use std::fmt::Debug;
use std::time::Duration;
use derive_more::Display;
use reqwest::redirect::Policy;
use reqwest::Client;
use reqwest::Error as ReqErr;
use serde::Deserialize;

#[derive(Clone)]
pub struct QueueClient {
    pub base_address : String,
    pub http_client : Client,
    pub version_timeout: Option<Duration>
}

impl Debug for QueueClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueClient").field("base_address", &self.base_address).field("http_client", &self.http_client).finish()
    }
}

#[derive(Debug,Deserialize,Clone)]
pub enum Redirect {
    Disabled
}

#[derive(Debug,Deserialize,Clone)]
pub struct QueueClientBuilder {
    pub base_address: Option<String>,
    pub connect_timeout: Option<Duration>,
    pub timeout: Option<Duration>,
    pub user_agent: Option<String>,
    pub gzip_autodecode_enable: Option<bool>,
    pub brotli_autodecode_enable: Option<bool>,
    pub deflate_autodecode_enable: Option<bool>,
    //pub accept_invalid_hostname: Option<bool>,
    pub redirect: Option<Redirect>,
    pub accept_invalid_certs: Option<bool>,
}

impl TryFrom<QueueClientBuilder> for QueueClient {
    type Error = Error;
    fn try_from(value: QueueClientBuilder) -> Result<QueueClient,Error> {
        let mut cb = Client::builder();
        if value.base_address.is_none() {
            return Err(Error::BuildClient(format!("base address not set")));
        }

        cb = match value.connect_timeout { Some(t) => cb.connect_timeout(t), _ => cb };
        cb = match value.timeout { Some(t) => cb.timeout(t), _ => cb };
        cb = match value.user_agent { Some(agent) => cb.user_agent(agent), _ => cb };
        cb = match value.gzip_autodecode_enable { Some(v) => cb.gzip(v), _ => cb };
        cb = match value.deflate_autodecode_enable { Some(v) => cb.deflate(v), _ => cb };
        cb = match value.brotli_autodecode_enable { Some(v) => cb.brotli(v), _ => cb };
        cb = match value.accept_invalid_certs { Some(v) => cb.danger_accept_invalid_certs(v), _ => cb };
        cb = match value.redirect { 
            Some(v) => match v {
                Redirect::Disabled => cb.redirect(Policy::none())
            }, 
            _ => cb 
        };

        let res = Self { 
            base_address: value.base_address.unwrap(), 
            http_client: cb.build().map_err(Error::from)?,
            version_timeout: None,
        };
        Ok(res)
    }
}

#[derive(Debug,Clone,Display)]
pub enum Error {
    #[display(fmt = "BuildClient: {}", _0)]
    BuildClient(String),

    #[display(fmt = "Body: {}", _0)]
    Body(String),

    #[display(fmt = "DecodeBody: {}", _0)]
    DecodeBody(String),

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

#[derive(Debug,Clone,Deserialize)]
pub struct Version {
    pub debug: bool,
    pub crate_name: String,
    pub crate_ver: String
}

impl QueueClient {
    pub fn new<S: Into<String>>( base_address: S ) -> Result<Self,Error> {
        let c = Client::builder().build().map_err(Error::from)?;
        Ok(QueueClient { 
            base_address: base_address.into(), 
            http_client: c,
            version_timeout: None,
        })
    }

    pub async fn version( &self ) -> Result<Version,Error> {
        let req = self.http_client
        .get(format!("{}/queue/version",self.base_address));

        let req = match self.version_timeout {
            Some(t) => req.timeout(t),
            None => req
        };

        let res = req
            .send()
            .await?.json::<Version>().await?;
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn version_direct() {
        use actix_rt::System;
        System::new().block_on(async {
            let client = QueueClient::new("http://localhost:8080").unwrap();
            let ver = client.version().await.unwrap();
            println!("ver {ver:?}");
        })
    }
}