use std::sync::Arc;

use actix_rt::spawn;
use chrono::Utc;
use log::info;
use log_http_client::{BlockWrite, QueueBlockId, QueueClient};
use logs::logqueue::*;

use crate::{queue_api::ApiErr, state::AppState, QUEUE, queue};
use super::*;

/// Составить план наката логов
async fn try_build_cargo( queue: QUEUE, client: QueueClient ) -> Result<Option<Cargo>,ApiErr> {
    let last_rec = { queue.lock()?.last_record()? };
    match last_rec {
        None => Ok(None),
        Some(last_rec) => {
            let queue_qbid = QueueBlockId::from(last_rec);
            let client_qbid = client.tail_id().await?;
            if client_qbid >= queue_qbid {
                // no need shipping
                // нечего доставлять
                Ok(None)
            } else {
                // need shipping
                let cargo = qblock_ids_downto(queue_qbid, client_qbid, queue.clone())?;
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

                Ok(Some(Arc::new(cargo)))
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

/// Запуск доставки лога
/// 
/// Аргументы
/// -------------
/// 
/// - client: QueueClient - куда доставлять
pub fn log_shipping_start( client: QueueClient, cargo: Transfer, queue: QUEUE ) 
{
    let log_data = cargo.log.clone();
    let hdl = cargo.current_proc.clone();

    let log = move | s:String | {
        match log_data.lock() {
            Err(err) => {
                log::error!("can't lock log_data in log_shipping_start {}", err.to_string());
            },
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

    {
        let res = hdl.lock();
        match res {
            Ok(mut hdl) => {
                (*hdl) = Some(h.abort_handle());
            },
            Err(err) => {
                log::error!("can't lock spawn handle of log_shipping_start {}", err.to_string());
            }
        }
    }
}

async fn log_shipping_impl( client: QueueClient, cargo: Transfer, queue: QUEUE ) -> Result<(),ApiErr> 
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

impl LogShipping {
    /// Старт доставки логово
    fn start( &self, qc: QueueClient, tr: Transfer, queue: QUEUE )  -> Result<TransferId,LogShippingError> {
        let tr_id = self.add_job(tr.clone())?;
        log_shipping_start(qc, tr.clone(), queue);
        Ok(tr_id)
    }

    /// Удалеяет уже завершенные задачи
    fn cleanup( &self ) -> Result<Vec<TransferId>,LogShippingError> {
        let mut jobs = self.jobs.lock()?;

        let mut remove_ids: Vec<TransferId> = Vec::new();
        for (k,tr) in jobs.iter() {
            match tr.is_finished()? {
                true => { remove_ids.push(k.clone()); },
                _ => {}
            }
        }

        for t_id in remove_ids.clone() {
            jobs.remove(&t_id);
        }

        Ok(remove_ids)
    }
}


impl AppState {
    /// Запуск асинхронной доставки логов на указанный узел
    pub async fn start_log_shipping( &self, target_node_id: &str ) -> Result<Option<(TransferId,Transfer)>,ApiErr> {
        let state = self.raft.lock()?;
        let target_node = state.find_node(target_node_id)?;
        let queue: QUEUE = queue(|queue| { queue.clone() });
        let target_client: QueueClient = target_node.client.clone();
        match try_build_cargo( queue.clone(), target_client.clone() ).await? {
            None => Ok(None),
            Some(cargo) => {
                let transfer = Transfer::from(cargo);
                let transfer_id = target_node.log_shipping.start(target_client, transfer.clone(), queue)?;
                Ok(Some((transfer_id,transfer)))
            }
        }
    }

    /// Удаляет уже завершенные задачи
    pub fn cleanup_log_shipping_jobs( &self ) -> Result<Vec<(String, Vec<TransferId>)>,ApiErr> {
        let state = self.raft.lock()?;
        let r: Vec<(String, Vec<TransferId>)> = state.nodes.iter().filter_map(|node| {
            match node.log_shipping.cleanup().ok() {
                Some(k) => Some( (node.id.clone(), k) ),
                _ => None
            }
        }).collect();
        Ok(r)
    }
}

