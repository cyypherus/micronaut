# Micron Markup Language Research

## Reference Implementation (Python)
- **Location**: https://github.com/markqvist/NomadNet/blob/master/nomadnet/ui/textui/MicronParser.py
- **Raw**: https://raw.githubusercontent.com/markqvist/NomadNet/master/nomadnet/ui/textui/MicronParser.py
- ~945 lines of Python, uses urwid for terminal rendering
- GPL-3.0 license

## Official Documentation
- **Location**: https://github.com/markqvist/NomadNet/blob/master/nomadnet/ui/textui/Guide.py
- TOPIC_MARKUP section contains full spec (lines 1418-1926)
- Embedded in Python as string literals

## Alternative Implementations
- **JS Port**: https://github.com/RFnexus/micron-parser-js - "1:1 implementation" per authors
- **Live Demo**: https://rfnexus.github.io/micron-parser-js/

## Test Files / Examples
- **No formal test suite** found in NomadNet repo
- Example pages: https://github.com/SebastianObi/NomadNet-Pages
- More examples: https://github.com/epenguins/NomadNet_pages

## Core Syntax (from source analysis)

### Line-start tags:
- `>` - Section heading (depth = count of >)
- `<` - Reset section depth to 0
- `-` - Horizontal divider (optional char after -)
- `#` - Comment (line not displayed)
- `\` - Escape first char

### Inline tags (backtick prefix):
- `` `! `` - Toggle bold
- `` `* `` - Toggle italic
- `` `_ `` - Toggle underline
- `` `F{xxx} `` - Set foreground color (3 hex chars)
- `` `f `` - Reset foreground
- `` `B{xxx} `` - Set background color
- `` `b `` - Reset background
- `` `` `` - Reset all formatting
- `` `c `` - Center align
- `` `l `` - Left align
- `` `r `` - Right align
- `` `a `` - Default align
- `` `= `` - Toggle literal mode (on own line)
- `` `[label`url`fields] `` - Link
- `` `<field_name`default> `` - Text input field
- `` `<!|field_name`default> `` - Masked field
- `` `<width|field_name`> `` - Sized field
- `` `<?|field_name|value`>label `` - Checkbox
- `` `<^|field_name|value`>label `` - Radio button
- `` `{url`refresh`fields} `` - Partial (async load)

### Escape:
- `\\` escapes next char
- `\`` outputs literal backtick

### Colors:
- 3-char hex: `000`-`fff`
- Grayscale: `g00`-`g99`
- 6-char hex also supported

### State Machine:
- Parser maintains: depth, fg_color, bg_color, formatting (bold/underline/italic), align
- Literal mode bypasses all parsing except `= toggle
- Section headings use predefined styles (heading1-3)

## Key Behaviors (from Python source):
1. Lines split on `\n`, processed individually
2. Empty lines produce empty text widgets
3. `>` removes heading status if line contains `<` (field)
4. Divider char control codes (<32) replaced with `─` (U+2500)
5. Field width capped at 256
6. Radio groups track state across document
7. Link fields can use `*` for "all fields"
8. Partials have optional refresh interval and field list

## Complete Test Examples (from sources)

### Basic Formatting Test
```micron
>Nomad Network

`c`*Communicate Freely.`*
`a

The intention with this program is to provide a tool to that allows you to build private and resilient communications platforms that are in complete control and ownership of the people that use them.
```

### Section Headings Test
```micron
>High Level Stuff
This is a section. It contains this text.

>>Another Level
This is a sub section.

>>>Going deeper
A sub sub section. We could continue, but you get the point.

>>>>
Wait! It's worth noting that we can also create sections without headings. They look like this.
```

### Alignment Test
```micron
`cThis line will be centered.
So will this.
`aThe alignment has now been returned to default.
`rThis will be aligned to the right
```

### Text Formatting Test
```micron
We shall soon see `!bold`! paragraphs of text decorated with `_underlines`_ and `*italics`*. Some even dare `!`*`_combine`` them!
```

### Color Test
```micron
You can use `B5d5`F222 color `f`b `Ff00f`Ff80o`Ffd0r`F9f0m`F0f2a`F0fdt`F07ft`F43fi`F70fn`Fe0fg`f for some fabulous effects.
```

### Color Gradient Test
```micron
`B100 `B200 `B300 `B400 `B500 `B600 `B700 `B800 `B900 `Ba00 `Bb00 `Bc00 `Bd00 `Be00 `Bf00`b
`B010 `B020 `B030 `B040 `B050 `B060 `B070 `B080 `B090 `B0a0 `B0b0 `B0c0 `B0d0 `B0e0 `B0f0`b
`B001 `B002 `B003 `B004 `B005 `B006 `B007 `B008 `B009 `B00a `B00b `B00c `B00d `B00e `B00f`b
`Bg06 `Bg13 `Bg20 `Bg26 `Bg33 `Bg40 `Bg46 `Bg53 `Bg59 `Bg66 `Bg73 `Bg79 `Bg86 `Bg92 `Bg99`b
```

### Divider Test
```micron
Plain divider:
-

Custom divider char:
-=

Wavy divider:
-~
```

### Links Test
```micron
Here is a link without any label: `[72914442a3689add83a09a767963f57c:/page/index.mu]

This is a `[labeled link`72914442a3689add83a09a767963f57c:/page/index.mu] to the same page

Here is `F00f`_`[a more visible link`72914442a3689add83a09a767963f57c:/page/index.mu]`_`f

Local link: `[other page`:/page/other.mu]
```

### Link with Fields Test
```micron
`[Submit Fields`:/page/fields.mu`*]
`[Submit Fields`:/page/fields.mu`username|auth_token]
`[Query the System`:/page/fields.mu`username|auth_token|action=view|amount=64]
```

### Input Fields Test
```micron
A simple input field: `B444`<user_input`Pre-defined data>`b
An empty input field: `B444`<demo_empty`>`b
A sized input field:  `B444`<16|with_size`>`b
A masked input field: `B444`<!|masked_demo`hidden text>`b
Full control: `B444`<!32|all_options`hidden text>`b
```

### Checkbox Test
```micron
`<?|field_name|value`>`b Label Text
`B444`<?|sign_up|1`>`b Sign me up
`B444`<?|checkbox|1|*`>`b Pre-checked checkbox
```

### Radio Group Test
```micron
`B900`<^|color|Red`>`b  Red

`B090`<^|color|Green`>`b Green

`B009`<^|color|Blue`>`b Blue
```

### Literal Block Test
```micron
`=
This text is literal.
No `!formatting`! is applied.
> Not a heading
`=
```

### Partials Test
```micron
`{f64a846313b874ee4a357040807f8c77:/page/partial_1.mu}
`{f64a846313b874ee4a357040807f8c77:/page/refreshing_partial.mu`10}
`{f64a846313b874e84a357039807f8c77:/page/hello_partial.mu`0`pid=32|user_name}
```

### Comments Test
```micron
# This line will not be displayed
This line will
```

### Page Headers Test
```micron
#!c=0
#!bg=444
#!fg=ddd
>Page Title
Content here
```

### Complex Real-World Example (Bug Tracker Template)
```micron
>`cBug-Tracker`c
{msg}
`cHere you can find all known bugs/problems with the app.

`[Refresh`:]`

Last Update: {date_time}

>>Filter
`[All`:`filter=all]`        `[Open`:`filter=open]`        `[Closed`:`filter=closed]`        `[Own`:`filter=own]`
{entrys}

`[Home`:/page/index.mu]`
```

### Message Board Header Example
```micron
`!`F222`Bddd`cNomadNet Message Board
-
`a`b`f

To add a message to the board just converse with the NomadNet Message Board at `[lxmf@{peer}]
Last Updated: {time}

>Messages
```

### Status Messages Example
```micron
>>>`cStatus: `F080Successfully added!`f

Your support is appreciated!`c

>>>`cStatus: `Ff00Error adding!`f`c
```

### Table-like List Entry (using `;` delimited format)
```micron
`;3s;;{title};{text};[b]Status:[/b] {state_name} [b]#:[/b] {votes};>{title}\n{text}\n\n>>Status: `F{color}{state}`f;`
```

### Display Test Page
```micron
>Markup & Color Display Test

`cYou can use this section to gauge how well your terminal reproduces the various types of formatting used by Nomad Network.
``

>>>>>>>>>>>>>>>>>>>>
-~
<

>>
`a`!This line should be bold, and aligned to the left`!

`c`*This one should be italic and centered`*

`r`_And this one should be underlined, aligned right`_
``
```

## Edge Cases to Test
1. Escaped backticks: `\`` should render as literal backtick
2. Escaped backslash: `\\` should render as single backslash
3. Empty sections: `>>` with no text after
4. Nested formatting: `!`*`_combined`_`*`!`
5. Color at line start vs mid-line
6. Field inside heading (strips heading)
7. Divider with control char (should default to ─)
8. Reset (double backtick) clears all state
9. Literal block contains fake tags
