#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use std::convert::TryFrom;

use genpdf::{elements, fonts, Document};

fn main() {
    let font_family = genpdf::fonts::from_files("themes/fonts/Cousine", "Cousine", None).unwrap();
    let mut doc = genpdf::Document::new(font_family);
    doc.set_title("Demo document");

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    doc.push(
        elements::Image::from_path("example.jpg")
            .expect("Failed to load test image")
            .with_alignment(genpdf::Alignment::Center)
            .with_scale(genpdf::Scale::new(1.2, 1.2)),
    );

    doc.push(elements::PageBreak::new());

    doc.push(
        elements::Image::from_path("example.jpg")
            .expect("Failed to load test image")
            .with_alignment(genpdf::Alignment::Center)
            .with_scale(genpdf::Scale::new(1.2, 1.2)),
    );

    doc.render_to_file("output.pdf")
        .expect("Failed to write PDF file");
}
