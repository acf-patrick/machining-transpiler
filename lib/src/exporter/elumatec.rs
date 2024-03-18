use crate::Export;

mod variant;
mod tag;

pub struct ElumatecExporter;

impl ElumatecExporter {
    pub fn new() -> Self {
        ElumatecExporter
    }
}

impl Export for ElumatecExporter {
    fn export(&self, project_uuid: &str, output_path: Option<String>) -> anyhow::Result<()> {
        todo!()
    }
}
