use serde::Serialize;
use logs::logqueue::*;

#[derive(Serialize)]
pub struct ID {
    pub log_id: String,
    pub block_id: String
}

impl From<RecID<LogQueueFileNumID>> for ID {
    fn from(value: RecID<LogQueueFileNumID>) -> Self {
        Self { 
            log_id: value.log_file_id.id.to_string(), 
            block_id: value.block_id.value().to_string() 
        }
    }
}

