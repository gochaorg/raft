use actix_web::{web, get, Error, HttpResponse};
use logs::logfile::block::BlockId;
use logs::logqueue::*;
use serde::Deserialize;
use futures::{future::ok, stream::once};

/// Получение тела записи
#[get("/record/{log:[0-9]+}/{block:[0-9]+}/raw")]
pub async fn read_block( path: web::Path<(String,u32)> ) -> HttpResponse {
    let (log_id, block_id) = path.into_inner();
    let log_id = u128::from_str_radix(&log_id,10).unwrap();
    
    let log_id = LogQueueFileNumID { id: log_id, previous: None };
    let block_id = BlockId::new(block_id);
    let rec_id = RecID { log_file_id: log_id, block_id: block_id };

    todo!()
}