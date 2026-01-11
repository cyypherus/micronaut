# Micron AST Design

## Core Types

```rust
pub struct Document {
    pub lines: Vec<Line>,
}

pub struct Line {
    pub kind: LineKind,
    pub indent_depth: u8,
    pub alignment: Alignment,
    pub elements: Vec<Element>,
}

pub enum LineKind {
    Normal,
    Heading(u8),      // 1-3
    Divider(char),
    Comment,
}

pub enum Alignment {
    Left,
    Center,
    Right,
}

pub enum Element {
    Text(StyledText),
    Link(Link),
    Field(Field),
    Partial(Partial),
}

pub struct StyledText {
    pub text: String,
    pub style: Style,
}

pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Link {
    pub label: String,
    pub url: String,
    pub fields: Vec<String>,
    pub style: Style,
}

pub struct Field {
    pub name: String,
    pub default: String,
    pub width: u16,
    pub masked: bool,
    pub kind: FieldKind,
}

pub enum FieldKind {
    Text,
    Checkbox { checked: bool },
    Radio { value: String, checked: bool },
}

pub struct Partial {
    pub url: String,
    pub refresh: Option<f64>,
    pub fields: Vec<String>,
}
```

## Parsing State

Parser must track:
```rust
struct ParseState {
    literal_mode: bool,
    depth: u8,
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
    italic: bool,
    underline: bool,
    alignment: Alignment,
    default_fg: Option<Color>,
    default_bg: Option<Color>,
}
```

State persists across lines within a document.
