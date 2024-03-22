use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::{util::find_files_with_extension, Export, Source};

mod elumatec;

pub struct Exporter {
    exporters: HashMap<String, Box<dyn Export>>,
}

impl Exporter {
    pub fn new() -> Self {
        let mut exporters: HashMap<String, Box<dyn Export>> = HashMap::new();

        exporters.insert(
            "elumatec".to_owned(),
            Box::new(elumatec::ElumatecExporter::new()),
        );

        Exporter { exporters }
    }

    pub fn vendors(&self) -> Vec<String> {
        self.exporters
            .keys()
            .map(|vendor| vendor.to_owned())
            .collect()
    }

    pub fn support(&self, vendor: &str) -> bool {
        self.get_key(vendor).is_some()
    }

    fn get_key(&self, vendor: &str) -> Option<String> {
        for key in self.exporters.keys() {
            if key.to_lowercase() == vendor.to_lowercase() {
                return Some(key.to_owned());
            }
        }

        None
    }

    pub fn transpile_folder(&self, folder: &str, vendor: &str) -> Result<()> {
        let files = find_files_with_extension(folder, "json")?;
        for file in files {
            let path = std::path::Path::new(&file);

            self.export(
                Source::File(file.clone()),
                vendor,
                path.with_extension("ncw")
                    .to_str()
                    .map(|path| path.to_owned()),
            )?;
        }

        todo!()
    }

    pub fn export(&self, source: Source, vendor: &str, output_path: Option<String>) -> Result<()> {
        let record_key = self.get_key(vendor);
        if record_key.is_none() {
            return Err(anyhow!("No exporter implemented for provider `{vendor}`"));
        }

        let exporter = self.exporters.get(&record_key.unwrap()).unwrap();

        exporter.export(source, output_path)
    }
}