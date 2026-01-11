mod ast;
mod parser;
#[cfg(feature = "ratatui")]
mod render;

pub use ast::*;
pub use parser::parse;
#[cfg(feature = "ratatui")]
pub use render::{
    FormState, Hitbox, HitboxTarget, RenderConfig, RenderOutput, render, render_with_hitboxes,
};
