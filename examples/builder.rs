use micronaut::{Color, Document, Field, Line, LinkElement, Style};
use std::io::Write;

fn main() {
    let path = std::env::args().nth(1).expect("Usage: builder <path>");

    let doc = build_page();
    let content = doc.to_string();

    let mut file = std::fs::File::create(&path).expect("Failed to create file");
    file.write_all(content.as_bytes())
        .expect("Failed to write file");

    println!("Wrote {} bytes to {}", content.len(), path);
}

fn build_page() -> Document {
    let mut doc = Document::new();

    doc.push(Line::heading(1).text("Welcome to Micronaut"));
    doc.push(Line::normal());
    doc.push(Line::normal().text("Edit build_page() and save - changes appear instantly!"));
    doc.push(Line::normal());
    doc.push(Line::divider());
    doc.push(Line::normal());

    doc.push(Line::heading(2).text("Styled Text"));
    doc.push(
        Line::normal()
            .text("You can have ")
            .bold("bold")
            .text(", ")
            .italic("italic")
            .text(", and ")
            .underline("underlined")
            .text(" text."),
    );
    doc.push(Line::normal());

    doc.push(Line::heading(2).text("Colors"));
    doc.push(
        Line::normal()
            .styled("Red ", Style::new().fg(Color::hex(0xFF0000)))
            .styled("Green ", Style::new().fg(Color::hex(0x00FF00)))
            .styled("Blue ", Style::new().fg(Color::hex(0x0000FF)))
            .styled("Yellow ", Style::new().fg(Color::hex(0xFFFF00)))
            .styled("Magenta", Style::new().fg(Color::hex(0xFF00FF))),
    );
    doc.push(Line::normal());

    doc.push(Line::heading(2).text("Links"));
    doc.push(
        Line::normal()
            .text("Visit ")
            .link(LinkElement::new("/page/about.mu").label("About"))
            .text(" or ")
            .link(LinkElement::new("/page/help.mu").label("Help")),
    );
    doc.push(Line::normal());

    doc.push(Line::heading(2).text("Form Fields"));
    doc.push(
        Line::normal()
            .text("Username: ")
            .field(Field::text("username").width(20)),
    );
    doc.push(
        Line::normal()
            .text("Password: ")
            .field(Field::password("password").width(20)),
    );
    doc.push(Line::normal());

    doc.push(Line::divider());
    doc.push(Line::normal().center().styled(
        "Built with Micronaut Builder API",
        Style::new().fg(Color::gray(50)),
    ));

    doc
}
