use actix_web::{web, Responder, get};
use actix_web::Result;
use serde::Serialize;

use crate::queue_api::ApiErr;
use crate::state::AppState;

/// Настройка маршрутов
pub fn route( cfg: &mut web::ServiceConfig ) {
    cfg.service(status);
}

#[get("/status")]
async fn status( state: web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    #[derive(Serialize)]
    struct Status {
        status: String
    }
    Ok(web::Json(
        Status {
            status: "ok".to_string()
        }
    ))
}