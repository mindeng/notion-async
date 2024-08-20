use std::collections::BTreeMap;

use monostate::MustBe;
use serde::{Deserialize, Serialize};

use crate::misc::{Icon, NotionFile, Property};
use crate::object::{Object, ObjectCommon};

/// Refer to:
/// - [Notion JSON conventions](https://developers.notion.com/reference/intro#json-conventions)
/// - [Block object](https://developers.notion.com/reference/block)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Page {
    object: MustBe!("page"),

    #[serde(flatten)]
    pub obj: ObjectCommon,

    pub properties: BTreeMap<String, Property>,
    pub url: String,

    pub public_url: Option<String>,
    pub icon: Option<Icon>,
    pub cover: Option<NotionFile>,
}

impl Object for Page {
    fn id(&self) -> &str {
        &self.obj.id
    }

    fn object_type(&self) -> crate::object::ObjectType {
        crate::object::ObjectType::Page
    }
}
