mod exporter;
pub mod util;

pub use exporter::Exporter;

pub trait Export {
    fn export(&self, project_uuid: &str, output_path: Option<String>) -> anyhow::Result<()>;
}
