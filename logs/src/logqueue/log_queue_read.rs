#[allow(unused)]
use std::{fmt::Debug, path::PathBuf};

#[allow(unused)]
use crate::logfile::{LogFile, FlatBuff, LogErr, block::{BlockId, BlockOptions}};

use super::{PreparedRecord, RecordInfo};
#[allow(unused)]
use super::{LogNavigationNear, log_id::{RecID, LogQueueFileId}, log_queue::LogFileQueue, LogNavigateLast, LogReading, LoqErr};

/// Реализация чтения логов для dyn LogFileQueue
impl<'a, LogId, FILE, BUFF> LogNavigationNear
for dyn LogFileQueue<LogId, FILE, LogFile<BUFF>> + 'a
where
    LogId: LogQueueFileId,
    BUFF: FlatBuff,
    FILE: Clone + Debug,
{
    type RecordId = RecID<LogId>;
    type FILE = FILE;
    type LogId = LogId;

    fn next_record( &self, record_id: RecID<LogId> ) -> 
    Result<Option<RecID<LogId>>,LoqErr<Self::FILE,Self::LogId>> {
        let res = 
        self.find_log(record_id.log_file_id.clone())?.and_then(|(_file,log)| {
            let count = log.count().ok()?; // TODO здесь теряется информация о ошибке
            if record_id.block_id.value() >= (count-1) {
                self.offset_log_id(record_id.log_file_id.clone(), 1).ok()? // TODO здесь теряется информация о ошибке
                .and_then(|next_log_id| {
                    Some(RecID{ 
                        log_file_id: next_log_id,
                        block_id: BlockId::new(0)
                    })
                })
            } else {
                Some(RecID { 
                    log_file_id: record_id.log_file_id.clone(), 
                    block_id: BlockId::new(record_id.block_id.value()+1)
                })
            }
        });
        Ok(res)
    }

    fn previous_record( &self, record_id: RecID<LogId> ) -> 
    Result<Option<RecID<LogId>>,LoqErr<Self::FILE,Self::LogId>> {
        let result =
        if record_id.block_id.value() == 0 {
            self.offset_log_id(record_id.log_file_id.clone(), -1)?
            .and_then(|prev_log_id|{
                self.find_log(prev_log_id.clone()).ok()? // TODO здесь теряется информация о ошибке
                .and_then(|(_,log)|{
                    let count = log.count().ok()?; // TODO здесь теряется информация о ошибке
                    if count > 0 {
                        Some(RecID{
                            log_file_id: prev_log_id.clone(),
                            block_id: BlockId::new(count-1)
                        })
                    } else {
                        None
                    }
                })
            })
        } else {
            Some(RecID{
                log_file_id: record_id.log_file_id.clone(),
                block_id: BlockId::new(record_id.block_id.value()-1)
            })
        };
        Ok(result)
    }
}

impl <'a,LogId,FILE,BUFF> LogNavigateLast<RecID<LogId>,FILE,LogId>
for dyn LogFileQueue<LogId, FILE, LogFile<BUFF>>  + 'a
where
    LogId: LogQueueFileId,
    BUFF: FlatBuff,
    FILE: Clone + Debug,    
{
    fn last_record( &self ) -> Result<Option<RecID<LogId>>,LoqErr<FILE,LogId>> {
        let (_id,file_name,tail) = self.tail();
        let file_name0 = file_name.clone();
        let cnt = tail.count()
            .map_err(|err| LoqErr::LogCountFail { 
                file: file_name0, 
                error: err 
        })?;

        if cnt==0 {
            return Ok(None)
        }

        let log_id = LogId::read(&file_name, &tail)?;
        
        Ok(Some(RecID {
            log_file_id: log_id,
            block_id: BlockId::new(cnt-1)
        }))
    }
}

impl <'a,LogId,FILE,BUFF> LogReading
for dyn LogFileQueue<LogId, FILE, LogFile<BUFF>> + 'a
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff,
{
    type RecordId = RecID<LogId>;
    type FILE = FILE;
    type LogId = LogId;

    fn read( &self, record_id: RecID<LogId> ) -> 
        Result<PreparedRecord, LoqErr<Self::FILE,Self::LogId>> 
    {
        match self.find_log(record_id.log_file_id.clone())? {
            None => {
                return Err(
                    LoqErr::LogIdNotMatched { log_id: record_id.log_file_id.clone() }
                )
            },
            Some( (file_name,log) ) => {
                let res = log.read_block(record_id.block_id.clone())
                    .map_err(|err| LoqErr::LogGetBlock { 
                        file: file_name.clone(), 
                        error: err,
                        block_id: record_id.block_id
                    })?;
                let opts = res.head.block_options.clone();

                let rec = PreparedRecord { data: res.data.as_ref().clone(), options: opts };
                Ok(rec)
            }
        }
    }

    fn info( &self, record_id: RecID<LogId> ) -> 
        Result<RecordInfo<Self::FILE,Self::LogId>, LoqErr<Self::FILE,Self::LogId>> 
    {
        match self.find_log(record_id.log_file_id.clone())? {
            None => {
                return Err( 
                    LoqErr::LogIdNotMatched { log_id: record_id.log_file_id.clone() }
                )
            },
            Some( (file_name,log) ) => {
                let res = log.read_block_header(record_id.block_id.clone())

                .map_err(|err| LoqErr::LogGetBlock { 
                    file: file_name.clone(), 
                    error: err,
                    block_id: record_id.block_id
                })?;

                Ok( RecordInfo { 
                    log_file: file_name.clone(), 
                    log_id: record_id.log_file_id.clone(), 
                    block_id: record_id.block_id.clone(), 
                    block_options: res.head.block_options, 
                    position: res.position, 
                    head_size: res.head_size, 
                    data_size: res.data_size, 
                    tail_size: res.tail_size 
                })
            }
        }
    }

    fn read_raw_bytes( &self, log_id: Self::LogId, pos: crate::logfile::block::FileOffset, data_consumer:&mut [u8] ) ->
        Result<u64, LoqErr<Self::FILE, Self::LogId>> 
    {
        match self.find_log(log_id.clone())? {
            None => {
                return Err( 
                    LoqErr::LogIdNotMatched { log_id: log_id.clone() }
                )
            },
            Some( (file_name,log) ) => {
                log.read_raw_bytes(pos.value(), data_consumer)
                    .map_err(|e| LoqErr::LogRawRead { 
                        file: file_name.clone(), 
                        log_id: log_id.clone(),
                        pos: pos, 
                        data_size: data_consumer.len(), 
                        error: e
                    })
            }
        }
    }    
}

