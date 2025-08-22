use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use lib::{
    util::{get_project_uuid, ProjectInfo},
    Exporter, Source,
};

#[derive(Parser)]
#[command(about = r#"
Transpiles Cover Datas to machine files.
By default, the CLI uses API data but file/folder can be used with the `transpile` subcommand.
"#)]
struct Cli {
    /// Path to output file
    #[arg(short, long)]
    output: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, PartialEq)]
enum Commands {
    /// List implemented providers
    Vendors,

    /// Fetch data from Cover API in provider format
    FromApi {
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
        vendor: String,
    },

    /// Transpile JSON file or entire folder.
    FromFile {
        #[arg(short, long, default_value = "false")]
        /// Recursive mode. Read JSON files within folder.
        recursive: bool,

        /// Name of the provider (Elumatec, ...)
        #[arg(short, long)]
        vendor: String,

        source: String,
    },
}

trait CheckVendor {
    fn check_vendor(&self, vendor: &str) -> Result<()>;
}

impl CheckVendor for Exporter {
    fn check_vendor(&self, vendor: &str) -> Result<()> {
        if !self.support(vendor) {
            return Err(anyhow!("No exporter implemented for provider `{vendor}`\n Use `vendors` subcommand to list implemented providers."));
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    dotenv().expect("Unable to load environnement variables, .env file not found");

    let cli = Cli::parse();
    let exporter = Exporter::new();

    match cli.command {
        Commands::FromApi {
            project_name,
            project_uuid,
            project_id,
            vendor,
        } => {
            exporter.check_vendor(&vendor)?;

            // Check project existence

            if project_name.is_none() && project_id.is_none() && project_uuid.is_none() {
                return Err(anyhow!(
                    "You have to set either project-name or project-id or project-uuid."
                ));
            }

            let mut project_uuid: Option<String> = None;
            if let Some(project_name) = project_name {
                project_uuid = get_project_uuid(ProjectInfo::Name(project_name));
            } else if let Some(project_id) = project_id {
                project_uuid = get_project_uuid(ProjectInfo::Id(project_id));
            } else if let Some(uuid) = project_uuid {
                project_uuid = get_project_uuid(ProjectInfo::Uuid(uuid));
            }

            if let Some(project_uuid) = project_uuid {
                println!("Using project {project_uuid}\n");
                exporter.export(Source::Api { project_uuid }, &vendor, cli.output)?;
            } else {
                return Err(anyhow!("Project not found"));
            }
        }

        Commands::FromFile {
            recursive,
            vendor,
            source,
        } => {
            exporter.check_vendor(&vendor)?;

            if recursive {
                exporter.transpile_folder(&source, &vendor, cli.output)?;
            } else {
                exporter.export(Source::File(source), &vendor, cli.output)?;
            }
        }

        Commands::Vendors => {
            let vendors = exporter.vendors();
            println!("Impemented providers : ");
            for vendor in vendors {
                println!("- {vendor}");
            }

            return Ok(());
        }
    }

    Ok(())
}
