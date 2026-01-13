use ratatui::style::{Color as RatColor, Modifier, Style as RatStyle};
use ratatui::text::{Line as RatLine, Span, Text};

use crate::micronaut::ast::*;
use crate::micronaut::browser::{RenderOutput, Renderer};
use crate::micronaut::types::{FormState, Hitbox, Interactable};

const SECTION_INDENT: u16 = 2;
const DEFAULT_FIELD_WIDTH: u16 = 24;

#[derive(Debug, Clone, Default)]
pub struct RatatuiRenderer;

impl Renderer for RatatuiRenderer {
    type Output = Text<'static>;

    fn render(
        &self,
        doc: &Document,
        width: u16,
        form_state: &FormState,
        selected_interactable: Option<usize>,
    ) -> RenderOutput<Self::Output> {
        render_document(doc, width, form_state, selected_interactable)
    }
}

fn render_document(
    doc: &Document,
    width: u16,
    form_state: &FormState,
    selected_interactable: Option<usize>,
) -> RenderOutput<Text<'static>> {
    let mut lines: Vec<RatLine> = Vec::new();
    let mut hitboxes: Vec<Hitbox> = Vec::new();
    let mut interactable_idx = 0usize;

    for line in &doc.lines {
        let row = lines.len();
        let (rendered, mut hits) = render_line_with_hitboxes(
            line,
            row,
            width,
            form_state,
            selected_interactable,
            &mut interactable_idx,
        );
        lines.extend(rendered);
        hitboxes.append(&mut hits);
    }

    RenderOutput {
        height: lines.len() as u16,
        content: Text::from(lines),
        hitboxes,
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

fn render_line_with_hitboxes(
    line: &Line,
    row: usize,
    width: u16,
    form_state: &FormState,
    selected_interactable: Option<usize>,
    interactable_idx: &mut usize,
) -> (Vec<RatLine<'static>>, Vec<Hitbox>) {
    match line.kind {
        LineKind::Comment => (vec![], vec![]),
        LineKind::Divider(ch) => (render_divider(ch, line.indent_depth, width), vec![]),
        LineKind::Heading(level) => (render_heading(line, level, width), vec![]),
        LineKind::Normal => render_normal_with_hitboxes(
            line,
            row,
            width,
            form_state,
            selected_interactable,
            interactable_idx,
        ),
    }
}

fn render_divider(ch: char, depth: u8, width: u16) -> Vec<RatLine<'static>> {
    let indent = depth.saturating_sub(1) as u16 * SECTION_INDENT;
    let div_width = width.saturating_sub(indent) as usize;
    let divider: String = std::iter::repeat_n(ch, div_width).collect();

    let mut spans = Vec::new();
    if indent > 0 {
        spans.push(Span::raw(" ".repeat(indent as usize)));
    }
    spans.push(Span::raw(divider));

    vec![RatLine::from(spans)]
}

fn render_heading(line: &Line, level: u8, width: u16) -> Vec<RatLine<'static>> {
    let indent = line.indent_depth.saturating_sub(1) as u16 * SECTION_INDENT;
    let content_width = width.saturating_sub(indent) as usize;
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
    interactable: Option<Interactable>,
}

fn render_normal_with_hitboxes(
    line: &Line,
    row: usize,
    width: u16,
    form_state: &FormState,
    selected_interactable: Option<usize>,
    interactable_idx: &mut usize,
) -> (Vec<RatLine<'static>>, Vec<Hitbox>) {
    let indent = line.indent_depth.saturating_sub(1) as u16 * SECTION_INDENT;
    let content_width = (width as usize).saturating_sub(indent as usize);

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
                    interactable: None,
                });
            }
            Element::Link(link) => {
                let selected = selected_interactable == Some(*interactable_idx);
                *interactable_idx += 1;
                let mut style = convert_style(&link.style);
                style = style.add_modifier(Modifier::UNDERLINED);
                if selected {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                wrapped_spans.push(WrappedSpan {
                    text: link.label.clone(),
                    style,
                    interactable: Some(Interactable::Link {
                        url: link.url.clone(),
                        fields: link.fields.clone(),
                    }),
                });
            }
            Element::Field(field) => {
                let selected = selected_interactable == Some(*interactable_idx);
                *interactable_idx += 1;
                let span = render_field(field, form_state, selected);
                let interactable = match &field.kind {
                    FieldKind::Text => Interactable::TextField {
                        name: field.name.clone(),
                        masked: field.masked,
                        default: field.default.clone(),
                    },
                    FieldKind::Checkbox { .. } => Interactable::Checkbox {
                        name: field.name.clone(),
                    },
                    FieldKind::Radio { value, .. } => Interactable::Radio {
                        name: field.name.clone(),
                        value: value.clone(),
                    },
                };
                wrapped_spans.push(WrappedSpan {
                    text: span.content.to_string(),
                    style: span.style,
                    interactable: Some(interactable),
                });
            }
            Element::Partial(partial) => {
                wrapped_spans.push(WrappedSpan {
                    text: format!("[partial:{}]", partial.url),
                    style: RatStyle::default(),
                    interactable: None,
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

            if let Some(ref interactable) = ws.interactable {
                hitboxes.push(Hitbox {
                    line: current_row,
                    col_start: current_col + indent as usize,
                    col_end: current_col + indent as usize + chunk_len,
                    interactable: interactable.clone(),
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

fn render_field(field: &Field, form_state: &FormState, selected: bool) -> Span<'static> {
    let width = field.width.unwrap_or(DEFAULT_FIELD_WIDTH) as usize;
    let mut style = RatStyle::default().fg(RatColor::Black).bg(RatColor::White);
    if selected {
        style = style.add_modifier(Modifier::REVERSED);
    }

    match &field.kind {
        FieldKind::Text => {
            let value = form_state
                .fields
                .get(&field.name)
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
            let is_checked = form_state
                .checkboxes
                .get(&field.name)
                .copied()
                .unwrap_or(*checked);

            let display = if is_checked { "[X]" } else { "[ ]" };
            Span::styled(display.to_string(), style)
        }
        FieldKind::Radio { value, checked } => {
            let is_checked = form_state
                .radios
                .get(&field.name)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::micronaut::parse;

    fn render(doc: &Document, width: u16) -> Text<'static> {
        render_document(doc, width, &FormState::default(), None).content
    }

    #[test]
    fn test_render_plain_text() {
        let doc = parse("Hello world");
        let text = render(&doc, 80);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_heading() {
        let doc = parse(">Heading 1");
        let text = render(&doc, 80);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_divider() {
        let doc = parse("-");
        let text = render(&doc, 10);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_styled_text() {
        let doc = parse("`!bold`* and `_underline");
        let text = render(&doc, 80);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_comment_excluded() {
        let doc = parse("# this is a comment\nvisible");
        let text = render(&doc, 80);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_link() {
        let doc = parse("`[Click here`http://example.com]");
        let text = render(&doc, 80);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_field() {
        let doc = parse("`<name`Enter name>");
        let text = render(&doc, 80);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_checkbox() {
        let doc = parse("`<!agree`I agree>");
        let text = render(&doc, 80);
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_indented() {
        let doc = parse(">\n>>\ntext");
        let text = render(&doc, 80);
        assert!(text.lines.len() >= 1);
    }

    #[test]
    fn test_hitbox_positions_simple() {
        let doc = parse("Hello `[Link`http://x]");
        let output = render_document(&doc, 80, &FormState::default(), None);
        assert_eq!(output.hitboxes.len(), 1);
        let hb = &output.hitboxes[0];
        assert_eq!(hb.line, 0);
        assert_eq!(hb.col_start, 6);
        assert_eq!(hb.col_end, 10);
    }

    #[test]
    fn test_hitbox_wrapped_link() {
        let doc = parse("Some text `[Click here now`http://x]");
        let output = render_document(&doc, 20, &FormState::default(), None);

        assert_eq!(
            output.hitboxes.len(),
            2,
            "Expected 2 hitboxes for wrapped link"
        );

        assert_eq!(output.hitboxes[0].line, 0);
        assert_eq!(output.hitboxes[0].col_start, 10);

        assert_eq!(output.hitboxes[1].line, 1);
        assert_eq!(output.hitboxes[1].col_start, 0);
    }
}
