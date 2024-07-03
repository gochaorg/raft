use std::collections::HashSet;

use actix_web::{get, post, web, HttpResponse};
use chrono::{Duration, Utc};
use log_http_client::QueueClient;
use serde::Serialize;
use crate::{queue, queue_api::ApiErr, raft::log_shipping::{try_build_cargo, Transfer, TransferId}, state::AppState, QUEUE};

#[post("/logShipping/{node}")]
pub async fn log_shipping_start_api( state: web::Data<AppState>, path: web::Path<String> ) -> Result<HttpResponse,ApiErr> {
    let state = state.raft.lock()?;
    let node_id = path.into_inner();

    #[derive(Serialize,Default)]
    struct Resp {
        job_id: Option<u32>,
        cargo_size: Option<usize>,
    }

    let node = state.find_node(&node_id)?;
    let qc: QueueClient = node.client.clone();
    let queue: QUEUE = queue(|queue| { queue.clone() });
    
    match try_build_cargo(queue.clone(), qc.clone()).await? {
        None => Ok(HttpResponse::Ok().json(Resp::default())),
        Some( cargo ) => {
            let transfer = Transfer::from(cargo);
            let transfer_id = node.log_shipping.start(qc, transfer.clone(), queue)?;

            Ok(HttpResponse::Ok().json(
                Resp {
                    job_id: Some(transfer_id.0),
                    cargo_size: Some(transfer.blocks_ids.len())
                }
            ))
        }
    }
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
    let state = state.raft.lock()?;

    let mut remove_set : HashSet<TransferId> = HashSet::new();
    for node in state.nodes.iter() {
        let log_ship_map = node.log_shipping.jobs.lock()?;
        for (k,cargo) in log_ship_map.iter() {
            let fin = cargo.finished.lock()?;
            if fin.is_some() {
                let fin = fin.unwrap();
                let dur = Utc::now().signed_duration_since(fin);
                if dur > Duration::seconds(30) {
                    remove_set.insert(k.clone());
                }
            }            
        }
    }

    Ok(HttpResponse::Ok().body("body"))
}


