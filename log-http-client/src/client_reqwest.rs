use core::str;
use std::fmt::Debug;
use std::time::Duration;
use reqwest::Client;
use serde::Deserialize;
use crate::errors::*;

/// Адрес для тестирования
#[cfg(test)]
const BASE_ADDR:&str = "http://localhost:8080";

/// Клиент очереди
#[derive(Clone)]
pub struct QueueClient {
    /// Базовый адрес
    pub base_address : String,

    /// http клиент
    pub http_client : Client,

    /// timeout ответа на запрос version
    pub version_timeout: Option<Duration>
}

impl Debug for QueueClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueClient")
        .field("base_address", &self.base_address)
        .field("http_client", &self.http_client)
        .finish()
    }
}

impl QueueClient {
    pub fn new<S: Into<String>>( base_address: S ) -> Result<Self,Error> {
        let c = Client::builder()
            .timeout(Duration::from_secs(10))
            .build().map_err(Error::from)?;

        Ok(QueueClient { 
            base_address: base_address.into(), 
            http_client: c,
            version_timeout: None,
        })
    }
}

///////////////////////////////////////////////////////////////
/// Версия log_http_service
#[derive(Debug,Clone,Deserialize)]
pub struct Version {
    pub debug: bool,
    pub crate_name: String,
    pub crate_ver: String
}

impl QueueClient {
    /// Запрос текущей версии ПО
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

#[test]
fn version_direct() {
    use actix_rt::System;
    System::new().block_on(async {
        let client = QueueClient::new(BASE_ADDR).unwrap();
        let ver = client.version().await.unwrap();
        println!("ver {ver:?}");
    })
}

////////////////////////////////////////////////////////////////
/// Лог файл
#[derive(Debug,Clone,Deserialize)]
pub struct LogFile {
    /// Идентификатор лог файла
    pub log_id: String,

    /// Расположение лог файла
    pub log_file: String,

    /// Кол-во записей
    pub items_count: Option<u32>,

    /// Кол-во байт
    pub bytes_count: Option<u64>,
}

#[derive(Debug,Clone,Deserialize)]
pub struct LogFiles {
    pub files: Vec<LogFile>,
}

impl QueueClient {
    /// Запрос лог файлов
    pub async fn log_files( &self ) -> Result<LogFiles,Error> {
        let req = self.http_client
        .get(format!("{}/queue/log/files",self.base_address));

        Ok(req.send().await?.json().await?)
    }
}

#[test]
fn log_files() {
    use actix_rt::System;
    System::new().block_on(async {
        let client = QueueClient::new(BASE_ADDR).unwrap();
        let result = client.log_files().await.unwrap();
        println!("files:\n {result:?}");
    })
}


////////////////////////////////////////////////
//// Получение QueueRecId текущей очереди - позиция конца очереди

/// Идентификатор записи в очереди
#[derive(Debug,Clone,Deserialize)]
pub struct QueueRecId {
    /// Идентификатор лог файла
    pub log_id: String,

    /// Идентификатор записи в логе
    pub block_id: String,
}

impl QueueClient {
    /// Запрос лог файлов
    pub async fn tail_id( &self ) -> Result<QueueRecId,Error> {
        let req = self.http_client
        .get(format!("{}/queue/tail/id",self.base_address));

        Ok(req.send().await?.json().await?)
    }    
}

#[test]
fn tail_id() {
    use actix_rt::System;
    System::new().block_on(async {
        let client = QueueClient::new(BASE_ADDR).unwrap();
        let result = client.tail_id().await.unwrap();
        println!("tail_id:\n {result:?}");
    })
}
