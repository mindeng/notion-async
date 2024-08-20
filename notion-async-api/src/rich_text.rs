use serde::{Deserialize, Serialize};

use crate::{
    misc::{DateProperty, IdData, UrlData},
    user::User,
};

/// Refer to:
/// - [Rich text](https://developers.notion.com/reference/rich-text)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RichText {
    #[serde(flatten)]
    pub rich_text_type: RichTextType,

    pub annotations: Annotations,
    pub plain_text: String,
    pub href: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Annotations {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub code: bool,
    pub color: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RichTextType {
    Equation { equation: EquationData },
    Mention { mention: MentionType },
    Text { text: TextData },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextData {
    content: String,
    link: Option<UrlData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EquationData {
    expression: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MentionType {
    Database { database: IdData },
    Page { page: IdData },
    User { user: User },
    Date { date: DateProperty },
    LinkPreview { link_preview: UrlData },
    TemplateMention { template_mention: MentionTypeData },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MentionTypeData {
    TemplateMentionDate(String),
    TemplateMentionUser(String),
}
