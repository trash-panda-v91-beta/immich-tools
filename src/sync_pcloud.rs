use crate::client::ImmichClient;
use anyhow::{Context, Result};
use clap::Args;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tracing::{info, warn};

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

    /// Path to config.toml listing base folders to scan
    #[arg(long, env = "CONFIG")]
    config: std::path::PathBuf,

    /// Directory that stores per-folder JSON ledgers of processed files
    #[arg(long, env = "STATE_DIR")]
    state_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Config {
    /// Exact pCloud folder paths to upload, e.g. "FOTKY/FOTKY - digitál/RODINA/.../High-Res"
    folders: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metadata {
    #[serde(default)]
    isfolder: bool,
    #[serde(default)]
    name: Option<String>,
    fileid: Option<i64>,
    hash: Option<i64>,
    size: Option<i64>,
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
    id: i64,
    name: String,
    hash: i64,
    size: i64,
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
            .query(&[("path", path), ("auth", &self.token)])
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

    async fn download_bytes(&self, id: i64) -> Result<Vec<u8>> {
        let link: FileLink = self
            .http
            .get(format!("https://{}/getfilelink", self.host))
            .query(&[
                ("fileid", id.to_string()),
                ("auth", self.token.clone()),
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
        self.http
            .get(url)
            .send()
            .await
            .context("file download request failed")?
            .error_for_status()
            .context("file download error status")?
            .bytes()
            .await
            .context("failed to read file bytes")
            .map(|b| b.to_vec())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct FolderState {
    files: BTreeMap<i64, FileEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileEntry {
    name: String,
    hash: i64,
    size: i64,
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

pub async fn run(args: PCloudArgs) -> Result<()> {
    let immich = ImmichClient::new(args.url, args.api_key)?;
    let pcloud = PCloud::new(args.token.clone(), args.host.clone())?;
    let state = args.state_dir.clone();
    std::fs::create_dir_all(&state).context("failed to create state dir")?;

    let raw = std::fs::read_to_string(&args.config)
        .with_context(|| format!("failed to read config {}", args.config.display()))?;
    let config: Config = toml::from_str(&raw).context("failed to parse config.toml")?;

    let mut uploaded = 0u32;
    let mut skipped = 0u32;

    for path in &config.folders {
        let path = path.trim();
        if path.is_empty() {
            continue;
        }

        let state_path = state.join(format!("{}.json", sanitize(path)));
        let mut folder_state: FolderState = match std::fs::read_to_string(&state_path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => FolderState::default(),
        };

        info!("scanning {path}");
        let folder = pcloud.list_folder(path).await?;
        let mut files = Vec::new();
        collect_files(&folder, &mut files);

        let mut changed = false;
        for f in &files {
            // skip files we already processed with the same content
            if folder_state.files.get(&f.id).map(|e| e.hash) == Some(f.hash) {
                skipped += 1;
                continue;
            }

            info!("upload: {}", f.name);
            match pcloud.download_bytes(f.id).await {
                Ok(bytes) => match immich
                    .upload_asset_bytes(&f.name, &format!("pcloud-{}", f.id), bytes)
                    .await
                {
                    Ok(_) => {
                        folder_state.files.insert(
                            f.id,
                            FileEntry {
                                name: f.name.clone(),
                                hash: f.hash,
                                size: f.size,
                                uploaded_at: chrono::Utc::now().to_rfc3339(),
                            },
                        );
                        uploaded += 1;
                        changed = true;
                    }
                    Err(e) => warn!("failed to upload {}: {e:#}", f.name),
                },
                Err(e) => warn!("failed to download {}: {e:#}", f.name),
            }
        }

        if changed {
            let json = serde_json::to_string_pretty(&folder_state)
                .context("failed to serialize folder state")?;
            std::fs::write(&state_path, json)
                .with_context(|| format!("failed to write state {}", state_path.display()))?;
        }
    }

    info!("done - {uploaded} uploaded, {skipped} skipped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let mut ids: Vec<i64> = files.iter().map(|f| f.id).collect();
        ids.sort_unstable();
        assert_eq!(ids, vec![1, 2]);
    }

    fn file(id: i64, name: &str, hash: i64) -> Metadata {
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
