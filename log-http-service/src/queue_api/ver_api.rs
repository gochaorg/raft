use actix_web::{get, post, web, Responder};
use actix_web::Result;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use std;
use std::time::Duration;

use crate::queue_api::ApiErr;
use crate::state::AppState;

#[cfg(debug_assertions)]
fn is_debug() -> bool { true }

#[cfg(not(debug_assertions))]
fn is_debug() -> bool { false }

#[allow(dead_code)]
pub const CRATE_NAME: &str = std::env!("CARGO_CRATE_NAME");

#[allow(dead_code)]
pub const CRATE_VER: &str = std::env!("CARGO_PKG_VERSION");


#[get("/version")]
pub async fn get_version( app:web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    #[derive(Serialize)]
    struct VerInfo {
        debug: bool,
        crate_name: String,
        crate_ver: String,
    }

    let sleep_dur = { 
        let x = app.debug.lock()?;
        x.version_delay.clone()
    };

    if sleep_dur.is_some() {
        sleep(sleep_dur.unwrap()).await;
    }

    Ok(web::Json(
        VerInfo {
            debug: is_debug(),
            crate_name: CRATE_NAME.to_string(),
            crate_ver: CRATE_VER.to_string(),
        }
    ))
}

#[derive(Debug,Deserialize,Serialize)]
pub enum VersionDelay {
    Non,
    MilliSeconds(u64),
    Seconds(u64)
}

#[post("/version/delay")]
pub async fn post_version_delay( app:web::Data<AppState>, req: web::Json<VersionDelay> ) -> Result<impl Responder,ApiErr> {
    let req = req.into_inner();
    {
        let mut dbg = app.debug.lock().unwrap();        
        match req {
            VersionDelay::Non => {
                dbg.version_delay = None;
            },
            VersionDelay::MilliSeconds(ms) => {
                dbg.version_delay = Some( Duration::from_millis(ms) )
            },
            VersionDelay::Seconds(sec) => {
                (*dbg).version_delay = Some( Duration::from_secs(sec) )
            },
        }
    }
    
    Ok(web::Json(""))
}

#[get("/version/delay")]
pub async fn get_version_delay( app:web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    let d = {
        let dbg = app.debug.lock()?;
        match dbg.version_delay {
            None => VersionDelay::Non,
            Some(d) => {
                let ms = d.as_millis();
                let rest = ms % 1000;
                if rest == 0 {
                    VersionDelay::Seconds((ms / 1000) as u64)
                } else {
                    VersionDelay::MilliSeconds(ms as u64)
                }
            }
        }      
    };
    
    Ok(web::Json(d))
}
