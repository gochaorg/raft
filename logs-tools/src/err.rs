use std::sync::{PoisonError, RwLockReadGuard};

use logs::{bbuff::absbuff::ABuffError, logfile::LogErr};
use logs::logfile::block::*;

use crate::actions::tag::TagAction;

#[derive(Debug, Clone)]
pub enum LogToolErr {
    Log(LogErr),
    BuffErr(ABuffError),
    IOError {
        message: String,
        os_error: Option<i32>,
    },
    FileSizeToBig,
    RwLockErr {
        message: String,
    },
    BlockError(BlockErr),
    ApplyTagFail {
        message: String,
        tag: TagAction,
    },
    NotImplemented(String)
}

impl From<BlockErr> for LogToolErr {
    fn from(value: BlockErr) -> Self {
        Self::BlockError(value.clone())
    }
}

impl From<ABuffError> for LogToolErr {
    fn from(value: ABuffError) -> Self {
        Self::BuffErr(value.clone())
    }
}

impl From<std::io::Error> for LogToolErr {
    fn from(value: std::io::Error) -> Self {
        Self::IOError {
            message: value.to_string(),
            os_error: value.raw_os_error(),
        }
    }
}

impl From<LogErr> for LogToolErr {
    fn from(value: LogErr) -> Self {
        Self::Log(value.clone())
    }
}

impl<A> From<PoisonError<RwLockReadGuard<'_, A>>> for LogToolErr {
    fn from(value: PoisonError<RwLockReadGuard<'_, A>>) -> Self {
        Self::RwLockErr {
            message: format!("{}", value.to_string()),
        }
    }
}
