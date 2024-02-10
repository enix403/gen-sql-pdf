use std::io::Cursor;

use crate::input::QueryList;
use crate::render::Engine;

use genpdf::{elements, fonts, Document};

pub async fn create_pdf(qlist: QueryList, engine: &mut Engine<'_>) {
    let font_family = genpdf::fonts::from_files("themes/fonts/Cousine", "Cousine", None).unwrap();
    let mut doc = genpdf::Document::new(font_family);
    doc.set_title("Demo document");

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    for (i, sql) in qlist.queries.iter().enumerate() {
        if i != 0 {
            doc.push(elements::PageBreak::new());
        }

        let mut image = engine
            .process_query(sql)
            .await
            .expect("Engine::process_query(...) error");

        let image = Cursor::new(image);

        let image_elem = elements::Image::from_reader(image)
            .expect("Failed to load page image")
            .with_alignment(genpdf::Alignment::Center)
            .with_scale(genpdf::Scale::new(1.2, 1.2));

        doc.push(image_elem);
    }

    doc.render_to_file("output.pdf")
        .expect("Failed to write PDF file");
}
