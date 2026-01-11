# Micron Syntax Reference

## Line-Level Commands (must be at start of line)

| Syntax | Meaning |
|--------|---------|
| `>` | Heading level 1 |
| `>>` | Heading level 2 |
| `>>>` | Heading level 3 |
| `<` | Reset heading depth to 0 |
| `-` | Horizontal divider (default char) |
| `-X` | Horizontal divider with char X |
| `#` | Comment (line ignored) |
| `\` | Escape (treat next char literally) |

## Inline Formatting (after backtick)

| Code | Effect |
|------|--------|
| `` `_ `` | Toggle underline |
| `` `! `` | Toggle bold |
| `` `* `` | Toggle italic |
| `` `Fxxx `` | Set foreground color (3 hex chars) |
| `` `f `` | Reset foreground to default |
| `` `Bxxx `` | Set background color (3 hex chars) |
| `` `b `` | Reset background to default |
| `` `` `` `` | Reset all formatting |
| `` `c `` | Align center |
| `` `l `` | Align left |
| `` `r `` | Align right |
| `` `a `` | Reset alignment to default |
| `` `= `` | Toggle literal mode (no parsing) |

## Links

```
`[label`url]
`[label`url`field1|field2]
```

Components:
- `label` — Display text (if empty, shows url)
- `url` — Link target
- `fields` — Optional pipe-separated field names for forms

## Input Fields

```
`<fieldname`default_value>
`<width|fieldname`default_value>
`<!width|fieldname`default_value>    # masked (password)
`<?|fieldname|value`label>           # checkbox
`<^|fieldname|value`label>           # radio button
`<?|fieldname|value|*`label>         # pre-checked checkbox
```

## Partials (Dynamic Includes)

```
`{url}
`{url`refresh_seconds}
`{url`refresh_seconds`field1|field2}
```

## Escaping

- `\`` — Literal backtick
- `\\` — Literal backslash
- `\>` — Literal > at line start

## Literal Mode

Between `` `= `` markers, no parsing occurs:
```
`=
This `!text`! is not bold
`=
```

## Color Format

3-character hex (RGB shorthand):
- `F00` = red
- `0F0` = green  
- `00F` = blue
- `FFF` = white
- `000` = black
- `g50` = 50% gray (special grayscale syntax)
