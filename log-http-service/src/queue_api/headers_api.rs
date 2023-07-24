use crate::queue;
use crate::queue_api::{ID, ApiErr};

use std::collections::HashMap;

use actix_web::{web, Responder, get};
use actix_web::Result;
use logs::logqueue::*;
use serde::Serialize;

// trait ShouldSkip { fn should_skip(&self) -> bool; }

// impl<T> ShouldSkip for T {
//     fn should_skip(&self) { false }
// }

// impl<T> ShouldSkip for Option<T> {
//     fn should_skip(&self) { self.is_none() }
// }

/// Просмотр заголовков последних n записей
#[get("/headers/last/{count}")]
pub async fn lasn_n_headers( path: web::Path<u32> ) -> Result<impl Responder,ApiErr> {
    let cnt: u32 = path.into_inner();
    queue(|q| {
        let q = q.lock()?;

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

                #[serde(skip_serializing_if="Option::is_none")]
                preview: Option<String>,
            },
            Fail(String)
        }

        #[derive(Serialize)]
        struct Result {
            values: Vec<Item>,

            #[serde(skip_serializing_if="Option::is_none")]
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
                            let enc = opts.block_options.get("mime").and_then(
                                |mime| {
                                    if mime.value().starts_with("text/") {
                                        opts.block_options.get("encoding").and_then(|enc_name| {
                                            encoding::all::encodings().into_iter()
                                            .find(|enc| enc.name() == enc_name.value() )
                                        })
                                    } else {
                                        None
                                    }
                                }
                            ).cloned();

                            let preview  =enc.and_then( |enc| q.read(rid.clone()).ok()
                                .map(|rec| enc.decode(&rec.data, encoding::DecoderTrap::Replace)
                                ))
                                .and_then(|res| res.ok());

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
                                        tail_size: opts.tail_size.0,
                                        preview: preview
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