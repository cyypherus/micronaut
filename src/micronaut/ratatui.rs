use ratatui::style::{Color as RatColor, Modifier, Style as RatStyle};
use ratatui::text::{Line as RatLine, Span, Text};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use crate::micronaut::ast::*;
use crate::micronaut::browser::{RenderOutput, Renderer};
use crate::micronaut::types::{FormState, Hitbox, Interactable};

const SECTION_INDENT: u16 = 2;
const DEFAULT_FIELD_WIDTH: u16 = 24;

#[derive(Debug, Clone, Default)]
pub struct RatatuiRenderer;

impl Renderer for RatatuiRenderer {
    type Output = Paragraph<'static>;

    fn render(
        &self,
        doc: &Document,
        width: u16,
        scroll: u16,
        form_state: &FormState,
        selected_interactable: Option<usize>,
    ) -> RenderOutput<Self::Output> {
        render_document(doc, width, scroll, form_state, selected_interactable)
    }
}

fn render_document(
    doc: &Document,
    width: u16,
    scroll: u16,
    form_state: &FormState,
    selected_interactable: Option<usize>,
) -> RenderOutput<Paragraph<'static>> {
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
        content: Paragraph::new(Text::from(lines)).scroll((scroll, 0)),
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
    interactable: Option<(usize, Interactable)>,
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
                let idx = *interactable_idx;
                let selected = selected_interactable == Some(idx);
                *interactable_idx += 1;
                let mut style = convert_style(&link.style);
                style = style.add_modifier(Modifier::UNDERLINED);
                if selected {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                wrapped_spans.push(WrappedSpan {
                    text: link.label.clone(),
                    style,
                    interactable: Some((
                        idx,
                        Interactable::Link {
                            url: link.url.clone(),
                            fields: link.fields.clone(),
                        },
                    )),
                });
            }
            Element::Field(field) => {
                let idx = *interactable_idx;
                let selected = selected_interactable == Some(idx);
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
                    interactable: Some((idx, interactable)),
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

            let (chunk, chunk_width) =
                take_by_width(&chars[char_idx..], remaining_width);
            let chars_taken = chunk.chars().count();

            if let Some((idx, ref interactable)) = ws.interactable {
                hitboxes.push(Hitbox {
                    line: current_row,
                    col_start: current_col + indent as usize,
                    col_end: current_col + indent as usize + chunk_width,
                    interactable: interactable.clone(),
                    interactable_idx: idx,
                });
            }

            current_line_spans.push(Span::styled(chunk, ws.style));
            current_col += chunk_width;
            char_idx += chars_taken;
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

fn take_by_width(chars: &[char], max_width: usize) -> (String, usize) {
    use unicode_width::UnicodeWidthChar;
    let mut result = String::new();
    let mut width = 0;
    for &ch in chars {
        let ch_width = ch.width().unwrap_or(0);
        if width + ch_width > max_width {
            break;
        }
        result.push(ch);
        width += ch_width;
    }
    (result, width)
}

fn pad_to_width(text: &str, width: usize, alignment: Alignment) -> String {
    let text_width = text.width();
    if text_width >= width {
        return text.to_string();
    }
    let padding = width - text_width;
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

    fn render(doc: &Document, width: u16, scroll: u16) -> Paragraph<'static> {
        render_document(doc, width, scroll, &FormState::default(), None).content
    }

    #[test]
    fn test_hitbox_positions_simple() {
        let doc = parse("Hello `[Link`http://x]");
        let output = render_document(&doc, 80, 0, &FormState::default(), None);
        assert_eq!(output.hitboxes.len(), 1);
        let hb = &output.hitboxes[0];
        assert_eq!(hb.line, 0);
        assert_eq!(hb.col_start, 6);
        assert_eq!(hb.col_end, 10);
    }

    #[test]
    fn test_hitbox_wrapped_link() {
        let doc = parse("Some text `[Click here now`http://x]");
        let output = render_document(&doc, 18, 0, &FormState::default(), None);
        assert_eq!(
            output.hitboxes.len(),
            2,
            "Expected 2 hitboxes for wrapped link"
        );

        assert_eq!(output.hitboxes[0].line, 0);
        assert_eq!(output.hitboxes[0].col_start, 10);
        assert_eq!(output.hitboxes[0].col_end, 18);

        assert_eq!(output.hitboxes[1].line, 1);
        assert_eq!(output.hitboxes[1].col_start, 0);
        assert_eq!(output.hitboxes[1].col_end, 6);
    }

    #[test]
    fn test_hitbox_after_emoji() {
        let doc = parse("🦀 `[Go`http://x]");
        let output = render_document(&doc, 80, 0, &FormState::default(), None);
        assert_eq!(output.hitboxes.len(), 1);
        let hb = &output.hitboxes[0];
        assert_eq!(hb.col_start, 3, "emoji is 2 cols wide + 1 space = col 3");
        assert_eq!(hb.col_end, 5, "Go is 2 chars wide");
    }

    #[test]
    fn test_hitbox_link_starts_on_wrapped_line() {
        let doc = parse("0123456789`[Link`http://x]");
        let output = render_document(&doc, 10, 0, &FormState::default(), None);

        assert_eq!(output.height, 2, "Should be 2 lines");
        assert_eq!(output.hitboxes.len(), 1);
        let hb = &output.hitboxes[0];
        assert_eq!(hb.line, 1, "Link should be on line 1 (after wrap)");
        assert_eq!(hb.col_start, 0);
        assert_eq!(hb.col_end, 4);
    }

    #[test]
    fn test_hitbox_link_wraps_at_exact_boundary() {
        let doc = parse("12345`[ABCDE`http://x]");
        let output = render_document(&doc, 10, 0, &FormState::default(), None);

        assert_eq!(output.height, 1, "Should be 1 line (exactly 10 chars)");
        assert_eq!(output.hitboxes.len(), 1, "Link fits on first line, no wrap");
        let hb = &output.hitboxes[0];
        assert_eq!(hb.line, 0);
        assert_eq!(hb.col_start, 5);
        assert_eq!(hb.col_end, 10);
    }

    #[test]
    fn test_hitbox_link_wraps_one_char_over() {
        let doc = parse("12345`[ABCDEF`http://x]");
        let output = render_document(&doc, 10, 0, &FormState::default(), None);

        assert_eq!(output.height, 2, "Should be 2 lines");
        assert_eq!(
            output.hitboxes.len(),
            2,
            "Link wraps, should have 2 hitboxes"
        );

        assert_eq!(output.hitboxes[0].line, 0);
        assert_eq!(output.hitboxes[0].col_start, 5);
        assert_eq!(output.hitboxes[0].col_end, 10);

        assert_eq!(output.hitboxes[1].line, 1);
        assert_eq!(output.hitboxes[1].col_start, 0);
        assert_eq!(output.hitboxes[1].col_end, 1);
    }

    #[test]
    fn test_hitbox_multiple_lines_before_link() {
        let doc = parse("Line one here.\nSecond line. `[Link`http://x]");
        let output = render_document(&doc, 80, 0, &FormState::default(), None);

        assert_eq!(output.hitboxes.len(), 1);
        let hb = &output.hitboxes[0];
        assert_eq!(hb.line, 1, "Link should be on second document line");
        assert_eq!(hb.col_start, 13);
        assert_eq!(hb.col_end, 17);
    }

    #[test]
    fn test_hitbox_wrapped_text_then_link() {
        let doc = parse("This is a long line of text `[Link`http://x]");
        let output = render_document(&doc, 15, 0, &FormState::default(), None);

        println!("Height: {}", output.height);
        for (i, hb) in output.hitboxes.iter().enumerate() {
            println!(
                "Hitbox {}: line={}, col_start={}, col_end={}",
                i, hb.line, hb.col_start, hb.col_end
            );
        }

        assert!(output.hitboxes.len() >= 1);
        let last_hb = output.hitboxes.last().unwrap();
        assert!(
            last_hb.col_end <= 15,
            "Hitbox should not exceed line width"
        );
    }

    #[test]
    fn test_hitbox_second_line_wrapped_link() {
        let doc = parse("First line\nSome text `[Click here`http://x]");
        let output = render_document(&doc, 14, 0, &FormState::default(), None);

        println!("Height: {}", output.height);
        for (i, hb) in output.hitboxes.iter().enumerate() {
            println!(
                "Hitbox {}: line={}, col_start={}, col_end={}",
                i, hb.line, hb.col_start, hb.col_end
            );
        }

        assert_eq!(output.hitboxes.len(), 2, "Link should wrap into 2 hitboxes");

        assert_eq!(
            output.hitboxes[0].line, 1,
            "First part of link on rendered line 1"
        );
        assert_eq!(output.hitboxes[0].col_start, 10);
        assert_eq!(output.hitboxes[0].col_end, 14);

        assert_eq!(
            output.hitboxes[1].line, 2,
            "Second part of link on rendered line 2"
        );
        assert_eq!(output.hitboxes[1].col_start, 0);
        assert_eq!(output.hitboxes[1].col_end, 6);
    }
}
