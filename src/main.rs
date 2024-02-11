#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use std::error::Error;
use std::path::PathBuf;

use clap::Parser;

pub mod database;
pub mod input;
pub mod pdfgen;
pub mod render;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path of SQLite database file
    #[arg(short, long)]
    db: PathBuf,

    /// Path of input file containing SQL statements
    #[arg(short, long)]
    infile: PathBuf,

    /// Theme
    #[arg(short, long)]
    theme: Option<String>,
}

#[derive(Debug)]
pub struct Environment {
    pub dbfile: PathBuf,
    pub infile: PathBuf,
    pub theme: String,

    pub data_dir: PathBuf,
    pub themes_dir: PathBuf,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let cwd = std::env::current_dir()?;

    let env = Environment {
        dbfile: args.db,
        infile: args.infile,
        theme: args.theme.unwrap_or_else(|| "default-light".to_string()),
        data_dir: cwd.join("data"),
        themes_dir: cwd.join("themes"),
    };

    let mut renderer = render::Renderer::new(&env).await;
    let qlist = input::QueryList::read_from_file(&env.infile);
    pdfgen::create_pdf(qlist, &mut renderer, cwd.join("output.pdf")).await;
    // pdfgen::save_image(
    //     &renderer.render_query(&qlist.queries[0]).await.unwrap(),
    //     "example.jpg",
    // );

    renderer.dispose().await;

    Ok(())
}
