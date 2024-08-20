use std::collections::BTreeMap;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use monostate::MustBe;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use thiserror::Error;

use crate::user::User;

pub trait Object: Send {
    fn id(&self) -> &str;
    fn object_type(&self) -> ObjectType;
}

/// Common object info shared between [`Block`](crate::Block),
/// [`Page`](crate::Page) & [`Database`](crate::Database).
///
/// Refer to:
/// - [Notion JSON conventions](https://developers.notion.com/reference/intro#json-conventions)
/// - [Block object](https://developers.notion.com/reference/block)
/// - [Page object](https://developers.notion.com/reference/page)
/// - [Database object](https://developers.notion.com/reference/database)
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObjectCommon {
    // #[serde_as(as = "DisplayFromStr")]
    // pub object: ObjectType,
    pub id: String,
    pub parent: Parent,

    pub created_time: DateTime<Utc>,
    pub created_by: User,
    pub last_edited_time: DateTime<Utc>,
    pub last_edited_by: User,

    pub archived: bool,
    pub in_trash: bool,
}

impl ObjectCommon {
    pub fn parent_type(&self) -> ParentType {
        self.parent.r#type()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectType {
    Block,
    Page,
    Database,
    User,
    Comment,
    List,
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(self).unwrap())
    }
}

#[derive(Debug, Error)]
#[error("UnsupportObjectError({0})")]
pub struct UnsupportObjectError(String);

#[allow(unused)]
#[derive(Deserialize, Debug, Clone)]
pub struct ObjectList<T> {
    object: ObjectType, // should be "list"
    pub results: Vec<T>,

    #[serde(rename = "type")]
    ttype: String,

    #[serde(skip)]
    pub start_index: usize,

    #[serde(flatten)]
    next_page_info: NextPageInfo,
}

/// See: [Pagination](https://developers.notion.com/reference/intro#pagination)
#[derive(Deserialize, Debug, Clone)]
struct NextPageInfo {
    next_cursor: Option<String>,
    has_more: bool,
}

pub trait NextCursor {
    fn next_cursor(&self) -> Option<&str>;
}

impl<T> NextCursor for ObjectList<T> {
    fn next_cursor(&self) -> Option<&str> {
        if !self.next_page_info.has_more {
            None
        } else {
            self.next_page_info.next_cursor.as_deref()
        }
    }
}

/// Refer to: [Parent object](https://developers.notion.com/reference/parent-object)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Parent {
    Block { block_id: String },
    Page { page_id: String },
    Database { database_id: String },
    Workspace { workspace: MustBe!(true) },
}

impl Parent {
    pub fn id(&self) -> &str {
        match self {
            Parent::Block { block_id } => block_id,
            Parent::Page { page_id } => page_id,
            Parent::Database { database_id } => database_id,
            Parent::Workspace { workspace: _ } => "workspace",
        }
    }

    pub fn r#type(&self) -> ParentType {
        match self {
            Parent::Block { block_id: _ } => ParentType::BlockId,
            Parent::Page { page_id: _ } => ParentType::PageId,
            Parent::Database { database_id: _ } => ParentType::DatabaseId,
            Parent::Workspace { workspace: _ } => ParentType::Workspace,
        }
    }

    pub fn workspace() -> Self {
        Self::Workspace {
            workspace: MustBe!(true),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParentType {
    DatabaseId,
    PageId,
    BlockId,
    Workspace,
}

impl Display for ParentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(self).unwrap())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImpossibleParseError;

impl Display for ImpossibleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ImpossibleParseError")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonObject {
    #[serde(flatten)]
    pub map: BTreeMap<String, serde_json::Value>,
}

impl Display for JsonObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).unwrap();
        f.write_str(&s)
    }
}
