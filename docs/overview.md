# Micron Parser

Micron is a lightweight markup language for NomadNet node pages. Bandwidth-efficient, designed for terminal rendering.

## Goal
Parse Micron markup into an AST of styled spans. Output should be library-agnostic (no ratatui dependency in parser).

## Crate
```toml
winnow = "0.7"
```

## Module Structure
```
src/micron/
├── mod.rs      # pub use, module declarations
├── ast.rs      # Types: Span, Line, Style, Color, Element
├── parser.rs   # Winnow combinators
```
