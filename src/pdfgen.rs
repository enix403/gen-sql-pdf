use std::io::Cursor;
use std::path::Path;

use crate::input::QueryList;
use crate::render::Renderer;

use genpdf::{elements, fonts, Alignment, Document, Scale, SimplePageDecorator};

pub async fn create_pdf<P: AsRef<Path>>(qlist: QueryList, renderer: &mut Renderer<'_>, outfile: P) {
    let font_family = genpdf::fonts::from_files("themes/fonts/Cousine", "Cousine", None).unwrap();
    let mut doc = genpdf::Document::new(font_family);

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    for (i, sql) in qlist.queries.iter().enumerate() {
        if i != 0 {
            doc.push(elements::PageBreak::new());
        }

        let mut image = renderer
            .render_query(sql)
            .await
            .expect("Renderer::render_query(...) error");

        let image_elem = elements::Image::from_reader(Cursor::new(image))
            .expect("Failed to load page image")
            .with_alignment(Alignment::Center)
            .with_scale(Scale::new(1.2, 1.2));

        doc.push(image_elem);
    }

    doc.render_to_file(outfile)
        .expect("Failed to write PDF file");
}
