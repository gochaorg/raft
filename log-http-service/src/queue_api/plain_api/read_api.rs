use std::collections::HashMap;

use actix_web::{get, web, Error, HttpResponse, Responder};
use logs::logfile::block::BlockId;
use logs::logqueue::*;
use serde::{Deserialize, Serialize};
use futures::{future::ok, stream::once};

use crate::{queue, queue_api::ApiErr};

#[derive(Deserialize,Clone)]
pub struct RawBodyOpts {
    /// Заголовки содержат опции блока
    opt2head: Option<bool>,

    /// Префикс в опциях блока
    opt_prefix: Option<String>
}

/// Получение тела записи
/// 
/// ```
/// query_string ::= [opt2head ['&' opt_prefix]]
/// opt2head ::= 'opt2head=' ( 'true' | 'false' )
/// opt_prefix ::= 'opt_prefix=' prefix
/// ```
#[get("/record/{log:[0-9]+}/{block:[0-9]+}/bytes")]
pub async fn read_bytes(path: web::Path<(String,u32)>, query:web::Query<RawBodyOpts>) -> Result<HttpResponse,ApiErr> {
    let raw_opt = query.into_inner();

    let (log_id, block_id) = path.into_inner();
    let log_id = u128::from_str_radix(&log_id,10).unwrap();
    
    let log_id = LogQueueFileNumID { id: log_id, previous: None };
    let block_id = BlockId::new(block_id);
    let rec_id = RecID { log_file_id: log_id, block_id: block_id };

    let prefix = raw_opt.clone().opt_prefix.unwrap_or("".to_string());

    queue(move |q| {
        let q = q.lock()?;
        let rec = q.read(rec_id.clone())?;

        let bytes = web::Bytes::from(rec.data);
        let body = once(ok::<_,Error>(bytes));

        let mut response = HttpResponse::Ok();

        let ct = || {
            rec.options.get("mime").map(|mime| {
                match mime.value() {
                    "text/plain" => "text/plain",
                    _ => "application/octet-stream"
                }
            }).unwrap_or("application/octet-stream")
        };

        let response = response.content_type(ct());

        let response = if raw_opt.clone().opt2head.unwrap_or(false) {
            let mut itr = rec.options.into_iter();
            loop {
                match itr.next() {
                    Some( (k,v) ) => {
                        let k = format!( "{pref}{key}",
                            key = k.value(),
                            pref = prefix
                        );
                        response.append_header((k, v.value()));
                    },
                    None => {
                        break response
                    }
                }                
            }
        } else {
            response
        };

        Ok(response.streaming(body))
    })
}

#[derive(Serialize)]
struct RecordHeader {
    pub log_file: String,
    pub log_id: String,
    pub block_id: u32,
    // фактически u64
    pub position: u64, 
    pub position_str: String, 
    pub head_size: u32,
    pub data_size: u32,
    pub tail_size: u16,
    pub block_options: HashMap<String,String>,
}

#[get("/record/{log:[0-9]+}/{block:[0-9]+}/headers")]
pub async fn header_of( path: web::Path<(String,u32)> ) -> Result<impl Responder,ApiErr> {
    let (log_id, block_id_src) = path.into_inner();
    let log_id = u128::from_str_radix(&log_id,10).unwrap();

    let log_id = LogQueueFileNumID { id: log_id, previous: None };
    let block_id = BlockId::new(block_id_src);
    let rec_id = RecID { log_file_id: log_id, block_id: block_id };

    queue(|q| {
        let q = q.lock()?;

        let info = q.info(rec_id.clone())?;
        let mut b_opts: HashMap<String,String> = HashMap::new();
        for (k,v) in info.block_options.into_iter() {
            b_opts.insert(k.to_string(), v.to_string());
        }

        Ok(web::Json(
            RecordHeader {
                log_file: info.log_file.to_str().map(|s| s.to_string()).unwrap_or("?".to_string()),
                log_id: info.log_id.id().to_string(),
                block_id: info.block_id.0,
                position: info.position.0,
                position_str: info.position.0.to_string(),
                head_size: info.head_size.0,
                data_size: info.data_size.0,
                tail_size: info.tail_size.0,
                block_options: b_opts,
            }
        ))
    })
}
