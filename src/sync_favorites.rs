use crate::client::ImmichClient;
use anyhow::Result;
use clap::Args;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Args)]
pub struct SyncArgs {
    /// Immich server URL
    #[arg(long, env = "IMMICH_URL")]
    url: String,

    /// Immich API key
    #[arg(long, env = "IMMICH_API_KEY")]
    api_key: String,

    /// Directory to sync favorites into
    #[arg(long, env = "FAVORITES_DIR")]
    dir: PathBuf,

    /// Re-run the sync on this interval (e.g. "6h"); unset = run once and exit
    #[arg(long, env = "SYNC_INTERVAL")]
    interval: Option<String>,
}

pub async fn run(args: SyncArgs) -> Result<()> {
    let SyncArgs {
        url,
        api_key,
        dir,
        interval,
    } = args;
    let client = ImmichClient::new(url, api_key)?;
    let interval = match interval.as_deref() {
        Some(s) => Some(humantime::parse_duration(s)?),
        None => None,
    };

    loop {
        sync_once(&client, &dir).await?;
        match interval {
            Some(d) => tokio::time::sleep(d).await,
            None => break,
        }
    }
    Ok(())
}

async fn sync_once(client: &ImmichClient, dir: &Path) -> Result<()> {
    let mut page = 1u32;
    let mut synced = 0u32;
    let mut skipped = 0u32;

    info!("Starting favorites sync into {}", dir.display());

    loop {
        let results = client.search_favorites(page).await?;
        let items = results.assets.items;

        if items.is_empty() {
            break;
        }

        for asset in &items {
            let dest = dir.join(&asset.original_file_name);
            if dest.exists() {
                skipped += 1;
                continue;
            }

            info!("download: {}", asset.original_file_name);
            match client.download_asset(&asset.id, &dest).await {
                Ok(()) => synced += 1,
                Err(e) => warn!("failed to download {}: {e:#}", asset.original_file_name),
            }
        }

        match results.assets.next_page {
            Some(_) => page += 1,
            None => break,
        }
    }

    info!("Done - {synced} downloaded, {skipped} skipped");
    Ok(())
}
