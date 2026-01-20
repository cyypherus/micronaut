use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct FormState {
    pub fields: HashMap<String, String>,
    pub checkboxes: HashMap<String, bool>,
    pub radios: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Hitbox {
    pub line: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub interactable: Interactable,
    pub interactable_idx: usize,
}

#[derive(Debug, Clone)]
pub enum Interactable {
    Link {
        url: String,
        fields: Vec<String>,
    },
    TextField {
        name: String,
        masked: bool,
        default: String,
    },
    Checkbox {
        name: String,
    },
    Radio {
        name: String,
        value: String,
    },
}

#[derive(Debug, Clone)]
pub struct Link {
    pub url: String,
    pub fields: Vec<String>,
    pub form_data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TextField {
    pub name: String,
    pub value: String,
    pub masked: bool,
}

#[derive(Debug, Clone)]
pub enum Interaction {
    Link(Link),
    EditField(TextField),
}
