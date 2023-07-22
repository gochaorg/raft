use std::collections::HashMap;

use actix_web::{web, Responder, get, post, Error, HttpResponse};
use actix_web::Result;
use chrono::{DateTime, Utc};
use date_format::{DateFormatParser, Format};
use logs::logfile::block::{BlockOptions, BlockId};
use logs::logqueue::*;
use parse::Parser;
use serde::{Serialize, Deserialize};
use encoding::all::UTF_8;
use encoding::{Encoding, EncoderTrap};
use futures::{future::ok, stream::once};

use crate::queue;

/// настройка ручек
pub fn queue_api_route( cfg: &mut web::ServiceConfig ) {
   cfg
    .service(get_queue_files)
    .service(get_cur_id)
    .service(insert_form)
    .service(lasn_n_headers)
    .service(raw_body);
}

/// Получение списка файлов
#[get("/log/files")]
async fn get_queue_files() -> Result<impl Responder> {
    #[derive(Serialize)]
    struct LogFileInfo {
        name: String,
        items_count: u32
    }

    #[derive(Serialize)]
    struct Res {
        files: Vec<LogFileInfo>
    }

    let res = queue(|q| {
        let q = q.lock().unwrap();
        Res {
            files: q.files().iter().map(|(f,l)|
                LogFileInfo { name: f.to_str().unwrap().to_string(), items_count:l.count().unwrap() }
            ).collect()
        }
    });

    Ok(web::Json(res))
}

#[derive(Serialize)]
struct ID {
    log_id: String,
    block_id: String
}

impl From<RecID<LogQueueFileNumID>> for ID {
    fn from(value: RecID<LogQueueFileNumID>) -> Self {
        Self { 
            log_id: value.log_file_id.id.to_string(), 
            block_id: value.block_id.value().to_string() 
        }
    }
}

/// Получение текущее id последней записи
#[get("/tail/id")]
async fn get_cur_id() -> Result<impl Responder> {
    Ok( 
        web::Json( queue(|q| { 
            let q = q.lock().unwrap(); 
            let rid = q.last_record().unwrap();
            rid.map(|rid| 
                ID::from(rid)
            )
        }))
    )
}

struct PlainText {
    content: String,
    time: DateTime<Utc>,
}

impl From<PlainText> for PreparedRecord {
    fn from(value: PlainText) -> Self {
        let res = UTF_8.encode(&value.content, EncoderTrap::Ignore).unwrap();
        let df: date_format::DateFormat = DateFormatParser::default().parse("utc:yyyy-mm-ddThh:mi:ss.s6zhm").unwrap().0;

        let mut opts = BlockOptions::default();
        opts.set("encoding", UTF_8.name()).unwrap();
        opts.set("time", value.time.format(df)).unwrap();
        opts.set("mime", "text/plain").unwrap();

        PreparedRecord { 
            data: res, 
            options: opts
        }
    }
}

/// Добавление plain записи
#[post("/insert/plain")]
async fn insert_form(req_body: String) -> Result<impl Responder> {
    queue(|q|{
        let q = q.lock().unwrap();
        let rid = q.write( PlainText { content: req_body.clone(), time: Utc::now() } ).unwrap();
        let id: ID = rid.into();
        Ok( web::Json(id) )
    })
}

/// Просмотр заголовков последних n записей
#[get("/headers/last/{count}")]
async fn lasn_n_headers( path: web::Path<u32> ) -> Result<impl Responder> {
    let cnt: u32 = path.into_inner();
    queue(|q| {
        let q = q.lock().unwrap();

        #[derive(Serialize)]
        struct Item {
            rid: ID,
            result:ItemValue
        }

        #[derive(Serialize)]
        enum ItemValue {
            Succ {
                log_file: String,
                log_id: String,
                block_id: String,
                options: HashMap<String,String>,
                position: String,
                head_size: u32,
                data_size: u32,
                tail_size: u16,
            },
            Fail(String)
        }

        #[derive(Serialize)]
        struct Result {
            values: Vec<Item>,
            navigate_error: Option<String>,
        }

        let mut res = Vec::<Item>::new();
        let mut nav_err: Option<String> = None;

        match q.last_record().unwrap() {
            None => {
                Ok( web::Json(Result{ values: res, navigate_error:nav_err }) )
            }
            Some(mut rid) => {
                let mut cnt = cnt;
                while cnt > 0 {
                    cnt -= 1;
                    match q.info(rid.clone()) {
                        Ok(opts) => {
                            res.push(
                                Item { 
                                    rid: rid.clone().into(), 
                                    result: ItemValue::Succ { 
                                        log_file: opts.log_file.to_str().unwrap().to_string(),
                                        log_id: opts.log_id.id.to_string(),
                                        block_id: opts.block_id.value().to_string(),
                                        options: opts.block_options.into(),
                                        head_size: opts.head_size.0,
                                        position: opts.position.value().to_string(),
                                        data_size: opts.data_size.0,
                                        tail_size: opts.tail_size.0
                                    } 
                                });
                        },
                        Err(err) => {
                            res.push(
                                Item { 
                                    rid: rid.clone().into(), 
                                    result: ItemValue::Fail( format!("{:?}",err) ) 
                                });
                        }
                    }
                    match q.previous_record(rid.clone()) {
                        Ok(n_rid) => match n_rid {
                            Some(n_rid) => {rid = n_rid;}
                            None => {break;}
                        }
                        Err(err) => {
                            nav_err = Some(format!("{:?}",err));
                            break;
                        }
                    }
                }
                Ok( web::Json(Result{ values: res, navigate_error:nav_err }) )
            }
        }
    })
}

#[derive(Deserialize,Clone)]
struct RawBodyOpts {
    /// Заголовки содержат опции блока
    opt2head: Option<bool>,

    /// Префикс в опциях блока
    opt_prefix: Option<String>
}

/// Получение тела запроса
#[get("/record/{log:[0-9]+}/{block:[0-9]+}")]
async fn raw_body(path: web::Path<(String,u32)>, query:web::Query<RawBodyOpts>) -> HttpResponse {
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
