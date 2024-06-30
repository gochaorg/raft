use core::str;
use std::fmt::Display;
use logs::{logfile::block::BlockId, logqueue::{LogQueueFileNumID, RecID}};
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use crate::errors::*;
use std::cmp::*;

/// Идентификатор записи в очереди
#[derive(Debug,Clone,Deserialize,Serialize,Eq,Copy)]
pub struct QueueBlockId {
    /// Идентификатор лог файла
    pub log_id: u128,

    /// Идентификатор записи в логе
    pub block_id: u32,
}

impl From<&QueueBlockId> for QueueBlockId {
    fn from(value: &QueueBlockId) -> Self {
        value.clone()
    }
}

impl Display for QueueBlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"QueueBlockId {{ log_id={log_id}, block_id={block_id} }}", 
            log_id=self.log_id, 
            block_id=self.block_id)
    }
}

impl QueueBlockId {
    /// Посылает подготовленный запрос и ожидает json ответ указанного типа QueueBlockId
    pub async fn try_send(request: RequestBuilder) -> Result<QueueBlockId, Error> {
        #[derive(Debug,Clone,Deserialize,Serialize)]
        struct QueueRecIdRaw {
            /// Идентификатор лог файла
            pub log_id: String,
        
            /// Идентификатор записи в логе
            pub block_id: String,
        }
        
        Ok(request.send().await?.json::<QueueRecIdRaw>().await?)
        .and_then(|rraw|{
            rraw.block_id.parse::<u32>()
            .map_err(|e| Error::DecodeBody(format!("can't decode block_id {b} as u32 {e:?}", b=rraw.block_id)))
            .and_then(|b_id| {
                rraw.log_id.parse::<u128>()
                    .map_err(|e| Error::DecodeBody(format!("can't decode log_id {l} as u128 {e:?}", l=rraw.log_id)))
                    .map(|l_id| 
                        QueueBlockId { log_id: l_id, block_id: b_id }
                    )
            })
        })
    }
}

impl QueueBlockId { 
    /// Возвращает
    /// --------------------
    /// - `true` - первый блок в очереди
    pub fn is_queue_first( &self ) -> bool {
        self.log_id == 0 && self.block_id == 0
    }

    pub fn is_same_log_id( &self, other: &QueueBlockId ) -> bool {
        self.log_id == other.log_id
    }

    /// Возвращает
    /// --------------------
    /// - `true` - первый блок в логе
    pub fn is_log_first( &self ) -> bool {
        self.block_id == 0
    }
}

impl PartialEq for QueueBlockId {
    fn eq(&self, other: &Self) -> bool {
        self.log_id == other.log_id && self.block_id == other.block_id
    }
}

impl PartialOrd for QueueBlockId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {        
        match self.log_id.partial_cmp(&other.log_id) {
            Some(Ordering::Equal) => {
                self.block_id.partial_cmp(&other.block_id)
            }
            ord => return ord,
        }
    }
}

impl From<RecID<LogQueueFileNumID>> for QueueBlockId {
    fn from(value: RecID<LogQueueFileNumID>) -> Self {
        Self {
            block_id: value.block_id.0,
            log_id: value.log_file_id.id
        }
    }
}

impl From<QueueBlockId> for RecID<LogQueueFileNumID> {
    fn from(value: QueueBlockId) -> Self {
        Self {
            block_id: BlockId::new(value.block_id),
            log_file_id: LogQueueFileNumID {
                id: value.log_id,
                previous: None
            }
        }
    }
}


