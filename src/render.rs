use async_std::task::JoinHandle;
use chromiumoxide::{
    cdp::browser_protocol::page::CaptureScreenshotFormat, page::ScreenshotParams, Browser,
    BrowserConfig,
};
use futures::StreamExt;
use std::path::PathBuf;
use tera::{Context as TeraContext, Tera};

use sqlformat::{format as format_sql, FormatOptions, Indent, QueryParams};

pub use crate::database::{self, Connection, QueryAnswer};
pub use crate::Environment;

// const WINDOW_SIZE: (u32, u32) = (1892, 1027);
const WINDOW_SIZE: (u32, u32) = (1892, 2054);
// const WINDOW_SIZE: (u32, u32) = (1516, 822);

pub const _TEMP_KEEP_BROWSER_OPEN: bool = false;

pub struct Renderer<'a> {
    env: &'a Environment,
    browser: Browser,
    tera: Tera,
    index_file: String,
    conn: Connection,
    pub event_task_handle: JoinHandle<()>,
}

impl<'a> Renderer<'a> {
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

        let (browser, mut handler) = Browser::launch({
            let mut builder = BrowserConfig::builder()
                .chrome_executable(exe)
                .window_size(WINDOW_SIZE.0, WINDOW_SIZE.1)
                .viewport(None);

            if _TEMP_KEEP_BROWSER_OPEN {
                builder = builder.with_head();
            }

            builder.build().expect("Failed to build browser config")
        })
        .await
        .expect("Failed to launch browser config");

        let task = async_std::task::spawn(async move {
            //
            while let Some(_) = handler.next().await {}
        });

        (browser, task)
    }

    pub async fn new(env: &'a Environment) -> Self {
        // Setup browser
        let (browser, task) = Self::open_browser(env).await;

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
            env,
            browser,
            tera,
            index_file,
            conn,
            event_task_handle: task,
        }
    }

    pub async fn render_query(&mut self, sql: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let query = QueryAnswer::from_sql(sql, &self.conn);

        let mut context = TeraContext::new();
        context.insert("DATA_DIR", &self.env.data_dir);
        context.insert("THEMES_DIR", &self.env.themes_dir);
        context.insert("theme", &self.env.theme);

        context.insert("sql", {
            format_sql(
                sql,
                &QueryParams::None,
                FormatOptions {
                    indent: Indent::Spaces(4),
                    uppercase: true,
                    lines_between_queries: 2,
                },
            )
            .as_str()
        });
        context.insert("headers", &query.headers);
        context.insert("rows", &query.rows);

        let html = self
            .tera
            .render("table", &context)
            .expect("Failed to render template");

        let page = self.browser.new_page(&self.index_file).await?;
        // println!("{}", html);
        page.set_content(html).await?;
        page.wait_for_navigation().await?;

        let image = page.screenshot(
            ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Jpeg)
                .quality(100)
                .full_page(false)
                // .omit_background(false)
                .build(),
        )
        .await?;

        if !_TEMP_KEEP_BROWSER_OPEN {
            page.close().await?;
        }

        Ok(image)
    }

    pub async fn dispose(mut self) {
        if _TEMP_KEEP_BROWSER_OPEN {
            self.event_task_handle.await;
        } else {
            self.browser.close().await.expect("Failed to close browser");
        }
    }
}
