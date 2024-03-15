mod exporter;

pub trait Export {
    fn export(&self, project_uuid: &str, output_path: Option<String>) -> anyhow::Result<()>;
}
