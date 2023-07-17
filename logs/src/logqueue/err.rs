#[allow(unused)]
use std::{path::PathBuf, fmt::Debug};

#[allow(unused)]
use crate::{logfile::{LogErr, LogFile, block::BlockId}, bbuff::absbuff::{ABuffError, FileBuff}};

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

    OpenFileBuff {
        file: FILE,
        error: ABuffError,
    },

    OpenLog {
        file: FILE,
        error: LogErr,
    },

    OpenTwoHeads {
        heads: Vec<(FILE,LogId)>
    },

    OpenNoHeads,

    /// Нельзя записть id файла, т.к. файл уже содержит данные
    LogNotEmpty {
        file: FILE,
    },

    OpenLogNotFound {
        id: LogId,
        logs: Vec<(FILE,LogId)>
    },

    LogIdWrite {
        file: FILE,
        error: LogIdReadWriteErr
    },

    LogIdWrite2 {
        file: FILE,
        error: LogErr,
    },

    LogDataWrite {
        file: FILE,
        error: LogErr
    },

    LogCountFail {
        file: FILE,
        error: LogErr
    },

    LogIdNotMatched {
        log_id: LogId
    },

    LogGetBlock {
        file: FILE,
        error: LogErr,
        block_id: BlockId,
    }
}
