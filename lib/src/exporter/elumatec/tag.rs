use super::variant::Variant;
use human_sort::sort;
use std::collections::HashMap;

pub struct Tag {
    name: String,
    attributes: HashMap<String, Variant>,
}

#[allow(dead_code)]
impl Tag {
    pub fn new(tag_name: &str) -> Self {
        Tag {
            name: tag_name.to_owned(),
            attributes: HashMap::new(),
        }
    }

    pub fn get(&self, attr: &str) -> Option<Variant> {
        self.attributes.get(attr).map(|value| value.clone())
    }

    pub fn set(&mut self, attr: &str, value: Variant) {
        self.attributes.insert(attr.to_owned(), value);
    }

    pub fn to_string(&self) -> String {
        let mut serialized = format!(":{}\n", self.name);

        let mut keys = self
            .attributes
            .keys()
            .map(|key| key.as_str())
            .collect::<Vec<_>>();
        let mut keys = keys.as_mut_slice();
        sort(&mut keys);

        for key in keys {
            let value = self.get(key).unwrap();
            serialized += &format!("{key}\t=\t{value}\n");
        }
        serialized += "\n";

        serialized
    }

    pub fn update_attributes(&mut self, line: &str) -> Option<(String, Variant)> {
        // Check for comment
        let parts = line.split("//").collect::<Vec<_>>();

        let left = parts[0].trim();
        if left.is_empty() {
            return None;
        }

        let parts = left.split("=").collect::<Vec<_>>();
        if parts.len() != 2 {
            return None;
        }

        let key = parts[0].trim();
        let value = Variant::from(parts[1].trim());

        self.set(key, value.clone());

        Some((key.to_owned(), value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_comment() {
        let mut tag = Tag::new("tag");
        assert_eq!(tag.update_attributes("// this is a comment"), None);
        assert!(tag.attributes.is_empty());
    }

    #[test]
    fn parse_line_with_comment() {
        let mut tag = Tag::new("tag");
        assert_eq!(
            tag.update_attributes("key = value // this is a comment"),
            Some(("key".to_owned(), Variant::String("value".to_owned())))
        );
        assert_eq!(
            tag.attributes.get("key"),
            Some(&Variant::String("value".to_owned()))
        );
    }

    #[test]
    fn parse_int() {
        let mut tag = Tag::new("tag");
        assert_eq!(
            tag.update_attributes("Int = 0"),
            Some(("Int".to_owned(), Variant::Int(0)))
        );
    }

    #[test]
    fn parse_float() {
        let mut tag = Tag::new("tag");
        assert_eq!(
            tag.update_attributes("Float = 0.0"),
            Some(("Float".to_owned(), Variant::Float(0.0)))
        );
    }

    #[test]
    fn parse_string() {
        let mut tag = Tag::new("tag");
        assert_eq!(
            tag.update_attributes("String = \"Some string\""),
            Some((
                "String".to_owned(),
                Variant::String("Some string".to_owned())
            ))
        );
    }

    #[test]
    fn parse_null_value() {
        let mut tag = Tag::new("tag");
        assert_eq!(
            tag.update_attributes("Null = // null attribute"),
            Some(("Null".to_owned(), Variant::Null))
        );
    }

    #[test]
    fn serialize() {
        let mut tag = Tag::new("TAG");
        tag.set("Int", Variant::Int(0));
        tag.set("Float", Variant::Float(0.1));
        tag.update_attributes("String = string");

        assert_eq!(
            tag.to_string(),
            ":TAG\nFloat\t=\t0.1\nInt\t=\t0\nString\t=\t\"string\"\n\n"
        );
    }
}
