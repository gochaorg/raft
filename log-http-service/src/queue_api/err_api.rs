use actix_web::{error, HttpResponse};
use logs::logqueue::LoqErr;
use std::fmt::Display;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::PoisonError;

use crate::state::RaftState;

#[derive(Debug)]
pub enum ApiErr 
{
    BlockErr(logs::logfile::block::BlockErr),
    RecIdNotMatch {
        expect_log_id: String,
        actual_log_id: String,
        expect_block_id: String,
        actual_block_id: String,
    },
    RawReadBlockDataTruncated {
        expected_size: u64,
        actual_size: u64,
    },
    CantLockQueue {
        error: String,
    },
    QueueIsEmpy,
    LoqErr(String),
    CantLockRaftState { error: String }
}

impl Display for ApiErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}", self)
    }
}

impl error::ResponseError for ApiErr {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {        
        HttpResponse::build(self.status_code())
        .body(match self {
            Self::BlockErr(err) => 
                format!("BlockErr {:?}",err),
            Self::RecIdNotMatch { expect_log_id, actual_log_id, expect_block_id, actual_block_id } => 
                format!("RecIdNotMatch expect_log_id={expect_log_id} actual_log_id={actual_log_id} expect_block_id={expect_block_id} actual_block_id={actual_block_id}"),
            Self::RawReadBlockDataTruncated { expected_size, actual_size } =>
                format!("RawReadBlockDataTruncated: expected_size={expected_size}, actual_size={actual_size}"),
            Self::CantLockQueue { error } =>
                format!("CantLockQueue: {error}"),
            Self::QueueIsEmpy =>
                format!("QueueIsEmpy"),
            Self::LoqErr(err) =>
                format!("LoqErr: {err}"),
            _ => format!("{:?}", self)
        })
    }

    fn status_code(&self) -> actix_swagger::StatusCode {
        match self {
            Self::BlockErr(_) => actix_swagger::StatusCode::INTERNAL_SERVER_ERROR,
            _ => actix_swagger::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl std::convert::From<logs::logfile::block::BlockErr> for ApiErr {
    fn from(err: logs::logfile::block::BlockErr) -> Self {
        ApiErr::BlockErr(err)
    }
}

impl<F,L> std::convert::From<PoisonError<std::sync::MutexGuard<'_, (dyn logs::logqueue::LogFileQueue<logs::logqueue::LogQueueFileNumID, F, L> + 'static)>>>
for ApiErr {
    fn from(value: PoisonError<std::sync::MutexGuard<'_, (dyn logs::logqueue::LogFileQueue<logs::logqueue::LogQueueFileNumID, F, L> + 'static)>>) -> Self {
        Self::CantLockQueue { error: value.to_string() }
    }
}

impl std::convert::From<LoqErr<PathBuf, logs::logqueue::LogQueueFileNumID>> for ApiErr {
    fn from(value: LoqErr<PathBuf, logs::logqueue::LogQueueFileNumID>) -> Self {
        Self::LoqErr(format!("{value:?}"))
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, RaftState>>> for ApiErr {
    fn from(value: PoisonError<std::sync::MutexGuard<'_, RaftState>>) -> Self {
        Self::CantLockRaftState { error: value.to_string() }
    }
}