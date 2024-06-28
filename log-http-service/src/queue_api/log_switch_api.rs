use actix_web::{post, web, HttpRequest, HttpResponse};
use actix_web::Result;
use logs::logqueue::*;
use serde::Serialize;

use crate::queue;
use crate::queue_api::{validate_raft_master, ApiErr};
use crate::state::AppState;

/// Переключение лог файла
#[post("/tail/switch")]
pub async fn log_switch( state: web::Data<AppState>, req: HttpRequest ) -> Result<HttpResponse,ApiErr> {
    validate_raft_master(&state, &req, || {
        queue(|q|{
            let mut q = q.lock()?;
            let res = q.switch()?;

            #[derive(Serialize)]
            struct Res {
                log_file: String,
                log_id: String,
            }

            Ok( HttpResponse::Ok().json(Res { log_file: res.0.to_str().unwrap().to_string(), log_id: res.1.id().to_string() }))
        })
    })
}

