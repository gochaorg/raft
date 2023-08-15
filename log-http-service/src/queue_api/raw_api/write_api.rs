use actix_web::{web, Responder};
use actix_web::post;
use logs::logfile::block::{BlockId, Block};
use logs::logqueue::*;
use crate::queue;
use crate::queue_api::{ID, ApiErr};

struct WriteBlock(Block);
impl From<WriteBlock> for PreparedRecord {
    fn from(value: WriteBlock) -> Self {
        let data : Vec<u8> = value.0.data.iter().cloned().collect();
        Self { data: data, options: value.0.head.block_options }
    }
}

#[post("/record/{log:[0-9]+}/{block:[0-9]+}/raw")]
pub async fn write_block( bytes:web::Bytes, path: web::Path<(String,u32)> ) -> Result<impl Responder,ApiErr> {
    let (log_id, block_id) = path.into_inner();
    let log_id = u128::from_str_radix(&log_id,10).unwrap();
    
    let log_id = LogQueueFileNumID { id: log_id, previous: None };
    let block_id = BlockId::new(block_id);
    let _rec_id = RecID { log_file_id: log_id, block_id: block_id };

    queue(|q|{
        let q = q.lock()?;

        let cur_id = match q.last_record()? {
            Some(v) => Ok(v),
            None => Err(ApiErr::QueueIsEmpy)
        }?;

        if cur_id.block_id!=block_id || cur_id.log_file_id.id()!=log_id.id() {
            return Err(ApiErr::RecIdNotMatch { 
                expect_block_id:cur_id.block_id.to_string(), actual_block_id: block_id.to_string(),
                expect_log_id: cur_id.log_file_id.id().to_string(), actual_log_id: log_id.id().to_string(),
            });
        }

        let bytes = bytes.to_vec();
        let block = Block::from_bytes(&bytes)?;

        let pr: PreparedRecord = WriteBlock(block).into();

        let rid = q.write(&pr)?;
        let rid: ID = rid.into();

        Ok(web::Json(rid))
    })
}