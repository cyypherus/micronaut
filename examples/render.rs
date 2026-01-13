use std::io::{self, stdout};

use crossterm::{
    ExecutableCommand,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    buffer::Buffer,
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use micronaut::{Browser, BrowserWidget, Interaction, Link, RatatuiRenderer};

enum Mode {
    Browse,
    Edit { field_name: String, masked: bool },
    Navigate { link: Link },
}

struct Modal<'a> {
    title: &'a str,
    content: Vec<Line<'a>>,
    buttons: Vec<(&'a str, Color)>,
    border_color: Color,
}

impl<'a> Modal<'a> {
    fn render(&self, area: Rect, buf: &mut Buffer) -> (Rect, Vec<Rect>) {
        let width = 50u16.min(area.width.saturating_sub(4));
        let height = (self.content.len() as u16 + 4).min(area.height.saturating_sub(4));
        let popup = Rect {
            x: area.x + (area.width.saturating_sub(width)) / 2,
            y: area.y + (area.height.saturating_sub(height)) / 2,
            width,
            height,
        };

        Clear.render(popup, buf);

        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.border_color));

        let inner = block.inner(popup);
        block.render(popup, buf);

        let content_height = inner.height.saturating_sub(2);
        Paragraph::new(self.content.clone()).render(
            Rect::new(inner.x, inner.y, inner.width, content_height),
            buf,
        );

        let button_y = inner.y + inner.height.saturating_sub(1);
        let mut button_rects = Vec::new();
        let total_len: usize = self.buttons.iter().map(|(l, _)| l.len() + 2).sum();
        let spacing = 2usize;
        let total = total_len + spacing * self.buttons.len().saturating_sub(1);
        let mut x = inner.x + (inner.width.saturating_sub(total as u16)) / 2;

        for (label, color) in &self.buttons {
            let label_text = format!(" {} ", label);
            let len = label_text.len() as u16;
            buf.set_string(
                x,
                button_y,
                &label_text,
                Style::default().fg(Color::Black).bg(*color),
            );
            button_rects.push(Rect::new(x, button_y, len, 1));
            x += len + spacing as u16;
        }

        (popup, button_rects)
    }
}

fn main() -> io::Result<()> {
    let content = std::env::args()
        .nth(1)
        .map(|path| std::fs::read_to_string(&path).expect("Failed to read file"))
        .unwrap_or_else(|| include_str!("../tests/example.mu").to_string());

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut browser = Browser::new(RatatuiRenderer);
    browser.set_content("file://example.mu", &content);
    let mut mode = Mode::Browse;
    let mut input = Input::default();
    let mut button_rects: Vec<Rect> = Vec::new();

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(BrowserWidget::new(&mut browser), area);

            match &mode {
                Mode::Browse => {
                    button_rects.clear();
                }
                Mode::Edit { field_name, masked } => {
                    let display_value = if *masked {
                        "*".repeat(input.value().len())
                    } else {
                        input.value().to_string()
                    };

                    let inner_width = 46usize;
                    let cursor = input.visual_cursor();
                    let scroll = cursor.saturating_sub(inner_width.saturating_sub(1));
                    let visible: String = display_value.chars().skip(scroll).collect();

                    let modal = Modal {
                        title: field_name,
                        content: vec![Line::from(visible)],
                        buttons: vec![("Cancel", Color::DarkGray), ("Confirm", Color::Green)],
                        border_color: Color::Cyan,
                    };

                    let (popup, btns) = modal.render(area, frame.buffer_mut());
                    button_rects = btns;

                    let cursor_x = popup.x + 1 + (cursor - scroll) as u16;
                    let cursor_y = popup.y + 1;
                    frame.set_cursor_position((cursor_x, cursor_y));
                }
                Mode::Navigate { link } => {
                    let url_display = if link.url.len() > 44 {
                        format!("{}...", &link.url[..41])
                    } else {
                        link.url.clone()
                    };

                    let mut lines = vec![Line::from(vec![
                        Span::styled("URL: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(url_display, Style::default().fg(Color::White)),
                    ])];

                    if !link.form_data.is_empty() {
                        lines.push(Line::from(""));
                        lines.push(Line::styled(
                            "Form data:",
                            Style::default().fg(Color::DarkGray),
                        ));
                        for (k, v) in &link.form_data {
                            lines.push(Line::from(vec![
                                Span::styled(
                                    format!("  {}: ", k),
                                    Style::default().fg(Color::Cyan),
                                ),
                                Span::styled(v, Style::default().fg(Color::White)),
                            ]));
                        }
                    }

                    let modal = Modal {
                        title: "Navigate",
                        content: lines,
                        buttons: vec![("Cancel", Color::DarkGray), ("Go", Color::Green)],
                        border_color: Color::Yellow,
                    };

                    let (_, btns) = modal.render(area, frame.buffer_mut());
                    button_rects = btns;
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            let evt = event::read()?;
            match &mode {
                Mode::Browse => match &evt {
                    Event::Key(key) => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Tab | KeyCode::Char('j') => browser.select_next(),
                        KeyCode::BackTab | KeyCode::Char('k') => browser.select_prev(),
                        KeyCode::Enter => {
                            if let Some(interaction) = browser.interact() {
                                match interaction {
                                    Interaction::Link(link) => {
                                        mode = Mode::Navigate { link };
                                    }
                                    Interaction::EditField(field) => {
                                        input = Input::new(field.value);
                                        mode = Mode::Edit {
                                            field_name: field.name,
                                            masked: field.masked,
                                        };
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    Event::Mouse(mouse) => match mouse.kind {
                        MouseEventKind::Down(_) => {
                            if let Some(interaction) = browser.click(mouse.column, mouse.row) {
                                match interaction {
                                    Interaction::Link(link) => {
                                        mode = Mode::Navigate { link };
                                    }
                                    Interaction::EditField(field) => {
                                        input = Input::new(field.value);
                                        mode = Mode::Edit {
                                            field_name: field.name,
                                            masked: field.masked,
                                        };
                                    }
                                }
                            }
                        }
                        MouseEventKind::ScrollDown => browser.scroll_by(3),
                        MouseEventKind::ScrollUp => browser.scroll_by(-3),
                        _ => {}
                    },
                    _ => {}
                },
                Mode::Edit { field_name, .. } => match &evt {
                    Event::Key(key) => match key.code {
                        KeyCode::Enter => {
                            browser.set_field_value(field_name, input.value().to_string());
                            input.reset();
                            mode = Mode::Browse;
                        }
                        KeyCode::Esc => {
                            input.reset();
                            mode = Mode::Browse;
                        }
                        _ => {
                            input.handle_event(&evt);
                        }
                    },
                    Event::Mouse(mouse) => {
                        if let MouseEventKind::Down(_) = mouse.kind {
                            let (x, y) = (mouse.column, mouse.row);
                            if button_rects
                                .get(0)
                                .is_some_and(|r| r.contains((x, y).into()))
                            {
                                input.reset();
                                mode = Mode::Browse;
                            } else if button_rects
                                .get(1)
                                .is_some_and(|r| r.contains((x, y).into()))
                            {
                                browser.set_field_value(field_name, input.value().to_string());
                                input.reset();
                                mode = Mode::Browse;
                            }
                        }
                    }
                    _ => {}
                },
                Mode::Navigate { .. } => match &evt {
                    Event::Key(key) => match key.code {
                        KeyCode::Enter | KeyCode::Char('y') => {
                            mode = Mode::Browse;
                        }
                        KeyCode::Esc | KeyCode::Char('n') => {
                            mode = Mode::Browse;
                        }
                        _ => {}
                    },
                    Event::Mouse(mouse) => {
                        if let MouseEventKind::Down(_) = mouse.kind {
                            let (x, y) = (mouse.column, mouse.row);
                            if button_rects
                                .get(0)
                                .is_some_and(|r| r.contains((x, y).into()))
                                || button_rects
                                    .get(1)
                                    .is_some_and(|r| r.contains((x, y).into()))
                            {
                                mode = Mode::Browse;
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
    }

    stdout().execute(DisableMouseCapture)?;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
