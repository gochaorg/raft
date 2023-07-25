use actix_web::{web, Responder, get};
use actix_web::Result;
use serde::Serialize;

use logs::logqueue::*;

use crate::queue;
use crate::queue_api::{ID, ApiErr};

/// Получение списка файлов
#[get("/log/files")]
pub async fn get_queue_files() -> Result<impl Responder,ApiErr> {
    #[derive(Serialize)]
    struct LogFileInfo {
        log_id: String,

        log_file: String,

        #[serde(skip_serializing_if="Option::is_none")]
        items_count: Option<u32>,

        #[serde(skip_serializing_if="Option::is_none")]
        bytes_count: Option<u64>,
    }

    #[derive(Serialize)]
    struct Res {
        files: Vec<LogFileInfo>
    }

    queue(|q| {
        let q = q.lock()?;
        Ok(web::Json(Res {
            files: q.files().iter().map(|(log_id,f,l)|
                LogFileInfo { 
                    log_id: log_id.id().to_string(),
                    log_file: f.to_str().unwrap().to_string(), 
                    items_count:
                        match l.count() {
                            Ok(v) => Some(v),
                            Err(_) => None
                        },
                    bytes_count:
                        match l.bytes_count() {
                            Ok(v) => Some(v),
                            Err(_) => None
                        }
                }
            ).collect()
        }))
    })
}

/// Получение текущее id последней записи
#[get("/tail/id")]
async fn get_cur_id() -> Result<impl Responder,ApiErr> {
    queue(|q| {
        let q = q.lock()?; 
        match q.last_record()? {
            Some(rid) => Ok(web::Json( ID::from(rid) )),
            None => Err(ApiErr::QueueIsEmpy)
        }
    })
}
