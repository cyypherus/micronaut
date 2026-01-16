
<div align="center">

# micronaut

**A rendering agnostic nomadnet micron parser & browser**

</div>

Micron is a markup webpage format spearheaded by [nomadnet](https://github.com/markqvist/NomadNet)

There is no official spec that I know of, but the reference implementation exists [in the nomadnet repo](https://github.com/markqvist/NomadNet/blob/master/nomadnet/ui/textui/MicronParser.py) and many micron pages are served over nomadnet itself.
There's a micron playground implemented [here](https://github.com/RFnexus/micron-parser-js)

# Features
- micronaut implements a standalone parser by default
- micronaut also _optionally_ implements a minimal browser implementation behind the `browser` feature.
- micronaut also _optionally_ implements a [ratatui](https://github.com/ratatui/ratatui) renderer, converting a parsed micron document into a ratatui widget for display in ratatui TUIs

# Parser
```rust
    let doc = micronaut::parse(include_str!("my_page.mu"))
    for line in doc.lines {
        dbg!(line);
    }
```

# Browser + Ratatui
```rust
    // Create the browser with a renderer
    let mut browser = Browser::new(RatatuiRenderer);
    // Pass the url, and the micron markup at that url
    browser.set_content("file://example.mu", &content);

    // Now we can render inside the draw loop
    // this is what rendering looks like using the built in ratatui feature
    let area = frame.area();
    self.browser.resize(area.width, area.height);
    let scroll = self.browser.scroll();
    if let Some(content) = self.browser.render().cloned() {
        Paragraph::new(content)
            .scroll((scroll, 0))
            .render(area, buf);
    } else {
        let content = Text::styled("No content", Style::default().fg(Color::DarkGray));
        Paragraph::new(content)
            .alignment(ratatui::layout::Alignment::Center)
            .render(area, buf);
    }
```

The `Renderer` trait's responsibility is _solely_ to turn a `Document` into something that you know how to show on your screen.
> *Do note: micronaut is not really concerned with how you get things onto your screen, one could implement an alternative HTML "renderer" for converting a micronaut Document to HTML & display micron in a standard browser*

The `Browser` struct handles simple, common browser functionality like forward / backward, rerendering and caching, scroll state, field interactions, and clicking.

