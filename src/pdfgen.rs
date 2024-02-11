use std::fs;
use std::io::Cursor;
use std::io::Write;
use std::path::Path;

use std::default::Default;

use crate::input::QueryList;
use crate::render::Renderer;

use genpdf::{elements, fonts, Alignment, Document, Scale, SimplePageDecorator};
use oxipng::{optimize_from_memory, Options as OptimizeOptions, StripChunks};

pub fn save_image<D: AsRef<[u8]>, P: AsRef<Path>>(data: D, outfile: P) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(outfile)
        .expect("Failed to open file");

    file.write_all(data.as_ref()).expect("Failed to save file");
}

pub async fn create_pdf<P: AsRef<Path>>(qlist: QueryList, renderer: &mut Renderer<'_>, outfile: P) {
    let font_family = genpdf::fonts::from_files("themes/fonts/Cousine", "Cousine", None).unwrap();
    let mut doc = genpdf::Document::new(font_family);

    let total = qlist.queries.len();

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

        // Command:
        //      oxipng -o 2 --strip all --alpha --scale16 input.png --out output.png
        // let optimized = optimize_from_memory(&image, &OptimizeOptions {
        //     strip: oxipng::StripChunks::All,
        //     optimize_alpha: true,
        //     fast_evaluation: true,
        //     scale_16: true,
        //     ..OptimizeOptions::from_preset(2)
        // }).expect("Failed to optimize image");
        let optimized = image;

        println!("Processed query {}/{}", i + 1, total);

        let image_elem = elements::Image::from_reader(Cursor::new(optimized))
            .expect("Failed to load page image")
            .with_alignment(Alignment::Center)
            .with_scale(Scale::new(1.2, 1.2));

        doc.push(image_elem);
    }

    doc.render_to_file(outfile)
        .expect("Failed to write PDF file");
}
