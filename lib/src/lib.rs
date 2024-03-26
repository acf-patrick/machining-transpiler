mod exporter;
pub mod util;

pub use exporter::Exporter;

pub enum Source {
    Api { project_uuid: String },
    File(String),
}

pub trait Export {
    fn export(&self, source: Source, output_path: Option<String>) -> anyhow::Result<()>;

    fn extension(&self) -> String;
}
