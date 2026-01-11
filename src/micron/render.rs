use ratatui::style::{Color as RatColor, Modifier, Style as RatStyle};
use ratatui::text::{Line as RatLine, Span, Text};
use std::collections::HashMap;

use crate::micron::ast::*;

const SECTION_INDENT: u16 = 2;

pub struct RenderConfig<'a> {
    pub width: u16,
    pub default_field_width: u16,
    pub form_state: Option<&'a FormState>,
}

impl Default for RenderConfig<'_> {
    fn default() -> Self {
        Self {
            width: 80,
            default_field_width: 24,
            form_state: None,
        }
    }
}

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
    pub target: HitboxTarget,
}

#[derive(Debug, Clone)]
pub enum HitboxTarget {
    Link { url: String },
    TextField { name: String, masked: bool },
    Checkbox { name: String },
    Radio { name: String, value: String },
}

pub struct RenderOutput {
    pub text: Text<'static>,
    pub hitboxes: Vec<Hitbox>,
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
    render_with_hitboxes(doc, config).text
}

pub fn render_with_hitboxes(doc: &Document, config: &RenderConfig) -> RenderOutput {
    let mut lines: Vec<RatLine> = Vec::new();
    let mut hitboxes: Vec<Hitbox> = Vec::new();

    for line in &doc.lines {
        let row = lines.len();
        let (rendered, mut hits) = render_line_with_hitboxes(line, row, config);
        lines.extend(rendered);
        hitboxes.append(&mut hits);
    }

    RenderOutput {
        text: Text::from(lines),
        hitboxes,
    }
}

fn render_line_with_hitboxes(
    line: &Line,
    row: usize,
    config: &RenderConfig,
) -> (Vec<RatLine<'static>>, Vec<Hitbox>) {
    match line.kind {
        LineKind::Comment => (vec![], vec![]),
        LineKind::Divider(ch) => (render_divider(ch, line.indent_depth, config), vec![]),
        LineKind::Heading(level) => (render_heading(line, level, config), vec![]),
        LineKind::Normal => render_normal_with_hitboxes(line, row, config),
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

struct WrappedSpan {
    text: String,
    style: RatStyle,
    hitbox: Option<HitboxTarget>,
}

fn render_normal_with_hitboxes(
    line: &Line,
    row: usize,
    config: &RenderConfig,
) -> (Vec<RatLine<'static>>, Vec<Hitbox>) {
    let indent = line.indent_depth.saturating_sub(1) as u16 * SECTION_INDENT;
    let width = config.width as usize;
    let content_width = width.saturating_sub(indent as usize);

    if content_width == 0 {
        return (vec![RatLine::from("")], vec![]);
    }

    let mut wrapped_spans: Vec<WrappedSpan> = Vec::new();

    for element in &line.elements {
        match element {
            Element::Text(styled) => {
                wrapped_spans.push(WrappedSpan {
                    text: styled.text.clone(),
                    style: convert_style(&styled.style),
                    hitbox: None,
                });
            }
            Element::Link(link) => {
                let mut style = convert_style(&link.style);
                style = style.add_modifier(Modifier::UNDERLINED);
                wrapped_spans.push(WrappedSpan {
                    text: link.label.clone(),
                    style,
                    hitbox: Some(HitboxTarget::Link {
                        url: link.url.clone(),
                    }),
                });
            }
            Element::Field(field) => {
                let span = render_field(field, config);
                let target = match &field.kind {
                    FieldKind::Text => HitboxTarget::TextField {
                        name: field.name.clone(),
                        masked: field.masked,
                    },
                    FieldKind::Checkbox { .. } => HitboxTarget::Checkbox {
                        name: field.name.clone(),
                    },
                    FieldKind::Radio { value, .. } => HitboxTarget::Radio {
                        name: field.name.clone(),
                        value: value.clone(),
                    },
                };
                wrapped_spans.push(WrappedSpan {
                    text: span.content.to_string(),
                    style: span.style,
                    hitbox: Some(target),
                });
            }
            Element::Partial(partial) => {
                wrapped_spans.push(WrappedSpan {
                    text: format!("[partial:{}]", partial.url),
                    style: RatStyle::default(),
                    hitbox: None,
                });
            }
        }
    }

    let mut lines: Vec<RatLine<'static>> = Vec::new();
    let mut hitboxes: Vec<Hitbox> = Vec::new();
    let mut current_line_spans: Vec<Span<'static>> = Vec::new();
    let mut current_col = 0usize;
    let mut current_row = row;

    if indent > 0 {
        current_line_spans.push(Span::raw(" ".repeat(indent as usize)));
    }

    for ws in wrapped_spans {
        let chars: Vec<char> = ws.text.chars().collect();
        let mut char_idx = 0;

        while char_idx < chars.len() {
            let remaining_width = content_width.saturating_sub(current_col);

            if remaining_width == 0 {
                lines.push(RatLine::from(std::mem::take(&mut current_line_spans)));
                current_row += 1;
                current_col = 0;
                if indent > 0 {
                    current_line_spans.push(Span::raw(" ".repeat(indent as usize)));
                }
                continue;
            }

            let chars_left = chars.len() - char_idx;
            let take_count = std::cmp::min(remaining_width, chars_left);
            let chunk: String = chars[char_idx..char_idx + take_count].iter().collect();
            let chunk_len = chunk.chars().count();

            if let Some(ref target) = ws.hitbox {
                hitboxes.push(Hitbox {
                    line: current_row,
                    col_start: current_col + indent as usize,
                    col_end: current_col + indent as usize + chunk_len,
                    target: target.clone(),
                });
            }

            current_line_spans.push(Span::styled(chunk, ws.style));
            current_col += chunk_len;
            char_idx += take_count;
        }
    }

    if !current_line_spans.is_empty() || (current_line_spans.is_empty() && lines.is_empty()) {
        lines.push(RatLine::from(current_line_spans));
    }

    (lines, hitboxes)
}

fn render_field(field: &Field, config: &RenderConfig) -> Span<'static> {
    let width = field.width.unwrap_or(config.default_field_width) as usize;
    let style = RatStyle::default().fg(RatColor::Black).bg(RatColor::White);

    match &field.kind {
        FieldKind::Text => {
            let value = config
                .form_state
                .and_then(|s| s.fields.get(&field.name))
                .map(|s| s.as_str())
                .unwrap_or(&field.default);

            let display = if field.masked {
                "*".repeat(value.len().min(width))
            } else {
                let mut s = value.to_string();
                s.truncate(width);
                s
            };
            let padded = format!("{:<width$}", display, width = width);
            Span::styled(padded, style)
        }
        FieldKind::Checkbox { checked } => {
            let is_checked = config
                .form_state
                .and_then(|s| s.checkboxes.get(&field.name))
                .copied()
                .unwrap_or(*checked);

            let display = if is_checked { "[X]" } else { "[ ]" };
            Span::styled(display.to_string(), style)
        }
        FieldKind::Radio { value, checked } => {
            let is_checked = config
                .form_state
                .and_then(|s| s.radios.get(&field.name))
                .map(|selected| selected == value)
                .unwrap_or(*checked);

            let display = if is_checked { "(X)" } else { "( )" };
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

    #[test]
    fn test_hitboxes_link_with_fields() {
        let doc = parse("`[John 3:17`:/page/bible.mu`single_verse=true|book=John]");
        let config = RenderConfig::default();
        let output = render_with_hitboxes(&doc, &config);
        assert_eq!(output.hitboxes.len(), 1);
        if let HitboxTarget::Link { url } = &output.hitboxes[0].target {
            assert_eq!(url, ":/page/bible.mu");
        } else {
            panic!("Expected Link hitbox");
        }
    }
}
