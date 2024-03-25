use std::fs;

use anyhow::Result;
use reqwest::{blocking::Client, Url};
use serde_json::Value;

pub enum ProjectInfo {
    Uuid(String),
    Id(u16),
    Name(String),
}

pub fn get_project_uuid(project_info: ProjectInfo) -> Option<String> {
    let base_url = Url::parse(&std::env::var("BASE_URL").expect("BASE_URL is not set"))
        .expect("Invalid URL provided as BASE_URL");
    println!("Cover API : {}", base_url.to_string());
    let client = Client::new();

    match project_info {
        ProjectInfo::Id(id) => {
            let url = base_url.join("/project/search/findById").unwrap();
            let res = client.get(url).query(&[("id", id)]).send().unwrap();

            if let Ok(json) = res.json::<Value>() {
                Some(json["uuid"].as_str().unwrap().to_owned())
            } else {
                None
            }
        }

        ProjectInfo::Name(name) => {
            let url = base_url.join("/project/search/findByName").unwrap();
            let res = client.get(url).query(&[("name", &name)]).send().unwrap();

            if let Ok(json) = res.json::<Value>() {
                let projects = json.as_array().unwrap();
                if projects.is_empty() {
                    None
                } else {
                    let project = &projects[0];
                    Some(project["uuid"].as_str().unwrap().to_owned())
                }
            } else {
                None
            }
        }

        ProjectInfo::Uuid(uuid) => {
            let url = base_url.join("/project/search/findByUuid").unwrap();
            let res = client.get(url).query(&[("uuid", &uuid)]).send().unwrap();

            if res.status().is_success() {
                Some(uuid)
            } else {
                None
            }
        }
    }
}

pub fn find_files_with_extension(folder: &str, extension: &str) -> Result<Vec<String>> {
    let mut files = vec![];

    let entries = fs::read_dir(folder)?;
    for entry in entries {
        let path = entry?.path();
        if path.is_dir() {
            files.extend(find_files_with_extension(folder, extension)?);
        } else if let Some(ext) = path.extension() {
            if ext == extension {
                files.push(path.to_str().unwrap().to_owned());
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    // UUID 0488bf92-813f-4bbd-8e5f-16885d5b75df
    // Name import
    // ID 8

    use super::*;

    fn set_base_url() {
        std::env::set_var("BASE_URL", "http://localhost:5000");
    }

    #[test]
    fn by_id_should_return_none() {
        set_base_url();
        let res = get_project_uuid(ProjectInfo::Id(1000));
        assert!(res.is_none());
    }

    #[test]
    fn by_id() {
        set_base_url();
        let uuid = get_project_uuid(ProjectInfo::Id(8));
        assert!(uuid.is_some());

        let uuid = uuid.unwrap();
        assert_eq!(uuid, "0488bf92-813f-4bbd-8e5f-16885d5b75df");
    }

    #[test]
    fn by_name_should_return_none() {
        set_base_url();
        let res = get_project_uuid(ProjectInfo::Name("inexistent-project".to_owned()));
        assert!(res.is_none());
    }

    #[test]
    fn by_name() {
        set_base_url();
        let uuid = get_project_uuid(ProjectInfo::Name("import".to_owned()));
        assert!(uuid.is_some());

        let uuid = uuid.unwrap();
        assert_eq!(uuid, "0488bf92-813f-4bbd-8e5f-16885d5b75df");
    }

    #[test]
    fn by_uuid_should_return_none() {
        set_base_url();
        let res = get_project_uuid(ProjectInfo::Uuid("non-existent-uuid".to_owned()));
        assert!(res.is_none());
    }

    #[test]
    fn by_uuid() {
        set_base_url();
        let uuid = get_project_uuid(ProjectInfo::Uuid(
            "0488bf92-813f-4bbd-8e5f-16885d5b75df".to_owned(),
        ));
        assert!(uuid.is_some());

        let uuid = uuid.unwrap();
        assert_eq!(uuid, "0488bf92-813f-4bbd-8e5f-16885d5b75df");
    }
}
