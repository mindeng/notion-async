use std::{collections::HashMap, env, path};

use clap::{Parser, Subcommand};
use futures::StreamExt;
use http::Uri;
use notion_async::{
    init_db, insert_or_update_block, insert_or_update_comment, insert_or_update_database,
    insert_or_update_page,
};
use notion_async_api::{Fetcher, Object};
use sqlx::SqliteConnection;

/// A notion sync tool, in `async` style.
#[derive(Parser, Debug)]
#[command(name = "notion-async")]
#[command(version = "0.1")]
struct Cli {
    /// Notion integration token, can get from
    /// https://www.notion.so/my-integrations. Read from env var NOTION_TOKEN
    /// if not set.
    #[arg(long)]
    token: Option<String>,

    /// Sqlite database file path
    #[arg(long, value_name = "FILE", default_value_t=String::from("notion.db"))]
    db: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Sync all pages/databases/comments into db, recursively.
    Sync {
        /// A LINK or ID of a notion page/database. Everything in it will be
        /// downloaded, in recursive way. Read from env var NOTION_ROOT_PAGE if
        /// not set.
        page: Option<String>,
    },
}

const NOTION_TOKEN: &str = "NOTION_TOKEN";
const NOTION_ROOT_PAGE: &str = "NOTION_ROOT_PAGE";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let _ = dotenvy::dotenv();

    let mut db = init_db(&cli.db).await?;
    cli.run(&mut db).await?;

    Ok(())
}

impl Cli {
    async fn run(&self, db: &mut SqliteConnection) -> Result<()> {
        match &self.command {
            Commands::Sync { page } => {
                let page = match page {
                    Some(id) => id.to_owned(),
                    None => {
                        let Ok(id) = env::var(NOTION_ROOT_PAGE) else {
                            return Err(
                                format!("Neither --id nor env {NOTION_ROOT_PAGE} is set.").into()
                            );
                        };
                        id
                    }
                };

                let page_id = if page.starts_with("https://") {
                    match page.parse::<Uri>() {
                        Ok(uri) => {
                            let p = uri.path();
                            let p = path::Path::new(p).file_name();
                            if let Some(last) = p.and_then(|x| x.to_str()) {
                                if let Some((_, id)) = last.rsplit_once("-") {
                                    id.to_owned()
                                } else {
                                    last.to_owned()
                                }
                            } else {
                                return Err(format!(
                                    "Can't extract ID from NOTION_ROOT_PAGE, which value is {page}"
                                )
                                .into());
                            }
                        }
                        Err(_) => page,
                    }
                } else {
                    page
                };

                run_sync(&self.get_token()?, &page_id, db).await;
            }
        };
        Ok(())
    }

    fn get_token(&self) -> Result<String> {
        let token = match self.token.as_deref() {
            Some(t) => t.to_owned(),
            None => {
                let Ok(token) = env::var(NOTION_TOKEN) else {
                    return Err(format!("Neither --token nor env {NOTION_TOKEN} is set.").into());
                };
                token
            }
        };
        Ok(token)
    }
}

async fn run_sync(token: &str, page_id: &str, db: &mut SqliteConnection) {
    let fetcher = Fetcher::new(token);
    let mut rx = fetcher.fetch(page_id).await;
    let mut objects = HashMap::<String, ()>::new();
    while let Some(obj) = rx.next().await {
        match obj {
            Ok(obj) => {
                if let std::collections::hash_map::Entry::Vacant(e) =
                    objects.entry(format!("{}-{}", obj.id(), obj.object_type()))
                {
                    e.insert(());
                    // println!("⏬ {} {}", obj.object_type(), obj.id());
                    // save_object(&obj, "testdata").await?;
                } else {
                    eprintln!("➡️ 🔁 repeated {} {}", obj.object_type(), obj.id());
                }

                match obj {
                    notion_async_api::AnyObject::Block(block) => {
                        println!(
                            "✔   {:8} {} {}",
                            block.object_type(),
                            block.id(),
                            block.block_type
                        );
                        insert_or_update_block(db, block).await.unwrap();
                    }
                    notion_async_api::AnyObject::Page(page) => {
                        println!("✔ 📃 {:8} {}", page.object_type(), page.id());
                        insert_or_update_page(db, page).await.unwrap();
                    }
                    notion_async_api::AnyObject::Database(database) => {
                        println!("✔   {:8} {}", database.object_type(), database.id());
                        insert_or_update_database(db, database).await.unwrap();
                    }
                    notion_async_api::AnyObject::User(user) => {
                        println!("✔️ 👤 {:8} {}", user.object_type(), user.id());
                    }
                    notion_async_api::AnyObject::Comment(comment) => {
                        println!("✔   {:8} {}", comment.object_type(), comment.id(),);
                        insert_or_update_comment(db, comment).await.unwrap();
                    }
                };
            }
            Err(e) => {
                eprintln!("❌ error {e}");
            }
        }
    }
}
