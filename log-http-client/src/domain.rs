use awc::error::{SendRequestError, JsonPayloadError, PayloadError};
use logs::logfile::block::BlockErr;
use serde::{Deserialize, Serialize};

/// Ошибки клиента
#[derive(Debug,Clone)]
pub enum ClientError {
    HttpClientError(String),
    Status{ code:u16, body:String },
    JsonParseError(String),
    RawBytesError(String),
    ParseRecId(String),
    ParseBlock(BlockErr)
}

impl From<SendRequestError> for ClientError {
    fn from(value: SendRequestError) -> Self {
        ClientError::HttpClientError(value.to_string())
    }
}

impl From<JsonPayloadError> for ClientError {
    fn from(value: JsonPayloadError) -> Self {
        ClientError::JsonParseError(value.to_string())
    }
}

impl From<PayloadError> for ClientError {
    fn from(value: PayloadError) -> Self {
        ClientError::RawBytesError(value.to_string())
    }
}

/// Версия сервера
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct QueueApiVersion {
    pub debug: bool,
    pub crate_name: String,
    pub crate_ver: String,
}

/// Лог файл
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct LogFileInfo {
    pub log_id: String,
    pub log_file: String,

    #[serde(default)]
    pub items_count: Option<u32>,

    #[serde(default)]
    pub bytes_count: Option<u64>,
}

/// Список лог файлов
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct LogFiles {
    pub files: Vec<LogFileInfo>
}

/// Идентификатор последней записи в логе
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct TailId {
    pub log_id: String,
    pub block_id: String,
}

/// Переключение лога
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct TailSwitch {
    pub log_file: String,
    pub log_id: String,
}

/// Идентификатор записи
#[derive(Debug,Clone,PartialEq,Eq,Copy,Deserialize,Serialize)]
pub struct RecId {
    pub log_id: u128,
    pub block_id: u32,
}

impl RecId {
    pub fn new( log_id: u128, block_id:u32 ) -> Self {
        Self { log_id: log_id, block_id: block_id }
    }
}

impl PartialOrd for RecId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.log_id.partial_cmp(&other.log_id) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.block_id.partial_cmp(&other.block_id)
    }
}

impl Ord for RecId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.log_id.cmp(&other.log_id) {
            core::cmp::Ordering::Equal => {}
            ord => return ord
        }
        self.block_id.cmp(&other.block_id)
    }
}

impl TryFrom<TailId> for RecId {
    type Error = ClientError;
    fn try_from(value: TailId) -> Result<Self, Self::Error> {
        let log_id = u128::from_str_radix(&value.log_id,10)
            .map_err(|e|
                ClientError::ParseRecId(e.to_string())
            )?;
        let block_id = u32::from_str_radix(&value.log_id,10)
        .map_err(|e|
            ClientError::ParseRecId(e.to_string())
        )?;
        Ok(Self { log_id: log_id, block_id: block_id })
    }    
}