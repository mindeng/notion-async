use monostate::MustBe;
use serde::{Deserialize, Serialize};

use crate::object::{JsonObject, Object};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    object: MustBe!("user"),
    id: String,

    pub r#type: Option<UserType>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    #[serde(flatten)]
    pub user_data: Option<UserTypeData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserType {
    Person,
    Bot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserTypeData {
    Person { email: Option<String> },
    Bot { owner: JsonObject },
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerType {
    Workspace,
    User,
}

impl Object for User {
    fn id(&self) -> &str {
        &self.id
    }
    fn object_type(&self) -> crate::object::ObjectType {
        crate::object::ObjectType::User
    }
}
