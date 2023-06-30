use std::fmt::Debug;
use std::str::FromStr;
use uuid::Uuid;
use crate::logfile::block::{String32, BlockErr};
use crate::logfile::block::Block;
use std::hash::Hash;

/// Запись идентификатора в блок
pub trait BlockWriter {
    type ERR;
    fn block_write( &self, block: &mut Block ) -> Result<(),Self::ERR>;
}

/// Чтение индентификатора из блока
pub trait BlockReader 
where
    Self: Sized
{
    type ERR;
    fn block_read( block: &Block ) -> Result<Self, Self::ERR>;
}

/// Идентификатор лог файла
pub trait LogQueueFileId : PartialEq + std::fmt::Display + Clone + Debug + BlockReader + BlockWriter + Hash {
    type ID: PartialEq;
    fn id( &self ) -> Self::ID;
    fn previous( &self ) -> Option<Self::ID>;
    fn new( prev:Option<Self::ID> ) -> Self;
}

/// Идентификатор лог файла - UUID
#[derive(Debug,Clone,PartialEq,Hash)]
pub struct LogQueueFileUUID {
    pub uuid: Uuid,
    pub previous: Option<Uuid>,
}

impl std::fmt::Display for LogQueueFileUUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"LogQueueFileUUID({})", match self.previous {
            Some(prev) => format!("{}, prev={}", self.uuid, prev),
            None => format!("{}", self.uuid)
        })
    }
}

impl LogQueueFileId for LogQueueFileUUID {
    type ID = Uuid;
    fn id( &self ) -> Self::ID {
        self.uuid.clone()
    }
    fn new( prev:Option<Self::ID> ) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            previous: prev
        }
    }
    fn previous( &self ) -> Option<Self::ID> {
        self.previous.clone()
    }
}

pub const LOG_FILE_ID_KEY: &str =      "log_file_id";
pub const LOG_FILE_ID_PREV_KEY: &str = "log_file_id_prev";
pub const LOG_FILE_ID_TYPE_KEY: &str = "log_file_id_type";
pub const LOG_FILE_UUID_TYPE: &str =    "LogQueueFileUUID";

/// Ошибки чтения / записи идентификатора
pub enum LogIdReadWriteErr {
    BlockErr(BlockErr),
    ValueAlreadyDefined(String32),
    ValueNotFound,
    TypeValueAlreadyDefined(String32),
    TypeValueNotMatched{expect:String32, actual:String32},
    TypeValueNotFound,
    PrevValueAlreadyDefined(String32),
    UuidParseError(String)
}

impl From<BlockErr> for LogIdReadWriteErr {
    fn from(value: BlockErr) -> Self {
        LogIdReadWriteErr::BlockErr(value.clone())
    }
}

impl From<uuid::Error> for LogIdReadWriteErr {
    fn from(value: uuid::Error) -> Self {
        LogIdReadWriteErr::UuidParseError(value.to_string())
    }
}

impl BlockWriter for LogQueueFileUUID {
    type ERR = LogIdReadWriteErr;

    fn block_write( &self, block: &mut Block ) -> Result<(),Self::ERR> {
        let value : Option<String32> = block.head.block_options.get(LOG_FILE_ID_KEY);
        if value.is_some() {
            return Err(LogIdReadWriteErr::ValueAlreadyDefined(value.unwrap()));
        }

        let type_of_value : Option<String32> = block.head.block_options.get(LOG_FILE_ID_TYPE_KEY);
        if type_of_value.is_some() {
            return Err(LogIdReadWriteErr::TypeValueAlreadyDefined(type_of_value.unwrap()));
        }

        let prev : Option<String32> = block.head.block_options.get(LOG_FILE_ID_PREV_KEY);
        if prev.is_some() {
            return Err(LogIdReadWriteErr::PrevValueAlreadyDefined(prev.unwrap()));
        }
        
        block.head.block_options.set(LOG_FILE_ID_TYPE_KEY, LOG_FILE_UUID_TYPE)?;
        block.head.block_options.set(LOG_FILE_ID_KEY, self.uuid.to_string())?;
        if self.previous.is_some() {
            block.head.block_options.set(LOG_FILE_ID_PREV_KEY, self.previous.unwrap().to_string())?;
        }

        Ok(())
    }
}

impl BlockReader for LogQueueFileUUID {
    type ERR = LogIdReadWriteErr;

    fn block_read( block: &Block ) -> Result<Self, Self::ERR> {
        let type_of_value : Option<String32> = block.head.block_options.get(LOG_FILE_ID_TYPE_KEY);
        if type_of_value.is_none() { return Err(LogIdReadWriteErr::TypeValueNotFound); };
        if type_of_value.clone().unwrap().value() != LOG_FILE_UUID_TYPE { 
            return Err(LogIdReadWriteErr::TypeValueNotMatched {
                expect: String32::try_from(LOG_FILE_UUID_TYPE).unwrap(),
                actual: type_of_value.unwrap().clone()
            }); 
        };

        let value : Option<String32> = block.head.block_options.get(LOG_FILE_ID_KEY);
        if value.is_none() {
            return Err(LogIdReadWriteErr::ValueNotFound);
        }
        let value = Uuid::from_str(value.unwrap().value())?;

        let prev : Option<String32> = block.head.block_options.get(LOG_FILE_ID_PREV_KEY);
        if prev.is_none() {
            Ok(LogQueueFileUUID { uuid: value, previous: None })
        } else {
            let prev = Uuid::from_str(prev.unwrap().value())?;
            Ok(LogQueueFileUUID { uuid: value, previous: Some(prev) })
        }
    }
}