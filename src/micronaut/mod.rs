mod ast;
#[cfg(feature = "browser")]
mod browser;
mod parser;
#[cfg(feature = "ratatui")]
mod ratatui;
#[cfg(feature = "browser")]
mod types;
#[cfg(feature = "ratatui")]
mod widget;

pub use ast::{
    Alignment, Color, Document, Element, Field, FieldKind, Line, LineKind, LinkElement, Partial,
    Style, StyledText,
};
pub use parser::parse;

#[cfg(feature = "browser")]
pub use browser::{Browser, Renderer};
#[cfg(feature = "browser")]
pub use types::{Interaction, Link, TextField};

#[cfg(feature = "ratatui")]
pub use self::ratatui::RatatuiRenderer;
#[cfg(feature = "ratatui")]
pub use self::widget::BrowserWidget;
