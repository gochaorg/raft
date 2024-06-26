use actix_web::{post, web, HttpRequest, Responder};
use actix_web::Result;
use chrono::{DateTime, Utc};
use date_format::{DateFormatParser, Format};
use logs::logfile::block::{BlockId, BlockOptions};
use logs::logqueue::*;
use parse::Parser;
use encoding::all::UTF_8;
use encoding::{Encoding, EncoderTrap};
use serde::Deserialize;

use crate::queue;
use crate::queue_api::{ID, ApiErr};

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
#[post("/insert/text_plain")]
pub async fn insert_text_plain(req_body: String) -> Result<impl Responder,ApiErr> {
    queue(|q|{
        let q = q.lock()?;
        let pr: PreparedRecord = PlainText { content: req_body.clone(), time: Utc::now() }.into();
        let rid = q.write( &pr )?;
        let id: ID = rid.into();
        Ok( web::Json(id) )
    })
}

#[derive(Deserialize,Clone)]
pub struct WriteBytesOpt {
    /// Заголовки содержат опции блока
    head2opt: Option<bool>,

    /// Префикс в опциях блока
    opt_prefix: Option<String>
}

/// Добавление записи в указанную позицию - в конец очереди
/// Проверяется корректность указания конца
#[post("/record/{log:[0-9]+}/{block:[0-9]+}/bytes")]
pub async fn write_bytes_at_tail( bytes:web::Bytes, path: web::Path<(String,u32)>, req: HttpRequest, query:web::Query<WriteBytesOpt> ) -> Result<impl Responder,ApiErr> {
    let (log_id, block_id) = path.into_inner();
    let log_id = u128::from_str_radix(&log_id,10).unwrap();

    let block_id = BlockId::new(block_id);
    let log_id = LogQueueFileNumID { id: log_id, previous: None };

    queue(|q|{
        let q = q.lock()?;

        // Проверка корректности указания позиции
        match q.last_record()? {
            Some(last_rec_id) => {
                match (last_rec_id.block_id == block_id, last_rec_id.log_file_id.id() == log_id.id()) {
                    (true,true) => Ok(()),
                    _ => Err(ApiErr::RecIdNotMatch { 
                        expect_log_id: last_rec_id.log_file_id.id().to_string(), 
                        actual_log_id: log_id.id().to_string(), 
                        expect_block_id: last_rec_id.block_id.to_string(), 
                        actual_block_id: block_id.to_string() 
                    })
                }
            },
            None => match block_id.0 {
                0 => Ok(()),
                _ => Err(ApiErr::RecIdNotMatch { 
                    expect_log_id: "0".to_string(), 
                    actual_log_id: log_id.id().to_string(), 
                    expect_block_id: "0".to_string(), 
                    actual_block_id: block_id.to_string() 
                })
            }
        }?;

        // формирование записи
        let bytes = bytes.to_vec();
        let mut b_opt = BlockOptions::default();

        let write_opt = query.clone().into_inner();
        if write_opt.head2opt.unwrap_or(false) {
            let pref = write_opt.opt_prefix.unwrap_or("".to_string());
            let pref_usize = pref.len();
            for (header_name,header_value) in  req.headers().into_iter() {
                if header_name.as_str().starts_with(&pref) {
                    let (_, key) = header_name.as_str().split_at(pref_usize);
                    let value = header_value.to_str()
                        .map_err(|e| ApiErr::BadRequest(
                            format!("can't decode header {}: {}", header_name.as_str(), e.to_string())))?.to_string();
                    b_opt.set(key, value)?;
                }
            }
        }

        let pr = PreparedRecord {
            data: bytes,
            options: b_opt 
        };

        let rid = q.write(&pr)?;
        let rid: ID = rid.into();

        Ok(web::Json(rid))
    })
}

/// Добавление записи в указанную позицию - в конец очереди
/// Проверяется корректность указания конца
#[post("/record/bytes")]
pub async fn write_bytes( bytes:web::Bytes, req: HttpRequest, query:web::Query<WriteBytesOpt> ) -> Result<impl Responder,ApiErr> {
    queue(|q|{
        let q = q.lock()?;

        // формирование записи
        let bytes = bytes.to_vec();
        let mut b_opt = BlockOptions::default();

        let write_opt = query.clone().into_inner();
        if write_opt.head2opt.unwrap_or(false) {
            let pref = write_opt.opt_prefix.unwrap_or("".to_string());
            let pref_usize = pref.len();
            for (header_name,header_value) in  req.headers().into_iter() {
                if header_name.as_str().starts_with(&pref) {
                    let (_, key) = header_name.as_str().split_at(pref_usize);
                    let value = header_value.to_str()
                        .map_err(|e| ApiErr::BadRequest(
                            format!("can't decode header {}: {}", header_name.as_str(), e.to_string())))?.to_string();
                    b_opt.set(key, value)?;
                }
            }
        }

        let pr = PreparedRecord {
            data: bytes,
            options: b_opt 
        };

        let rid = q.write(&pr)?;
        let rid: ID = rid.into();

        Ok(web::Json(rid))
    })
}

