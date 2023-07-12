use std::path::PathBuf;

use crate::{logfile::{LogErr, LogFile}, bbuff::absbuff::{ABuffError, FileBuff}};

use super::{LogIdReadWriteErr, LogQueueFileNumID};

/// Ошибки очереди логов
#[derive(Clone,Debug)]
pub enum LoqErr {
    /// Не возможно прочитать кол-во записей в логе
    CantReadRecordsCount {
        file: PathBuf,
        error: LogErr
    },

    CantReadLogId {
        file: PathBuf,        
        error: LogErr,
        log_id_type: String,
    },

    CantParseLogId {
        file: PathBuf,
        error: LogIdReadWriteErr,
        log_id_type: String,
    },

    OpenFileBuff {
        file: PathBuf,
        error: ABuffError,
    },

    OpenLog {
        file: PathBuf,
        error: LogErr,
    },

    OpenTwoHeads {
        heads: Vec<(PathBuf,LogQueueFileNumID)>
    },

    OpenNoHeads,

    OpenLogNotFound {
        id: LogQueueFileNumID,
        logs: Vec<(PathBuf,LogQueueFileNumID)>
    },

    LogIdWriteFailed {
        file: PathBuf,
        error: LogIdReadWriteErr
    },

    LogIdWriteFailed2 {
        file: PathBuf,
        error: LogErr,
    }
}