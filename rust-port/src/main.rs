use anyhow::Result;
use clap::Parser;
use printing_press_rs::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "printing_press_rs=info".into()),
        )
        .without_time()
        .compact()
        .init();

    let cli = Cli::parse();
    printing_press_rs::commands::run(cli).await
}
