use std::sync::{PoisonError, RwLockReadGuard, RwLockWriteGuard};
#[allow(unused)]
use std::{path::PathBuf, fmt::Debug};

use crate::logfile::{block::FileOffset, FlatBuff};
#[allow(unused)]
use crate::{logfile::{LogErr, LogFile, block::BlockId}, bbuff::absbuff::{ABuffError, FileBuff}};

use super::{new_file::NewFileGeneratorErr, LogFileQueue};
#[allow(unused)]
use super::{LogIdReadWriteErr, LogQueueFileNumID, LogWriteErr};

/// Ошибки очереди логов
#[derive(Clone,Debug)]
pub enum LoqErr<FILE,LogId>
where 
    FILE: Clone + Debug,
    LogId: Clone + Debug
{
    /// Не возможно прочитать кол-во записей в логе
    CantReadRecordsCount {
        file: FILE,
        error: LogErr
    },

    /// Ошибка чтения идентификатора файла - не возможно прочитать блок
    CantReadLogId {
        file: FILE,        
        error: LogErr,
        log_id_type: String,
    },

    /// Ошибка чтения идентификатора файла - не возможно распознать значения записанные в блоке
    CantParseLogId {
        file: FILE,
        error: LogIdReadWriteErr,
        log_id_type: String,
    },

    /// Ошибка парсинга шаблона имени файла
    CantParsePathTemplate {
        error: String,
        template: String,
    },

    /// Ошибка генерации 
    CantGenerateNewFile {
        error: NewFileGeneratorErr,
    },

    /// Ошибка блокировки
    CantCaptureWriteLock {
        error: String,
    },

    /// Ошибка открытия файлового лог файла 
    OpenFileBuff {
        file: FILE,
        error: ABuffError,
    },

    /// Ошибка открытия лог файла
    OpenLog {
        file: FILE,
        error: LogErr,
    },

    /// При открытии лог файлов, обнаружены две "головы" - скорей всего присуствубт лишние лог файлы
    OpenTwoHeads {
        heads: Vec<(FILE,LogId)>
    },

    /// Не найдена голова - косяк с лог файлами, появилась цикличная ссылка
    OpenNoHeads,

    /// Нельзя записть id файла, т.к. файл уже содержит данные
    LogNotEmpty {
        file: FILE,
    },

    /// дублирование id в лог файлах
    OpenLogDuplicateId {
        id: LogId,
        files: Vec<FILE>
    },

    /// В списке лог файлов, пропущен необходимй лог файл
    OpenLogNotFound {
        prev_file: FILE,
        prev_id: LogId,
        next_file: FILE,
        next_id: LogId,
    },

    /// Ошибка записи log id в файл
    LogIdWrite {
        file: FILE,
        error: LogIdReadWriteErr
    },

    /// Ошибка записи log id в файл
    LogIdWrite2 {
        file: FILE,
        error: LogErr,
    },

    /// Ошибка записи данных в лог
    LogDataWrite {
        file: FILE,
        error: LogErr
    },

    /// Ощибка подсчета кол-ва элементов в лог файле
    LogCountFail {
        file: FILE,
        error: LogErr
    },

    /// Не найден лог файл с указанным id
    LogIdNotMatched {
        log_id: LogId
    },

    /// Ошибка чтения лог файла
    LogGetBlock {
        file: FILE,
        error: LogErr,
        block_id: BlockId,
    },

    /// Ошибка чтения сырого набора байтов
    LogRawRead {
        file: FILE,
        log_id: LogId,
        pos: FileOffset,
        data_size: usize,
        error: LogErr,
    }
}

impl<FILE,LogId,BUFF> From<PoisonError<RwLockReadGuard<'_, dyn LogFileQueue<LogId, FILE, LogFile<BUFF>>>>> for LoqErr<FILE,LogId>
where 
    FILE: Clone + Debug,
    LogId: Clone + Debug,
    BUFF: FlatBuff,
{
    fn from(value: PoisonError<RwLockReadGuard<'_, dyn LogFileQueue<LogId, FILE, LogFile<BUFF>>>>) -> Self {
        todo!()
    }
}

impl<FILE,LogId,BUFF> From<PoisonError<RwLockWriteGuard<'_, dyn LogFileQueue<LogId, FILE, LogFile<BUFF>>>>> for LoqErr<FILE,LogId>
where 
    FILE: Clone + Debug,
    LogId: Clone + Debug,
    BUFF: FlatBuff,
{
    fn from(value: PoisonError<RwLockWriteGuard<'_, dyn LogFileQueue<LogId, FILE, LogFile<BUFF>>>>) -> Self {
        todo!()
    }
}

// impl<FILE,LogId,BUFF> FromResidual<Result<Infallible, PoisonError<RwLockReadGuard<'_, dyn log_queue::LogFileQueue<LogId, FILE, logfile::logfile::LogFile<BUFF>>>>>> for  LoqErr<FILE,LogId>
// where 
//     FILE: Clone + Debug,
//     LogId: Clone + Debug,
//     BUFF: FlatBuff,
// {

// }