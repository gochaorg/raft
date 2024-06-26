use core::str;
use std::fmt::{format, write, Display};
use std::{collections::HashMap, fmt::Debug};
use std::time::Duration;
use logs::logfile::block::{BlockErr, BlockOptions, String16, String32};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
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

//#region version()

/// Версия log_http_service
#[derive(Debug,Clone,Deserialize,Serialize)]
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

//#endregion

//#region log_files()

/// Лог файл
#[derive(Debug,Clone,Deserialize,Serialize)]
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

#[derive(Debug,Clone,Deserialize,Serialize)]
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

//#endregion

//#region tail_id()

/// Идентификатор записи в очереди
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct QueueBlockId {
    /// Идентификатор лог файла
    pub log_id: String,

    /// Идентификатор записи в логе
    pub block_id: u32,
}

impl Display for QueueBlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"QueueBlockId {{ log_id={log_id}, block_id={block_id} }}", 
            log_id=self.log_id, 
            block_id=self.block_id)
    }
}

impl QueueBlockId {
    async fn try_send(request: RequestBuilder) -> Result<QueueBlockId, Error> {
        #[derive(Debug,Clone,Deserialize,Serialize)]
        struct QueueRecIdRaw {
            /// Идентификатор лог файла
            pub log_id: String,
        
            /// Идентификатор записи в логе
            pub block_id: String,
        }
        
        
        Ok(request.send().await?.json::<QueueRecIdRaw>().await?)
        .and_then(|rraw|{
            rraw.block_id.parse::<u32>()
            .map_err(|e| Error::DecodeBody(format!("{e:?}")))
            .map(|b_id| {
                QueueBlockId { log_id: rraw.log_id.clone(), block_id: b_id }
            })
        })
    }
}

impl QueueClient {
    /// Получение QueueBlockId текущей очереди - позиция конца очереди
    pub async fn tail_id( &self ) -> Result<QueueBlockId,Error> {
        let req = self.http_client
        .get(format!("{}/queue/tail/id",self.base_address));

        QueueBlockId::try_send(req).await
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

//#endregion

//#region block_info()

//// Заголовки записи 
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct QueueBlockInfo {
    pub log_file: String,
    pub log_id: String,
    pub block_id: u32,
    pub position: u64,
    pub head_size: u32,
    pub data_size: u32,
    pub tail_size: u32,
    pub block_options: HashMap<String,String>,
}

impl QueueClient {
    /// Запрос информации о записи
    pub async fn block_info( &self, block_id: &QueueBlockId ) -> Result<QueueBlockInfo,Error> {
        let req = self.http_client
        .get(format!("{addr}/queue/record/{log_id}/{block_id}/headers",
            addr=self.base_address,
            log_id=block_id.log_id.clone(),
            block_id=block_id.block_id
        ));

        Ok(req.send().await?.json().await?)
    }    
}

#[test]
fn block_info() {
    use actix_rt::System;
    System::new().block_on(async {
        let client = QueueClient::new(BASE_ADDR).unwrap();
        let result = client.block_info(
            &QueueBlockId { log_id: "0".to_string(), block_id: 1 }
        ).await.unwrap();
        println!("result:\n {result:?}");
    })
}

//#endregion

//#region block_read()

#[derive(Clone)]
pub struct BlockRead {
    /// Данные блока
    pub bytes: Vec<u8>,

    /// Пользовательские заголовки
    pub options: HashMap<String,String>,

    /// Идентификатор лог файла
    pub log_id: String,

    /// Номер блока в логе
    pub block_id: u32,
}

impl Display for BlockRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        
        s.push_str("Block {\n");
        s.push_str(&format!("  log_id = {}\n", self.log_id));
        s.push_str(&format!("  block_id = {}\n", self.block_id));
        s.push_str(&format!("  bytes size {}\n", self.bytes.len()));

        if self.options.len()>0 {
            s.push_str("  options {\n");
            for (k,v) in self.options.clone().into_iter() {
                s.push_str(&format!("    {k} = {v}\n"));
            }
            s.push_str("  }\n");
        }

        s.push_str("}\n");
        write!(f, "{}", s)
    }
}

impl QueueClient {
    /// Чтение блока
    pub async fn block_read( &self, block_id: &QueueBlockId ) -> Result<BlockRead,Error> {
        let pref="b_opt_";
        let pref_usize = pref.len();

        let res = self.http_client.get(
            format!(
                "{addr}/queue/record/{log_id}/{block_id}/plain?opt2head=true&opt_prefix={pref}",
                addr=self.base_address,
                log_id=block_id.log_id.clone(),
                block_id=block_id.block_id,
                pref=pref
            )
        ).send().await?;

        let mut block_opts: HashMap<String,String> = HashMap::new();

        for (header_name, header_value) in res.headers().into_iter() {
            if header_name.as_str().starts_with(pref) {
                let (_, key) = header_name.as_str().split_at(pref_usize);
                let value = header_value.to_str()
                    .map_err(|e| Error::DecodeBody(format!(
                        "can't decode header {hdr}: {err}", hdr=header_name.as_str(), err=e.to_string())))?;
                block_opts.insert(key.to_string(), value.to_string());
            }
        }

        let bytes = res.bytes().await?.to_vec();

        Ok(BlockRead {
            bytes: bytes,
            options: block_opts,
            log_id: block_id.log_id.clone(),
            block_id: block_id.block_id,
        })
    }
}

#[test]
fn block_read() {
    use actix_rt::System;
    System::new().block_on(async {
        let client = QueueClient::new(BASE_ADDR).unwrap();
        let result = client.block_read(
            &QueueBlockId { log_id: "0".to_string(), block_id: 1 }
        ).await.unwrap();
        println!("result:\n {result}");
    })
}

//#endregion

//#region block_write()
pub struct BlockWrite {
    pub expect_tail: Option<QueueBlockId>,
    pub options: BlockOptions,
    pub data: Vec<u8>,
}

impl BlockWrite {
    pub fn expect_tail( self, qbid: &QueueBlockId ) -> Self {
        Self { expect_tail: Some(qbid.clone()), ..self }
    }

    pub fn option<
    K: TryInto<String16, Error = BlockErr>,
    V: TryInto<String32, Error = BlockErr>
    >( 
        self, k:K, v:V 
    ) -> Result<Self,BlockErr> {
        let mut opt = self.options.clone();
        opt.set(k, v)?;
        Ok(Self { options: opt, ..self })
    }
}

impl QueueClient {
    pub async fn block_write<T: Into<BlockWrite>>( &self, t:T )  -> Result<QueueBlockId,Error> {
        let block_wr: BlockWrite = t.into();
        let pref = "bo_";

        let mut url = match block_wr.expect_tail {
            None => format!(
                "{addr}/queue/record/bytes",
                addr=self.base_address,
            ),
            Some(QueueBlockId { log_id, block_id }) => format!(
                "{addr}/queue/record/{log_id}/{block_id}/bytes",
                addr=self.base_address,
            )
        };

        url.push_str(&format!("?head2opt=true&opt_prefix={pref}"));

        let mut req = self.http_client.post(url);
        for (k,v) in block_wr.options {
            req = req.header(format!("{pref}{k}"), format!("{v}"));
        }

        req = req.body(block_wr.data);

        QueueBlockId::try_send(req).await
    }
}

impl From<Vec<u8>> for BlockWrite {
    fn from(value: Vec<u8>) -> Self {
        Self { data:value, expect_tail:None, options: BlockOptions::default() }
    }
}

impl From<&[u8]> for BlockWrite {
    fn from(value: &[u8]) -> Self {
        let mut bytes: Vec<u8> = vec![];
        bytes.extend_from_slice(value);

        Self { data:bytes, expect_tail:None, options: BlockOptions::default() }
    }
}

impl From<&str> for BlockWrite {
    fn from(value: &str) -> Self {
        let mut bytes: Vec<u8> = vec![];
        bytes.extend_from_slice(value.as_bytes());

        Self { data:bytes, expect_tail:None, options: BlockOptions::default() }
    }
}

#[test]
fn block_write_1() {
    use actix_rt::System;
    System::new().block_on(async {
        let client = QueueClient::new(BASE_ADDR).unwrap();
        let result = client.block_write("blabla").await.unwrap();
        println!("result:\n {result}");
    })
}

#[test]
fn block_write_2() {
    use actix_rt::System;
    System::new().block_on(async {
        let client = QueueClient::new(BASE_ADDR).unwrap();
        let tid = client.tail_id().await.unwrap();
        let result = client.block_write(
            BlockWrite::from("foo").expect_tail( &tid ).option("k", "v").unwrap()
        ).await.unwrap();
        println!("result:\n {result}");
    })
}

//#endregion

//#region log_switch()

/// Переключение логов
#[derive(Debug,Clone,Deserialize)]
pub struct LogSwitched {
    pub log_file: String,
    pub log_id: String,
}

impl Display for LogSwitched {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LogSwitched {{ log_file={log_file}, log_id={log_id} }}", 
            log_file=self.log_file, 
            log_id=self.log_id)
    }
}

impl QueueClient {
    pub async fn log_switch( &self )  -> Result<LogSwitched,Error> {
        let req = self.http_client.post(
            format!("{addr}/queue/tail/switch", addr=self.base_address)
        );

        Ok(req.send().await?.json().await?)
    }
}

#[test]
fn log_switch() {
    use actix_rt::System;
    System::new().block_on(async {
        let client = QueueClient::new(BASE_ADDR).unwrap();
        let result = client.log_switch().await.unwrap();
        println!("result:\n {result}");
    })
}

//#endregion
