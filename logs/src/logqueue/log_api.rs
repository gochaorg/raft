use crate::logfile::{block::BlockOptions, LogErr};
use core::fmt::Debug;
use super::LoqErr;

/// Навигация по смеженным записям
/// 
/// Типы
/// - `Err` - Ошибки
pub trait LogNavigationNear<ERR> {
    /// Идентификатор записи
    type RecordId: Sized;

    /// Получение id следующей записи
    fn next_record( &self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,ERR>;

    /// Получение id предыдущей записи
    fn previous_record( &self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,ERR>;    
}

/// Навигация в конец
pub trait LogNavigateLast<ERR> {
    /// Идентификатор записи
    type RecordId: Sized;

    /// Получение последней записи в log queue
    fn last_record( &self ) -> Result<Option<Self::RecordId>,ERR>;
}

/// Чтение отдельных записей
pub trait LogReading<Record,RecordOptions> {
    /// Идентификатор записи
    type RecordId: Sized;

    type FILE: Clone + Debug;
    type LogId: Clone + Debug;

    /// Чтение записи и ее опций
    fn read_record( &self, record_id: Self::RecordId ) -> 
        Result<(Record,RecordOptions), LoqErr<Self::FILE,Self::LogId>>;

    /// Чтение опций записи
    fn read_options( &self, record_id: Self::RecordId ) -> 
        Result<RecordOptions, LoqErr<Self::FILE,Self::LogId>>;
}

/// Подготовленные данные для записи
pub struct PreparedRecord {
    pub data: Box<[u8]>,
    pub options: BlockOptions,
}

pub struct LogWriteErr(pub LogErr);

/// Запись в лог
pub trait LogWriting<RecordId> 
{
    type FILE: Clone + Debug;
    type LogId: Clone + Debug;

    fn write<Record>( self, record:Record ) -> Result<RecordId,LoqErr<Self::FILE,Self::LogId>>
    where Record: Into<PreparedRecord>;
}
