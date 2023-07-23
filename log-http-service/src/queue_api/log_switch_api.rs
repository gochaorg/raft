use actix_web::{web, Responder, post};
use actix_web::Result;
use logs::logqueue::*;
use serde::Serialize;

use crate::queue;

/// Переключение лог файла
#[post("/tail/switch")]
pub async fn log_switch() -> Result<impl Responder> {
    queue(|q|{
        let mut q = q.lock().unwrap();
        let res = q.switch().unwrap();

        #[derive(Serialize)]
        struct Res {
            log_file: String,
            log_id: String,
        }

        Ok(web::Json(Res { log_file: res.0.to_str().unwrap().to_string(), log_id: res.1.id().to_string() }))
    })
}

