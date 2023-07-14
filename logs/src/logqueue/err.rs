use std::{path::PathBuf, fmt::Debug};

use crate::{logfile::{LogErr, LogFile, block::BlockId}, bbuff::absbuff::{ABuffError, FileBuff}};

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

    CantReadLogId {
        file: FILE,        
        error: LogErr,
        log_id_type: String,
    },

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

    OpenLogNotFound {
        id: LogId,
        logs: Vec<(FILE,LogId)>
    },

    LogIdWriteFailed {
        file: FILE,
        error: LogIdReadWriteErr
    },

    LogIdWriteFailed2 {
        file: FILE,
        error: LogErr,
    },

    LogDataWriteFailed {
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

impl<FILE,LogId> From<LogWriteErr> for LoqErr<FILE,LogId> 
where 
    FILE: Clone + Debug,
    LogId: Clone + Debug,
{
    fn from(value: LogWriteErr) -> Self {
        LoqErr::LogDataWriteFailed { error: value.0 }
    }
}