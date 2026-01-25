use crate::{
    Alignment, Color, Document, Element, Field, FieldKind, Line, LineKind, LinkElement, Partial,
    Style, StyledText,
};

impl Document {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn push(&mut self, line: Line) {
        self.lines.push(line);
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Line {
    pub fn new(kind: LineKind) -> Self {
        Self {
            kind,
            indent_depth: 0,
            alignment: Alignment::Left,
            elements: Vec::new(),
        }
    }

    pub fn normal() -> Self {
        Self::new(LineKind::Normal)
    }

    pub fn heading(level: u8) -> Self {
        Self::new(LineKind::Heading(level.clamp(1, 3)))
    }

    pub fn divider() -> Self {
        Self::new(LineKind::Divider('\u{2500}'))
    }

    pub fn divider_char(ch: char) -> Self {
        Self::new(LineKind::Divider(ch))
    }

    pub fn comment() -> Self {
        Self::new(LineKind::Comment)
    }

    pub fn indent(mut self, depth: u8) -> Self {
        self.indent_depth = depth.min(3);
        self
    }

    pub fn align(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn center(self) -> Self {
        self.align(Alignment::Center)
    }

    pub fn right(self) -> Self {
        self.align(Alignment::Right)
    }

    pub fn text(mut self, s: &str) -> Self {
        self.elements.push(Element::Text(StyledText {
            text: s.to_string(),
            style: Style::default(),
        }));
        self
    }

    pub fn styled(mut self, s: &str, style: Style) -> Self {
        self.elements.push(Element::Text(StyledText {
            text: s.to_string(),
            style,
        }));
        self
    }

    pub fn bold(mut self, s: &str) -> Self {
        self.elements.push(Element::Text(StyledText {
            text: s.to_string(),
            style: Style {
                bold: true,
                ..Default::default()
            },
        }));
        self
    }

    pub fn italic(mut self, s: &str) -> Self {
        self.elements.push(Element::Text(StyledText {
            text: s.to_string(),
            style: Style {
                italic: true,
                ..Default::default()
            },
        }));
        self
    }

    pub fn underline(mut self, s: &str) -> Self {
        self.elements.push(Element::Text(StyledText {
            text: s.to_string(),
            style: Style {
                underline: true,
                ..Default::default()
            },
        }));
        self
    }

    pub fn link(mut self, link: LinkElement) -> Self {
        self.elements.push(Element::Link(link));
        self
    }

    pub fn field(mut self, field: Field) -> Self {
        self.elements.push(Element::Field(field));
        self
    }

    pub fn partial(mut self, partial: Partial) -> Self {
        self.elements.push(Element::Partial(partial));
        self
    }

    pub fn element(mut self, element: Element) -> Self {
        self.elements.push(element);
        self
    }
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn gray(pct: u8) -> Self {
        let v = (pct.min(99) as u32 * 255 / 99) as u8;
        Self { r: v, g: v, b: v }
    }

    pub fn hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
        }
    }
}

impl LinkElement {
    pub fn new(url: impl Into<String>) -> Self {
        let url = url.into();
        Self {
            label: url.clone(),
            url,
            fields: Vec::new(),
            style: Style::default(),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn field(mut self, name: impl Into<String>) -> Self {
        self.fields.push(name.into());
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Field {
    pub fn text(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: String::new(),
            width: None,
            masked: false,
            kind: FieldKind::Text,
        }
    }

    pub fn password(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: String::new(),
            width: None,
            masked: true,
            kind: FieldKind::Text,
        }
    }

    pub fn checkbox(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: value.into(),
            width: None,
            masked: false,
            kind: FieldKind::Checkbox { checked: false },
        }
    }

    pub fn radio(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: String::new(),
            width: None,
            masked: false,
            kind: FieldKind::Radio {
                value: value.into(),
                checked: false,
            },
        }
    }

    pub fn default(mut self, default: impl Into<String>) -> Self {
        self.default = default.into();
        self
    }

    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    pub fn checked(mut self) -> Self {
        match &mut self.kind {
            FieldKind::Checkbox { checked } => *checked = true,
            FieldKind::Radio { checked, .. } => *checked = true,
            FieldKind::Text => {}
        }
        self
    }
}

impl Partial {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            refresh: None,
            fields: Vec::new(),
        }
    }

    pub fn refresh(mut self, seconds: u32) -> Self {
        self.refresh = Some(seconds);
        self
    }

    pub fn field(mut self, name: impl Into<String>) -> Self {
        self.fields.push(name.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_simple_doc() {
        let mut doc = Document::new();
        doc.push(Line::heading(1).text("Hello World"));
        doc.push(Line::normal().text("Some ").bold("bold").text(" text"));
        doc.push(Line::divider());

        assert_eq!(doc.to_string(), ">Hello World\nSome `!bold`! text\n-");
    }

    #[test]
    fn build_link() {
        let mut doc = Document::new();
        doc.push(
            Line::normal()
                .text("Click ")
                .link(LinkElement::new("https://example.com").label("here")),
        );

        assert_eq!(doc.to_string(), "Click `[here`https://example.com]");
    }

    #[test]
    fn build_with_color() {
        let mut doc = Document::new();
        doc.push(Line::normal().styled("red text", Style::new().fg(Color::hex(0xFF0000))));

        assert_eq!(doc.to_string(), "`Ff00red text");
    }

    #[test]
    fn build_field() {
        let mut doc = Document::new();
        doc.push(
            Line::normal()
                .text("Name: ")
                .field(Field::text("username").width(20).default("guest")),
        );

        assert_eq!(doc.to_string(), "Name: `<20|username`guest>");
    }
}
