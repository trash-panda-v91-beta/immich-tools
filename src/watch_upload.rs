use crate::client::ImmichClient;
use anyhow::{Context, Result};
use clap::Args;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::{info, warn};

#[derive(Args)]
pub struct WatchArgs {
    /// Immich server URL
    #[arg(long, env = "IMMICH_URL")]
    url: String,

    /// Immich API key
    #[arg(long, env = "IMMICH_API_KEY")]
    api_key: String,

    /// Directory to watch for new files
    #[arg(long, env = "WATCH_DIR")]
    dir: PathBuf,
}

pub async fn run(args: WatchArgs) -> Result<()> {
    let client = ImmichClient::new(args.url, args.api_key)?;
    let (tx, mut rx) = mpsc::channel::<notify::Result<Event>>(128);

    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.blocking_send(res);
    })
    .context("failed to create filesystem watcher")?;

    watcher
        .watch(&args.dir, RecursiveMode::Recursive)
        .with_context(|| format!("failed to watch {}", args.dir.display()))?;

    info!("Watching {} for new files", args.dir.display());

    while let Some(event) = rx.recv().await {
        let event = match event {
            Ok(e) => e,
            Err(e) => {
                warn!("watch error: {e}");
                continue;
            }
        };

        // Only act on file creation/moves-into
        if !matches!(
            event.kind,
            EventKind::Create(_)
                | EventKind::Modify(notify::event::ModifyKind::Name(
                    notify::event::RenameMode::To
                ))
        ) {
            continue;
        }

        for path in event.paths {
            if !path.is_file() {
                continue;
            }

            info!("upload: {}", path.display());
            match client.upload_asset(&path).await {
                Ok(Some(id)) => info!("uploaded {} → asset {id}", path.display()),
                Ok(None) => info!("duplicate, skipped: {}", path.display()),
                Err(e) => warn!("failed to upload {}: {e:#}", path.display()),
            }
        }
    }

    Ok(())
}
