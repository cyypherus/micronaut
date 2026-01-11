mod ast;
mod browser;
mod parser;
#[cfg(feature = "ratatui")]
mod ratatui;
mod types;
#[cfg(feature = "ratatui")]
mod widget;

pub use ast::{Alignment, Color, Document, Element, Field, FieldKind, Line, LineKind, LinkElement, Partial, Style, StyledText};
pub use browser::{Browser, Renderer};
pub use parser::parse;
pub use types::{InputResult, Link};

#[cfg(feature = "ratatui")]
pub use self::ratatui::RatatuiRenderer;
#[cfg(feature = "ratatui")]
pub use self::widget::BrowserWidget;
