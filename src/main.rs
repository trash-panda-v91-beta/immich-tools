mod client;
mod sync_favorites;
mod sync_pcloud;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "immich-tools", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Download all favorited assets from Immich to a local directory
    SyncFavorites(sync_favorites::SyncArgs),
    /// Upload files from pCloud High-Res folders to Immich
    SyncPcloud(sync_pcloud::PCloudArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("immich_tools=info".parse()?))
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::SyncFavorites(args) => sync_favorites::run(args).await,
        Command::SyncPcloud(args) => sync_pcloud::run(args).await,
    }
}
