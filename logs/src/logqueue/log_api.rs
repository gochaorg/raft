use crate::logfile::{block::{BlockOptions, BlockId, FileOffset, BlockHeadSize, BlockDataSize, BlockTailSize}, LogErr};
use core::fmt::Debug;
use super::{LoqErr, LogFileQueue};

/// Навигация по смеженным записям
/// 
/// Типы
/// - `Err` - Ошибки
pub trait LogNavigationNear {
    /// Идентификатор записи
    type RecordId: Sized;

    /// Тип файла, имееться виду PathBuf
    type FILE: Clone + Debug;

    /// Тип идентификатора лог файла, имеется ввиду LogQueueFileNumID
    type LogId: Clone + Debug;

    /// Получение id следующей записи
    fn next_record( &self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,LoqErr<Self::FILE,Self::LogId>>;

    /// Получение id предыдущей записи
    fn previous_record( &self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,LoqErr<Self::FILE,Self::LogId>>;    
}

/// Навигация в конец
pub trait LogNavigateLast<RecordId,FILE,LogId> 
where
    FILE: Clone + Debug,
    LogId: Clone + Debug
{
    /// Идентификатор записи
    // type RecordId: Sized;

    /// Тип файла, имееться виду PathBuf
    // type FILE: Clone + Debug;

    /// Тип идентификатора лог файла, имеется ввиду LogQueueFileNumID
    // type LogId: Clone + Debug;

    /// Получение последней записи в log queue
    fn last_record( &self ) -> Result<Option<RecordId>,LoqErr<FILE,LogId>>;
}

/// Информация о записи
pub struct RecordInfo<FILE,LogId> 
where
    FILE: Clone+Debug,
    LogId: Clone+Debug,
{
    /// Лог файл
    pub log_file: FILE,

    /// Идентификатор лог файла
    pub log_id: LogId,

    /// Идентификатор блока
    pub block_id: BlockId,

    /// Опции блока
    pub block_options: BlockOptions,

    /// Смещение в файле
    pub position: FileOffset,

    /// Размер заголовка
    pub head_size: BlockHeadSize,

    /// Размер данных после заголовка
    pub data_size: BlockDataSize,

    /// Размер хвоста после данных
    pub tail_size: BlockTailSize,
}

/// Чтение отдельных записей
pub trait LogReading {
    /// Идентификатор записи
    type RecordId: Sized;

    /// Тип файла, имееться виду PathBuf
    type FILE: Clone + Debug;

    /// Тип идентификатора лог файла, имеется ввиду LogQueueFileNumID
    type LogId: Clone + Debug;

    /// Чтение записи и ее опций
    /// 
    /// Аргументы
    /// - `record_id` идентификатор записи
    /// 
    /// Результат - запись
    fn read( &self, record_id: Self::RecordId ) -> 
        Result<PreparedRecord, LoqErr<Self::FILE,Self::LogId>>;

    /// Чтение опций записи
    fn info( &self, record_id: Self::RecordId ) -> 
        Result<RecordInfo<Self::FILE,Self::LogId>, LoqErr<Self::FILE,Self::LogId>>;

    /// Чтение байтов лог файла
    /// 
    /// Аргументы
    /// - `log_id` - идентификатор лог файла
    /// - `pos` - позиция в лог файле
    /// - `data_consumer` - куда записывать данные
    /// 
    /// Результат
    /// - Кол-во прочитанных байтов
    fn read_raw_bytes( &self, log_id: Self::LogId, pos: FileOffset, data_consumer:&mut [u8] ) ->
        Result<u64, LoqErr<Self::FILE, Self::LogId>>;
}

/// Подготовленные данные для записи
#[derive(Clone)]
pub struct PreparedRecord {
    pub data: Vec<u8>,
    pub options: BlockOptions,
}

pub struct LogWriteErr(pub LogErr);

/// Запись в лог
pub trait LogWriting<RecordId> 
{
    /// Тип файла, имееться виду PathBuf
    type FILE: Clone + Debug;

    /// Тип идентификатора лог файла, имеется ввиду LogQueueFileNumID
    type LogId: Clone + Debug;

    /// Запись данных в лог
    fn write( &self, record:&PreparedRecord ) -> Result<RecordId,LoqErr<Self::FILE,Self::LogId>>;
}

/// Общий API лог очереди
pub trait LogQueue<RecordId,LogId,FILE,LOG> 
: LogNavigateLast<RecordId,FILE,LogId>
+ LogWriting<RecordId, FILE = FILE, LogId = LogId>
+ LogReading<RecordId = RecordId, FILE = FILE, LogId = LogId>
+ LogNavigationNear<RecordId = RecordId, FILE = FILE, LogId = LogId>
+ LogFileQueue<LogId, FILE, LOG>
where
    RecordId: Sized,
    LogId: Clone + Debug,
    FILE: Clone + Debug,
{}

