use std::collections::HashSet;

use actix_web::{get, post, web, HttpResponse};
use chrono::{Duration, Utc};
use serde::Serialize;
use crate::{queue_api::ApiErr, raft::log_shipping::TransferId, state::AppState};

#[post("/logShipping/{node}")]
pub async fn log_shipping_start( state: web::Data<AppState>, path: web::Path<String> ) -> Result<HttpResponse,ApiErr> {
    let node_id = path.into_inner();
    let transfer_id = state.start_log_shipping(&node_id).await?;

    #[derive(Serialize,Default)]
    struct Resp {
        has_job: bool,                
        job_id: Option<u32>,
        cargo_size: Option<usize>,
    }

    Ok(HttpResponse::Ok().json(match transfer_id {
        None => Resp { has_job:false, job_id:None, cargo_size:None },
        Some((transfer_id,transfer)) => Resp { 
            has_job:true, 
            job_id:Some(transfer_id.0), 
            cargo_size:Some(transfer.blocks_ids.len())
        }
    }))
}

#[get("/logShipping/{node}/{job}/log")]
pub async fn log_shipping_state( state: web::Data<AppState>, path: web::Path<(String,u32)> ) -> Result<HttpResponse,ApiErr> {
    let state = state.raft.lock()?;
    let (node_id,job_id) = path.into_inner();
    let node = state.nodes.iter().find(|n| n.id == node_id );

    match node {
        None => Err(ApiErr::BadRequest(format!("node not found: {}", node_id))),
        Some(node) => {
            let log_ship_map = 
            node.log_shipping.jobs.lock().map_err(
                |e| ApiErr::MutexErr(format!("can't lock node.log_shipping {e}",e=e.to_string())))?;

            match log_ship_map.get(&TransferId(job_id)) {
                None => Err(ApiErr::BadRequest(format!("job not found: {}", job_id))),
                Some(cargo) => {
                    let log = cargo.log.lock()?;

                    let mut log_text = String::new();
                    for s in log.iter() {
                        log_text.push_str(&s);
                        log_text.push_str("\n");
                    }

                    Ok(HttpResponse::Ok().body(log_text))
                }
            }
        }
    }
}

#[post("/logShipping/clean")]
pub async fn log_shipping_clean( state: web::Data<AppState> ) -> Result<HttpResponse,ApiErr> {
    let removed = state.cleanup_log_shipping_jobs()?;
    Ok(HttpResponse::Ok().json(removed))
}


