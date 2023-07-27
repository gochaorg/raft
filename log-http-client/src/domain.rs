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
    ParseBlockId { from: String, error: String },
    ParseBlock(BlockErr),
    ParseLogId { from: String, error: String },
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
pub struct LogFileInfoRaw {
    /// Идентификатор лог файла - по факту должно быть число u128
    pub log_id: String,

    /// Имя лог файла
    pub log_file: String,

    /// Кол-во записей
    #[serde(default)]
    pub items_count: Option<u32>,

    /// Размер лог файла в байтах
    #[serde(default)]
    pub bytes_count: Option<u64>,
}

/// Лог файл
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct LogFileInfo {
    /// Идентификатор лог файла
    pub log_id: u128,

    /// Имя лог файла
    pub log_file: String,

    /// Кол-во записей
    #[serde(default)]
    pub items_count: Option<u32>,

    /// Размер лог файла в байтах
    #[serde(default)]
    pub bytes_count: Option<u64>,
}

impl TryFrom<LogFileInfoRaw> for LogFileInfo {
    type Error = ClientError;
    fn try_from(value: LogFileInfoRaw) -> Result<Self, Self::Error> {
        let lid = 
            u128::from_str_radix(&value.log_id, 10)
            .map_err(|e| ClientError::ParseLogId { 
                from: value.log_id.clone(), 
                error: e.to_string()
            })?;

        Ok(Self { 
            log_id: lid, 
            log_file: value.log_file, 
            items_count: value.items_count, 
            bytes_count: value.bytes_count 
        })
    }
}

/// Список лог файлов
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct LogFilesRaw {
    pub files: Vec<LogFileInfoRaw>
}

/// Список лог файлов
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct LogFiles {
    pub files: Vec<LogFileInfo>
}

impl TryFrom<LogFilesRaw> for LogFiles {
    type Error = ClientError;
    fn try_from(value: LogFilesRaw) -> Result<Self, Self::Error> {
        let sum : Result<Vec<LogFileInfo>,ClientError> = Ok(vec![]);
        let files = value.files.iter().fold(
            sum, 
            |sum,it| {                
                sum.and_then(|mut sum| {
                    sum.push(it.clone().try_into()?);
                    Ok(sum)
                })
            })?;
        Ok(Self { files: files })
    }
}

/// Идентификатор последней записи в логе
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct TailIdRaw {
    /// Идентификатор лог файла - по факту должно быть число u128
    pub log_id: String,

    /// Индентификатор блока - по факту должно быть числов u32
    pub block_id: String,
}

/// Переключение лога
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct TailSwitch {
    /// Новый лог файл
    pub log_file: String,

    /// Идентификатор лог файла - по факту должно быть число u128
    pub log_id: String,
}

/// Идентификатор записи
#[derive(Debug,Clone,PartialEq,Eq,Copy,Deserialize,Serialize)]
pub struct RecId {
    /// Идентификатор лог файла
    pub log_id: u128,

    /// Индентификатор блока
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

impl TryFrom<TailIdRaw> for RecId {
    type Error = ClientError;
    fn try_from(value: TailIdRaw) -> Result<Self, Self::Error> {
        let log_id = u128::from_str_radix(&value.log_id,10)
            .map_err(|e|
                ClientError::ParseLogId{ from: value.log_id.clone(), error: e.to_string() }
            )?;
        let block_id = u32::from_str_radix(&value.block_id,10)
        .map_err(|e|
            ClientError::ParseBlockId {
                from: value.block_id.clone(),
                error: e.to_string(),
            }
        )?;        
        Ok(Self { log_id: log_id, block_id: block_id })
    }    
}