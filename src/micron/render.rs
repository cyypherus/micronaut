use ratatui::style::{Color as RatColor, Modifier, Style as RatStyle};
use ratatui::text::{Line as RatLine, Span, Text};

use crate::micron::ast::*;

const SECTION_INDENT: u16 = 2;

pub struct RenderConfig {
    pub width: u16,
    pub default_field_width: u16,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 80,
            default_field_width: 24,
        }
    }
}

struct HeadingStyle {
    fg: RatColor,
    bg: RatColor,
}

fn heading_style(level: u8) -> HeadingStyle {
    match level {
        1 => HeadingStyle {
            fg: RatColor::Rgb(0x22, 0x22, 0x22),
            bg: RatColor::Rgb(0xbb, 0xbb, 0xbb),
        },
        2 => HeadingStyle {
            fg: RatColor::Rgb(0x11, 0x11, 0x11),
            bg: RatColor::Rgb(0x99, 0x99, 0x99),
        },
        _ => HeadingStyle {
            fg: RatColor::Rgb(0x00, 0x00, 0x00),
            bg: RatColor::Rgb(0x77, 0x77, 0x77),
        },
    }
}

fn convert_color(color: Option<Color>) -> RatColor {
    match color {
        Some(c) => RatColor::Rgb(c.r, c.g, c.b),
        None => RatColor::Reset,
    }
}

fn convert_style(style: &Style) -> RatStyle {
    let mut rat_style = RatStyle::default()
        .fg(convert_color(style.fg))
        .bg(convert_color(style.bg));

    let mut modifiers = Modifier::empty();
    if style.bold {
        modifiers |= Modifier::BOLD;
    }
    if style.italic {
        modifiers |= Modifier::ITALIC;
    }
    if style.underline {
        modifiers |= Modifier::UNDERLINED;
    }
    rat_style = rat_style.add_modifier(modifiers);
    rat_style
}

pub fn render(doc: &Document, config: &RenderConfig) -> Text<'static> {
    let lines: Vec<RatLine> = doc
        .lines
        .iter()
        .flat_map(|line| render_line(line, config))
        .collect();
    Text::from(lines)
}

fn render_line(line: &Line, config: &RenderConfig) -> Vec<RatLine<'static>> {
    match line.kind {
        LineKind::Comment => vec![],
        LineKind::Divider(ch) => render_divider(ch, line.indent_depth, config),
        LineKind::Heading(level) => render_heading(line, level, config),
        LineKind::Normal => render_normal(line, config),
    }
}

fn render_divider(ch: char, depth: u8, config: &RenderConfig) -> Vec<RatLine<'static>> {
    let indent = depth.saturating_sub(1) as u16 * SECTION_INDENT;
    let width = config.width.saturating_sub(indent) as usize;
    let divider: String = std::iter::repeat_n(ch, width).collect();

    let mut spans = Vec::new();
    if indent > 0 {
        spans.push(Span::raw(" ".repeat(indent as usize)));
    }
    spans.push(Span::raw(divider));

    vec![RatLine::from(spans)]
}

fn render_heading(line: &Line, level: u8, config: &RenderConfig) -> Vec<RatLine<'static>> {
    let indent = line.indent_depth.saturating_sub(1) as u16 * SECTION_INDENT;
    let content_width = config.width.saturating_sub(indent) as usize;
    let hs = heading_style(level);

    let text_content = collect_text(&line.elements);
    let padded = pad_to_width(&text_content, content_width, line.alignment);

    let style = RatStyle::default().fg(hs.fg).bg(hs.bg);

    let mut spans = Vec::new();
    if indent > 0 {
        spans.push(Span::raw(" ".repeat(indent as usize)));
    }
    spans.push(Span::styled(padded, style));

    vec![RatLine::from(spans)]
}

fn render_normal(line: &Line, config: &RenderConfig) -> Vec<RatLine<'static>> {
    let indent = line.indent_depth.saturating_sub(1) as u16 * SECTION_INDENT;

    let mut spans: Vec<Span<'static>> = Vec::new();

    if indent > 0 {
        spans.push(Span::raw(" ".repeat(indent as usize)));
    }

    for element in &line.elements {
        match element {
            Element::Text(styled) => {
                spans.push(Span::styled(
                    styled.text.clone(),
                    convert_style(&styled.style),
                ));
            }
            Element::Link(link) => {
                let mut style = convert_style(&link.style);
                style = style.add_modifier(Modifier::UNDERLINED);
                spans.push(Span::styled(link.label.clone(), style));
            }
            Element::Field(field) => {
                spans.push(render_field(field, config));
            }
            Element::Partial(partial) => {
                spans.push(Span::raw(format!("[partial:{}]", partial.url)));
            }
        }
    }

    if spans.is_empty() || (spans.len() == 1 && indent > 0) {
        return vec![RatLine::from(spans)];
    }

    vec![RatLine::from(spans).alignment(convert_alignment(line.alignment))]
}

fn render_field(field: &Field, config: &RenderConfig) -> Span<'static> {
    let width = field.width.unwrap_or(config.default_field_width) as usize;
    let style = RatStyle::default().fg(RatColor::Black).bg(RatColor::White);

    match &field.kind {
        FieldKind::Text => {
            let display = if field.masked {
                "*".repeat(field.default.len().min(width))
            } else {
                let mut s = field.default.clone();
                s.truncate(width);
                s
            };
            let padded = format!("{:<width$}", display, width = width);
            Span::styled(padded, style)
        }
        FieldKind::Checkbox { checked } => {
            let display = if *checked { "[X]" } else { "[ ]" };
            Span::styled(display.to_string(), style)
        }
        FieldKind::Radio { checked, .. } => {
            let display = if *checked { "(X)" } else { "( )" };
            Span::styled(display.to_string(), style)
        }
    }
}

fn collect_text(elements: &[Element]) -> String {
    elements
        .iter()
        .filter_map(|e| match e {
            Element::Text(t) => Some(t.text.as_str()),
            Element::Link(l) => Some(l.label.as_str()),
            _ => None,
        })
        .collect()
}

fn pad_to_width(text: &str, width: usize, alignment: Alignment) -> String {
    let len = text.chars().count();
    if len >= width {
        return text.to_string();
    }
    let padding = width - len;
    match alignment {
        Alignment::Left => format!("{}{}", text, " ".repeat(padding)),
        Alignment::Right => format!("{}{}", " ".repeat(padding), text),
        Alignment::Center => {
            let left = padding / 2;
            let right = padding - left;
            format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
        }
    }
}

fn convert_alignment(alignment: Alignment) -> ratatui::layout::Alignment {
    match alignment {
        Alignment::Left => ratatui::layout::Alignment::Left,
        Alignment::Center => ratatui::layout::Alignment::Center,
        Alignment::Right => ratatui::layout::Alignment::Right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::micron::parse;

    #[test]
    fn test_render_plain_text() {
        let doc = parse("Hello world");
        let config = RenderConfig::default();
        let text = render(&doc, &config);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_heading() {
        let doc = parse(">Heading 1");
        let config = RenderConfig::default();
        let text = render(&doc, &config);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_divider() {
        let doc = parse("-");
        let config = RenderConfig {
            width: 10,
            ..Default::default()
        };
        let text = render(&doc, &config);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_styled_text() {
        let doc = parse("`!bold`* and `_underline");
        let config = RenderConfig::default();
        let text = render(&doc, &config);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_comment_excluded() {
        let doc = parse("# this is a comment\nvisible");
        let config = RenderConfig::default();
        let text = render(&doc, &config);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_link() {
        let doc = parse("`[Click here`http://example.com]");
        let config = RenderConfig::default();
        let text = render(&doc, &config);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_field() {
        let doc = parse("`<name`Enter name>");
        let config = RenderConfig::default();
        let text = render(&doc, &config);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_checkbox() {
        let doc = parse("`<!agree`I agree>");
        let config = RenderConfig::default();
        let text = render(&doc, &config);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_indented() {
        let doc = parse(">\n>>\ntext");
        let config = RenderConfig::default();
        let text = render(&doc, &config);
        assert!(text.lines.len() >= 1);
    }
}
