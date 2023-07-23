use std::fmt::Debug;
use std::num::ParseIntError;
use std::str::FromStr;
use crate::logfile::{LogFile, FlatBuff};
use crate::logfile::block::{String32, BlockErr, BlockId, BlockOptions};
use crate::logfile::block::Block;
use std::hash::Hash;
use std::any::type_name;

use super::LoqErr;

/// Запись идентификатора в блок
pub trait BlockWriter {
    fn block_write( &self, options: &mut BlockOptions, data: &mut Vec<u8> ) -> Result<(),LogIdReadWriteErr>;
}

/// Чтение индентификатора из блока
pub trait BlockReader 
where
    Self: Sized
{
    fn block_read( block: &Block ) -> Result<Self, LogIdReadWriteErr>;
}

/// Идентификатор лог файла
pub trait LogQueueFileId : Eq + std::fmt::Display + Clone + Copy + Debug + BlockReader + BlockWriter + Hash + Ord {
    type ID: Eq + Clone;

    /// Получение идентификатора
    fn id( &self ) -> Self::ID;

    /// Получение идентификатора предыдущего блока
    fn previous( &self ) -> Option<Self::ID>;

    /// Генерация нового идентификатора
    fn new( prev:Option<Self::ID> ) -> Self;

    /// Чтение идентификатора из лог файла
    /// 
    /// Читает первый блок в лог файле
    fn read<FILE,BUFF>( filename:&FILE, log:&LogFile<BUFF> ) -> Result<Self,LoqErr<FILE,Self>> 
    where
        FILE: Clone + Debug,
        BUFF: FlatBuff,
    {
        let id_type = type_name::<Self>().to_string();

        let block = 
            log.read_block(BlockId::new(0))
            .map_err(|err| LoqErr::CantReadLogId { 
                file: filename.clone(), 
                error: err, 
                log_id_type: id_type.clone() 
            })?;

        let id = Self::block_read(&block)
        .map_err(|err| LoqErr::CantParseLogId { 
            file: filename.clone(), 
            error: err, 
            log_id_type: id_type.clone() 
        })?;
        
        Ok(id)
    }

    /// Запись идентификатора лог файла, первым блоком 
    /// файл должен быть пустым
    fn write<FILE,BUFF>( &self, filename:&FILE, log:&mut LogFile<BUFF> ) -> Result<(),LoqErr<FILE,Self>>
    where
        FILE: Clone+Debug,
        BUFF: FlatBuff,
    {
        let mut options = BlockOptions::default();
        let mut data = Vec::<u8>::new();

        self.block_write(&mut options, &mut data).
            map_err(|err| LoqErr::LogIdWrite { 
            file: filename.clone(),
            error: err
        })?;

        let count = log.count().map_err(|e| LoqErr::LogCountFail { file: filename.clone(), error: e })?;
        if count>0 {
            return Err(LoqErr::LogNotEmpty { file: filename.clone() });
        }

        log.write_block(&options, &data)
            .map_err(|err|
            LoqErr::LogIdWrite2 { 
                file: filename.clone(), 
                error: err 
            })?;

        Ok(())
    }
}

///  Идентификатор лог файла - число
#[derive(Debug,Clone,PartialEq,Hash)]
pub struct LogQueueFileNumID {
    pub id:u128,
    pub previous:Option<u128>
}

impl std::fmt::Display for LogQueueFileNumID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"LogQueueFileNumID({})", match self.previous {
            Some(prev) => format!("{}, prev={}", self.id, prev),
            None => format!("{}", self.id)
        })
    }
}

impl Eq for LogQueueFileNumID {}

impl Copy for LogQueueFileNumID {}

impl LogQueueFileId for LogQueueFileNumID {
    type ID = u128;
    fn id( &self ) -> Self::ID {
        self.id.clone()
    }
    fn new( prev:Option<Self::ID> ) -> Self {
        match prev {
            Some(id_prev) => Self {
                id: id_prev+1,
                previous: prev
            },
            None => Self { id: 0u128, previous: None }
        }
    }
    fn previous( &self ) -> Option<Self::ID> {
        self.previous.clone()
    }
}

impl Ord for LogQueueFileNumID {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for LogQueueFileNumID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.id.partial_cmp(&other.id) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.previous.partial_cmp(&other.previous)
    }
}

impl BlockWriter for LogQueueFileNumID {
    fn block_write( &self, options: &mut BlockOptions, _data: &mut Vec<u8> ) -> Result<(),LogIdReadWriteErr> {
        let value : Option<String32> = options.get(LOG_FILE_ID_KEY);
        if value.is_some() {
            return Err(LogIdReadWriteErr::ValueAlreadyDefined(value.unwrap()));
        }

        let type_of_value : Option<String32> = options.get(LOG_FILE_ID_TYPE_KEY);
        if type_of_value.is_some() {
            return Err(LogIdReadWriteErr::TypeValueAlreadyDefined(type_of_value.unwrap()));
        }

        let prev : Option<String32> = options.get(LOG_FILE_ID_PREV_KEY);
        if prev.is_some() {
            return Err(LogIdReadWriteErr::PrevValueAlreadyDefined(prev.unwrap()));
        }
        
        options.set(LOG_FILE_ID_TYPE_KEY, LOG_FILE_NUM_TYPE)?;
        options.set(LOG_FILE_ID_KEY, self.id.to_string())?;
        if self.previous.is_some() {
            options.set(LOG_FILE_ID_PREV_KEY, self.previous.unwrap().to_string())?;
        }

        Ok(())
    }
}

impl BlockReader for LogQueueFileNumID {
    fn block_read( block: &Block ) -> Result<Self, LogIdReadWriteErr> {
        let type_of_value : Option<String32> = block.head.block_options.get(LOG_FILE_ID_TYPE_KEY);
        if type_of_value.is_none() { return Err(LogIdReadWriteErr::TypeValueNotFound); };
        if type_of_value.clone().unwrap().value() != LOG_FILE_NUM_TYPE { 
            return Err(LogIdReadWriteErr::TypeValueNotMatched {
                expect: String32::try_from(LOG_FILE_NUM_TYPE).unwrap(),
                actual: type_of_value.unwrap().clone()
            }); 
        };

        let value : Option<String32> = block.head.block_options.get(LOG_FILE_ID_KEY);
        if value.is_none() {
            return Err(LogIdReadWriteErr::ValueNotFound);
        }
        let value = u128::from_str(value.unwrap().value())?;

        let prev : Option<String32> = block.head.block_options.get(LOG_FILE_ID_PREV_KEY);
        if prev.is_none() {
            Ok(LogQueueFileNumID { id: value, previous: None })
        } else {
            let prev = u128::from_str(prev.unwrap().value())?;
            Ok(LogQueueFileNumID { id: value, previous: Some(prev) })
        }
    }
}

pub const LOG_FILE_ID_KEY: &str =       "log_file_id";
pub const LOG_FILE_ID_PREV_KEY: &str =  "log_file_id_prev";
pub const LOG_FILE_ID_TYPE_KEY: &str =  "log_file_id_type";
pub const LOG_FILE_NUM_TYPE: &str =     "LogQueueFileNumID";

/// Ошибки чтения / записи идентификатора
#[derive(Debug,Clone)]
pub enum LogIdReadWriteErr {
    BlockErr(BlockErr),
    ValueAlreadyDefined(String32),
    ValueNotFound,
    TypeValueAlreadyDefined(String32),
    TypeValueNotMatched{expect:String32, actual:String32},
    TypeValueNotFound,
    PrevValueAlreadyDefined(String32),
    IdParseError(String)
}

impl From<BlockErr> for LogIdReadWriteErr {
    fn from(value: BlockErr) -> Self {
        LogIdReadWriteErr::BlockErr(value.clone())
    }
}

impl From<ParseIntError> for LogIdReadWriteErr {
    fn from(value: ParseIntError) -> Self {
        LogIdReadWriteErr::IdParseError(value.to_string())
    }
}

/// Указатель на запись в log queue
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RecID<LogId> 
where
    LogId: LogQueueFileId
{
    /// Идентификатор лог - файла
    pub log_file_id: LogId,

    /// Идентификатор записи в лог файле
    pub block_id: BlockId,
}


