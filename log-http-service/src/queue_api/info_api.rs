use actix_web::{web, Responder, get};
use actix_web::Result;
use serde::Serialize;

use logs::logqueue::*;

use crate::queue;
use crate::queue_api::ID;

/// Получение списка файлов
#[get("/log/files")]
pub async fn get_queue_files() -> Result<impl Responder> {
    #[derive(Serialize)]
    struct LogFileInfo {
        log_file: String,
        items_count: u32
    }

    #[derive(Serialize)]
    struct Res {
        files: Vec<LogFileInfo>
    }

    let res = queue(|q| {
        let q = q.lock().unwrap();
        Res {
            files: q.files().iter().map(|(f,l)|
                LogFileInfo { log_file: f.to_str().unwrap().to_string(), items_count:l.count().unwrap() }
            ).collect()
        }
    });

    Ok(web::Json(res))
}

/// Получение текущее id последней записи
#[get("/tail/id")]
async fn get_cur_id() -> Result<impl Responder> {
    Ok( 
        web::Json( queue(|q| { 
            let q = q.lock().unwrap(); 
            let rid = q.last_record().unwrap();
            rid.map(|rid| 
                ID::from(rid)
            )
        }))
    )
}
