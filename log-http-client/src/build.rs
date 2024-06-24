use std::time::Duration;
use serde::*;
use crate::QueueClient;
use crate::errors::*;
use reqwest::redirect::Policy;
use reqwest::Client;

/// Настройки перенаправления клиента
#[derive(Debug,Deserialize,Clone)]
pub enum Redirect {
    Disabled
}

/// Структура для создания клиента
#[derive(Debug,Deserialize,Clone)]
pub struct QueueClientBuilder {
    /// Базовый адрес сервиса, например `http://service:8080`
    pub base_address: Option<String>,

    /// Таймаут соединения
    pub connect_timeout: Option<Duration>,

    /// Таймаут запроса
    pub timeout: Option<Duration>,

    /// Имя клиента
    pub user_agent: Option<String>,

    /// Поддержка кодирования gzip
    pub gzip_autodecode_enable: Option<bool>,

    /// Поддержка кодирования brotli
    pub brotli_autodecode_enable: Option<bool>,

    /// Поддержка кодирования deflate
    pub deflate_autodecode_enable: Option<bool>,

    /// Настройки redirect
    pub redirect: Option<Redirect>,

    /// Допускать self signed сертификаты
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
