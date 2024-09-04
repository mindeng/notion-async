use std::fmt::Display;

use crate::api::RequestError;

#[derive(Debug)]
pub enum NotionError {
    InvalidObject(String),
    RequestFailed(RequestError),
}

impl NotionError {
    pub fn invalid_object(s: impl Into<String>) -> Self {
        Self::InvalidObject(s.into())
    }

    pub fn key_not_found(s: impl Into<String>) -> Self {
        Self::InvalidObject(format!("key `{}` not found", s.into()))
    }

    pub fn invalid_request(s: impl Into<String>) -> Self {
        Self::RequestFailed(RequestError::InvalidRequest(s.into()))
    }

    pub fn invalid_response(s: impl Into<String>) -> Self {
        Self::RequestFailed(RequestError::invalid_response(s))
    }

    pub fn retry_after(secs: u64) -> Self {
        Self::RequestFailed(RequestError::RetryAfter(secs))
    }
}

impl Display for NotionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotionError::InvalidObject(s) => format!("invalid notion object: {s}").fmt(f),
            NotionError::RequestFailed(e) => e.fmt(f),
        }
    }
}

impl From<&'static str> for NotionError {
    fn from(value: &'static str) -> Self {
        NotionError::InvalidObject(value.into())
    }
}

impl From<reqwest::Error> for NotionError {
    fn from(value: reqwest::Error) -> Self {
        Self::RequestFailed(RequestError::Other(value))
    }
}

impl serde::de::Error for NotionError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        NotionError::invalid_object(msg.to_string())
    }
}

impl std::error::Error for NotionError {}
