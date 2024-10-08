use std::collections::BTreeMap;
use std::fmt::Display;

use monostate::MustBe;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;

use crate::{
    misc::Unquotes,
    object::{Object, ObjectCommon},
};

/// Refer to:
/// - [Notion JSON conventions](https://developers.notion.com/reference/intro#json-conventions)
/// - [Block object](https://developers.notion.com/reference/block)
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    object: MustBe!("block"),

    #[serde(flatten)]
    pub obj: ObjectCommon,

    // custom field, index in parent
    #[serde(default)]
    pub child_index: usize,

    pub has_children: bool,

    #[serde(rename = "type")]
    pub block_type: BlockType,

    #[serde(flatten)]
    pub type_data: BlockTypeData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    ChildPage,
    ChildDatabase,
    Bookmark,
    Breadcrumb,
    BulletedListItem,
    Callout,
    Code,
    Column,
    ColumnList,
    Divider,
    Embed,
    Equation,
    File,

    #[serde(rename = "heading_1")]
    Heading1,
    #[serde(rename = "heading_2")]
    Heading2,
    #[serde(rename = "heading_3")]
    Heading3,

    Image,
    LinkPreview,
    LinkToPreview,
    Mention,
    NumberedListItem,
    Paragraph,
    Pdf,
    Quote,
    SyncedBlock,
    Table,
    TableRow,
    TableOfContents,
    Template,
    ToDo,
    Toggle,
    Video,
    Unsupported,
}

impl Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).unwrap_or("".to_owned());
        s.unquotes().fmt(f)
    }
}

/// Refer to: [Block type](https://developers.notion.com/reference/block)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockTypeData {
    ChildPage {
        title: String,
    },
    ChildDatabase {
        title: String,
    },
    Bookmark(BTreeMap<String, Value>),
    Breadcrumb(BTreeMap<String, Value>),
    BulletedListItem(BTreeMap<String, Value>),
    Callout(BTreeMap<String, Value>),
    Code(BTreeMap<String, Value>),
    Column(BTreeMap<String, Value>),
    ColumnList(BTreeMap<String, Value>),
    Divider(BTreeMap<String, Value>),
    Embed(BTreeMap<String, Value>),
    Equation(BTreeMap<String, Value>),
    File(BTreeMap<String, Value>),

    #[serde(rename = "heading_1")]
    Heading1(BTreeMap<String, Value>),
    #[serde(rename = "heading_2")]
    Heading2(BTreeMap<String, Value>),
    #[serde(rename = "heading_3")]
    Heading3(BTreeMap<String, Value>),

    Image(BTreeMap<String, Value>),
    LinkPreview(BTreeMap<String, Value>),
    LinkToPreview(BTreeMap<String, Value>),
    Mention(BTreeMap<String, Value>),
    NumberedListItem(BTreeMap<String, Value>),
    Paragraph(BTreeMap<String, Value>),
    Pdf(BTreeMap<String, Value>),
    Quote(BTreeMap<String, Value>),
    SyncedBlock(BTreeMap<String, Value>),
    Table(BTreeMap<String, Value>),
    TableRow(BTreeMap<String, Value>),
    TableOfContents(BTreeMap<String, Value>),
    Template(BTreeMap<String, Value>),
    ToDo(BTreeMap<String, Value>),
    Toggle(BTreeMap<String, Value>),
    Video(BTreeMap<String, Value>),
    Unsupported(BTreeMap<String, Value>),
}

impl Object for Block {
    fn id(&self) -> &str {
        &self.obj.id
    }

    fn object_type(&self) -> crate::object::ObjectType {
        crate::object::ObjectType::Block
    }
}
