use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "printing-press-rs")]
#[command(about = "Rust-native spike for the CLI Printing Press", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Research an API or website before generation
    Research(TargetArgs),
    /// Generate a printed CLI from a named API, spec, or URL
    Generate(GenerateArgs),
    /// Run deterministic checks against generated output
    Verify(VerifyArgs),
    /// Score a generated CLI against structural rules
    Scorecard(VerifyArgs),
    /// Run structural CLI self-tests
    Dogfood(VerifyArgs),
    /// Publish a validated CLI artifact
    Publish(PublishArgs),
    /// Discover undocumented API behavior from a site capture path or URL
    Sniff(TargetArgs),
}

#[derive(Debug, Clone, Args)]
pub struct TargetArgs {
    pub target: String,
}

#[derive(Debug, Clone, Args)]
pub struct GenerateArgs {
    pub target: String,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub force: bool,
}

#[derive(Debug, Clone, Args)]
pub struct VerifyArgs {
    pub target: PathBuf,
    #[arg(long)]
    pub spec: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct PublishArgs {
    pub target: PathBuf,
    #[arg(long)]
    pub destination: Option<PathBuf>,
}
