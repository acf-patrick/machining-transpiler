use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use self::{tag::Tag, variant::Variant};
use crate::{Export, Source};
use anyhow::{anyhow, Result};
use reqwest::{blocking::Client, Url};

mod tag;
mod variant;

#[derive(Clone)]
pub struct ElumatecExporter {
    tags: Vec<Tag>,
}

impl ElumatecExporter {
    pub fn new() -> Self {
        match Self::from_template() {
            Ok(exporter) => exporter,

            Err(err) => {
                eprintln!("Unable to read template file : \n{}", err.to_string());
                Self::default()
            }
        }
    }

    fn has_tag(&self, tag_name: &str) -> bool {
        for tag in &self.tags {
            if tag.name == tag_name {
                return true;
            }
        }

        false
    }

    fn set_attribute(&mut self, tag_name: &str, attr: &str, value: Variant) {
        if !self.has_tag(tag_name) {
            self.tags.push(Tag::new(tag_name));
        }

        for tag in &mut self.tags {
            if tag.name == tag_name {
                tag.set(attr, value);
                return;
            }
        }
    }

    fn update_cuts(&mut self, project_uuid: &str) -> Result<()> {
        let base_url = std::env::var("BASE_URL")?;
        let url =
            Url::parse(&base_url)?.join("/documentData/search/findProjectDataByProjectUuid")?;

        let res = Client::new()
            .get(url)
            .query(&[("projectUuid", project_uuid)])
            .send()?;
        let data = res.json::<serde_json::Value>()?;

        let structure_views = &data["structureViews"];
        if structure_views.is_null() {
            return Err(anyhow!("Unable to update cuts, `structureViews` is null"));
        }

        let structure_views = structure_views.as_array().unwrap();
        if structure_views.is_empty() {
            return Err(anyhow!("Unable to update cuts, no profile found"));
        }

        let structure_view = &structure_views[0];
        let profiles = &structure_view["nomenclature"]["profiles"];

        if profiles.is_null() {
            return Err(anyhow!("Unable to update cuts, `profiles` is null"));
        }

        let mut cut_tags = vec![];

        let profiles = profiles.as_array().unwrap();
        for profile in profiles {
            let length = profile["length"].as_f64().unwrap() as f32;

            let left = profile["extremity1"]["cuts"].as_array().unwrap();
            let right = profile["extremity2"]["cuts"].as_array().unwrap();

            for i in 0..left.len() {
                let mut cut = Tag::new("CUT");
                cut.set("CLength", Variant::Float(length as f32));

                let cut_index = cut_tags.len() + 1;
                cut.set("CNo", Variant::Int(cut_index as i32));

                let cut_object = &left[i];
                cut.set(
                    "CAngleLH",
                    Variant::Float(cut_object["h"].as_f64().unwrap() as f32),
                );
                cut.set(
                    "CAngleLV",
                    Variant::Float(cut_object["v"].as_f64().unwrap() as f32),
                );
                cut.set(
                    "CutLossL",
                    Variant::Float(cut_object["z"].as_f64().unwrap() as f32),
                );

                let cut_object = &right[i];
                cut.set(
                    "CAngleRH",
                    Variant::Float(cut_object["h"].as_f64().unwrap() as f32),
                );
                cut.set(
                    "CAngleRV",
                    Variant::Float(cut_object["v"].as_f64().unwrap() as f32),
                );
                cut.set(
                    "CutLossR",
                    Variant::Float(length + cut_object["z"].as_f64().unwrap() as f32),
                );

                cut_tags.push(cut);
            }
        }

        let cut_count = cut_tags.len() as i32;
        for tag in &mut cut_tags {
            tag.set("CCount", Variant::Int(cut_count));
            self.tags.push(tag.clone());
        }

        Ok(())
    }

    fn update_macros(&mut self, project_uuid: &str) -> Result<()> {
        let base_url = std::env::var("BASE_URL")?;
        let url =
            Url::parse(&base_url)?.join("/documentData/search/findProjectDataByProjectUuid")?;

        let res = Client::new()
            .get(url)
            .query(&[("projectUuid", project_uuid)])
            .send()?;
        let data = res.json::<serde_json::Value>()?;

        let structure_views = &data["structureViews"];
        if structure_views.is_null() {
            return Err(anyhow!("Unable to update macros, `structureViews` is null"));
        }

        let structure_views = structure_views.as_array().unwrap();
        for structure_view in structure_views {
            let nomenclature = &structure_view["nomenclature"];
            if nomenclature.is_null() {
                continue;
            }

            let profiles = &nomenclature["profiles"];
            if profiles.is_null() {
                continue;
            }

            let profiles = profiles.as_array().unwrap();
            for profile in profiles {
                let element = &profile["element"];
                if element.is_null() {
                    continue;
                }

                let machinings = &element["machinings"];
                if machinings.is_null() {
                    continue;
                }

                let machinings = machinings.as_array().unwrap();
                for machining in machinings {
                    let operations = &machining["operations"];
                    if operations.is_null() {
                        continue;
                    }

                    let operations = operations.as_array().unwrap();
                    for operation in operations {
                        let params = &operation["params"];
                        if params.is_null() {
                            continue;
                        }

                        let params = params.as_object().unwrap();
                        let mut var_set = false;

                        for (key, value) in params {
                            if key.as_bytes()[0] != b'v' {
                                continue;
                            }

                            var_set = true;

                            let var_index = (&key[1..]).parse::<u16>()?;
                            self.set_attribute(
                                "JOB",
                                &format!("Var{}", var_index - 1),
                                Variant::from(value.as_str().unwrap()),
                            );
                        }

                        if var_set {
                            return Ok(());
                        }
                    }
                }
            }
        }

        Err(anyhow!("Unable to update macros : no machinings set"))
    }

    fn update_from_api(&mut self, project_uuid: &str) -> Result<()> {
        if let Err(err) = self.update_macros(project_uuid) {
            eprintln!("{}", err.to_string());
        }

        if let Err(err) = self.update_cuts(project_uuid) {
            eprintln!("{}", err.to_string());
        }

        // other substitutions that should be done

        Ok(())
    }

    fn update_from_file(&mut self, file: &str) -> Result<()> {
        self.set_attribute("OPTIONS", "OScale", Variant::Int(1));
        self.set_attribute("OPTIONS", "OCreator", Variant::String("Elucad".to_owned()));

        let mut file = File::open(file)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let json: serde_json::Value = serde_json::from_str(&contents)?;

        let mut cut_tags = vec![];

        let articles = json["articles"].as_array().unwrap();
        for article in articles {
            match article["type"].as_str().unwrap() {
                "profile" => {
                    let mut cut = Tag::new("CUT");

                    let cut_index = cut_tags.len() + 1;
                    cut.set("CNo", Variant::Int(cut_index as i32));

                    let length = article["length"]["value"].as_f64().unwrap();
                    cut.set("CLength", Variant::Float(length as f32));

                    let cuts = article["cuts"].as_array().unwrap();

                    cut.set(
                        "CAngleLH",
                        Variant::Float(cuts[0][0]["h"]["value"].as_f64().unwrap() as f32),
                    );
                    cut.set(
                        "CAngleRH",
                        Variant::Float(cuts[1][0]["h"]["value"].as_f64().unwrap() as f32),
                    );

                    cut.set(
                        "CAngleLV",
                        Variant::Float(cuts[0][0]["v"]["value"].as_f64().unwrap() as f32),
                    );
                    cut.set(
                        "CAngleRV",
                        Variant::Float(cuts[1][0]["v"]["value"].as_f64().unwrap() as f32),
                    );

                    cut.set("CRotation", Variant::Float(0.0));
                    cut.set("CSawRotation", Variant::Float(0.0));

                    cut_tags.push(cut);
                }

                _ => {
                    // à voir
                }
            }
        }

        let cut_count = cut_tags.len() as i32;
        for mut tag in cut_tags {
            tag.set("CCount", Variant::Int(cut_count));
            self.tags.push(tag);
        }

        Ok(())
    }

    fn to_string(&self) -> String {
        let mut serialized = String::new();

        for tag in &self.tags {
            serialized += &tag.to_string();
        }

        serialized
    }

    fn from_template() -> Result<Self> {
        let template = Self::load_template()?;
        Self::read_template(&template)
    }

    fn load_template() -> Result<String> {
        let template_path = std::env::var("TEMPLATE_PATH")?;
        let path = Path::new(&template_path).join("elumatec");

        let mut file = File::open(path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        Ok(buffer)
    }

    fn read_template(template: &str) -> Result<Self> {
        let buffer = template.to_owned();

        let mut tags = vec![];
        let mut curr_tag: Option<Tag> = None;

        for (line_index, line) in buffer.split("\n").enumerate() {
            // Check comments
            let line = line.split("//").next().unwrap_or_default().trim();
            if line.is_empty() {
                continue;
            }

            if line.as_bytes()[0] == b':' {
                if let Some(tag) = &curr_tag {
                    tags.push(tag.clone());
                }

                curr_tag = Some(Tag::new(&line[1..]));
            } else if let Some(tag) = &mut curr_tag {
                if tag.update_attributes(line).is_none() {
                    return Err(anyhow!(
                        "{line}\n^ Invalid syntax on line {} : unable to read template file",
                        line_index + 1
                    ));
                }
            }
        }

        if let Some(tag) = curr_tag {
            tags.push(tag);
        }

        Ok(Self { tags })
    }
}

impl Default for ElumatecExporter {
    fn default() -> Self {
        ElumatecExporter { tags: vec![] }
    }
}

impl Export for ElumatecExporter {
    fn export(&self, source: Source, output_path: Option<String>) -> Result<()> {
        if let Source::File(file) = &source {
            let path = Path::new(file);
            if !path.is_file() {
                return Err(anyhow!("Source must be a file"));
            }
        }

        let mut exporter = self.clone();

        match source {
            Source::Api { project_uuid } => {
                exporter.update_from_api(&project_uuid)?;
            }

            Source::File(path) => {
                exporter.tags.clear();
                exporter.update_from_file(&path)?;
            }
        }

        let serialized = exporter.to_string();
        if let Some(output_path) = output_path {
            fs::write(output_path, serialized)?;
        } else {
            println!("{serialized}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tests::variant::Variant;

    use super::*;

    #[test]
    fn serialize() {
        let mut exporter = ElumatecExporter::default();
        let mut tag = Tag::new("TAG");

        tag.set("Int", Variant::Int(0));
        tag.set("Float", Variant::Float(0.0));
        tag.set("String", Variant::String("string".to_owned()));
        tag.set("V1", Variant::Int(0));
        tag.set("V10", Variant::Int(1));
        tag.set("V2", Variant::Int(2));

        exporter.tags.push(tag);
        let serialized = exporter.to_string();
        assert_eq!(serialized, ":TAG\nFloat\t=\t0\nInt\t=\t0\nString\t=\t\"string\"\nV1\t=\t0\nV2\t=\t2\nV10\t=\t1\n\n");
    }

    #[test]
    fn parse_string() {
        let res = ElumatecExporter::read_template(
            r#"
            :TAG // mock tag
    // line comment
            Int = 0
            Float = 0.0
            String = "string"
        "#,
        );

        assert!(res.is_ok());
        let exporter = res.unwrap();

        assert_eq!(exporter.tags.len(), 1);
        let tag = &exporter.tags[0];
        assert_eq!(tag.get("Int"), Some(Variant::Int(0)));
        assert_eq!(tag.get("Float"), Some(Variant::Float(0.0)));
        assert_eq!(
            tag.get("String"),
            Some(Variant::String("string".to_owned()))
        );
    }

    #[test]
    fn parse_empty_tag() {
        let res = ElumatecExporter::read_template(":TAG");
        assert!(res.is_ok());

        let exporter = res.unwrap();
        assert_eq!(exporter.tags.len(), 1);

        let tag = &exporter.tags[0];
        assert!(tag.is_empty());
    }

    #[test]
    fn should_ignore_headless_tag() {
        let res = ElumatecExporter::read_template(
            r#"
            Foo = test
            Bar = test
///////////////////////
            :TAG
            Int = 0
        "#,
        );

        assert!(res.is_ok());
        let exporter = res.unwrap();
        assert_eq!(exporter.tags.len(), 1);

        let tag = &exporter.tags[0];
        assert_eq!(tag.get("Int"), Some(Variant::Int(0)));
    }
}
