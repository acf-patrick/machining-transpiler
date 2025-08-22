use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Variant {
    Int(i32),
    Float(f32),
    String(String),
    Null,
}

impl Variant {
    pub fn to_string(&self) -> String {
        match self {
            Self::Int(int) => format!("{int}"),
            Self::Float(float) => format!("{float}"),
            Self::String(string) => format!("\"{string}\""),
            Self::Null => String::new(),
        }
    }
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<&str> for Variant {
    fn from(value: &str) -> Self {
        if value.is_empty() {
            return Variant::Null;
        }

        if let Ok(integer) = value.parse::<i32>() {
            Variant::Int(integer)
        } else if let Ok(float) = value.parse::<f32>() {
            Variant::Float(float)
        } else if value.is_empty() {
            Variant::Null
        } else {
            Variant::String(value.trim_matches('"').to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_integer() {
        let value = Variant::from("0");
        assert_eq!(value, Variant::Int(0));
    }

    #[test]
    fn parse_float() {
        let value = Variant::from("0.0");
        assert_eq!(value, Variant::Float(0.0));
    }

    #[test]
    fn parse_string() {
        let value = Variant::from("\"This is a string\"");
        assert_eq!(value, Variant::String("This is a string".to_owned()));
    }

    #[test]
    fn serialize() {
        assert_eq!(Variant::Int(0).to_string(), "0".to_owned());
        assert_eq!(Variant::Float(0.5).to_string(), "0.5".to_owned());
        assert_eq!(
            Variant::String("string".to_owned()).to_string(),
            "\"string\"".to_owned()
        );
        assert_eq!(Variant::Null.to_string(), String::new());
    }
}
