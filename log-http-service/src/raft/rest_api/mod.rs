use std::time::Duration;

use actix_web::{web, Responder, get, post};
use actix_web::Result;
use serde::Serialize;

use crate::queue_api::ApiErr;
use crate::state::AppState;

/// Настройка маршрутов
pub fn route( cfg: &mut web::ServiceConfig ) {
    cfg
        .service(status)
        .service(bg_job_stop)
        .service(bg_job_start)
        ;
}

#[get("/status")]
async fn status( state: web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Status {
        status: String,
        bg_job: BgJobStatus,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct BgJobStatus {
        running: bool,
        timeout: Duration
    }

    let state = state.raft.lock().unwrap();

    let (bg_running,bg_timeout) = state.bg_job.as_ref().map(|b| 
        ( b.is_running(), b.get_timeout().clone() )
    ).unwrap_or((false,Duration::ZERO));    

    Ok(web::Json(
        Status {
            status: "ok".to_string(),
            bg_job: BgJobStatus {
                running: bg_running,
                timeout: bg_timeout
            }
        }
    ))
}

#[post("/bg/stop")]
async fn bg_job_stop( state: web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    let mut state = state.raft.lock().unwrap();
    state.bg_job.as_mut().map(|j| j.stop());
    Ok(web::Json("try stop"))
}

#[post("/bg/start")]
async fn bg_job_start( state: web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    let mut state = state.raft.lock().unwrap();
    state.bg_job.as_mut().map(|j| j.start());
    Ok(web::Json("try stop"))
}