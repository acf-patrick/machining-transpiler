use std::collections::HashMap;

use super::variant::Variant;

pub struct Tag {
    attributes: HashMap<String, Variant>,
}
