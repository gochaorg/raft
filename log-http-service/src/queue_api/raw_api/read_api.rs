use actix_web::{web, get, Error, HttpResponse};
use logs::logfile::block::BlockId;
use logs::logqueue::*;
use futures::{future::ok, stream::once};
use crate::queue_api::ApiErr;
use crate::queue;

const CACHE_1DAY_TTL: &str = "max-age=86400";

/// Получение тела записи
#[get("/record/{log:[0-9]+}/{block:[0-9]+}/raw")]
pub async fn read_block( path: web::Path<(String,u32)> ) -> Result<HttpResponse,ApiErr> {
    let (log_id, block_id) = path.into_inner();
    let log_id = u128::from_str_radix(&log_id,10).unwrap();
    
    let log_id = LogQueueFileNumID { id: log_id, previous: None };
    let block_id = BlockId::new(block_id);
    let rec_id = RecID { log_file_id: log_id, block_id: block_id };

    queue(|q|{
        let q = q.lock().unwrap();
        let b_info = q.info(rec_id.clone()).unwrap();
        let tot_size = 
            b_info.data_size.0 as u64 + 
            b_info.head_size.0 as u64 +
            b_info.tail_size.0 as u64;

        let mut bytes = Vec::<u8>::with_capacity(tot_size as usize);
        bytes.resize(tot_size as usize, 0u8);

        let offset = b_info.position;

        let reads_size = q.read_raw_bytes(log_id, offset, &mut bytes).unwrap();
        if reads_size != tot_size as u64 {
            Err(ApiErr::RawReadBlockDataTruncated { expected_size: tot_size, actual_size: reads_size })
        }else{
            let bytes = web::Bytes::from(bytes);
            let body = once(ok::<_,Error>(bytes));

            Ok(HttpResponse::Ok()
                .content_type("application/octet-stream")
                .append_header(("Cache-Control",CACHE_1DAY_TTL))
                .streaming(body))
        }
    })
}



