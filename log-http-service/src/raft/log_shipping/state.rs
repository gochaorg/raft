use std::{collections::HashMap, sync::{atomic::AtomicU32, Arc, Mutex}};

use chrono::{DateTime, Utc};
use log_http_client::{QueueBlockId, QueueClient};
use tokio::task::AbortHandle;

use crate::QUEUE;

use super::{log_shipping_start, LogShippingError};

/// Состояние доставки логов
#[derive(Debug,Clone)]
pub struct LogShipping {
    pub jobs: Arc<Mutex<HashMap<TransferId,Transfer>>>,
    job_idseq: Arc<AtomicU32>,
}

impl LogShipping {
    fn add_job( &self, tr: Transfer ) -> Result<TransferId,LogShippingError> {
        let t_id = TransferId(self.job_idseq.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
        let mut jobs = self.jobs.lock()?;
        jobs.insert(t_id, tr);
        Ok(t_id)
    }

    pub fn start( &self, qc: QueueClient, tr: Transfer, queue: QUEUE )  -> Result<TransferId,LogShippingError> {
        let tr_id = self.add_job(tr.clone())?;
        log_shipping_start(qc, tr.clone(), queue);
        Ok(tr_id)
    }
}

impl Default for LogShipping {
    fn default() -> Self {
        Self { jobs: Default::default(), job_idseq: Default::default() }
    }
}

#[derive(Debug,PartialEq,Eq,Hash,Clone,Copy)]
pub struct TransferId(pub u32);

/// Доставляемая порция груза
#[derive(Debug,Clone,Copy)]
pub enum CargoItem {
    /// Доставка блока
    BlockTransfer {
        /// Исходный блок из очереди
        source: QueueBlockId,

        /// Ожидаемое значение tail_id на целевой системе
        expected: QueueBlockId,
    },

    /// Переключение логов
    Switch
}

/// Доставляемый груз
pub type Cargo = Arc<Vec<CargoItem>>;

/// "Груз" который надо доставить на клиента
#[derive(Debug,Clone)]
pub struct Transfer {
    /// Упарадоченная последовательность - в порядке возрастания: от младшего к старшему.
    pub blocks_ids: Cargo,

    /// Лог
    pub log: Arc<Mutex<Vec<String>>>,

    /// Дата начала
    pub started: Arc<Mutex<Option<DateTime<Utc>>>>,

    /// Дата завершения
    pub finished: Arc<Mutex<Option<DateTime<Utc>>>>,

    /// Текущий процесс доставки
    pub current_proc: Arc<Mutex<Option<AbortHandle>>>,
}

impl From<Cargo> for Transfer {
    fn from(value: Cargo) -> Self {
        Self { 
            blocks_ids: value, 
            log: Arc::new(Mutex::new(Vec::new())), 
            started: Arc::new(Mutex::new(None)), 
            finished: Arc::new(Mutex::new(None)), 
            current_proc: Arc::new(Mutex::new(None)) 
        }
    }
}