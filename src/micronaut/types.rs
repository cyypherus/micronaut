use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct FormState {
    pub fields: HashMap<String, String>,
    pub checkboxes: HashMap<String, bool>,
    pub radios: HashMap<String, String>,
    pub editing_field: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Hitbox {
    pub line: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub target: HitboxTarget,
}

#[derive(Debug, Clone)]
pub enum HitboxTarget {
    Link { url: String, fields: Vec<String> },
    TextField { name: String, masked: bool },
    Checkbox { name: String },
    Radio { name: String, value: String },
}

#[derive(Debug, Clone)]
pub struct Link {
    pub url: String,
    pub fields: Vec<String>,
    pub form_data: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputResult {
    Consumed,
    Ignored,
}
