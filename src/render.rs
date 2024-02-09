use chromiumoxide::{
    cdp::browser_protocol::page::{CaptureScreenshotFormat, PrintToPdfParams},
    page::ScreenshotParams,
    Browser, BrowserConfig, Handler,
};
use futures::StreamExt;
use std::path::{Path, PathBuf};

use async_std::task::JoinHandle;

use tera::{Context as TeraContext, Tera};

pub use crate::database::{self, Connection, QueryAnswer};
pub use crate::Environment;

const WINDOW_SIZE: (u32, u32) = (1892, 1027);

pub struct Engine {
    browser: Browser,
    tera: Tera,
    asset_root: PathBuf,
    index_file: String,
    conn: Connection,
}

impl Engine {

    #[cfg(target_os = "linux")]
    fn get_chrome_exe(env: &Environment) -> PathBuf {
        let mut exe = env.data_dir.clone();
        exe.push("chrome-linux64");
        exe.push("chrome");
        exe
    }

    // #[cfg(target_os = "windows")]
    // fn get_chrome_exe(env: &Environment) -> PathBuf {
    //     let mut exe = env.data_dir.clone();
    //     exe.push("chrome-linux64");
    //     exe.push("chrome");
    //     exe
    // }

    async fn open_browser(env: &Environment) -> (Browser, JoinHandle<()>) {
        let exe = Self::get_chrome_exe(env);

        let (mut browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .chrome_executable(exe)
                // .with_head()
                .window_size(WINDOW_SIZE.0, WINDOW_SIZE.1)
                .viewport(None)
                .build()
                .expect("Failed to launch browser config"),
        )
        .await
        .expect("Failed to launch browser config");

        let handler = async_std::task::spawn(async move {
            //
            while let Some(_) = handler.next().await {}
        });

        (browser, handler)
    }

    pub async fn new(env: &Environment) -> Self {
        // Setup browser
        let (mut browser, mut handler) = Self::open_browser(env).await;

        // Setup Tera template system
        let templates_dir = env.data_dir.join("templates");

        let mut tera = Tera::default();
        tera.add_template_file(templates_dir.join("table.html"), Some("table"))
            .expect("Failed to compile template");

        let index_file = templates_dir.join("empty.html");
        let index_file = format!(
            "file://{}",
            index_file.into_os_string().into_string().unwrap()
        );

        // Setup Sqlite Connection
        let conn = database::create_connection(&env.dbfile);

        Self {
            browser,
            tera,
            asset_root: templates_dir,
            index_file,
            conn,
        }
    }

    pub async fn process_query(&mut self, sql: &str) -> Result<(), Box<dyn std::error::Error>> {
        let query = QueryAnswer::from_sql(sql, &self.conn);

        let mut context = TeraContext::new();
        context.insert("headers", &query.headers);
        context.insert("rows", &query.rows);
        context.insert("ASSET_ROOT", &self.asset_root);

        let html = self
            .tera
            .render("table", &context)
            .expect("Failed to render template");

        let page = self.browser.new_page(&self.index_file).await?;
        // println!("{}", html);
        page.set_content(html).await?;
        page.wait_for_navigation().await?;

        page.save_screenshot(
            ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .full_page(false)
                .omit_background(false)
                .build(),
            "example.png",
        )
        .await?;

        page.close().await?;

        Ok(())
    }

    pub async fn close(&mut self) {
        self.browser.close().await.expect("Failed to close browser");
    }
}