use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use lib::{
    util::{get_project_uuid, ProjectInfo},
    Exporter,
};

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
    project_id: Option<u16>,

    /// Name of the provider (Elumatec, ...)
    #[arg(short, long)]
    vendor: Option<String>,

    /// Path to output file
    #[arg(short, long)]
    output: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List implemented providers
    Vendors,
}

fn main() -> Result<()> {
    dotenv().expect("Unable to load environnement variables, .env file not found");

    let mut cli = Cli::parse();
    cli.project_name = Some("import".to_owned());
    cli.vendor = Some("elumatec".to_owned());

    let exporter = Exporter::new();

    if let Some(command) = cli.command {
        match command {
            Commands::Vendors => {
                let vendors = exporter.vendors();
                println!("Impemented providers : ");
                for vendor in vendors {
                    println!("- {vendor}");
                }
            }
        }

        return Ok(());
    }

    if cli.vendor.is_none() {
        return Err(anyhow!(
            "You have to provider a provider to use. Use -v or --vendor argument"
        ));
    }

    let vendor = cli.vendor.unwrap();
    if !exporter.support(&vendor) {
        return Err(anyhow!("No exporter implemented for provider `{vendor}`\n Use `vendors` subcommand to list implemented providers."));
    }

    // Check project existence

    if cli.project_name.is_none() && cli.project_id.is_none() && cli.project_uuid.is_none() {
        return Err(anyhow!(
            "You have to set either project-name or project-id or project-uuid."
        ));
    }

    let mut project_uuid: Option<String> = None;
    if let Some(project_name) = cli.project_name {
        project_uuid = get_project_uuid(ProjectInfo::Name(project_name));
    } else if let Some(project_id) = cli.project_id {
        project_uuid = get_project_uuid(ProjectInfo::Id(project_id));
    } else if let Some(uuid) = cli.project_uuid {
        project_uuid = get_project_uuid(ProjectInfo::Uuid(uuid));
    }

    if let Some(project_uuid) = project_uuid {
        println!("Using project {project_uuid}\n");
        exporter.export(&project_uuid, &vendor, cli.output)?;

        Ok(())
    } else {
        Err(anyhow!("Project not found"))
    }
}
