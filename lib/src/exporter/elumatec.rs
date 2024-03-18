use std::{fs::File, io::Read, path::Path};

use self::tag::Tag;
use crate::Export;
use anyhow::{anyhow, Result};

mod tag;
mod variant;

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

    fn update_from_api(mut self) -> Self {
        todo!()
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
                        "{line}\n^ Invalid syntax on line {line_index} : unable to read template file"
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
    fn export(&self, project_uuid: &str, output_path: Option<String>) -> anyhow::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use tests::variant::Variant;

    use super::*;

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
    fn should_fail_with_invalid_line() {
        let res = ElumatecExporter::read_template(
            r#"
        :TAG
        Test == 5
        "#,
        );
        assert!(res.is_err());
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
