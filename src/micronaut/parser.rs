use winnow::Parser;
use winnow::combinator::{opt, preceded};
use winnow::error::ModalResult;
use winnow::stream::Stateful;
use winnow::token::{take, take_while};

use crate::micronaut::ast::*;

type Stream<'a> = Stateful<&'a str, ParseState>;

#[derive(Debug, Clone, Default)]
struct ParseState {
    literal_mode: bool,
    depth: u8,
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
    italic: bool,
    underline: bool,
    alignment: Alignment,
    first_text_alignment: Option<Alignment>,
}

impl ParseState {
    fn current_style(&self) -> Style {
        Style {
            fg: self.fg,
            bg: self.bg,
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
        }
    }

    fn reset_style(&mut self) {
        self.fg = None;
        self.bg = None;
        self.bold = false;
        self.italic = false;
        self.underline = false;
        self.alignment = Alignment::Left;
    }
}

pub fn parse(input: &str) -> Document {
    let mut state = ParseState::default();
    let lines: Vec<Line> = input
        .lines()
        .filter_map(|line| parse_line(line, &mut state))
        .collect();
    Document { lines }
}

fn parse_line(line: &str, state: &mut ParseState) -> Option<Line> {
    let mut line = line;
    let mut pre_escape = false;

    if line == "`=" {
        state.literal_mode = !state.literal_mode;
        return None;
    }

    if !state.literal_mode {
        if line.starts_with('>') && line.contains("`<") {
            line = line.trim_start_matches('>');
        }

        if let Some(rest) = line.strip_prefix('\\') {
            line = rest;
            pre_escape = true;
        }

        if !pre_escape {
            if line.starts_with('#') {
                return Some(Line {
                    kind: LineKind::Comment,
                    indent_depth: state.depth,
                    alignment: state.alignment,
                    elements: vec![],
                });
            }

            if line.starts_with("`{") {
                let (elements, alignment) = parse_elements(line, state);
                return Some(Line {
                    kind: LineKind::Normal,
                    indent_depth: state.depth,
                    alignment,
                    elements,
                });
            }

            if let Some(rest) = line.strip_prefix('<') {
                state.depth = 0;
                let (elements, alignment) = parse_elements(rest, state);
                return Some(Line {
                    kind: LineKind::Normal,
                    indent_depth: 0,
                    alignment,
                    elements,
                });
            }

            if let Some(rest) = line.strip_prefix(">>>") {
                state.depth = 3;
                if rest.is_empty() {
                    return Some(Line {
                        kind: LineKind::Heading(3),
                        indent_depth: 3,
                        alignment: state.alignment,
                        elements: vec![],
                    });
                }
                let (elements, alignment) = parse_elements(rest, state);
                return Some(Line {
                    kind: LineKind::Heading(3),
                    indent_depth: 3,
                    alignment,
                    elements,
                });
            }

            if let Some(rest) = line.strip_prefix(">>") {
                state.depth = 2;
                if rest.is_empty() {
                    return Some(Line {
                        kind: LineKind::Heading(2),
                        indent_depth: 2,
                        alignment: state.alignment,
                        elements: vec![],
                    });
                }
                let (elements, alignment) = parse_elements(rest, state);
                return Some(Line {
                    kind: LineKind::Heading(2),
                    indent_depth: 2,
                    alignment,
                    elements,
                });
            }

            if let Some(rest) = line.strip_prefix('>') {
                state.depth = 1;
                if rest.is_empty() {
                    return Some(Line {
                        kind: LineKind::Heading(1),
                        indent_depth: 1,
                        alignment: state.alignment,
                        elements: vec![],
                    });
                }
                let (elements, alignment) = parse_elements(rest, state);
                return Some(Line {
                    kind: LineKind::Heading(1),
                    indent_depth: 1,
                    alignment,
                    elements,
                });
            }

            if let Some(rest) = line.strip_prefix('-') {
                let ch = rest.chars().next().unwrap_or('\u{2500}');
                let ch = if ch < ' ' { '\u{2500}' } else { ch };
                return Some(Line {
                    kind: LineKind::Divider(ch),
                    indent_depth: state.depth,
                    alignment: state.alignment,
                    elements: vec![],
                });
            }
        }
    }

    let (elements, alignment) = parse_elements_with_escape(line, state, pre_escape);
    Some(Line {
        kind: LineKind::Normal,
        indent_depth: state.depth,
        alignment,
        elements,
    })
}

fn parse_elements(input: &str, state: &mut ParseState) -> (Vec<Element>, Alignment) {
    parse_elements_with_escape(input, state, false)
}

fn parse_elements_with_escape(
    input: &str,
    state: &mut ParseState,
    pre_escape: bool,
) -> (Vec<Element>, Alignment) {
    let initial_alignment = state.alignment;
    state.first_text_alignment = None;

    let mut stream = Stateful {
        input,
        state: state.clone(),
    };

    let result = parse_elements_inner(&mut stream, pre_escape);
    *state = stream.state;

    let line_alignment = state.first_text_alignment.unwrap_or(initial_alignment);

    (result.unwrap_or_default(), line_alignment)
}

fn parse_elements_inner<'a>(input: &mut Stream<'a>, pre_escape: bool) -> ModalResult<Vec<Element>> {
    let mut elements = Vec::new();
    let mut text_buf = String::new();
    let mut escape = pre_escape;

    while !input.input.is_empty() {
        if input.state.literal_mode {
            if input.input == "\\`=" {
                text_buf.push_str("`=");
                let _ = take(3usize).parse_next(input)?;
                continue;
            }
            if let Some(ch) = input.input.chars().next() {
                text_buf.push(ch);
                let _ = take(1usize).parse_next(input)?;
            }
            continue;
        }

        if let Some(ch) = input.input.chars().next() {
            if ch == '\\' {
                if escape {
                    text_buf.push(ch);
                    escape = false;
                } else {
                    escape = true;
                }
                let _ = take(1usize).parse_next(input)?;
                continue;
            }

            if ch == '`' {
                if escape {
                    text_buf.push(ch);
                    escape = false;
                    let _ = take(1usize).parse_next(input)?;
                    continue;
                }

                flush_text(&mut text_buf, &mut input.state, &mut elements);

                if let Ok(elem) = parse_backtick_sequence(input) {
                    if let Some(e) = elem {
                        if input.state.first_text_alignment.is_none() {
                            input.state.first_text_alignment = Some(input.state.alignment);
                        }
                        elements.push(e);
                    }
                    continue;
                }
            }

            text_buf.push(ch);
            escape = false;
            let _ = take(1usize).parse_next(input)?;
        }
    }

    flush_text(&mut text_buf, &mut input.state, &mut elements);
    Ok(elements)
}

fn flush_text(buf: &mut String, state: &mut ParseState, elements: &mut Vec<Element>) {
    if !buf.is_empty() {
        if state.first_text_alignment.is_none() {
            state.first_text_alignment = Some(state.alignment);
        }
        elements.push(Element::Text(StyledText {
            text: std::mem::take(buf),
            style: state.current_style(),
        }));
    }
}

fn parse_backtick_sequence<'a>(input: &mut Stream<'a>) -> ModalResult<Option<Element>> {
    let _ = '`'.parse_next(input)?;

    if input.input.is_empty() {
        return Ok(None);
    }

    let next_char = input.input.chars().next().unwrap();

    match next_char {
        '!' => {
            let _ = take(1usize).parse_next(input)?;
            input.state.bold = !input.state.bold;
            Ok(None)
        }
        '*' => {
            let _ = take(1usize).parse_next(input)?;
            input.state.italic = !input.state.italic;
            Ok(None)
        }
        '_' => {
            let _ = take(1usize).parse_next(input)?;
            input.state.underline = !input.state.underline;
            Ok(None)
        }
        'F' => {
            let _ = take(1usize).parse_next(input)?;
            if input.input.len() >= 3
                && let Ok(color) = parse_color(input)
            {
                input.state.fg = Some(color);
            }
            Ok(None)
        }
        'f' => {
            let _ = take(1usize).parse_next(input)?;
            input.state.fg = None;
            Ok(None)
        }
        'B' => {
            let _ = take(1usize).parse_next(input)?;
            if input.input.len() >= 3
                && let Ok(color) = parse_color(input)
            {
                input.state.bg = Some(color);
            }
            Ok(None)
        }
        'b' => {
            let _ = take(1usize).parse_next(input)?;
            input.state.bg = None;
            Ok(None)
        }
        '`' => {
            let _ = take(1usize).parse_next(input)?;
            input.state.reset_style();
            Ok(None)
        }
        'c' => {
            let _ = take(1usize).parse_next(input)?;
            input.state.alignment = Alignment::Center;
            Ok(None)
        }
        'l' => {
            let _ = take(1usize).parse_next(input)?;
            input.state.alignment = Alignment::Left;
            Ok(None)
        }
        'r' => {
            let _ = take(1usize).parse_next(input)?;
            input.state.alignment = Alignment::Right;
            Ok(None)
        }
        'a' => {
            let _ = take(1usize).parse_next(input)?;
            input.state.alignment = Alignment::Left;
            Ok(None)
        }
        '=' => {
            let _ = take(1usize).parse_next(input)?;
            Ok(None)
        }
        '[' => {
            let _ = take(1usize).parse_next(input)?;
            let link = parse_link(input)?;
            Ok(Some(Element::Link(link)))
        }
        '<' => {
            let _ = take(1usize).parse_next(input)?;
            let field = parse_field(input)?;
            Ok(Some(Element::Field(field)))
        }
        '{' => {
            let _ = take(1usize).parse_next(input)?;
            let partial = parse_partial(input)?;
            Ok(Some(Element::Partial(partial)))
        }
        _ => Ok(None),
    }
}

fn parse_color<'a>(input: &mut Stream<'a>) -> ModalResult<Color> {
    let hex: &str = take(3usize).parse_next(input)?;

    if let Some(gray) = hex.strip_prefix('g') {
        let brightness = gray.parse::<u8>().unwrap_or(0);
        let val = (brightness as u16 * 255 / 99) as u8;
        return Ok(Color {
            r: val,
            g: val,
            b: val,
        });
    }

    let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(0) * 17;
    let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(0) * 17;
    let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(0) * 17;
    Ok(Color { r, g, b })
}

fn parse_link<'a>(input: &mut Stream<'a>) -> ModalResult<LinkElement> {
    let link_data: &str = take_while(0.., |c| c != ']').parse_next(input)?;
    let _ = ']'.parse_next(input)?;

    let components: Vec<&str> = link_data.split('`').collect();

    let (label, url, fields) = match components.len() {
        1 => ("", components[0], ""),
        2 => (components[0], components[1], ""),
        3 => (components[0], components[1], components[2]),
        _ => ("", "", ""),
    };

    let effective_label = if label.is_empty() {
        url.to_string()
    } else {
        label.to_string()
    };

    Ok(LinkElement {
        label: effective_label,
        url: url.to_string(),
        fields: if fields.is_empty() {
            vec![]
        } else {
            fields.split('|').map(String::from).collect()
        },
        style: input.state.current_style(),
    })
}

fn parse_field<'a>(input: &mut Stream<'a>) -> ModalResult<Field> {
    let masked = opt('!').parse_next(input)?.is_some();
    let is_checkbox = opt('?').parse_next(input)?.is_some();
    let is_radio = opt('^').parse_next(input)?.is_some();

    if is_checkbox || is_radio {
        let _ = opt('|').parse_next(input)?;
        let name: &str = take_while(0.., |c| c != '|').parse_next(input)?;
        let _ = '|'.parse_next(input)?;
        let value: &str = take_while(0.., |c| c != '`' && c != '|').parse_next(input)?;
        let checked = opt(preceded('|', '*')).parse_next(input)?.is_some();
        let _ = '`'.parse_next(input)?;
        let label: &str = take_while(0.., |c| c != '>').parse_next(input)?;
        let _ = '>'.parse_next(input)?;

        let effective_value = if value.is_empty() {
            label.to_string()
        } else {
            value.to_string()
        };

        return Ok(Field {
            name: name.to_string(),
            default: label.to_string(),
            width: None,
            masked: false,
            kind: if is_checkbox {
                FieldKind::Checkbox { checked }
            } else {
                FieldKind::Radio {
                    value: effective_value,
                    checked,
                }
            },
        });
    }

    let width_and_name: &str = take_while(0.., |c| c != '`').parse_next(input)?;
    let (width, name) = if let Some((w, n)) = width_and_name.split_once('|') {
        (w.parse().ok(), n)
    } else {
        (None, width_and_name)
    };

    let _ = '`'.parse_next(input)?;
    let default: &str = take_while(0.., |c| c != '>').parse_next(input)?;
    let _ = '>'.parse_next(input)?;

    Ok(Field {
        name: name.to_string(),
        default: default.to_string(),
        width,
        masked,
        kind: FieldKind::Text,
    })
}

fn parse_partial<'a>(input: &mut Stream<'a>) -> ModalResult<Partial> {
    let url: &str = take_while(0.., |c| c != '`' && c != '}').parse_next(input)?;

    let refresh = if opt('`').parse_next(input)?.is_some() {
        let r: &str = take_while(0.., |c| c != '`' && c != '}').parse_next(input)?;
        r.parse().ok()
    } else {
        None
    };

    let fields = if opt('`').parse_next(input)?.is_some() {
        let f: &str = take_while(0.., |c| c != '}').parse_next(input)?;
        f.split('|').map(String::from).collect()
    } else {
        vec![]
    };

    let _ = '}'.parse_next(input)?;

    Ok(Partial {
        url: url.to_string(),
        refresh,
        fields,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let doc = parse("Hello world");
        assert_eq!(doc.lines.len(), 1);
        assert_eq!(doc.lines[0].kind, LineKind::Normal);
        assert_eq!(doc.lines[0].elements.len(), 1);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "Hello world");
            assert_eq!(t.style, Style::default());
        } else {
            panic!("Expected Text element");
        }
    }

    #[test]
    fn test_heading_level_1() {
        let doc = parse(">Main Title");
        assert_eq!(doc.lines[0].kind, LineKind::Heading(1));
        assert_eq!(doc.lines[0].indent_depth, 1);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "Main Title");
        }
    }

    #[test]
    fn test_heading_level_2() {
        let doc = parse(">>Sub Title");
        assert_eq!(doc.lines[0].kind, LineKind::Heading(2));
        assert_eq!(doc.lines[0].indent_depth, 2);
    }

    #[test]
    fn test_heading_level_3() {
        let doc = parse(">>>Deep Title");
        assert_eq!(doc.lines[0].kind, LineKind::Heading(3));
        assert_eq!(doc.lines[0].indent_depth, 3);
    }

    #[test]
    fn test_depth_reset() {
        let doc = parse(">>Sub\n<Reset");
        assert_eq!(doc.lines[0].indent_depth, 2);
        assert_eq!(doc.lines[1].indent_depth, 0);
    }

    #[test]
    fn test_depth_persists() {
        let doc = parse(">>Sub\nNext line");
        assert_eq!(doc.lines[0].indent_depth, 2);
        assert_eq!(doc.lines[1].indent_depth, 2);
    }

    #[test]
    fn test_comment() {
        let doc = parse("# This is a comment");
        assert_eq!(doc.lines[0].kind, LineKind::Comment);
        assert!(doc.lines[0].elements.is_empty());
    }

    #[test]
    fn test_divider_default() {
        let doc = parse("-");
        assert_eq!(doc.lines[0].kind, LineKind::Divider('\u{2500}'));
    }

    #[test]
    fn test_divider_custom() {
        let doc = parse("-=");
        assert_eq!(doc.lines[0].kind, LineKind::Divider('='));
    }

    #[test]
    fn test_bold() {
        let doc = parse("This is `!bold`! text");
        assert_eq!(doc.lines[0].elements.len(), 3);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert!(!t.style.bold);
            assert_eq!(t.text, "This is ");
        }
        if let Element::Text(t) = &doc.lines[0].elements[1] {
            assert!(t.style.bold);
            assert_eq!(t.text, "bold");
        }
        if let Element::Text(t) = &doc.lines[0].elements[2] {
            assert!(!t.style.bold);
            assert_eq!(t.text, " text");
        }
    }

    #[test]
    fn test_italic() {
        let doc = parse("`*italic`*");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert!(t.style.italic);
            assert_eq!(t.text, "italic");
        }
    }

    #[test]
    fn test_underline() {
        let doc = parse("`_underlined`_");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert!(t.style.underline);
            assert_eq!(t.text, "underlined");
        }
    }

    #[test]
    fn test_foreground_color() {
        let doc = parse("`Ff00red`f");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.style.fg, Some(Color { r: 255, g: 0, b: 0 }));
            assert_eq!(t.text, "red");
        }
    }

    #[test]
    fn test_background_color() {
        let doc = parse("`Bff0yellow`b");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(
                t.style.bg,
                Some(Color {
                    r: 255,
                    g: 255,
                    b: 0
                })
            );
            assert_eq!(t.text, "yellow");
        }
    }

    #[test]
    fn test_grayscale_color() {
        let doc = parse("`Fg50gray`f");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            let c = t.style.fg.unwrap();
            assert_eq!(c.r, c.g);
            assert_eq!(c.g, c.b);
            assert!((125..=130).contains(&c.r));
        }
    }

    #[test]
    fn test_reset_all() {
        let doc = parse("`!`*`_styled`` plain");
        if let Element::Text(t) = &doc.lines[0].elements[1] {
            assert!(!t.style.bold);
            assert!(!t.style.italic);
            assert!(!t.style.underline);
            assert_eq!(t.text, " plain");
        }
    }

    #[test]
    fn test_alignment_center() {
        let doc = parse("`ccentered");
        assert_eq!(doc.lines[0].alignment, Alignment::Center);
    }

    #[test]
    fn test_alignment_right() {
        let doc = parse("`rright");
        assert_eq!(doc.lines[0].alignment, Alignment::Right);
    }

    #[test]
    fn test_alignment_left() {
        let doc = parse("`r`lleft");
        assert_eq!(doc.lines[0].alignment, Alignment::Left);
    }

    #[test]
    fn test_alignment_reset() {
        let doc = parse("`r`areset");
        assert_eq!(doc.lines[0].alignment, Alignment::Left);
    }

    #[test]
    fn test_alignment_persists() {
        let doc = parse("`ccentered\nstill centered");
        assert_eq!(doc.lines[0].alignment, Alignment::Center);
        assert_eq!(doc.lines[1].alignment, Alignment::Center);
    }

    #[test]
    fn test_link_simple() {
        let doc = parse("`[Home`/]");
        if let Element::Link(l) = &doc.lines[0].elements[0] {
            assert_eq!(l.label, "Home");
            assert_eq!(l.url, "/");
            assert!(l.fields.is_empty());
        } else {
            panic!("Expected Link");
        }
    }

    #[test]
    fn test_link_with_fields() {
        let doc = parse("`[Submit`/submit`name|email]");
        if let Element::Link(l) = &doc.lines[0].elements[0] {
            assert_eq!(l.label, "Submit");
            assert_eq!(l.url, "/submit");
            assert_eq!(l.fields, vec!["name", "email"]);
        } else {
            panic!("Expected Link");
        }
    }

    #[test]
    fn test_link_inherits_style() {
        let doc = parse("`!`[Bold Link`/]");
        if let Element::Link(l) = &doc.lines[0].elements[0] {
            assert!(l.style.bold);
        }
    }

    #[test]
    fn test_field_text_simple() {
        let doc = parse("`<name`John>");
        if let Element::Field(f) = &doc.lines[0].elements[0] {
            assert_eq!(f.name, "name");
            assert_eq!(f.default, "John");
            assert_eq!(f.kind, FieldKind::Text);
            assert!(!f.masked);
            assert!(f.width.is_none());
        } else {
            panic!("Expected Field");
        }
    }

    #[test]
    fn test_field_with_width() {
        let doc = parse("`<20|username`>");
        if let Element::Field(f) = &doc.lines[0].elements[0] {
            assert_eq!(f.name, "username");
            assert_eq!(f.width, Some(20));
        }
    }

    #[test]
    fn test_field_masked() {
        let doc = parse("`<!8|password`>");
        if let Element::Field(f) = &doc.lines[0].elements[0] {
            assert_eq!(f.name, "password");
            assert!(f.masked);
            assert_eq!(f.width, Some(8));
        }
    }

    #[test]
    fn test_field_checkbox() {
        let doc = parse("`<?|remember|yes`Keep me logged in>");
        if let Element::Field(f) = &doc.lines[0].elements[0] {
            assert_eq!(f.name, "remember");
            assert_eq!(f.default, "Keep me logged in");
            assert_eq!(f.kind, FieldKind::Checkbox { checked: false });
        }
    }

    #[test]
    fn test_field_checkbox_checked() {
        let doc = parse("`<?|accept|yes|*`I accept>");
        if let Element::Field(f) = &doc.lines[0].elements[0] {
            assert_eq!(f.kind, FieldKind::Checkbox { checked: true });
        }
    }

    #[test]
    fn test_field_radio() {
        let doc = parse("`<^|color|red`Red>");
        if let Element::Field(f) = &doc.lines[0].elements[0] {
            assert_eq!(f.name, "color");
            if let FieldKind::Radio { value, checked } = &f.kind {
                assert_eq!(value, "red");
                assert!(!checked);
            } else {
                panic!("Expected Radio");
            }
        }
    }

    #[test]
    fn test_field_radio_checked() {
        let doc = parse("`<^|color|blue|*`Blue>");
        if let Element::Field(f) = &doc.lines[0].elements[0]
            && let FieldKind::Radio { checked, .. } = &f.kind
        {
            assert!(checked);
        }
    }

    #[test]
    fn test_partial_simple() {
        let doc = parse("`{/api/status}");
        if let Element::Partial(p) = &doc.lines[0].elements[0] {
            assert_eq!(p.url, "/api/status");
            assert!(p.refresh.is_none());
            assert!(p.fields.is_empty());
        } else {
            panic!("Expected Partial");
        }
    }

    #[test]
    fn test_partial_with_refresh() {
        let doc = parse("`{/api/clock`5}");
        if let Element::Partial(p) = &doc.lines[0].elements[0] {
            assert_eq!(p.url, "/api/clock");
            assert_eq!(p.refresh, Some(5));
        }
    }

    #[test]
    fn test_partial_with_fields() {
        let doc = parse("`{/api/data`10`user_id|session}");
        if let Element::Partial(p) = &doc.lines[0].elements[0] {
            assert_eq!(p.refresh, Some(10));
            assert_eq!(p.fields, vec!["user_id", "session"]);
        }
    }

    #[test]
    fn test_literal_mode() {
        let doc = parse("`=\n`!not bold`!\n`=");
        assert_eq!(doc.lines.len(), 1);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "`!not bold`!");
            assert!(!t.style.bold);
        }
    }

    #[test]
    fn test_literal_mode_multiline() {
        let doc = parse("`=\n`!still literal\n`=\n`!now bold`!");
        assert_eq!(doc.lines.len(), 2);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "`!still literal");
        }
        if let Element::Text(t) = &doc.lines[1].elements[0] {
            assert!(t.style.bold);
        }
    }

    #[test]
    fn test_escape_backtick() {
        let doc = parse("\\`not a command");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "`not a command");
        }
    }

    #[test]
    fn test_escape_backslash() {
        let doc = parse("\\\\double");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "\\double");
        }
    }

    #[test]
    fn test_nested_formatting() {
        let doc = parse("`!`*bold and italic`*`!");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert!(t.style.bold);
            assert!(t.style.italic);
            assert_eq!(t.text, "bold and italic");
        }
    }

    #[test]
    fn test_color_rgb() {
        let doc = parse("`F0f0green`f");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.style.fg, Some(Color { r: 0, g: 255, b: 0 }));
        }
    }

    #[test]
    fn test_color_blue() {
        let doc = parse("`F00fblue`f");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.style.fg, Some(Color { r: 0, g: 0, b: 255 }));
        }
    }

    #[test]
    fn test_multiple_lines() {
        let doc = parse("line 1\nline 2\nline 3");
        assert_eq!(doc.lines.len(), 3);
    }

    #[test]
    fn test_style_persists_across_lines() {
        let doc = parse("`!bold start\nstill bold`!");
        if let Element::Text(t) = &doc.lines[1].elements[0] {
            assert!(t.style.bold);
        }
    }

    #[test]
    fn test_mixed_content() {
        let doc = parse("Visit `[here`/page] for `!info`!");
        assert_eq!(doc.lines[0].elements.len(), 4);
        assert!(matches!(&doc.lines[0].elements[0], Element::Text(_)));
        assert!(matches!(&doc.lines[0].elements[1], Element::Link(_)));
        assert!(matches!(&doc.lines[0].elements[2], Element::Text(_)));
        assert!(matches!(&doc.lines[0].elements[3], Element::Text(_)));
    }

    #[test]
    fn test_empty_input() {
        let doc = parse("");
        assert!(doc.lines.is_empty());
    }

    #[test]
    fn test_empty_line() {
        let doc = parse("a\n\nb");
        assert_eq!(doc.lines.len(), 3);
    }

    #[test]
    fn test_link_empty_label() {
        let doc = parse("`[`/page]");
        if let Element::Link(l) = &doc.lines[0].elements[0] {
            assert_eq!(l.label, "/page");
            assert_eq!(l.url, "/page");
        }
    }

    #[test]
    fn test_complete_page() {
        let input = r#">Node Name
`cWelcome!`a

-

>>About
This is `!NomadNet`!.

>>Links
`[Home`/]
`[Messages`/messages]

-=

`c`g50Updated 2024`f`a"#;

        let doc = parse(input);
        assert_eq!(doc.lines[0].kind, LineKind::Heading(1));
        assert_eq!(doc.lines[1].alignment, Alignment::Center);
        assert_eq!(doc.lines[3].kind, LineKind::Divider('\u{2500}'));
        assert_eq!(doc.lines[5].kind, LineKind::Heading(2));
        assert_eq!(doc.lines[12].kind, LineKind::Divider('='));
    }

    #[test]
    fn test_backtick_at_end_resets() {
        let doc = parse("`!bold`");
        assert_eq!(doc.lines[0].elements.len(), 1);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert!(t.style.bold);
        }
    }

    #[test]
    fn test_field_empty_default() {
        let doc = parse("`<name`>");
        if let Element::Field(f) = &doc.lines[0].elements[0] {
            assert_eq!(f.default, "");
        }
    }

    #[test]
    fn test_external_link() {
        let doc = parse("`[Example`https://example.com]");
        if let Element::Link(l) = &doc.lines[0].elements[0] {
            assert_eq!(l.url, "https://example.com");
        }
    }

    #[test]
    fn test_color_white() {
        let doc = parse("`Ffffwhite`f");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(
                t.style.fg,
                Some(Color {
                    r: 255,
                    g: 255,
                    b: 255
                })
            );
        }
    }

    #[test]
    fn test_color_black() {
        let doc = parse("`F000black`f");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.style.fg, Some(Color { r: 0, g: 0, b: 0 }));
        }
    }

    #[test]
    fn test_fg_and_bg_combined() {
        let doc = parse("`Ff00`B00fboth`f`b");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.style.fg, Some(Color { r: 255, g: 0, b: 0 }));
            assert_eq!(t.style.bg, Some(Color { r: 0, g: 0, b: 255 }));
        }
    }

    #[test]
    fn test_all_styles_combined() {
        let doc = parse("`!`*`_`Ff00styled`f`_`*`!");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert!(t.style.bold);
            assert!(t.style.italic);
            assert!(t.style.underline);
            assert_eq!(t.style.fg, Some(Color { r: 255, g: 0, b: 0 }));
        }
    }

    #[test]
    fn test_escape_at_line_start() {
        let doc = parse("\\>not heading");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, ">not heading");
        }
    }

    #[test]
    fn test_literal_preserves_all() {
        let doc = parse("`=\n`F00`!`*all preserved\n`=");
        assert_eq!(doc.lines.len(), 1);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "`F00`!`*all preserved");
            assert!(!t.style.bold);
        }
    }

    #[test]
    fn test_partial_url_only() {
        let doc = parse("`{/simple}");
        if let Element::Partial(p) = &doc.lines[0].elements[0] {
            assert_eq!(p.url, "/simple");
            assert!(p.refresh.is_none());
            assert!(p.fields.is_empty());
        }
    }

    #[test]
    fn test_depth_after_heading_change() {
        let doc = parse(">H1\n>>H2\n>>>H3\nnormal");
        assert_eq!(doc.lines[0].indent_depth, 1);
        assert_eq!(doc.lines[1].indent_depth, 2);
        assert_eq!(doc.lines[2].indent_depth, 3);
        assert_eq!(doc.lines[3].indent_depth, 3);
    }

    #[test]
    fn test_comment_preserves_depth() {
        let doc = parse(">>section\n# comment\ntext");
        assert_eq!(doc.lines[1].kind, LineKind::Comment);
        assert_eq!(doc.lines[1].indent_depth, 2);
        assert_eq!(doc.lines[2].indent_depth, 2);
    }

    #[test]
    fn test_divider_inherits_depth() {
        let doc = parse(">>section\n-");
        assert_eq!(doc.lines[1].kind, LineKind::Divider('\u{2500}'));
        assert_eq!(doc.lines[1].indent_depth, 2);
    }

    #[test]
    fn test_reset_depth_with_content() {
        let doc = parse(">>>deep\n<back to top with text");
        assert_eq!(doc.lines[1].indent_depth, 0);
        if let Element::Text(t) = &doc.lines[1].elements[0] {
            assert_eq!(t.text, "back to top with text");
        }
    }

    #[test]
    fn test_unknown_backtick_command() {
        let doc = parse("`zunknown");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "zunknown");
        }
    }

    #[test]
    fn test_multiple_resets() {
        let doc = parse("`!bold`` more`!");
        assert_eq!(doc.lines[0].elements.len(), 2);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert!(t.style.bold);
        }
        if let Element::Text(t) = &doc.lines[0].elements[1] {
            assert!(!t.style.bold);
        }
    }

    #[test]
    fn test_link_in_styled_context() {
        let doc = parse("`Ff00`[Red Link`/page]`f");
        if let Element::Link(l) = &doc.lines[0].elements[0] {
            assert_eq!(l.style.fg, Some(Color { r: 255, g: 0, b: 0 }));
        }
    }

    #[test]
    fn test_field_in_form() {
        let doc = parse("Name: `<20|name`Default>`[Submit`/form`name]");
        assert!(matches!(&doc.lines[0].elements[1], Element::Field(_)));
        assert!(matches!(&doc.lines[0].elements[2], Element::Link(_)));
    }

    #[test]
    fn test_grayscale_zero() {
        let doc = parse("`Fg00black`f");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            let c = t.style.fg.unwrap();
            assert_eq!(c.r, 0);
        }
    }

    #[test]
    fn test_grayscale_full() {
        let doc = parse("`Fg99white`f");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            let c = t.style.fg.unwrap();
            assert_eq!(c.r, 255);
        }
    }

    #[test]
    fn test_consecutive_escapes() {
        let doc = parse("\\`\\`two backticks");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "``two backticks");
        }
    }

    #[test]
    fn test_heading_with_formatting() {
        let doc = parse(">>`!Bold Heading`!");
        assert_eq!(doc.lines[0].kind, LineKind::Heading(2));
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert!(t.style.bold);
            assert_eq!(t.text, "Bold Heading");
        }
    }

    #[test]
    fn test_divider_with_special_char() {
        let doc = parse("-*");
        assert_eq!(doc.lines[0].kind, LineKind::Divider('*'));
    }

    #[test]
    fn test_style_toggle_sequence() {
        let doc = parse("`!on`!off`!on again`!");
        assert_eq!(doc.lines[0].elements.len(), 3);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert!(t.style.bold);
            assert_eq!(t.text, "on");
        }
        if let Element::Text(t) = &doc.lines[0].elements[1] {
            assert!(!t.style.bold);
            assert_eq!(t.text, "off");
        }
        if let Element::Text(t) = &doc.lines[0].elements[2] {
            assert!(t.style.bold);
            assert_eq!(t.text, "on again");
        }
    }

    #[test]
    fn test_alignment_without_text() {
        let doc = parse("`c");
        assert_eq!(doc.lines[0].alignment, Alignment::Left);
        assert!(doc.lines[0].elements.is_empty());
    }

    #[test]
    fn test_alignment_persists_after_reset() {
        let doc = parse("`ccentered`a\nstill left");
        assert_eq!(doc.lines[0].alignment, Alignment::Center);
        assert_eq!(doc.lines[1].alignment, Alignment::Left);
    }

    #[test]
    fn test_link_single_field() {
        let doc = parse("`[Go`/action`field1]");
        if let Element::Link(l) = &doc.lines[0].elements[0] {
            assert_eq!(l.fields, vec!["field1"]);
        }
    }

    #[test]
    fn test_link_many_fields() {
        let doc = parse("`[Go`/action`a|b|c|d]");
        if let Element::Link(l) = &doc.lines[0].elements[0] {
            assert_eq!(l.fields, vec!["a", "b", "c", "d"]);
        }
    }

    #[test]
    fn test_partial_with_many_fields() {
        let doc = parse("`{/api`60`f1|f2|f3}");
        if let Element::Partial(p) = &doc.lines[0].elements[0] {
            assert_eq!(p.refresh, Some(60));
            assert_eq!(p.fields, vec!["f1", "f2", "f3"]);
        }
    }

    #[test]
    fn test_field_no_width_with_pipe() {
        let doc = parse("`<field`value>");
        if let Element::Field(f) = &doc.lines[0].elements[0] {
            assert_eq!(f.name, "field");
            assert!(f.width.is_none());
        }
    }

    #[test]
    fn test_checkbox_with_default_label() {
        let doc = parse("`<?|notify|yes`Send notifications>");
        if let Element::Field(f) = &doc.lines[0].elements[0] {
            assert_eq!(f.name, "notify");
            assert_eq!(f.default, "Send notifications");
            assert_eq!(f.kind, FieldKind::Checkbox { checked: false });
        }
    }

    #[test]
    fn test_radio_group() {
        let doc = parse("`<^|size|s`Small> `<^|size|m`Medium> `<^|size|l|*`Large>");
        if let Element::Field(f) = &doc.lines[0].elements[0]
            && let FieldKind::Radio { value, checked } = &f.kind
        {
            assert_eq!(value, "s");
            assert!(!checked);
        }
        if let Element::Field(f) = &doc.lines[0].elements[4]
            && let FieldKind::Radio { value, checked } = &f.kind
        {
            assert_eq!(value, "l");
            assert!(checked);
        }
    }

    #[test]
    fn test_text_between_elements() {
        let doc = parse("before`[link`/]after");
        assert_eq!(doc.lines[0].elements.len(), 3);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "before");
        }
        if let Element::Text(t) = &doc.lines[0].elements[2] {
            assert_eq!(t.text, "after");
        }
    }

    #[test]
    fn test_only_whitespace_line() {
        let doc = parse("   ");
        assert_eq!(doc.lines.len(), 1);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "   ");
        }
    }

    #[test]
    fn test_unicode_text() {
        let doc = parse("Hello \u{4E16}\u{754C}");
        assert_eq!(doc.lines.len(), 1);
        assert_eq!(doc.lines[0].elements.len(), 1);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "Hello \u{4E16}\u{754C}");
        }
    }

    #[test]
    fn test_style_reset_clears_all() {
        let doc = parse("`!`*`_`Ff00`B00fstyle`` clean");
        assert_eq!(doc.lines[0].elements.len(), 2);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert!(t.style.bold);
            assert!(t.style.italic);
            assert!(t.style.underline);
            assert!(t.style.fg.is_some());
            assert!(t.style.bg.is_some());
        }
        if let Element::Text(t) = &doc.lines[0].elements[1] {
            assert!(!t.style.bold);
            assert!(!t.style.italic);
            assert!(!t.style.underline);
            assert!(t.style.fg.is_none());
            assert!(t.style.bg.is_none());
        }
    }

    #[test]
    fn test_heading_empty_content() {
        let doc = parse(">");
        assert_eq!(doc.lines[0].kind, LineKind::Heading(1));
        assert!(doc.lines[0].elements.is_empty());
    }

    #[test]
    fn test_literal_mode_toggle() {
        let doc = parse("before\n`=\nliteral\n`=\nafter");
        assert_eq!(doc.lines.len(), 3);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "before");
        }
        if let Element::Text(t) = &doc.lines[1].elements[0] {
            assert_eq!(t.text, "literal");
        }
        if let Element::Text(t) = &doc.lines[2].elements[0] {
            assert_eq!(t.text, "after");
        }
    }

    #[test]
    fn test_escape_in_middle() {
        let doc = parse("hello\\`world");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "hello`world");
        }
    }

    #[test]
    fn test_color_mid_word() {
        let doc = parse("he`Ff00ll`fo");
        assert_eq!(doc.lines[0].elements.len(), 3);
        if let Element::Text(t) = &doc.lines[0].elements[1] {
            assert_eq!(t.style.fg, Some(Color { r: 255, g: 0, b: 0 }));
            assert_eq!(t.text, "ll");
        }
    }

    #[test]
    fn test_literal_in_heading() {
        let doc = parse(">Heading Title");
        assert_eq!(doc.lines[0].kind, LineKind::Heading(1));
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "Heading Title");
        }
    }

    #[test]
    fn test_escape_greater_at_start() {
        let doc = parse("\\>not a heading");
        assert_eq!(doc.lines[0].kind, LineKind::Normal);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, ">not a heading");
        }
    }

    #[test]
    fn test_escape_dash_at_start() {
        let doc = parse("\\-not a divider");
        assert_eq!(doc.lines[0].kind, LineKind::Normal);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "-not a divider");
        }
    }

    #[test]
    fn test_escape_hash_at_start() {
        let doc = parse("\\#not a comment");
        assert_eq!(doc.lines[0].kind, LineKind::Normal);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "#not a comment");
        }
    }

    #[test]
    fn test_partial_empty_fields() {
        let doc = parse("`{/api`30`}");
        if let Element::Partial(p) = &doc.lines[0].elements[0] {
            assert_eq!(p.refresh, Some(30));
            assert_eq!(p.fields, vec![""]);
        }
    }

    #[test]
    fn test_link_empty_url() {
        let doc = parse("`[Label`]");
        if let Element::Link(l) = &doc.lines[0].elements[0] {
            assert_eq!(l.label, "Label");
            assert_eq!(l.url, "");
        }
    }

    #[test]
    fn test_multiline_styles() {
        let doc = parse("`!bold\nstill bold\n`!not bold");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert!(t.style.bold);
        }
        if let Element::Text(t) = &doc.lines[1].elements[0] {
            assert!(t.style.bold);
        }
        if let Element::Text(t) = &doc.lines[2].elements[0] {
            assert!(!t.style.bold);
        }
    }

    #[test]
    fn test_color_persists_lines() {
        let doc = parse("`Ff00red\nstill red`f");
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.style.fg, Some(Color { r: 255, g: 0, b: 0 }));
        }
        if let Element::Text(t) = &doc.lines[1].elements[0] {
            assert_eq!(t.style.fg, Some(Color { r: 255, g: 0, b: 0 }));
        }
    }

    #[test]
    fn test_heading_depth_sequence() {
        let doc = parse(">\n>>\n>>>\n<\nnormal");
        assert_eq!(doc.lines[0].indent_depth, 1);
        assert_eq!(doc.lines[1].indent_depth, 2);
        assert_eq!(doc.lines[2].indent_depth, 3);
        assert_eq!(doc.lines[3].indent_depth, 0);
        assert_eq!(doc.lines[4].indent_depth, 0);
    }

    #[test]
    fn test_divider_inherits_alignment() {
        let doc = parse("`c\n-");
        assert_eq!(doc.lines[1].alignment, Alignment::Center);
    }

    #[test]
    fn test_comment_after_styled() {
        let doc = parse("`!bold\n# comment\nstill bold`!");
        assert_eq!(doc.lines[1].kind, LineKind::Comment);
        if let Element::Text(t) = &doc.lines[2].elements[0] {
            assert!(t.style.bold);
        }
    }

    #[test]
    fn test_literal_multiline_complex() {
        let doc = parse("`=\n`!\n`=\n`!actual bold`!");
        assert_eq!(doc.lines.len(), 2);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert!(!t.style.bold);
            assert_eq!(t.text, "`!");
        }
        if let Element::Text(t) = &doc.lines[1].elements[0] {
            assert!(t.style.bold);
        }
    }

    #[test]
    fn test_field_label_with_special_chars() {
        let doc = parse("`<?|field|val`Label with spaces & symbols!>");
        if let Element::Field(f) = &doc.lines[0].elements[0] {
            assert_eq!(f.default, "Label with spaces & symbols!");
        }
    }

    #[test]
    fn test_link_url_special_chars() {
        let doc = parse("`[Go`https://example.com/path?q=1&b=2]");
        if let Element::Link(l) = &doc.lines[0].elements[0] {
            assert_eq!(l.url, "https://example.com/path?q=1&b=2");
        }
    }

    #[test]
    fn test_double_backtick_resets_alignment() {
        let doc = parse("`ccentered`` left");
        assert_eq!(doc.lines[0].alignment, Alignment::Center);
        if let Element::Text(t) = &doc.lines[0].elements[1] {
            assert_eq!(t.text, " left");
        }
        let doc2 = parse("`ccentered``\nnext line");
        assert_eq!(doc2.lines[1].alignment, Alignment::Left);
    }

    #[test]
    fn test_radio_value_fallback_to_label() {
        let doc = parse("`<^|choice|`Option A>");
        if let Element::Field(f) = &doc.lines[0].elements[0]
            && let FieldKind::Radio { value, .. } = &f.kind
        {
            assert_eq!(value, "Option A");
        }
    }

    #[test]
    fn test_radio_explicit_value() {
        let doc = parse("`<^|choice|val`Option A>");
        if let Element::Field(f) = &doc.lines[0].elements[0]
            && let FieldKind::Radio { value, .. } = &f.kind
        {
            assert_eq!(value, "val");
        }
    }

    #[test]
    fn test_heading_with_field_strips_prefix() {
        let doc = parse(">`<name`default>");
        assert_eq!(doc.lines[0].kind, LineKind::Normal);
        assert!(matches!(&doc.lines[0].elements[0], Element::Field(_)));
    }

    #[test]
    fn test_color_incomplete_ignored() {
        let doc = parse("`Fx");
        assert_eq!(doc.lines[0].elements.len(), 1);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "x");
            assert!(t.style.fg.is_none());
        }
    }

    #[test]
    fn test_color_at_end_of_line() {
        let doc = parse("text`F");
        assert_eq!(doc.lines[0].elements.len(), 1);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "text");
        }
    }

    #[test]
    fn test_bg_color_incomplete_ignored() {
        let doc = parse("`Bxy");
        assert_eq!(doc.lines[0].elements.len(), 1);
        if let Element::Text(t) = &doc.lines[0].elements[0] {
            assert_eq!(t.text, "xy");
            assert!(t.style.bg.is_none());
        }
    }

    #[test]
    fn test_link_colon_path() {
        let doc = parse("`[Home`:/page/index.mu]");
        if let Element::Link(l) = &doc.lines[0].elements[0] {
            assert_eq!(l.label, "Home");
            assert_eq!(l.url, ":/page/index.mu");
        } else {
            panic!("Expected Link");
        }
    }
}

#[test]
fn test_styled_link() {
    // This is the format from the actual page: `!`[Home`:/page/index.mu]`!
    let doc = parse(r#"`!`[Home`:/page/index.mu]`!"#);
    println!("Elements: {:?}", doc.lines[0].elements);
    assert!(!doc.lines[0].elements.is_empty());
    let has_link = doc.lines[0]
        .elements
        .iter()
        .any(|e| matches!(e, Element::Link(_)));
    assert!(has_link, "Should have a link element");
    if let Element::Link(l) = &doc.lines[0].elements[0] {
        assert_eq!(l.label, "Home");
        assert_eq!(l.url, ":/page/index.mu");
    }
}

#[test]
fn test_file_list_with_color_underline_link() {
    // First test a simpler case: just color and link
    let doc = parse(r#"`F0f0`[Test`/path]"#);
    println!("Simple test - Elements: {:?}", doc.lines[0].elements);
    assert!(
        doc.lines[0]
            .elements
            .iter()
            .any(|e| matches!(e, Element::Link(_))),
        "Simple case should have link"
    );

    // Test with underline
    let doc = parse(r#"`_`[Test`/path]`_"#);
    println!("Underline test - Elements: {:?}", doc.lines[0].elements);
    assert!(
        doc.lines[0]
            .elements
            .iter()
            .any(|e| matches!(e, Element::Link(_))),
        "Underline case should have link"
    );

    // Test link with no label (URL becomes label) - no backtick needed per reference impl
    let doc = parse(r#"`[:/file/test.mp3]"#);
    println!("No-label test - Elements: {:?}", doc.lines[0].elements);
    assert!(
        doc.lines[0]
            .elements
            .iter()
            .any(|e| matches!(e, Element::Link(_))),
        "No-label case should have link"
    );

    // Format: " -  `F0f0`_`[:/file/Baby_Got_Back.mp3]`_`f (11M)"
    // This is a list item with: fg color, underline start, link (no label), underline end, fg reset, text
    let doc = parse(r#" -  `F0f0`_`[:/file/Baby_Got_Back.mp3]`_`f (11M)"#);

    println!("Full test - Parsed {} lines", doc.lines.len());
    for (i, line) in doc.lines.iter().enumerate() {
        println!("Line {}: {:?}", i, line);
        for (j, elem) in line.elements.iter().enumerate() {
            println!("  Element {}: {:?}", j, elem);
        }
    }

    assert_eq!(doc.lines.len(), 1);
    let has_link = doc.lines[0]
        .elements
        .iter()
        .any(|e| matches!(e, Element::Link(_)));
    assert!(has_link, "Should have a link element");

    // Find the link element and verify its URL
    let link = doc.lines[0]
        .elements
        .iter()
        .find_map(|e| match e {
            Element::Link(l) => Some(l),
            _ => None,
        })
        .expect("Should have a link");
    assert_eq!(link.url, ":/file/Baby_Got_Back.mp3");
}

#[test]
fn test_user_form_fields() {
    let content = "`<20|username`Guest_ccbc>`[Submit`:/page/test.mu`username]";
    let doc = parse(content);
    println!("Lines: {}", doc.lines.len());
    println!("Elements: {}", doc.lines[0].elements.len());
    for (i, elem) in doc.lines[0].elements.iter().enumerate() {
        println!("  Element {}: {:?}", i, elem);
    }
    assert!(matches!(&doc.lines[0].elements[0], Element::Field(_)));
    assert!(matches!(&doc.lines[0].elements[1], Element::Link(_)));
}

#[test]
fn test_exact_user_content() {
    let content = "`B317 Nickname: `Baac`F000`<20|username`Guest_ccbc>`b`f";
    let doc = parse(content);
    println!("Lines: {}", doc.lines.len());
    println!("Elements: {}", doc.lines[0].elements.len());
    for (i, elem) in doc.lines[0].elements.iter().enumerate() {
        match elem {
            Element::Text(t) => println!("  Element {}: Text({:?})", i, t.text),
            Element::Field(f) => println!(
                "  Element {}: Field(name={}, default={}, width={:?})",
                i, f.name, f.default, f.width
            ),
            Element::Link(l) => println!("  Element {}: Link({})", i, l.url),
            _ => println!("  Element {}: {:?}", i, elem),
        }
    }
    // Check for field
    let has_field = doc.lines[0]
        .elements
        .iter()
        .any(|e| matches!(e, Element::Field(_)));
    assert!(has_field, "Should have a field element");
}

#[test]
fn test_alignment_persists_to_link_lines() {
    let doc = parse("`ctext\n`[Link`/a]");
    assert_eq!(doc.lines[0].alignment, Alignment::Center, "line with text");
    assert_eq!(doc.lines[1].alignment, Alignment::Center, "line with link");
}

#[test]
fn test_alignment_persists_through_format_only_lines() {
    let doc = parse("`F8ff`B222`c\n\n`[Link`/a]");
    assert_eq!(doc.lines.len(), 3);
    assert_eq!(
        doc.lines[2].alignment,
        Alignment::Center,
        "link after format-only line"
    );
}
