use std::{
    fmt::{Debug, Display},
    sync::LazyLock,
};

use reqwest::{header, Client, Method, Response, StatusCode, Url};
use serde::de::DeserializeOwned;

use crate::{
    block::Block,
    comment::Comment,
    database::Database,
    error::NotionError,
    fetcher::AnyObject,
    object::{NextCursor, ObjectList},
    page::Page,
    user::User,
};

const NOTION_API_VERSION: &str = "2022-06-28";

/// Low-level notion Api.
#[derive(Clone)]
pub struct Api {
    client: Client,
}

#[derive(Debug)]
pub enum RequestError {
    InvalidRequest(String),
    InvalidResponse(String),
    RetryAfter(u64), // seconds
    Other(reqwest::Error),
}

impl RequestError {
    pub fn invalid_response(s: impl Into<String>) -> Self {
        Self::InvalidResponse(s.into())
    }
}

impl Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestError::InvalidRequest(s) => write!(f, "invalid request: {s}"),
            RequestError::InvalidResponse(s) => write!(f, "invalid response: {s}"),
            RequestError::RetryAfter(s) => write!(f, "retry after: {s}"),
            RequestError::Other(e) => write!(f, "request error: {e:?}"),
        }
    }
}

impl std::error::Error for RequestError {}

impl Api {
    pub fn new(token: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Notion-Version",
            header::HeaderValue::from_static(NOTION_API_VERSION),
        );
        let bearer = format!("Bearer {}", token);
        let mut auth_value = header::HeaderValue::from_str(&bearer)
            .expect("token: only visible ASCII characters (32-127) are permitted");
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);

        Api {
            client: Client::builder().default_headers(headers).build().unwrap(),
        }
    }

    pub async fn get_object<T>(&self, id: &str) -> Result<T, NotionError>
    where
        T: DeserializeOwned + Requestable,
    {
        let res = self.client.get(T::url(id)).send().await?;
        check_retry_after(&res)?;
        let res = check_status_code(res).await?;

        res.json::<T>().await.map_err(|e| {
            NotionError::RequestFailed(RequestError::InvalidResponse(format!(
                "decode failed: {e:?}, {}",
                T::url(id),
            )))
        })
    }

    pub async fn list<T, P>(&self, pagination: &P) -> Result<PaginationResult<T>, NotionError>
    where
        T: DeserializeOwned,
        P: Pagination<T> + NextCursor,
    {
        pagination.next_page(&self.client).await
    }
}

fn check_retry_after(res: &Response) -> Result<(), NotionError> {
    if res.status() == StatusCode::TOO_MANY_REQUESTS {
        // extract Retry-After
        let Some(retry_after) = res.headers().get(header::RETRY_AFTER) else {
            return Err(NotionError::invalid_response(
                "encounter rate limited error without Retry-After",
            ));
        };
        let after: u64 = retry_after
            .to_str()
            .map_err(|_| NotionError::invalid_response("invalid Retry-After header"))
            .and_then(|s| {
                s.parse()
                    .map_err(|_| NotionError::invalid_response("invalid Retry-After header"))
            })?;
        return Err(NotionError::retry_after(after));
    };
    Ok(())
}

async fn check_status_code(res: Response) -> Result<Response, NotionError> {
    if !res.status().is_success() {
        let url = res.url().clone();
        Err(NotionError::invalid_response(format!(
            "status: {}, body: {}, url: {url}",
            res.status(),
            res.text().await?,
        )))
    } else {
        Ok(res)
    }
}

pub trait Pagination<Item>: Debug {
    fn next_page(
        &self,
        client: &Client,
    ) -> impl std::future::Future<Output = Result<PaginationResult<Item>, NotionError>> + Send;
}

#[derive(Clone)]
pub struct PaginationInfo {
    cursor: Option<String>,
    url: Url,
    method: Method,
    start_index: usize,
}

impl PaginationInfo {
    pub fn new<R>(id: &str) -> Self
    where
        R: Requestable,
    {
        Self::build(R::url(id), R::method())
    }

    fn build(url: Url, method: Method) -> Self {
        Self {
            cursor: None,
            url,
            method,
            start_index: 0,
        }
    }

    fn cursor(mut self, cursor: String) -> Self {
        self.cursor = Some(cursor);
        self
    }

    fn start_index(mut self, index: usize) -> Self {
        self.start_index = index;
        self
    }
}

impl NextCursor for PaginationInfo {
    fn next_cursor(&self) -> Option<&str> {
        self.cursor.as_deref()
    }
}

impl Debug for PaginationInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PaginationInfo")
            .field("params", &self.cursor)
            .field("url", &self.url)
            .finish()
    }
}

impl<T> Pagination<T> for PaginationInfo
where
    T: DeserializeOwned + Send,
{
    async fn next_page(&self, client: &Client) -> Result<PaginationResult<T>, NotionError> {
        let mut url = self.url.clone();

        if let Some(ref next_cursor) = self.cursor {
            // set start_cursor
            let q = self.url.query_pairs().filter(|(k, _)| k != "start_cursor");
            url.query_pairs_mut()
                .clear()
                .extend_pairs(q)
                .append_pair("start_cursor", next_cursor)
                .finish();
        };

        let res = client.request(self.method.clone(), url).send().await?;
        check_retry_after(&res)?;
        let res = check_status_code(res).await?;

        let mut res: ObjectList<T> = res.json().await?;
        res.start_index = self.start_index;
        let next_page = res.next_cursor().map(|x| {
            PaginationInfo::build(self.url.clone(), self.method.clone())
                .cursor(x.to_owned())
                .start_index(self.start_index + res.results.len())
        });

        Ok(PaginationResult::<T> {
            result: res,
            pagination: next_page,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PaginationResult<T> {
    pub result: ObjectList<T>,
    pub pagination: Option<PaginationInfo>,
}

pub trait Requestable {
    fn url(id: &str) -> Url;
    fn method() -> Method {
        Method::GET
    }
}

static BASE_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://api.notion.com/v1/").unwrap());

impl Requestable for Block {
    fn url(id: &str) -> Url {
        BASE_URL.join(&format!("blocks/{id}")).unwrap()
    }
}

impl Requestable for Page {
    fn url(id: &str) -> Url {
        BASE_URL.join(&format!("pages/{id}")).unwrap()
    }
}

impl Requestable for Database {
    fn url(id: &str) -> Url {
        BASE_URL.join(&format!("databases/{id}")).unwrap()
    }
}

impl Requestable for ObjectList<Block> {
    fn url(id: &str) -> Url {
        BASE_URL.join(&format!("blocks/{id}/children")).unwrap()
    }
}

impl Requestable for ObjectList<AnyObject> {
    fn url(id: &str) -> Url {
        BASE_URL.join(&format!("databases/{id}/query")).unwrap()
    }

    fn method() -> Method {
        Method::POST
    }
}

impl Requestable for ObjectList<Comment> {
    fn url(id: &str) -> Url {
        let mut url = BASE_URL.join("comments").unwrap();
        url.query_pairs_mut().append_pair("block_id", id).finish();
        url
    }
}

impl Requestable for User {
    fn url(id: &str) -> Url {
        BASE_URL.join(&format!("users/{id}")).unwrap()
    }
}

impl Requestable for ObjectList<User> {
    fn url(_: &str) -> Url {
        BASE_URL.join("users").unwrap()
    }
}
