use std::{collections::HashMap, env};

use clap::{Parser, Subcommand};
use futures::StreamExt;
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
    /// Notion integration token, can get from:
    /// https://www.notion.so/my-integrations. If it's not set, will read from
    /// env var NOTION_TOKEN.
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
        /// Specify the root notion page/database ID, can be find in the
        /// page/database link. If it's not set, will read from env var
        /// NOTION_ROOT_ID.
        #[arg(long)]
        id: Option<String>,
    },
}

const NOTION_TOKEN: &str = "NOTION_TOKEN";
const NOTION_ROOT_ID: &str = "NOTION_ROOT_ID";

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
            Commands::Sync { id } => {
                let page_id = match id {
                    Some(id) => id.to_owned(),
                    None => {
                        let Ok(id) = env::var(NOTION_ROOT_ID) else {
                            return Err(
                                format!("Neither --id nor env {NOTION_ROOT_ID} is set.").into()
                            );
                        };
                        id
                    }
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
                    println!("🔁 repeated {} {}", obj.object_type(), obj.id());
                }

                match obj {
                    notion_async_api::AnyObject::Block(block) => {
                        println!("⏬ 🆎 block {} {}", block.id(), block.block_type);
                        insert_or_update_block(db, block).await.unwrap();
                    }
                    notion_async_api::AnyObject::Page(page) => {
                        println!("⏬ 📃 page {}", page.id());
                        insert_or_update_page(db, page).await.unwrap();
                    }
                    notion_async_api::AnyObject::Database(database) => {
                        println!("⏬ 🗐 database {}", database.id());
                        insert_or_update_database(db, database).await.unwrap();
                    }
                    notion_async_api::AnyObject::User(user) => {
                        println!("⏬ 👤 user {}", user.id());
                    }
                    notion_async_api::AnyObject::Comment(comment) => {
                        println!("⏬ 📝 comment {}", comment.id(),);
                        insert_or_update_comment(db, comment).await.unwrap();
                    }
                };
            }
            Err(e) => {
                println!("❌ {e}");
            }
        }
    }
}
