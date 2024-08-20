use notion_async_api::{Block, Comment, Database, Object, Page};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteQueryResult},
    Connection, SqliteConnection,
};

pub async fn init_db(path: &str) -> sqlx::Result<SqliteConnection> {
    let sql = SQL_SCHEMA;
    // let mut sql = String::new();
    // File::open("notion-async/schema.sql")
    //     .await
    //     .unwrap()
    //     .read_to_string(&mut sql)
    //     .await
    //     .unwrap();

    // let mut conn = SqliteConnection::connect("sqlite::memory:").await?;
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);
    let mut conn = SqliteConnection::connect_with(&options).await?;
    sqlx::query(sql).execute(&mut conn).await?;

    Ok(conn)
}

pub async fn insert_or_update_block(
    db: &mut SqliteConnection,
    block: Block,
) -> sqlx::error::Result<SqliteQueryResult> {
    sqlx::query(
        "insert or replace into blocks \
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
    )
    .bind(block.id().to_owned())
    .bind(block.obj.parent_type().to_string())
    .bind(block.obj.parent.id())
    .bind(block.obj.created_time)
    .bind(block.obj.created_by.id())
    .bind(block.obj.last_edited_time)
    .bind(block.obj.last_edited_by.id())
    .bind(block.obj.archived)
    .bind(block.obj.in_trash)
    .bind(block.child_index as i64)
    .bind(block.has_children)
    .bind(block.block_type.to_string())
    .bind(serde_json::to_string(&block.type_data).unwrap())
    .execute(db)
    .await
}

pub async fn insert_or_update_page(
    db: &mut SqliteConnection,
    page: Page,
) -> sqlx::error::Result<SqliteQueryResult> {
    sqlx::query(
        "insert or replace into pages \
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
    )
    .bind(page.id().to_owned())
    .bind(page.obj.parent_type().to_string())
    .bind(page.obj.parent.id())
    .bind(page.obj.created_time)
    .bind(page.obj.created_by.id())
    .bind(page.obj.last_edited_time)
    .bind(page.obj.last_edited_by.id())
    .bind(page.obj.archived)
    .bind(page.obj.in_trash)
    .bind(serde_json::to_string(&page.properties).unwrap())
    .bind(page.url)
    .bind(page.public_url)
    .bind(page.icon.map(|x| x.to_string()))
    .bind(page.cover.map(|x| x.to_string()))
    .execute(db)
    .await
}

pub async fn insert_or_update_database(
    db: &mut SqliteConnection,
    database: Database,
) -> sqlx::error::Result<SqliteQueryResult> {
    sqlx::query(
        "insert or replace into databases \
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)",
    )
    .bind(database.id().to_owned())
    .bind(database.obj.parent_type().to_string())
    .bind(database.obj.parent.id())
    .bind(database.obj.created_time)
    .bind(database.obj.created_by.id())
    .bind(database.obj.last_edited_time)
    .bind(database.obj.last_edited_by.id())
    .bind(database.obj.archived)
    .bind(database.obj.in_trash)
    .bind(serde_json::to_string(&database.properties).unwrap())
    .bind(database.url)
    .bind(database.public_url)
    .bind(database.icon.map(|x| x.to_string()))
    .bind(database.cover.map(|x| x.to_string()))
    .bind(database.is_inline)
    .bind(serde_json::to_string(&database.title).unwrap())
    .bind(serde_json::to_string(&database.description).unwrap())
    .execute(db)
    .await
}

pub async fn insert_or_update_comment(
    db: &mut SqliteConnection,
    comment: Comment,
) -> sqlx::error::Result<SqliteQueryResult> {
    sqlx::query(
        "insert or replace into comments \
         values ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(comment.id().to_owned())
    .bind(comment.parent.r#type().to_string())
    .bind(comment.parent.id())
    .bind(comment.created_time)
    .bind(comment.created_by.id())
    .bind(comment.last_edited_time)
    .bind(comment.discussion_id)
    .bind(serde_json::to_string(&comment.rich_text).unwrap())
    .execute(db)
    .await
}

// async fn save_object(obj: impl AnyObject, dir: &str) -> Result<(), Box<dyn Error>> {
//     // save
//     let name = format!("{}-{}.json", obj.object_type(), obj.id());
//     let p = path::Path::new(dir).join(name);
//     let mut file = File::create(p).await?;
//     let data = serde_json::to_vec_pretty(&obj)?;
//     file.write_all(&data).await?;

//     Ok(())
// }

const SQL_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS blocks (
    -- `rowid` will be the actual primary key, `id` is just a unique index
    id TEXT not null primary key,

    parent_type TEXT not null,
    parent_id TEXT not null,

    created_time TEXT not null,
    created_by TEXT not null,
    last_edited_time TEXT not null,
    last_edited_by TEXT not null,

    -- 1: true, 0: false
    archived INTEGER not null,
    in_trash INTEGER not null,

    -------------------------------
    -- index in parent
    child_index INTEGER not null,

    has_children INTEGER not null,

    -- child_page, child_database, paragraph, etc.
    block_type TEXT not null,
    type_data TEXT not null
);

CREATE TABLE IF NOT EXISTS pages (
    id TEXT not null primary key,

    parent_type TEXT not null,
    parent_id TEXT not null,

    created_time TEXT not null,
    created_by TEXT not null,
    last_edited_time TEXT not null,
    last_edited_by TEXT not null,

    -- 1: true, 0: false
    archived INTEGER not null,
    in_trash INTEGER not null,

    -------------------------------
    -- json object
    properties TEXT not null,
    url TEXT not null,

    public_url TEXT,
    icon TEXT,
    cover TEXT
);

CREATE TABLE IF NOT EXISTS databases (
    id TEXT not null primary key,

    parent_type TEXT not null,
    parent_id TEXT not null,

    created_time TEXT not null,
    created_by TEXT not null,
    last_edited_time TEXT not null,
    last_edited_by TEXT not null,

    -- 1: true, 0: false
    archived INTEGER not null,
    in_trash INTEGER not null,

    -------------------------------
    -- json object
    properties TEXT not null,
    url TEXT not null,

    public_url TEXT,
    icon TEXT,
    cover TEXT,

    is_inline INTEGER not null,
    -- array of rich text objects
    title TEXT not null,
    -- array of rich text objects
    description TEXT not null
);

CREATE TABLE IF NOT EXISTS comments (
    id TEXT not null primary key,

    parent_type TEXT not null,
    parent_id TEXT not null,

    created_time TEXT not null,
    created_by TEXT not null,
    last_edited_time TEXT not null,

    -------------------------------
    discussion_id TEXT not null,
    -- array of rich text objects
    rich_text TEXT not null
);
"#;
