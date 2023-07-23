use std::fmt::Display;
use std::sync::{Arc, RwLock};

use actix_web::http::header::ContentType;
use actix_web::{web, error, HttpResponse};
use actix_web::post;
use logs::bbuff::absbuff::ByteBuff;
use logs::logfile::block::{BlockId, Block};
use logs::logqueue::*;

use crate::queue;
use crate::queue_api::ID;

#[derive(Debug)]
struct RawErr(pub String);

impl Display for RawErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}", self.0)
    }
}

impl error::ResponseError for RawErr {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).body(self.0.clone())
    }

    fn status_code(&self) -> actix_swagger::StatusCode {
        actix_swagger::StatusCode::INTERNAL_SERVER_ERROR
    }
}

struct WriteBlock(Block);
impl From<WriteBlock> for PreparedRecord {
    fn from(value: WriteBlock) -> Self {
        let data : Vec<u8> = value.0.data.iter().cloned().collect();
        Self { data: data, options: value.0.head.block_options }
    }
}

#[post("/record/{log:[0-9]+}/{block:[0-9]+}/raw")]
pub async fn write_block( bytes:web::Bytes, path: web::Path<(String,u32)> ) -> HttpResponse {
    let (log_id, block_id) = path.into_inner();
    let log_id = u128::from_str_radix(&log_id,10).unwrap();
    
    let log_id = LogQueueFileNumID { id: log_id, previous: None };
    let block_id = BlockId::new(block_id);
    let _rec_id = RecID { log_file_id: log_id, block_id: block_id };

    queue(|q|{
        let q = q.lock().unwrap();
        let cur_id = q.last_record().unwrap().unwrap();

        if cur_id.block_id!=block_id || cur_id.log_file_id.id()!=log_id.id() {
            return HttpResponse::BadRequest().body("log_id/block_id not matched")
        }

        let bytes = bytes.to_vec();
        let bbuff = ByteBuff {
            data: Arc::new(RwLock::new(bytes)),
            resizeable: true,
            max_size: None
        };

        let block = match Block::read_from(0u64, &bbuff) {
            Err(err) => return HttpResponse::BadRequest().body(format!("can't read block {err:?}")),
            Ok(block) => block.0
        };

        let rid = q.write(WriteBlock(block)).unwrap();
        let rid: ID = rid.into();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&rid).unwrap())
    })
}