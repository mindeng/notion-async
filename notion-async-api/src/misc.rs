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

#[cfg(test)]
mod tests {
    use chrono::{DateTime, FixedOffset};

    use super::NotionFile;

    #[test]
    fn notion_file() {
        let js = r#"{
  "type": "file",
  "file": {
    "url": "https://s3.us-west-2.amazonaws.com/secure.notion-static.com/7b8b0713-dbd4-4962-b38b-955b6c49a573/My_test_image.png?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Content-Sha256=UNSIGNED-PAYLOAD&X-Amz-Credential=AKIAT73L2G45EIPT3X45%2F20221024%2Fus-west-2%2Fs3%2Faws4_request&X-Amz-Date=20221024T205211Z&X-Amz-Expires=3600&X-Amz-Signature=208aa971577ff05e75e68354e8a9488697288ff3fb3879c2d599433a7625bf90&X-Amz-SignedHeaders=host&x-id=GetObject",
    "expiry_time": "2022-10-24T22:49:22.765Z"
  }
}"#;
        let file: NotionFile = serde_json::from_str(js).unwrap();
        let url = "https://s3.us-west-2.amazonaws.com/secure.notion-static.com/7b8b0713-dbd4-4962-b38b-955b6c49a573/My_test_image.png?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Content-Sha256=UNSIGNED-PAYLOAD&X-Amz-Credential=AKIAT73L2G45EIPT3X45%2F20221024%2Fus-west-2%2Fs3%2Faws4_request&X-Amz-Date=20221024T205211Z&X-Amz-Expires=3600&X-Amz-Signature=208aa971577ff05e75e68354e8a9488697288ff3fb3879c2d599433a7625bf90&X-Amz-SignedHeaders=host&x-id=GetObject";
        let t: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339("2022-10-24T22:49:22.765Z").unwrap();
        let t = t.to_utc();
        assert!(
            matches!(file, NotionFile::File { file } if file.url == url && file.expiry_time == t)
        );
    }

    #[test]
    fn notion_external_file() {
        let js = r#"{
  "type": "external",
  "external": {
    "url": "https://images.unsplash.com/photo-1525310072745-f49212b5ac6d?ixlib=rb-1.2.1&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=1065&q=80"
  }
}"#;
        let file: NotionFile = serde_json::from_str(js).unwrap();
        let url = "https://images.unsplash.com/photo-1525310072745-f49212b5ac6d?ixlib=rb-1.2.1&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=1065&q=80";
        assert!(matches!(file, NotionFile::External { external } if external.url == url));
    }
}
