use std::io::{self, stdout};

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;

use micronaut::{Browser, BrowserWidget, RatatuiRenderer};

fn main() -> io::Result<()> {
    let input = std::env::args()
        .nth(1)
        .map(|path| std::fs::read_to_string(&path).expect("Failed to read file"))
        .unwrap_or_else(|| include_str!("../tests/example.mu").to_string());

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut browser = Browser::new(RatatuiRenderer);
    browser.set_content("file://example.mu", &input);

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(BrowserWidget::new(&mut browser), area);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down | KeyCode::Char('j') => browser.scroll_by(1),
                    KeyCode::Up | KeyCode::Char('k') => browser.scroll_by(-1),
                    KeyCode::PageDown | KeyCode::Char(' ') => browser.scroll_by(10),
                    KeyCode::PageUp => browser.scroll_by(-10),
                    KeyCode::Home => browser.scroll_to(0),
                    KeyCode::End => browser.scroll_to(u16::MAX),
                    KeyCode::Tab => browser.select_next(),
                    KeyCode::BackTab => browser.select_prev(),
                    KeyCode::Enter => {
                        if let Some(link) = browser.interact() {
                            eprintln!("Navigate to: {} (form: {:?})", link.url, link.form_data);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
