# Winnow Parser Hints

## Basic Setup

```rust
use winnow::prelude::*;
use winnow::combinator::{alt, opt, preceded, delimited, repeat};
use winnow::token::{any, take, take_while, one_of};
use winnow::ascii::{hex_digit, digit};
```

## Stateful Parsing

Micron needs state (bold on/off, colors). Use winnow's Stateful wrapper:

```rust
use winnow::stream::Stateful;

type Stream<'a> = Stateful<&'a str, ParseState>;

fn parse_document<'a>(input: &mut Stream<'a>) -> PResult<Document> {
    // input.state is accessible here
}
```

## Useful Combinators

```rust
// Match literal
">>".parse_next(input)?;

// Match one of several chars
one_of(['!', '_', '*']).parse_next(input)?;

// Take N chars
take(3usize).parse_next(input)?;  // for color codes

// Take while condition
take_while(1.., |c| c != '`').parse_next(input)?;

// Optional
opt(parse_color).parse_next(input)?;

// Alternatives
alt((parse_bold, parse_italic, parse_underline)).parse_next(input)?;

// Preceded (skip prefix, return second)
preceded('`', parse_format_code).parse_next(input)?;
```

## Line-by-Line

Process document as lines, parse each:

```rust
pub fn parse(input: &str) -> Result<Document, Error> {
    let mut state = ParseState::default();
    let lines: Vec<Line> = input
        .lines()
        .filter_map(|line| parse_line(line, &mut state))
        .collect();
    Ok(Document { lines })
}
```

## Color Parsing

```rust
fn parse_color<'a>(input: &mut Stream<'a>) -> PResult<Color> {
    let hex: &str = take(3usize).parse_next(input)?;
    let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(0) * 17;
    let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(0) * 17;
    let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(0) * 17;
    Ok(Color { r, g, b })
}
```

## Escape Handling

```rust
fn parse_escaped_char<'a>(input: &mut Stream<'a>) -> PResult<char> {
    preceded('\\', any).parse_next(input)
}
```

## Link Parsing

```rust
fn parse_link<'a>(input: &mut Stream<'a>) -> PResult<Link> {
    // Expect: [label`url] or [label`url`fields]
    let _ = '['.parse_next(input)?;
    let label = take_while(0.., |c| c != '`').parse_next(input)?;
    let _ = '`'.parse_next(input)?;
    let url = take_while(0.., |c| c != '`' && c != ']').parse_next(input)?;
    let fields = opt(preceded('`', take_while(0.., |c| c != ']'))).parse_next(input)?;
    let _ = ']'.parse_next(input)?;
    
    Ok(Link {
        label: label.to_string(),
        url: url.to_string(),
        fields: fields.map(|f| f.split('|').map(String::from).collect()).unwrap_or_default(),
        style: input.state.current_style(),
    })
}
```
