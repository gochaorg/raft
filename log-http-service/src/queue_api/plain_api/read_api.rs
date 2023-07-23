use actix_web::{web, get, Error, HttpResponse};
use logs::logfile::block::BlockId;
use logs::logqueue::*;
use serde::Deserialize;
use futures::{future::ok, stream::once};

use crate::queue;

#[derive(Deserialize,Clone)]
struct RawBodyOpts {
    /// Заголовки содержат опции блока
    opt2head: Option<bool>,

    /// Префикс в опциях блока
    opt_prefix: Option<String>
}

/// Получение тела записи
#[get("/record/{log:[0-9]+}/{block:[0-9]+}/plain")]
pub async fn read_plain(path: web::Path<(String,u32)>, query:web::Query<RawBodyOpts>) -> HttpResponse {
    let raw_opt = query.into_inner();

    let (log_id, block_id) = path.into_inner();
    let log_id = u128::from_str_radix(&log_id,10).unwrap();
    
    let log_id = LogQueueFileNumID { id: log_id, previous: None };
    let block_id = BlockId::new(block_id);
    let rec_id = RecID { log_file_id: log_id, block_id: block_id };

    let prefix = raw_opt.clone().opt_prefix.unwrap_or("".to_string());

    queue(move |q| {
        let q = q.lock().unwrap();
        let rec = q.read(rec_id.clone()).unwrap();

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

        response.streaming(body)
    })
}

