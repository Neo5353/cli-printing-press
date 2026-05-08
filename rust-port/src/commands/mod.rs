use anyhow::Result;
use tracing::info;

use crate::cli::{Cli, Command};

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Research { target } => placeholder("research", &target).await,
        Command::Generate { target } => placeholder("generate", &target).await,
        Command::Verify { target } => placeholder("verify", &target).await,
        Command::Scorecard { target } => placeholder("scorecard", &target).await,
        Command::Dogfood { target } => placeholder("dogfood", &target).await,
        Command::Publish { target } => placeholder("publish", &target).await,
        Command::Sniff { target } => placeholder("sniff", &target).await,
    }
}

async fn placeholder(command: &str, target: &str) -> Result<()> {
    info!(command, target, "rust-port placeholder invoked");
    println!("printing-press-rs: {command} for {target}");
    println!("status: placeholder — command surface is wired, implementation comes next");
    Ok(())
}
