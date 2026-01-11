use crate::micronaut::ast::Document;
use crate::micronaut::parser::parse;
use crate::micronaut::types::{FormState, Hitbox, HitboxTarget, InputResult, Link};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct HistoryEntry {
    url: String,
    content: String,
    scroll: u16,
}

pub struct Browser<R: Renderer> {
    url: Option<String>,
    content: Option<String>,
    scroll: u16,
    back_stack: Vec<HistoryEntry>,
    forward_stack: Vec<HistoryEntry>,
    selected: usize,
    hitboxes: Vec<Hitbox>,
    field_values: HashMap<String, String>,
    checkbox_states: HashMap<String, bool>,
    radio_states: HashMap<String, String>,
    editing_field: Option<usize>,
    width: u16,
    height: u16,
    content_height: u16,
    renderer: R,
    cached_output: Option<R::Output>,
    render_dirty: bool,
}

pub trait Renderer {
    type Output;
    fn render(&self, doc: &Document, width: u16, form_state: &FormState) -> RenderOutput<Self::Output>;
}

pub struct RenderOutput<T> {
    pub content: T,
    pub hitboxes: Vec<Hitbox>,
    pub height: u16,
}

impl<R: Renderer> Browser<R> {
    pub fn new(renderer: R) -> Self {
        Self {
            url: None,
            content: None,
            scroll: 0,
            back_stack: Vec::new(),
            forward_stack: Vec::new(),
            selected: 0,
            hitboxes: Vec::new(),
            field_values: HashMap::new(),
            checkbox_states: HashMap::new(),
            radio_states: HashMap::new(),
            editing_field: None,
            width: 80,
            height: 24,
            content_height: 0,
            renderer,
            cached_output: None,
            render_dirty: false,
        }
    }

    pub fn set_content(&mut self, url: &str, content: &str) {
        if let (Some(old_url), Some(old_content)) = (self.url.take(), self.content.take()) {
            self.back_stack.push(HistoryEntry {
                url: old_url,
                content: old_content,
                scroll: self.scroll,
            });
        }
        self.forward_stack.clear();
        self.url = Some(url.to_string());
        self.content = Some(content.to_string());
        self.scroll = 0;
        self.clear_form_state();
        self.rebuild();
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    fn clear_form_state(&mut self) {
        self.field_values.clear();
        self.checkbox_states.clear();
        self.radio_states.clear();
        self.selected = 0;
        self.editing_field = None;
    }

    fn form_state(&self) -> FormState {
        FormState {
            fields: self.field_values.clone(),
            checkboxes: self.checkbox_states.clone(),
            radios: self.radio_states.clone(),
            editing_field: self.editing_field.and_then(|idx| {
                self.hitboxes.get(idx).and_then(|hb| match &hb.target {
                    HitboxTarget::TextField { name, .. } => Some(name.clone()),
                    _ => None,
                })
            }),
        }
    }

    fn rebuild(&mut self) {
        let Some(ref content) = self.content else {
            self.hitboxes.clear();
            self.content_height = 0;
            self.cached_output = None;
            self.render_dirty = false;
            return;
        };

        let doc = parse(content);
        let output = self.renderer.render(&doc, self.width, &self.form_state());
        self.hitboxes = output.hitboxes;
        self.content_height = output.height;
        self.cached_output = Some(output.content);
        self.render_dirty = false;

        for hitbox in &self.hitboxes {
            match &hitbox.target {
                HitboxTarget::TextField { name, default, .. } => {
                    self.field_values.entry(name.clone()).or_insert_with(|| default.clone());
                }
                HitboxTarget::Checkbox { name } => {
                    self.checkbox_states.entry(name.clone()).or_insert(false);
                }
                HitboxTarget::Radio { name, value } => {
                    self.radio_states
                        .entry(name.clone())
                        .or_insert_with(|| value.clone());
                }
                HitboxTarget::Link { .. } => {}
            }
        }
    }

    fn rerender(&mut self) {
        let Some(ref content) = self.content else {
            return;
        };
        let doc = parse(content);
        let output = self.renderer.render(&doc, self.width, &self.form_state());
        self.cached_output = Some(output.content);
        self.render_dirty = false;
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        let width_changed = self.width != width;
        self.width = width;
        self.height = height;
        if width_changed && self.content.is_some() {
            self.rebuild();
        }
    }

    pub fn render(&mut self) -> Option<&R::Output> {
        if self.render_dirty {
            self.rerender();
        }
        self.cached_output.as_ref()
    }

    pub fn back(&mut self) -> bool {
        let Some(entry) = self.back_stack.pop() else {
            return false;
        };
        if let (Some(url), Some(content)) = (self.url.take(), self.content.take()) {
            self.forward_stack.push(HistoryEntry {
                url,
                content,
                scroll: self.scroll,
            });
        }
        self.url = Some(entry.url);
        self.content = Some(entry.content);
        self.scroll = entry.scroll;
        self.clear_form_state();
        self.rebuild();
        true
    }

    pub fn forward(&mut self) -> bool {
        let Some(entry) = self.forward_stack.pop() else {
            return false;
        };
        if let (Some(url), Some(content)) = (self.url.take(), self.content.take()) {
            self.back_stack.push(HistoryEntry {
                url,
                content,
                scroll: self.scroll,
            });
        }
        self.url = Some(entry.url);
        self.content = Some(entry.content);
        self.scroll = entry.scroll;
        self.clear_form_state();
        self.rebuild();
        true
    }

    pub fn can_go_back(&self) -> bool {
        !self.back_stack.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward_stack.is_empty()
    }

    pub fn scroll_to(&mut self, y: u16) {
        let max = self.content_height.saturating_sub(self.height);
        self.scroll = y.min(max);
    }

    pub fn scroll_by(&mut self, delta: i32) {
        let new = (self.scroll as i32).saturating_add(delta);
        self.scroll_to(new.max(0) as u16);
    }

    pub fn scroll(&self) -> u16 {
        self.scroll
    }

    pub fn select_next(&mut self) {
        if !self.hitboxes.is_empty() {
            self.selected = (self.selected + 1) % self.hitboxes.len();
            self.ensure_selected_visible();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.hitboxes.is_empty() {
            self.selected = self
                .selected
                .checked_sub(1)
                .unwrap_or(self.hitboxes.len() - 1);
            self.ensure_selected_visible();
        }
    }

    fn ensure_selected_visible(&mut self) {
        if let Some(hitbox) = self.hitboxes.get(self.selected) {
            let line = hitbox.line as u16;
            if line < self.scroll {
                self.scroll = line;
            } else if line >= self.scroll + self.height {
                self.scroll = line.saturating_sub(self.height) + 1;
            }
        }
    }

    pub fn interact(&mut self) -> Option<Link> {
        let hitbox = self.hitboxes.get(self.selected)?;

        match &hitbox.target {
            HitboxTarget::Link { url, fields } => Some(Link {
                url: url.clone(),
                fields: fields.clone(),
                form_data: self.collect_form_data(fields),
            }),
            HitboxTarget::TextField { .. } => {
                self.editing_field = Some(self.selected);
                self.render_dirty = true;
                None
            }
            HitboxTarget::Checkbox { name } => {
                let current = self.checkbox_states.get(name).copied().unwrap_or(false);
                self.checkbox_states.insert(name.clone(), !current);
                self.render_dirty = true;
                None
            }
            HitboxTarget::Radio { name, value } => {
                self.radio_states.insert(name.clone(), value.clone());
                self.render_dirty = true;
                None
            }
        }
    }

    pub fn click(&mut self, x: u16, y: u16) -> Option<Link> {
        let doc_y = (y as usize).saturating_add(self.scroll as usize);
        let doc_x = x as usize;

        for (idx, hitbox) in self.hitboxes.iter().enumerate() {
            if hitbox.line == doc_y && doc_x >= hitbox.col_start && doc_x < hitbox.col_end {
                self.selected = idx;
                return self.interact();
            }
        }
        
        if self.editing_field.is_some() {
            self.editing_field = None;
            self.render_dirty = true;
        }
        None
    }

    fn collect_form_data(&self, field_specs: &[String]) -> HashMap<String, String> {
        let mut data = HashMap::new();
        if field_specs.is_empty() {
            return data;
        }

        let include_all = field_specs.iter().any(|f| f == "*");
        let mut requested: Vec<&str> = Vec::new();

        for spec in field_specs {
            if let Some((key, value)) = spec.split_once('=') {
                data.insert(key.to_string(), value.to_string());
            } else if spec != "*" {
                requested.push(spec);
            }
        }

        for (name, value) in &self.field_values {
            if include_all || requested.iter().any(|f| f == name) {
                data.insert(name.clone(), value.clone());
            }
        }

        for (name, checked) in &self.checkbox_states {
            if *checked && (include_all || requested.iter().any(|f| f == name)) {
                data.insert(name.clone(), "1".to_string());
            }
        }

        for (name, value) in &self.radio_states {
            if include_all || requested.iter().any(|f| f == name) {
                data.insert(name.clone(), value.clone());
            }
        }

        data
    }

    pub fn is_editing(&self) -> bool {
        self.editing_field.is_some()
    }

    pub fn cancel_edit(&mut self) {
        if self.editing_field.is_some() {
            self.editing_field = None;
            self.render_dirty = true;
        }
    }

    pub fn input_char(&mut self, c: char) -> InputResult {
        let Some(idx) = self.editing_field else {
            return InputResult::Ignored;
        };
        let Some(hitbox) = self.hitboxes.get(idx) else {
            return InputResult::Ignored;
        };
        if let HitboxTarget::TextField { name, .. } = &hitbox.target {
            self.field_values.entry(name.clone()).or_default().push(c);
            self.render_dirty = true;
            InputResult::Consumed
        } else {
            InputResult::Ignored
        }
    }

    pub fn input_backspace(&mut self) -> InputResult {
        let Some(idx) = self.editing_field else {
            return InputResult::Ignored;
        };
        let Some(hitbox) = self.hitboxes.get(idx) else {
            return InputResult::Ignored;
        };
        if let HitboxTarget::TextField { name, .. } = &hitbox.target {
            if let Some(val) = self.field_values.get_mut(name) {
                val.pop();
                self.render_dirty = true;
            }
            InputResult::Consumed
        } else {
            InputResult::Ignored
        }
    }

    pub fn selected_link(&self) -> Option<&str> {
        let hitbox = self.hitboxes.get(self.selected)?;
        match &hitbox.target {
            HitboxTarget::Link { url, .. } => Some(url),
            _ => None,
        }
    }

    pub fn has_content(&self) -> bool {
        self.content.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::micronaut::ast::{Element, FieldKind};

    struct NullRenderer;

    impl Renderer for NullRenderer {
        type Output = ();

        fn render(&self, doc: &Document, _width: u16, _form_state: &FormState) -> RenderOutput<()> {
            let mut hitboxes = Vec::new();
            for (line_idx, line) in doc.lines.iter().enumerate() {
                let mut col = 0;
                for element in &line.elements {
                    match element {
                        Element::Link(link) => {
                            let len = link.label.len();
                            hitboxes.push(Hitbox {
                                line: line_idx,
                                col_start: col,
                                col_end: col + len,
                                target: HitboxTarget::Link {
                                    url: link.url.clone(),
                                    fields: link.fields.clone(),
                                },
                            });
                            col += len;
                        }
                        Element::Field(field) => {
                            let len = 24;
                            let target = match &field.kind {
                                FieldKind::Text => HitboxTarget::TextField {
                                    name: field.name.clone(),
                                    masked: field.masked,
                                    default: field.default.clone(),
                                },
                                FieldKind::Checkbox { .. } => {
                                    HitboxTarget::Checkbox {
                                        name: field.name.clone(),
                                    }
                                }
                                FieldKind::Radio { value, .. } => {
                                    HitboxTarget::Radio {
                                        name: field.name.clone(),
                                        value: value.clone(),
                                    }
                                }
                            };
                            hitboxes.push(Hitbox {
                                line: line_idx,
                                col_start: col,
                                col_end: col + len,
                                target,
                            });
                            col += len;
                        }
                        Element::Text(t) => {
                            col += t.text.len();
                        }
                        Element::Partial(_) => {}
                    }
                }
            }
            RenderOutput {
                content: (),
                hitboxes,
                height: doc.lines.len() as u16,
            }
        }
    }

    fn form_state(browser: &mut Browser<NullRenderer>) -> FormState {
        browser.render();
        FormState {
            fields: browser.field_values.clone(),
            checkboxes: browser.checkbox_states.clone(),
            radios: browser.radio_states.clone(),
            editing_field: browser.editing_field.and_then(|idx| {
                browser.hitboxes.get(idx).and_then(|hb| match &hb.target {
                    HitboxTarget::TextField { name, .. } => Some(name.clone()),
                    _ => None,
                })
            }),
        }
    }

    #[test]
    fn initial_state() {
        let browser = Browser::new(NullRenderer);
        assert!(!browser.has_content());
        assert!(browser.url().is_none());
        assert!(!browser.can_go_back());
        assert!(!browser.can_go_forward());
    }

    #[test]
    fn set_content_and_back() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/page1", "Page 1");
        assert_eq!(browser.url(), Some("/page1"));
        
        browser.set_content("/page2", "Page 2");
        assert_eq!(browser.url(), Some("/page2"));
        assert!(browser.can_go_back());
        assert!(!browser.can_go_forward());
        
        browser.back();
        assert_eq!(browser.url(), Some("/page1"));
        assert!(browser.can_go_forward());
        
        browser.forward();
        assert_eq!(browser.url(), Some("/page2"));
        assert!(!browser.can_go_forward());
    }

    #[test]
    fn scroll() {
        let mut browser = Browser::new(NullRenderer);
        browser.resize(80, 10);
        browser.set_content("/test", "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl\nm\nn\no");
        
        browser.scroll_by(5);
        assert_eq!(browser.scroll(), 5);
        
        browser.scroll_by(-3);
        assert_eq!(browser.scroll(), 2);
        
        browser.scroll_to(0);
        assert_eq!(browser.scroll(), 0);
    }

    #[test]
    fn click_link() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/test", "`[Click Me`http://target]");
        
        let link = browser.click(3, 0);
        assert!(link.is_some());
        assert_eq!(link.unwrap().url, "http://target");
    }

    #[test]
    fn checkbox_toggle() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/test", "`<?|agree|yes`I agree>");
        
        assert!(!form_state(&mut browser).checkboxes.get("agree").copied().unwrap_or(false));
        browser.interact();
        assert!(form_state(&mut browser).checkboxes.get("agree").copied().unwrap_or(false));
    }

    #[test]
    fn text_input() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/test", "`<|name`>");
        
        browser.interact();
        assert!(browser.is_editing());
        
        browser.input_char('H');
        browser.input_char('i');
        assert_eq!(form_state(&mut browser).fields.get("name"), Some(&"Hi".to_string()));
        
        browser.input_backspace();
        assert_eq!(form_state(&mut browser).fields.get("name"), Some(&"H".to_string()));
    }

    #[test]
    fn text_field_with_default() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/test", "`<|name`John>");
        
        assert_eq!(form_state(&mut browser).fields.get("name"), Some(&"John".to_string()));
        
        browser.interact();
        browser.input_char('!');
        assert_eq!(form_state(&mut browser).fields.get("name"), Some(&"John!".to_string()));
    }

    #[test]
    fn form_data_collection() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/test", "`<|user`>\n`<|msg`>\n`[Submit`/send`user|msg|action=go]");
        
        browser.interact();
        browser.input_char('A');
        browser.cancel_edit();
        
        browser.select_next();
        browser.interact();
        browser.input_char('B');
        browser.cancel_edit();
        
        browser.select_next();
        let link = browser.interact().unwrap();
        
        assert_eq!(link.url, "/send");
        assert_eq!(link.form_data.get("user"), Some(&"A".to_string()));
        assert_eq!(link.form_data.get("msg"), Some(&"B".to_string()));
        assert_eq!(link.form_data.get("action"), Some(&"go".to_string()));
    }

    #[test]
    fn select_next_prev_cycles() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/test", "`[A`/a]\n`[B`/b]\n`[C`/c]");
        
        assert_eq!(browser.selected_link(), Some("/a"));
        browser.select_next();
        assert_eq!(browser.selected_link(), Some("/b"));
        browser.select_next();
        assert_eq!(browser.selected_link(), Some("/c"));
        browser.select_next();
        assert_eq!(browser.selected_link(), Some("/a"));
        
        browser.select_prev();
        assert_eq!(browser.selected_link(), Some("/c"));
    }

    #[test]
    fn radio_button_selection() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/test", "`<^|color|red`Red>\n`<^|color|blue`Blue>\n`<^|color|green`Green>");
        
        assert_eq!(form_state(&mut browser).radios.get("color"), Some(&"red".to_string()));
        
        browser.select_next();
        browser.interact();
        assert_eq!(form_state(&mut browser).radios.get("color"), Some(&"blue".to_string()));
        
        browser.select_next();
        browser.interact();
        assert_eq!(form_state(&mut browser).radios.get("color"), Some(&"green".to_string()));
    }

    #[test]
    fn resize_triggers_rebuild() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/test", "Hello world");
        browser.render();
        
        browser.resize(40, 20);
        assert!(browser.render().is_some());
    }

    #[test]
    fn navigation_clears_form_state() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/page1", "`<|name`>");
        
        browser.interact();
        browser.input_char('X');
        browser.cancel_edit();
        assert_eq!(form_state(&mut browser).fields.get("name"), Some(&"X".to_string()));
        
        browser.set_content("/page2", "`<|name`>");
        assert_eq!(form_state(&mut browser).fields.get("name"), Some(&"".to_string()));
    }

    #[test]
    fn back_preserves_scroll_position() {
        let mut browser = Browser::new(NullRenderer);
        browser.resize(80, 10);
        browser.set_content("/page1", "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl\nm\nn\no");
        browser.scroll_to(5);
        
        browser.set_content("/page2", "Page 2");
        assert_eq!(browser.scroll(), 0);
        
        browser.back();
        assert_eq!(browser.scroll(), 5);
    }

    #[test]
    fn directory_navigation_simulation() {
        let mut browser = Browser::new(NullRenderer);
        
        let index = r#">Welcome
`[Documents`/docs]
`[Settings`/settings]
`[About`/about]"#;
        
        let docs = r#">Documents
`[Back`/]
-
`[Report.pdf`/docs/report]
`[Notes.txt`/docs/notes]"#;
        
        let report = r#">Report
`[Back to Documents`/docs]
-
This is the report content."#;
        
        browser.set_content("/", index);
        assert_eq!(browser.url(), Some("/"));
        assert_eq!(browser.selected_link(), Some("/docs"));
        
        let link = browser.interact().unwrap();
        assert_eq!(link.url, "/docs");
        browser.set_content("/docs", docs);
        assert_eq!(browser.url(), Some("/docs"));
        
        browser.select_next();
        assert_eq!(browser.selected_link(), Some("/docs/report"));
        
        let link = browser.interact().unwrap();
        browser.set_content(&link.url, report);
        assert_eq!(browser.url(), Some("/docs/report"));
        
        assert!(browser.can_go_back());
        browser.back();
        assert_eq!(browser.url(), Some("/docs"));
        
        browser.back();
        assert_eq!(browser.url(), Some("/"));
        
        browser.forward();
        assert_eq!(browser.url(), Some("/docs"));
    }

    #[test]
    fn login_form_simulation() {
        let mut browser = Browser::new(NullRenderer);
        
        let login_page = r#">Login
`<|username`>
`<!|password`>
`[Login`/auth`username|password]"#;
        
        browser.set_content("/login", login_page);
        
        browser.interact();
        for c in "alice".chars() {
            browser.input_char(c);
        }
        browser.cancel_edit();
        
        browser.select_next();
        browser.interact();
        for c in "secret123".chars() {
            browser.input_char(c);
        }
        browser.cancel_edit();
        
        browser.select_next();
        let link = browser.interact().unwrap();
        
        assert_eq!(link.url, "/auth");
        assert_eq!(link.form_data.get("username"), Some(&"alice".to_string()));
        assert_eq!(link.form_data.get("password"), Some(&"secret123".to_string()));
    }

    #[test]
    fn search_with_wildcard_fields() {
        let mut browser = Browser::new(NullRenderer);
        
        let search_page = r#"`<|query`>
`<?|exact|1`Exact match>
`[Search`/search`*]"#;
        
        browser.set_content("/search", search_page);
        
        browser.interact();
        for c in "rust".chars() {
            browser.input_char(c);
        }
        browser.cancel_edit();
        
        browser.select_next();
        browser.interact();
        
        browser.select_next();
        let link = browser.interact().unwrap();
        
        assert_eq!(link.url, "/search");
        assert_eq!(link.form_data.get("query"), Some(&"rust".to_string()));
        assert_eq!(link.form_data.get("exact"), Some(&"1".to_string()));
    }

    #[test]
    fn empty_content_handling() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/empty", "");
        
        assert!(browser.has_content());
        assert!(browser.render().is_some());
        assert!(browser.selected_link().is_none());
        
        browser.select_next();
        browser.select_prev();
        assert!(browser.interact().is_none());
    }

    #[test]
    fn input_ignored_when_not_editing() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/test", "`[Link`/target]");
        
        assert_eq!(browser.input_char('x'), InputResult::Ignored);
        assert_eq!(browser.input_backspace(), InputResult::Ignored);
    }

    #[test]
    fn cancel_edit_when_not_editing() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/test", "`<|name`>");
        
        browser.cancel_edit();
        assert!(!browser.is_editing());
    }

    #[test]
    fn click_outside_hitbox() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/test", "`[Link`/target]");
        
        let result = browser.click(100, 100);
        assert!(result.is_none());
    }

    #[test]
    fn click_outside_defocuses_field() {
        let mut browser = Browser::new(NullRenderer);
        browser.set_content("/test", "`<|name`Enter name>");
        
        browser.interact();
        assert!(browser.is_editing());
        
        browser.click(100, 100);
        assert!(!browser.is_editing());
    }

    #[test]
    fn multiple_back_forward() {
        let mut browser = Browser::new(NullRenderer);
        
        browser.set_content("/a", "A");
        browser.set_content("/b", "B");
        browser.set_content("/c", "C");
        browser.set_content("/d", "D");
        
        assert_eq!(browser.url(), Some("/d"));
        
        browser.back();
        browser.back();
        assert_eq!(browser.url(), Some("/b"));
        
        browser.forward();
        assert_eq!(browser.url(), Some("/c"));
        
        browser.set_content("/e", "E");
        assert!(!browser.can_go_forward());
        assert_eq!(browser.url(), Some("/e"));
        
        browser.back();
        assert_eq!(browser.url(), Some("/c"));
    }
}
