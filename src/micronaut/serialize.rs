use std::fmt::{self, Write};

use crate::{
    Alignment, Color, Document, Element, Field, FieldKind, Line, LineKind, LinkElement, Partial,
    Style, StyledText,
};

#[derive(Default)]
struct SerializeState {
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
    italic: bool,
    underline: bool,
    alignment: Alignment,
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = SerializeState::default();
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                f.write_char('\n')?;
            }
            serialize_line(line, &mut state, f)?;
        }
        Ok(())
    }
}

fn serialize_line(
    line: &Line,
    state: &mut SerializeState,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match line.kind {
        LineKind::Normal => {}
        LineKind::Heading(level) => {
            for _ in 0..level {
                f.write_char('>')?;
            }
        }
        LineKind::Divider(ch) => {
            f.write_char('-')?;
            if ch != '\u{2500}' {
                f.write_char(ch)?;
            }
            return Ok(());
        }
        LineKind::Comment => {
            f.write_char('#')?;
            if let Some(Element::Text(text)) = line.elements.first() {
                f.write_str(&text.text)?;
            }
            return Ok(());
        }
    }

    if line.alignment != state.alignment {
        state.alignment = line.alignment;
        match line.alignment {
            Alignment::Left => f.write_str("`a")?,
            Alignment::Center => f.write_str("`c")?,
            Alignment::Right => f.write_str("`r")?,
        }
    }

    for element in &line.elements {
        serialize_element(element, state, f)?;
    }

    Ok(())
}

fn serialize_element(
    element: &Element,
    state: &mut SerializeState,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match element {
        Element::Text(text) => serialize_styled_text(text, state, f),
        Element::Link(link) => serialize_link(link, state, f),
        Element::Field(field) => serialize_field(field, f),
        Element::Partial(partial) => serialize_partial(partial, f),
    }
}

fn serialize_styled_text(
    text: &StyledText,
    state: &mut SerializeState,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    emit_style_changes(&text.style, state, f)?;
    escape_text(&text.text, f)
}

fn emit_style_changes(
    target: &Style,
    state: &mut SerializeState,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    if state.bold != target.bold {
        f.write_str("`!")?;
        state.bold = target.bold;
    }
    if state.italic != target.italic {
        f.write_str("`*")?;
        state.italic = target.italic;
    }
    if state.underline != target.underline {
        f.write_str("`_")?;
        state.underline = target.underline;
    }
    if state.fg != target.fg {
        match target.fg {
            Some(color) => {
                f.write_str("`F")?;
                write_color(color, f)?;
            }
            None => f.write_str("`f")?,
        }
        state.fg = target.fg;
    }
    if state.bg != target.bg {
        match target.bg {
            Some(color) => {
                f.write_str("`B")?;
                write_color(color, f)?;
            }
            None => f.write_str("`b")?,
        }
        state.bg = target.bg;
    }
    Ok(())
}

fn write_color(color: Color, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if color.r == color.g && color.g == color.b {
        let pct = (color.r as u32 * 99 + 127) / 255;
        write!(f, "g{:02}", pct)
    } else {
        let r = (color.r as u32 + 8) / 17;
        let g = (color.g as u32 + 8) / 17;
        let b = (color.b as u32 + 8) / 17;
        write!(f, "{:x}{:x}{:x}", r, g, b)
    }
}

fn escape_text(text: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for ch in text.chars() {
        match ch {
            '\\' => f.write_str("\\\\")?,
            '`' => f.write_str("\\`")?,
            _ => f.write_char(ch)?,
        }
    }
    Ok(())
}

fn serialize_link(
    link: &LinkElement,
    state: &mut SerializeState,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    emit_style_changes(&link.style, state, f)?;

    f.write_str("`[")?;
    if link.label != link.url {
        escape_text(&link.label, f)?;
        f.write_char('`')?;
    }
    escape_text(&link.url, f)?;
    if !link.fields.is_empty() {
        f.write_char('`')?;
        for (i, field) in link.fields.iter().enumerate() {
            if i > 0 {
                f.write_char('|')?;
            }
            f.write_str(field)?;
        }
    }
    f.write_char(']')
}

fn serialize_field(field: &Field, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("`<")?;

    match &field.kind {
        FieldKind::Text => {
            if field.masked {
                f.write_char('!')?;
            }
            if let Some(width) = field.width {
                write!(f, "{}|", width)?;
            }
            f.write_str(&field.name)?;
            if !field.default.is_empty() {
                f.write_char('`')?;
                f.write_str(&field.default)?;
            }
        }
        FieldKind::Checkbox { checked } => {
            f.write_str("?|")?;
            f.write_str(&field.name)?;
            f.write_char('|')?;
            f.write_str(&field.default)?;
            if *checked {
                f.write_str("|*")?;
            }
        }
        FieldKind::Radio { value, checked } => {
            f.write_str("^|")?;
            f.write_str(&field.name)?;
            f.write_char('|')?;
            f.write_str(value)?;
            if *checked {
                f.write_str("|*")?;
            }
            if !field.default.is_empty() {
                f.write_char('`')?;
                f.write_str(&field.default)?;
            }
        }
    }

    f.write_char('>')
}

fn serialize_partial(partial: &Partial, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("`{")?;
    f.write_str(&partial.url)?;
    if partial.refresh.is_some() || !partial.fields.is_empty() {
        f.write_char('`')?;
        if let Some(refresh) = partial.refresh {
            write!(f, "{}", refresh)?;
        }
        if !partial.fields.is_empty() {
            f.write_char('`')?;
            for (i, field) in partial.fields.iter().enumerate() {
                if i > 0 {
                    f.write_char('|')?;
                }
                f.write_str(field)?;
            }
        }
    }
    f.write_char('}')
}

#[cfg(test)]
mod tests {
    #[test]
    fn roundtrip_simple() {
        let input = "Hello world";
        let doc = crate::parse(input);
        assert_eq!(doc.to_string(), input);
    }

    #[test]
    fn roundtrip_heading() {
        let input = ">Heading 1";
        let doc = crate::parse(input);
        assert_eq!(doc.to_string(), input);
    }

    #[test]
    fn roundtrip_bold() {
        let input = "`!bold`! normal";
        let doc = crate::parse(input);
        assert_eq!(doc.to_string(), input);
    }

    #[test]
    fn roundtrip_link() {
        let input = "`[click here`https://example.com]";
        let doc = crate::parse(input);
        assert_eq!(doc.to_string(), input);
    }

    #[test]
    fn roundtrip_escape() {
        let input = "backtick: \\` backslash: \\\\";
        let doc = crate::parse(input);
        assert_eq!(doc.to_string(), input);
    }

    #[test]
    fn roundtrip_divider() {
        let input = "-";
        let doc = crate::parse(input);
        assert_eq!(doc.to_string(), input);
    }

    #[test]
    fn roundtrip_multiline() {
        let input = ">Title\nsome text\n-\nmore text";
        let doc = crate::parse(input);
        assert_eq!(doc.to_string(), input);
    }
}
