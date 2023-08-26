use actix_web::{web, Responder, post, get};
use serde::Serialize;
use crate::{state::AppState, queue_api::ApiErr};

pub fn raft_api_route( cfg: &mut web::ServiceConfig ) {
    cfg .service(node_register)
        .service(node_registry)
        .service(get_state);
}

/// Регистрация узла кластера
#[post("/register")]
pub async fn node_register( state:web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    Ok(web::Json(()))
}

/// Получение информации о регмстрации
#[get("/registry")]
pub async fn node_registry( state:web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    Ok(web::Json(()))
}

/// Просмотр состояния raft
#[get("/state")]
pub async fn get_state( state:web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    let state = state.raft.lock()?;
    #[derive(Serialize)]
    struct StateInfo {
        id: String,
        base_url: String,
    }
    Ok(web::Json(StateInfo {
        id: state.id.clone(),
        base_url: state.base_url.clone(),
    }))
}
