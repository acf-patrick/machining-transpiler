use std::collections::HashMap;

use crate::Export;

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
}
