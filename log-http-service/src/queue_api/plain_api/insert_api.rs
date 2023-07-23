use actix_web::{web, Responder, post};
use actix_web::Result;
use chrono::{DateTime, Utc};
use date_format::{DateFormatParser, Format};
use logs::logfile::block::BlockOptions;
use logs::logqueue::*;
use parse::Parser;
use encoding::all::UTF_8;
use encoding::{Encoding, EncoderTrap};

use crate::queue;
use crate::queue_api::ID;

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
pub async fn insert_plain(req_body: String) -> Result<impl Responder> {
    queue(|q|{
        let q = q.lock().unwrap();
        let rid = q.write( PlainText { content: req_body.clone(), time: Utc::now() } ).unwrap();
        let id: ID = rid.into();
        Ok( web::Json(id) )
    })
}
