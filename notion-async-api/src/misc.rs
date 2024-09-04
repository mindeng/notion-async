use std::collections::BTreeMap;
use std::ops::Deref;
use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;
use serde_with::{DisplayFromStr, MapSkipError};
use thiserror::Error;

/// Refer to:
/// - [Property object](https://developers.notion.com/reference/property-object)
/// - [Page properties](https://developers.notion.com/reference/page-property-values)
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Property {
    pub id: String,

    pub r#type: String,

    #[serde(flatten)]
    #[serde_as(as = "MapSkipError<DisplayFromStr, _>")]
    pub type_data: BTreeMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Icon {
    Emoji { emoji: String },
    File(NotionFile),
}

impl Display for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).unwrap();
        s.unquotes().fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum NotionFile {
    File { file: NotionFileData },
    External { external: UrlData },
}

impl Display for NotionFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).unwrap();
        s.unquotes().fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NotionFileData {
    pub url: String,
    pub expiry_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NotionFileType {
    File,
    External,
}

impl Display for NotionFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NotionFileType::File => "file",
            NotionFileType::External => "external",
        };
        s.fmt(f)
    }
}

impl FromStr for NotionFileType {
    type Err = UnsupportFileTypeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s {
            "file" => Self::File,
            "external" => Self::External,
            x => return Err(UnsupportFileTypeError(x.to_owned())),
        };
        Ok(res)
    }
}

#[derive(Debug, Error)]
#[error("UnsupportFileTypeError({0})")]
pub struct UnsupportFileTypeError(String);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct IdData {
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DateProperty {
    start: DateTime<Utc>,
    end: Option<DateTime<Utc>>,
    // optional field `time_zone` is ignored
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct UrlData {
    url: String,
}

pub(crate) trait Unquotes {
    fn unquotes(&self) -> &str;
}

// impl Unquotes for &str {
//     fn unquotes(&self) -> String {
//         let s = match self.strip_prefix("\"").and_then(|x| x.strip_suffix("\"")) {
//             Some(ss) => ss,
//             None => self,
//         };
//         s.to_owned()
//     }
// }

impl<T> Unquotes for T
where
    T: Deref<Target = str>,
{
    fn unquotes(&self) -> &str {
        match self.strip_prefix("\"").and_then(|x| x.strip_suffix("\"")) {
            Some(ss) => ss,
            None => self,
        }
    }
}
