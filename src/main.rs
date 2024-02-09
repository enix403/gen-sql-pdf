#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use std::error::Error;
use std::path::{Path, PathBuf};

use clap::Parser;

pub mod database;
pub mod input;
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
}

#[derive(Debug)]
pub struct Environment {
    pub dbfile: PathBuf,
    pub infile: PathBuf,
    pub data_dir: PathBuf,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut cwd = std::env::current_dir()?;

    let env = Environment {
        dbfile: args.db,
        infile: args.infile,
        data_dir: cwd.join("data"),
    };

    let mut engine = render::Engine::new(&env).await;

    let sql = r#"
        select
            ename,
            sal,
            sal + comm as "total salary",
            sal * 1.25 as "new salary",
            sal * 1.25 - sal as "bonus",
            sal * 2 as "more bonus"
        from
            emp
    "#;

    engine.process_query(sql).await?;

    engine.close().await;
    // engine.event_task_handle.await;

    Ok(())
}
