use actix_web::{error, HttpResponse};
use logs::logqueue::LoqErr;
use serde::Serialize;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::PoisonError;
use crate::raft::rest_api::sync::LogShippingError;
use crate::raft as raft_state;
use crate::raft::RaftError;
use derive_more::Display;
use actix_swagger::StatusCode;

#[derive(Debug,Display,Serialize)]
pub enum ApiErr 
{    
    #[display(fmt="BlockErr {:?}", _0)]
    BlockErr(logs::logfile::block::BlockErr),

    #[display(fmt=
        "RecIdNotMatch log_id: expect={} actual={}, block_id: expect={} actual={}", 
        expect_log_id, actual_log_id, expect_block_id, actual_block_id)]
    RecIdNotMatch {
        expect_log_id: String,
        actual_log_id: String,
        expect_block_id: String,
        actual_block_id: String,
    },

    #[display(fmt="RawReadBlockDataTruncated size: expect={} actual={}", expected_size, actual_size)]
    RawReadBlockDataTruncated {
        expected_size: u64,
        actual_size: u64,
    },

    #[display(fmt="CantLockQueue {}", error)]
    CantLockQueue {
        error: String,
    },

    #[display(fmt="QueueIsEmpy")]
    QueueIsEmpy,

    #[display(fmt="LoqErr {}", _0)]
    LoqErr(String),

    #[display(fmt="MutexErr {}", _0)]
    MutexErr(String),

    #[display(fmt="BadRequest {}", _0)]
    BadRequest(String),

    #[display(fmt="Raft {}", _0)]
    Raft(RaftError),

    #[display(fmt="ClientError {}", _0)]
    ClientError(log_http_client::Error),
}

impl error::ResponseError for ApiErr {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {     
        HttpResponse::build(self.status_code())
        .body(match self {
            Self::BlockErr(err) => format!("BlockErr {:?}",err),
            Self::RecIdNotMatch { expect_log_id, actual_log_id, expect_block_id, actual_block_id } =>  
                format!("RecIdNotMatch expect_log_id={expect_log_id} actual_log_id={actual_log_id} expect_block_id={expect_block_id} actual_block_id={actual_block_id}"),
            Self::RawReadBlockDataTruncated { expected_size, actual_size } => format!("RawReadBlockDataTruncated: expected_size={expected_size}, actual_size={actual_size}"),
            Self::CantLockQueue { error } => format!("CantLockQueue: {error}"),
            Self::QueueIsEmpy => format!("QueueIsEmpy"),
            Self::LoqErr(err) => format!("LoqErr: {err}"),
            Self::MutexErr(err) => format!("MutexErr: {err}"),
            Self::BadRequest(err) => format!("BadRequest {err}"),
            Self::Raft(err) => format!("Raft {err}"),
            Self::ClientError(err) => format!("Client {err}"),
        })
    }

    fn status_code(&self) -> actix_swagger::StatusCode {
        match self {
            Self::BlockErr(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Raft(RaftError::LogShipping(LogShippingError::CargoPlanFailBlockNotFound(_))) => StatusCode::BAD_REQUEST,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::RecIdNotMatch { expect_log_id:_, actual_log_id:_, expect_block_id:_, actual_block_id:_ } => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR
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
        //serde_json::to_string(&value);
        Self::LoqErr(format!("{value:?}"))
    }
}

impl std::convert::From<PoisonError<std::sync::MutexGuard<'_, crate::state::Debug>>> for ApiErr {
    fn from(value: PoisonError<std::sync::MutexGuard<'_, crate::state::Debug>>) -> Self {
        ApiErr::MutexErr(format!("can't lock debug in AppState: {}",value.to_string()))
    }
}

impl std::convert::From<PoisonError<std::sync::MutexGuard<'_, raft_state::RaftState>>> for ApiErr {
    fn from(value: PoisonError<std::sync::MutexGuard<'_, raft_state::RaftState>>) -> Self {
        ApiErr::MutexErr(format!("can't lock raft in AppState: {}",value.to_string()))
    }
}

impl From<RaftError> for ApiErr {
    fn from(value: RaftError) -> Self {
        ApiErr::Raft(value)
    }
}

impl From<log_http_client::Error> for ApiErr {
    fn from(value: log_http_client::Error) -> Self {
        Self::ClientError(value)
    }
}

impl From<LogShippingError> for ApiErr {
    fn from(value: LogShippingError) -> Self {
        Self::Raft(RaftError::LogShipping(value))
    }
}
