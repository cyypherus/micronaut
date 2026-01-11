#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub kind: LineKind,
    pub indent_depth: u8,
    pub alignment: Alignment,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Normal,
    Heading(u8),
    Divider(char),
    Comment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Text(StyledText),
    Link(Link),
    Field(Field),
    Partial(Partial),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyledText {
    pub text: String,
    pub style: Style,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub label: String,
    pub url: String,
    pub fields: Vec<String>,
    pub style: Style,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub default: String,
    pub width: Option<u16>,
    pub masked: bool,
    pub kind: FieldKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Checkbox { checked: bool },
    Radio { value: String, checked: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Partial {
    pub url: String,
    pub refresh: Option<u32>,
    pub fields: Vec<String>,
}
