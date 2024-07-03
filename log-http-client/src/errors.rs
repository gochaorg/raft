use derive_more::Display;
use reqwest::StatusCode;
use serde::Serialize;

#[derive(Debug,Clone,Display,Serialize)]
pub enum Error {
    #[display(fmt = "UnExpectedStatus: {}", _0)]
    UnExpectedStatus(String),

    #[display(fmt = "BuildClient: {}", _0)]
    BuildClient(String),

    #[display(fmt = "Body: {}", _0)]
    Body(String),

    #[display(fmt = "DecodeBody: {}", _0)]
    DecodeBody(String),

    #[display(fmt = "JsonDecodeBody: {}", _0)]
    JsonDecodeBody(String),

    #[display(fmt = "Status: {}", _0)]
    Status(String),

    #[display(fmt = "Timeout: {}", _0)]
    Timeout(String),

    #[display(fmt = "Connect: {}", _0)]
    Connect(String),

    #[display(fmt = "Redirect: {}", _0)]
    Redirect(String),

    #[display(fmt = "Request: {}", _0)]
    Request(String),

    #[display(fmt = "Undefined: {}", _0)]
    Undefined(String)
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        if value.is_builder() { return Self::BuildClient(value.to_string()); }
        if value.is_body() { return Self::Body(value.to_string()); }
        if value.is_decode() { return Self::DecodeBody(value.to_string()); }
        if value.is_status() { return Self::Status(value.to_string()); }
        if value.is_timeout() { return Self::Timeout(value.to_string()); }
        if value.is_connect() { return Self::Connect(value.to_string()); }
        if value.is_redirect() { return Self::Redirect(value.to_string()); }
        if value.is_request() { return Self::Request(value.to_string()); }

        Error::Undefined(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::JsonDecodeBody(value.to_string())
    }
}

impl Error {
    pub fn un_expected_status( status: StatusCode, body: &str ) -> Self {
        Self::UnExpectedStatus(format!("status = {}\nbody:\n{}", status, body ))
    }
}

