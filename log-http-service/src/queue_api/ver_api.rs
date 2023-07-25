use actix_web::{web, Responder, get};
use actix_web::Result;
use serde::Serialize;
use std;

use crate::queue_api::ApiErr;

#[cfg(debug_assertions)]
fn is_debug() -> bool { true }

#[cfg(not(debug_assertions))]
fn is_debug() -> bool { false }

#[allow(dead_code)]
pub const CRATE_NAME: &str = std::env!("CARGO_CRATE_NAME");

#[allow(dead_code)]
pub const CRATE_VER: &str = std::env!("CARGO_PKG_VERSION");


#[get("/version")]
pub async fn get_version() -> Result<impl Responder,ApiErr> {
    #[derive(Serialize)]
    struct VerInfo {
        debug: bool,
        crate_name: String,
        crate_ver: String,
    }

    Ok(web::Json(
        VerInfo {
            debug: is_debug(),
            crate_name: CRATE_NAME.to_string(),
            crate_ver: CRATE_VER.to_string(),
        }
    ))
}
