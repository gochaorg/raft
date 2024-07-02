use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex, PoisonError}};

use actix_rt::spawn;
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Duration, DurationRound, Utc};
use log::info;
use log_http_client::{BlockWrite, QueueBlockId, QueueClient};
use logs::logqueue::*;
use serde::Serialize;

use crate::{queue, queue_api::ApiErr, state::AppState, QUEUE};
use derive_more::Display;

#[derive(Debug,Display,Serialize)]
pub enum LogShippingError {
    #[display(fmt="Can't create cargo plan, block {} not found in queue", _0)]
    CargoPlanFailBlockNotFound(QueueBlockId)
}

impl From<PoisonError<std::sync::MutexGuard<'_, HashMap<u32, Cargo>>>> for ApiErr {
    fn from(value: PoisonError<std::sync::MutexGuard<'_, HashMap<u32, Cargo>>>) -> Self {
        ApiErr::MutexErr(format!("can't lock node log_ship: {}", value.to_string()))
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, Vec<std::string::String>>>> for ApiErr {
    fn from(value: PoisonError<std::sync::MutexGuard<'_, Vec<std::string::String>>>) -> Self {
        ApiErr::MutexErr(format!("can't lock node log_ship, cargo log {}", value.to_string()))
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, std::option::Option<chrono::DateTime<chrono::Utc>>>>> for ApiErr {
    fn from(value: PoisonError<std::sync::MutexGuard<'_, std::option::Option<chrono::DateTime<chrono::Utc>>>>) -> Self {
        ApiErr::MutexErr(format!("can't lock node log_ship, ___ {}", value.to_string()))
    }
}

#[derive(Debug,Clone,Copy)]
pub enum CargoItem {
    BlockTransfer {
        source: QueueBlockId,
        expected: QueueBlockId,
    },
    Switch
}

/// "Груз" который надо доставить на клиента
#[derive(Debug,Clone)]
pub struct Cargo {
    /// Упарадоченная последовательность - в порядке возрастания: от младшего к старшему.
    pub blocks_ids: Arc<Vec<CargoItem>>,

    /// Лог
    pub log: Arc<Mutex<Vec<String>>>,

    /// Дата начала
    pub started: Arc<Mutex<Option<DateTime<Utc>>>>,

    /// Дата завершения
    pub finished: Arc<Mutex<Option<DateTime<Utc>>>>,
}

/// Составить план наката логов
async fn try_build_cargo( queue: QUEUE, client: QueueClient ) -> Result<Option<Cargo>,ApiErr> {
    let last_rec = { queue.lock()?.last_record()? };
    match last_rec {
        None => Ok(None),
        Some(last_rec) => {
            let queue_qbid = QueueBlockId::from(last_rec);
            let client_qbid = client.tail_id().await?;
            if( client_qbid >= queue_qbid ){
                // no need shipping
                // нечего доставлять
                Ok(None)
            } else {
                // need shipping
                let mut cargo = qblock_ids_downto(queue_qbid, client_qbid, queue.clone())?;
                if cargo.is_empty() {
                    return Ok(None);
                }

                if ! cargo.first().map(|bid| bid.clone()==client_qbid ).unwrap_or(false) {
                    // не найден блок с которого произовдить накат логов
                    return Err(LogShippingError::CargoPlanFailBlockNotFound(client_qbid).into());
                }

                let cargo: Vec<CargoItem> = cargo.iter().zip(cargo.iter().skip(1))
                    .map(|(expect,source )| {
                        if source.is_log_first() {
                            CargoItem::Switch
                        } else {
                            CargoItem::BlockTransfer { 
                                source: source.clone(), 
                                expected: expect.clone(),
                            }
                        }
                    }).collect();

                if cargo.is_empty() { return Ok(None); }

                println!("-----------------------------------");
                println!("cargo:");
                println!("  queue  tail id: {queue_qbid}");
                println!("  client tail id: {client_qbid}");
                for (c_idx,item) in cargo.iter().enumerate() {
                    println!("  block#{c_idx} {item:?}");
                }

                Ok(Some(Cargo {
                    blocks_ids: Arc::new(cargo),
                    log: Arc::new(Mutex::new(vec![])),
                    started: Arc::new(Mutex::new(None)),
                    finished: Arc::new(Mutex::new(None)),
                }))
            }
        }
    }
}

/// Поиск идентификаторов блоков, с конца (from_qbid), к началу до указанного (downto_qbid) включительно.
/// 
/// Результат
/// ---------------
/// - Вектор в порядке возрастания: от младшего к старшему.
/// - Возможно не будет найден downto_qbid
fn qblock_ids_downto( from_qbid: QueueBlockId, downto_qbid: QueueBlockId, queue: QUEUE, ) -> Result<Vec<QueueBlockId>,ApiErr> {
    let mut result : Vec<QueueBlockId> = vec![];
    let queue = queue.lock()?;

    let mut cur = from_qbid.clone();
    result.push(cur);
    loop {
        if cur == downto_qbid { break; }

        match queue.previous_record(cur.into())? {
            None => break,
            Some(prev) => {
                cur = prev.into();
                result.push(cur);
            }
        }
    }

    result.reverse();
    Ok(result)
}

fn log_shipping_start0( client: QueueClient, cargo: Cargo, queue: QUEUE ) 
{
    let log_data = cargo.log.clone();
    let log = move | s:String | {
        match log_data.lock() {
            Err(_) => {},
            Ok(mut log) => {
                log.push(s);
            }
        }
    };

    let h = spawn( async move {
        match cargo.started.lock() {
            Ok(mut started) => { (*started) = Some(Utc::now()); }
            _ => {}
        }

        let shipping_finish = cargo.finished.clone();
        let mark_finish = || {
            match shipping_finish.lock() {
                Ok(mut started) => { (*started) = Some(Utc::now()); }
                _ => {}
            }
        };

        match log_shipping_impl(
            client, cargo, queue
        ).await {
            Ok(_) => {
                log(format!("log shipping finished successfuly"));
                mark_finish();
            },
            Err(err) => {
                log(format!("log shipping finished with error: {err}"));
                mark_finish();
            }
        }
    });
}

async fn log_shipping_impl( client: QueueClient, cargo: Cargo, queue: QUEUE ) -> Result<(),ApiErr> 
{
    let log_data = cargo.log.clone();
    let log = | s:String | {
        info!("{s}");
        match log_data.lock() {
            Err(_) => {},
            Ok(mut log) => {
                log.push(s);
            }
        }
    };

    let block_count = cargo.blocks_ids.len();
    log(format!("start log shipping, cargo size: {} blocks", block_count));

    for (idx, qbid) in cargo.blocks_ids.iter().enumerate() {
        let cargo_item = qbid.clone();
        log(format!("shipping block [{idx}/{block_count}] {cargo_item:?}", idx=idx+1 ));

        let queue = queue.lock()?;

        match cargo_item {
            CargoItem::Switch => {
                log(format!("log switch"));
                client.log_switch().await?;
            }
            CargoItem::BlockTransfer { source, expected } => {
                log(format!("block read from queue"));
                let block: BlockWrite = queue.read(source.into())?.into();
                let block = block.expect_tail(expected);

                log(format!("block write to client"));
                client.block_write(block).await?;
            }
        }
    }
    Ok(())
}

#[post("/logShipping/{node}")]
pub async fn log_shipping_start( state: web::Data<AppState>, path: web::Path<String> ) -> Result<HttpResponse,ApiErr> {
    let state = state.raft.lock()?;
    let node_id = path.into_inner();

    #[derive(Serialize,Default)]
    struct Resp {
        job_id: Option<u32>,
        cargo_size: Option<usize>,
    }

    let node = state.nodes.iter().find(|n| n.id == node_id );
    match node {
        None => Err(ApiErr::BadRequest(format!("node not found: {}", node_id))),
        Some(node) => {
            let mut qc: QueueClient = node.client.clone();
            qc.raft_master_id = Some( state.id.clone() );

            let queue: QUEUE = queue(|queue| { queue.clone() });
            match try_build_cargo(queue.clone(), qc.clone()).await? {
                None => Ok(HttpResponse::Ok().json(Resp::default())),
                Some( cargo ) => {
                    let job_id = node.log_shipping_idseq.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let mut log_ship_map = 
                        node.log_shipping.lock().map_err(
                            |e| ApiErr::MutexErr(format!("can't lock node.log_shipping {e}",e=e.to_string())))?;

                    log_ship_map.insert(job_id, cargo.clone());                    
                    log_shipping_start0(qc, cargo.clone(), queue);

                    Ok(HttpResponse::Ok().json(
                        Resp {
                            job_id: Some(job_id),
                            cargo_size: Some(cargo.blocks_ids.len())
                        }
                    ))
                }
            }
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
            node.log_shipping.lock().map_err(
                |e| ApiErr::MutexErr(format!("can't lock node.log_shipping {e}",e=e.to_string())))?;

            match log_ship_map.get(&job_id) {
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

    let mut remove_set : HashSet<u32> = HashSet::new();
    for node in state.nodes.iter() {
        let mut log_ship_map = node.log_shipping.lock()?;
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


