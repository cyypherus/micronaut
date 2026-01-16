// use ratatui::buffer::Buffer;
// use ratatui::layout::Rect;
// use ratatui::style::{Color, Style};
// use ratatui::text::Text;
// use ratatui::widgets::{Paragraph, Widget};

// use crate::micronaut::browser::Browser;
// use crate::micronaut::ratatui::RatatuiRenderer;

// pub struct BrowserWidget<'a> {
//     browser: &'a mut Browser<RatatuiRenderer>,
// }

// impl<'a> BrowserWidget<'a> {
//     pub fn new(browser: &'a mut Browser<RatatuiRenderer>) -> Self {
//         Self { browser }
//     }
// }

// impl Widget for BrowserWidget<'_> {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         self.browser.resize(area.width, area.height);
//         let scroll = self.browser.scroll();

//         if let Some(content) = self.browser.render().cloned() {
//             Paragraph::new(content)
//                 .scroll((scroll, 0))
//                 .render(area, buf);
//         } else {
//             let content = Text::styled("No content", Style::default().fg(Color::DarkGray));
//             Paragraph::new(content)
//                 .alignment(ratatui::layout::Alignment::Center)
//                 .render(area, buf);
//         }
//     }
// }
