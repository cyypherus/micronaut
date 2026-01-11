# Micron Examples

## Basic Formatting

Input:
```
>Welcome to My Node

This is `!bold`! and `*italic`* text.

`_Underlined`_ with `Ff00red color`f.
```

## Nested Sections

Input:
```
>Main Heading
Some intro text.

>>Sub Section
Indented content here.

>>>Deep Section  
Even more indented.

<Back to top level.
```

## Links

Input:
```
Visit `[Home`/]
Or `[External`https://example.com]
Form link: `[Submit`/submit`name|email]
```

## Form Fields

Input:
```
Name: `<name`John Doe>
Password: `<!8|password`>
Remember me: `<?|remember|yes`Keep me logged in>
Choice: `<^|color|red`Red> `<^|color|blue`Blue>
```

## Colors

Input:
```
`Ff00Red `F0f0Green `F00fBlue`f

`Bff0Yellow background`b

`g50Gray text (50% brightness)`f
```

## Dividers

Input:
```
-
Above is default divider.

-=
Above uses = character.

->
Above uses > character.
```

## Literal Mode

Input:
```
`=
Everything here is literal.
`!This is NOT bold`!
Backticks don't work: `F00
`=
Normal parsing resumes.
```

## Partial Include

Input:
```
`{/api/status}
`{/api/clock`5}
`{/api/data`10`user_id}
```

## Alignment

Input:
```
`lLeft aligned (default)
`cCentered text
`rRight aligned
`aBack to default
```

## Complete Page Example

```
>Node Name
`cWelcome to my node!`a

-

>>About
This is a simple node running on `!NomadNet`!.

>>Links
`[Home`/]
`[Messages`/messages]
`[Files`/files]

-=

`c`g50Last updated: 2024`f`a
```
