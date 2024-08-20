use chrono::{DateTime, Utc};
use monostate::MustBe;
use serde::{Deserialize, Serialize};

use crate::object::{Object, Parent};
use crate::rich_text::RichText;
use crate::user::User;

/// Refer to:
/// - [Comment](https://developers.notion.com/reference/comment-object)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Comment {
    object: MustBe!("comment"),
    pub id: String,
    pub parent: Parent,

    pub created_time: DateTime<Utc>,
    pub created_by: User,
    pub last_edited_time: DateTime<Utc>,

    pub discussion_id: String,
    pub rich_text: Vec<RichText>,
    // pub rich_text: Vec<Value>,
}

impl Object for Comment {
    fn id(&self) -> &str {
        &self.id
    }

    fn object_type(&self) -> crate::object::ObjectType {
        crate::object::ObjectType::Comment
    }
}
