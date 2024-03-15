use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    /// Name of the project to export
    #[arg(short = 'n', long)]
    project_name: Option<String>,

    /// UUID of the project to export
    #[arg(short = 'u', long)]
    project_uuid: Option<String>,

    /// ID of the project to export
    #[arg(short = 'i', long)]
    project_id: Option<u32>,

    /// Name of the provider (Elumatec, ...)
    #[arg(short, long)]
    vendor: Option<String>,

    /// Path to output file
    #[arg(short, long)]
    output: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List implemented providers
    Vendors,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    todo!()
}
