use crate::client::ImmichClient;
use anyhow::{bail, Context, Result};
use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::post,
    Router,
};
use clap::Args;
use futures_util::{stream, StreamExt};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tracing::{info, warn};
use unicode_normalization::UnicodeNormalization;

/// Maximum attempts (including the first) for each file before giving up.
const MAX_ATTEMPTS: u32 = 3;

#[derive(Args)]
pub struct PCloudArgs {
    /// Immich server URL
    #[arg(long, env = "IMMICH_URL")]
    url: String,

    /// Immich API key
    #[arg(long, env = "IMMICH_API_KEY")]
    api_key: String,

    /// pCloud OAuth access token
    #[arg(long, env = "PCLOUD_TOKEN")]
    token: String,

    /// pCloud API host - api.pcloud.com (US) or eapi.pcloud.com (Europe)
    #[arg(long, env = "PCLOUD_HOST", default_value = "api.pcloud.com")]
    host: String,

    /// pCloud folder path to sync (one-shot CLI mode only; the HTTP API takes it per request)
    #[arg(long, env = "FOLDER")]
    folder: Option<String>,

    /// Directory that stores per-folder JSON ledgers of processed files
    #[arg(long, env = "STATE_DIR")]
    state_dir: PathBuf,

    /// Address to bind the HTTP API on (serve mode)
    #[arg(long, env = "LISTEN", default_value = "0.0.0.0:8080")]
    listen: String,

    /// Maximum concurrent downloads/uploads per folder
    #[arg(long, env = "CONCURRENCY", default_value_t = 4)]
    concurrency: usize,

    /// Directory to sync Immich favorites into; enables background favorites sync in serve
    #[arg(long, env = "FAVORITES_DIR")]
    favorites_dir: Option<PathBuf>,

    /// How often to run the background favorites sync (e.g. "6h"); requires --favorites-dir
    #[arg(long, env = "SYNC_INTERVAL")]
    favorites_interval: Option<String>,

    /// Bearer token required to call the API
    #[arg(long, env = "API_TOKEN")]
    api_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metadata {
    #[serde(default)]
    isfolder: bool,
    #[serde(default)]
    name: Option<String>,
    fileid: Option<u64>,
    hash: Option<u64>,
    size: Option<u64>,
    #[serde(default)]
    contents: Vec<Metadata>,
}

#[derive(Debug, Deserialize)]
struct ListFolder {
    result: i32,
    metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize)]
struct FileLink {
    hosts: Vec<String>,
    path: String,
}

struct FileRef {
    id: u64,
    name: String,
    hash: u64,
    size: u64,
}

struct PCloud {
    http: HttpClient,
    token: String,
    host: String,
}

impl PCloud {
    fn new(token: String, host: String) -> Result<Self> {
        let http = HttpClient::builder()
            .build()
            .context("failed to build pCloud client")?;
        Ok(Self { http, token, host })
    }

    async fn list_folder(&self, path: &str) -> Result<Metadata> {
        let resp: ListFolder = self
            .http
            .get(format!("https://{}/listfolder", self.host))
            .query(&[("path", path), ("access_token", &self.token)])
            .send()
            .await
            .context("listfolder request failed")?
            .error_for_status()
            .context("listfolder error status")?
            .json()
            .await
            .context("failed to parse listfolder response")?;
        resp.metadata
            .with_context(|| format!("listfolder failed (result {})", resp.result))
    }

    /// Stream a file to disk, computing its SHA-1 checksum on the way.
    /// Returns (bytes written, checksum hex).
    async fn download_and_hash(&self, id: u64, dest: &Path) -> Result<(u64, String)> {
        let link: FileLink = self
            .http
            .get(format!("https://{}/getfilelink", self.host))
            .query(&[
                ("fileid", id.to_string()),
                ("access_token", self.token.clone()),
            ])
            .send()
            .await
            .context("getfilelink request failed")?
            .error_for_status()
            .context("getfilelink error status")?
            .json()
            .await
            .context("failed to parse getfilelink response")?;
        let host = link
            .hosts
            .first()
            .context("getfilelink returned no hosts")?;
        let url = format!("https://{host}{}", link.path);
        let resp = self
            .http
            .get(url)
            .send()
            .await
            .context("file download request failed")?
            .error_for_status()
            .context("file download error status")?;

        let mut file = tokio::fs::File::create(dest)
            .await
            .context("failed to create temp file")?;
        let mut hasher = Sha1::new();
        let mut written: u64 = 0;
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("download chunk error")?;
            hasher.update(&chunk);
            written += chunk.len() as u64;
            file.write_all(&chunk)
                .await
                .context("failed to write temp file")?;
        }
        file.flush().await.context("failed to flush temp file")?;
        let digest = hasher.finalize();
        let checksum = digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        Ok((written, checksum))
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct FolderState {
    files: BTreeMap<u64, FileEntry>,
}

#[derive(Debug)]
enum Outcome {
    Skipped,
    Uploaded,
    Duplicate,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileEntry {
    name: String,
    hash: u64,
    size: u64,
    uploaded_at: String,
}

/// Collect the file entries directly inside one already-known folder.
fn collect_files(folder: &Metadata, out: &mut Vec<FileRef>) {
    for file in &folder.contents {
        if !file.isfolder {
            if let (Some(id), Some(name), Some(hash)) = (file.fileid, file.name.clone(), file.hash)
            {
                out.push(FileRef {
                    id,
                    name,
                    hash,
                    size: file.size.unwrap_or(0),
                });
            }
        }
    }
}

/// Map a pCloud path to a safe on-disk state filename.
fn sanitize(path: &str) -> String {
    path.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Normalize a folder path to NFD. pCloud stores names decomposed (NFD, e.g.
/// `digita\u{301}l`), so folder lookup (`listfolder`) fails with result 2005 if
/// the stored path uses composed (NFC) accents. Normalize everywhere a path is
/// read or written so all comparisons and lookups are consistent.
fn nfd(path: &str) -> String {
    path.nfd().collect()
}

#[derive(Clone)]
struct Shared {
    args: Arc<PCloudArgs>,
    lock: Arc<Mutex<()>>,
}

#[derive(Deserialize)]
struct SyncBody {
    path: String,
}

async fn handle_sync(
    State(state): State<Shared>,
    Json(body): Json<SyncBody>,
) -> Result<Json<Value>, StatusCode> {
    let _guard = state.lock.lock().await;
    match sync_folder(&state.args, &body.path).await {
        Ok((uploaded, skipped)) => Ok(Json(json!({ "uploaded": uploaded, "skipped": skipped }))),
        Err(e) => {
            warn!("sync failed: {e:#}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

fn authorized(req: &Request<Body>, token: &str) -> bool {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        == Some(token)
}

async fn require_auth(
    State(token): State<Arc<str>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if authorized(&request, &token) {
        next.run(request).await
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

pub async fn serve(args: PCloudArgs) -> Result<()> {
    if let Some(dir) = args.favorites_dir.clone() {
        spawn_favorites_sync(
            args.url.clone(),
            args.api_key.clone(),
            dir,
            args.favorites_interval.as_deref(),
        )?;
    }
    let addr = args.listen.clone();
    let api_token = args.api_token.clone();
    let shared = Shared {
        args: Arc::new(args),
        lock: Arc::new(Mutex::new(())),
    };
    let app = Router::new()
        .route("/sync", post(handle_sync))
        .with_state(shared)
        .layer(middleware::from_fn_with_state(
            Arc::<str>::from(api_token),
            require_auth,
        ));
    info!("API auth enabled (bearer token)");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    info!("pcloud API listening on {addr}");
    axum::serve(listener, app).await.context("server error")
}

/// Run a favorites sync in the background on the given interval.
fn spawn_favorites_sync(
    url: String,
    api_key: String,
    dir: PathBuf,
    interval: Option<&str>,
) -> Result<()> {
    let client = ImmichClient::new(url, api_key)?;
    let interval = match interval {
        Some(s) => Some(humantime::parse_duration(s)?),
        None => None,
    };
    tokio::spawn(async move {
        loop {
            if let Err(e) = crate::sync_favorites::run_once(&client, &dir).await {
                warn!("favorites sync failed: {e:#}");
            }
            match interval {
                Some(d) => tokio::time::sleep(d).await,
                None => break,
            }
        }
    });
    Ok(())
}

pub async fn run(args: PCloudArgs) -> Result<()> {
    let path = args
        .folder
        .as_deref()
        .context("a folder path is required: pass --folder <path>")?;
    sync_folder(&args, path).await.map(|_| ())
}

/// Sync a single pCloud folder to Immich.
async fn sync_folder(args: &PCloudArgs, path: &str) -> Result<(u32, u32)> {
    // Normalize to NFD so pCloud listfolder matches its decomposed names.
    let path = nfd(path).trim().to_string();
    if path.is_empty() {
        bail!("folder path must not be empty");
    }

    let immich = ImmichClient::new(args.url.clone(), args.api_key.clone())?;
    let pcloud = PCloud::new(args.token.clone(), args.host.clone())?;
    let state = args.state_dir.clone();
    std::fs::create_dir_all(&state).context("failed to create state dir")?;

    let mut uploaded = 0u32;
    let mut skipped = 0u32;

    let state_path = state.join(format!("{}.json", sanitize(&path)));
    let mut folder_state: FolderState = match std::fs::read_to_string(&state_path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => FolderState::default(),
    };

    info!("scanning {path}");
    let folder = pcloud.list_folder(&path).await?;
    let mut files = Vec::new();
    collect_files(&folder, &mut files);

    let tmp_dir = std::env::temp_dir();
    let outcomes: Vec<(FileRef, Outcome)> = stream::iter(files)
        .map(|f| {
            let pcloud = &pcloud;
            let immich = &immich;
            let folder_state = &folder_state;
            let tmp_dir = &tmp_dir;
            async move {
                if folder_state.files.get(&f.id).map(|e| e.hash) == Some(f.hash) {
                    return (f, Outcome::Skipped);
                }

                let tmp = tmp_dir.join(format!("immich-pcloud-{}.tmp", f.id));
                // Retry each file a bounded number of times, with a per-attempt
                // timeout, so a transient stall fails that file (and is retried)
                // instead of hanging the whole sync. Files still failing stay out
                // of the ledger and are picked up on a later run.
                let mut last_err: Option<String> = None;
                let mut outcome = Outcome::Failed;
                for attempt in 1..=MAX_ATTEMPTS {
                    let result = tokio::time::timeout(Duration::from_secs(900), async {
                        let (size, checksum) = pcloud.download_and_hash(f.id, &tmp).await?;
                        immich
                            .upload_asset_file(
                                &f.name,
                                &format!("pcloud-{}", f.id),
                                &tmp,
                                size,
                                &checksum,
                            )
                            .await
                    })
                    .await;

                    let _ = tokio::fs::remove_file(&tmp).await;
                    match result {
                        Ok(Ok(Some(_))) => {
                            outcome = Outcome::Uploaded;
                            break;
                        }
                        Ok(Ok(None)) => {
                            outcome = Outcome::Duplicate;
                            break;
                        }
                        Ok(Err(e)) => {
                            last_err = Some(format!("{e:#}"));
                            warn!("failed to process {} (attempt {attempt}): {e:#}", f.name);
                        }
                        Err(_) => {
                            last_err = Some("timed out (>900s)".into());
                            warn!("timed out processing {} (attempt {attempt})", f.name);
                        }
                    }
                }
                if matches!(outcome, Outcome::Failed) {
                    if let Some(e) = last_err {
                        warn!("giving up on {} after {MAX_ATTEMPTS} attempts: {e}", f.name);
                    }
                } else {
                    info!("processed {}: {:?}", f.name, outcome);
                }
                (f, outcome)
            }
        })
        .buffer_unordered(args.concurrency.max(1))
        .collect()
        .await;

    let mut changed = false;
    for (f, outcome) in outcomes {
        match outcome {
            Outcome::Skipped => skipped += 1,
            Outcome::Failed => {}
            Outcome::Uploaded | Outcome::Duplicate => {
                folder_state.files.insert(
                    f.id,
                    FileEntry {
                        name: f.name.clone(),
                        hash: f.hash,
                        size: f.size,
                        uploaded_at: chrono::Utc::now().to_rfc3339(),
                    },
                );
                changed = true;
                if matches!(outcome, Outcome::Uploaded) {
                    uploaded += 1;
                } else {
                    skipped += 1;
                }
            }
        }
    }

    if changed {
        let json = serde_json::to_string_pretty(&folder_state)
            .context("failed to serialize folder state")?;
        std::fs::write(&state_path, json)
            .with_context(|| format!("failed to write state {}", state_path.display()))?;
    }

    info!("done - {uploaded} uploaded, {skipped} skipped");
    Ok((uploaded, skipped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_requires_bearer_match() {
        let with = |h: &str| {
            Request::builder()
                .header(header::AUTHORIZATION, h)
                .body(Body::empty())
                .unwrap()
        };
        assert!(authorized(&with("Bearer secret"), "secret"));
        assert!(!authorized(&with("Bearer wrong"), "secret"));
        assert!(!authorized(&with("Basic abc"), "secret"));
        assert!(!authorized(
            &Request::builder().body(Body::empty()).unwrap(),
            "secret"
        ));
    }

    #[test]
    fn normalizes_nfc_to_nfd() {
        // Composed (NFC) accents, including capital/caron letters, decompose to NFD
        // so pCloud listfolder matches its stored names.
        let nfc = "/FOTKY/FOTKY - digitál/KONCERTY, PŘEDSTAVENÍ/Rodinné";
        let decomposed = nfd(nfc);
        assert_ne!(
            nfc, decomposed,
            "expected NFC input to differ from its NFD form"
        );
        // NFD is a fixed point: normalizing twice is unchanged.
        assert_eq!(nfd(&decomposed), decomposed);
        // spot-check the decomposed form
        assert_eq!(
            decomposed,
            "/FOTKY/FOTKY - digita\u{301}l/KONCERTY, PR\u{30c}EDSTAVENI\u{301}/Rodinne\u{301}"
        );
    }

    #[test]
    fn collects_only_direct_files() {
        // exact configured folder - files are collected, nested folders are ignored
        let folder = Metadata {
            isfolder: true,
            name: None,
            fileid: None,
            hash: None,
            size: None,
            contents: vec![
                file(1, "otto.jpg", 1000),
                file(2, "maria.jpg", 2000),
                Metadata {
                    isfolder: true,
                    name: None,
                    fileid: None,
                    hash: None,
                    size: None,
                    contents: vec![file(3, "nested.jpg", 3000)],
                },
            ],
        };

        let mut files = Vec::new();
        collect_files(&folder, &mut files);

        let mut ids: Vec<u64> = files.iter().map(|f| f.id).collect();
        ids.sort_unstable();
        assert_eq!(ids, vec![1, 2]);
    }

    fn file(id: u64, name: &str, hash: u64) -> Metadata {
        Metadata {
            isfolder: false,
            name: Some(name.into()),
            fileid: Some(id),
            hash: Some(hash),
            size: Some(10),
            contents: vec![],
        }
    }
}
