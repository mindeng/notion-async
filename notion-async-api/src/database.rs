use std::collections::BTreeMap;

use monostate::MustBe;
use serde::{Deserialize, Serialize};

use crate::misc::{Icon, NotionFile, Property};
use crate::object::{Object, ObjectCommon};
use crate::rich_text::RichText;

/// Refer to:
/// - [database](https://developers.notion.com/reference/database)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Database {
    object: MustBe!("database"),

    #[serde(flatten)]
    pub obj: ObjectCommon,

    pub properties: BTreeMap<String, Property>,
    pub url: String,

    pub public_url: Option<String>,
    pub icon: Option<Icon>,
    pub cover: Option<NotionFile>,

    pub is_inline: bool,
    pub title: Vec<RichText>,
    pub description: Vec<RichText>,
}

impl Object for Database {
    fn id(&self) -> &str {
        &self.obj.id
    }

    fn object_type(&self) -> crate::object::ObjectType {
        crate::object::ObjectType::Database
    }
}
