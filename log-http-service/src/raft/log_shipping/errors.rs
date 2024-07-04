use derive_more::Display;
use serde::Serialize;
use std::{collections::HashMap, sync::PoisonError};
use log_http_client::QueueBlockId;

use crate::{queue_api::ApiErr, raft::RaftError};

use super::*;

/// Ошибки доставки логов
#[derive(Debug,Display,Serialize)]
pub enum LogShippingError {
    #[display(fmt="Can't create cargo plan, block {} not found in queue", _0)]
    CargoPlanFailBlockNotFound(QueueBlockId),

    #[display(fmt="Can't lock transfer map {}", _0)]
    TransferLockError(String)
}

impl From<PoisonError<std::sync::MutexGuard<'_, HashMap<u32, Transfer>>>> for ApiErr {
    fn from(value: PoisonError<std::sync::MutexGuard<'_, HashMap<u32, Transfer>>>) -> Self {
        ApiErr::MutexErr(format!("can't lock node log_ship: {}", value.to_string()))
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, Vec<std::string::String>>>> for ApiErr {
    fn from(value: PoisonError<std::sync::MutexGuard<'_, Vec<std::string::String>>>) -> Self {
        ApiErr::MutexErr(format!("can't lock node log_ship, cargo log {}", value.to_string()))
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, std::option::Option<chrono::DateTime<chrono::Utc>>>>> for ApiErr {
    fn from(value: PoisonError<std::sync::MutexGuard<'_, std::option::Option<chrono::DateTime<chrono::Utc>>>>) -> Self {
        ApiErr::MutexErr(format!("can't lock node log_ship, ___ {}", value.to_string()))
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, HashMap<TransferId, Transfer>>>> for LogShippingError {
    fn from(value: PoisonError<std::sync::MutexGuard<'_, HashMap<TransferId, Transfer>>>) -> Self {
        Self::TransferLockError(format!("lock error: {}", value.to_string()))
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, HashMap<TransferId, Transfer>>>> for ApiErr {
    fn from(value: PoisonError<std::sync::MutexGuard<'_, HashMap<TransferId, Transfer>>>) -> Self {
        Self::Raft(
            RaftError::LogShipping(
                LogShippingError::TransferLockError(
                    format!("{}", value.to_string())
                )
            )
        )
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, std::option::Option<tokio::task::AbortHandle>>>> for LogShippingError {
    fn from(value: PoisonError<std::sync::MutexGuard<'_, std::option::Option<tokio::task::AbortHandle>>>) -> Self {
        Self::TransferLockError(format!("lock error: {}", value.to_string()))
    }
}