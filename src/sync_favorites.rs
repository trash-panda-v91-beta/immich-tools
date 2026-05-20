use crate::client::ImmichClient;
use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
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
}

pub async fn run(args: SyncArgs) -> Result<()> {
    let client = ImmichClient::new(args.url, args.api_key)?;
    let mut page = 1u32;
    let mut synced = 0u32;
    let mut skipped = 0u32;

    info!("Starting favorites sync into {}", args.dir.display());

    loop {
        let results = client.search_favorites(page).await?;
        let items = results.assets.items;

        if items.is_empty() {
            break;
        }

        for asset in &items {
            let dest = args.dir.join(&asset.original_file_name);
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

    info!("Done — {synced} downloaded, {skipped} skipped");
    Ok(())
}
