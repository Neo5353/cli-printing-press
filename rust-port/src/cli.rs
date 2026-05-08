use clap::{Parser, Subcommand};

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
    Research { target: String },
    /// Generate a printed CLI from a named API, spec, or URL
    Generate { target: String },
    /// Run deterministic checks against generated output
    Verify { target: String },
    /// Score a generated CLI against structural rules
    Scorecard { target: String },
    /// Run structural CLI self-tests
    Dogfood { target: String },
    /// Publish a validated CLI artifact
    Publish { target: String },
    /// Discover undocumented API behavior from a site capture path or URL
    Sniff { target: String },
}
