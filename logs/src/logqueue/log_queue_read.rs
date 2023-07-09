use crate::logfile::{LogFile, FlatBuff, LogErr, block::{BlockId, BlockOptions}};

use super::{LogNavigationNear, log_id::{RecID, LogQueueFileId}, log_queue::LogFileQueue, LogNavigateLast, LogReading};

/// Реализация чтения логов для dyn LogFileQueue
impl<ERR,LogId, FILE, BUFF> LogNavigationNear<ERR,RecID<LogId>> 
for dyn LogFileQueue<ERR, LogId, FILE, LogFile<BUFF>>
where
    LogId: LogQueueFileId,
    BUFF: FlatBuff,
    ERR: From<LogErr>,
{
    fn next_record( &self, record_id: RecID<LogId> ) -> Result<Option<RecID<LogId>>,ERR> {
        let res = 
        self.find_log(record_id.file_id.clone())?.and_then(|(_file,log)| {
            let count = log.count().ok()?; // TODO здесь теряется информация о ошибке
            if record_id.block_id.value() >= (count-1) {
                self.offset_log_id(record_id.file_id.clone(), 1).ok()? // TODO здесь теряется информация о ошибке
                .and_then(|next_log_id| {
                    Some(RecID{ 
                        file_id: next_log_id,
                        block_id: BlockId::new(0)
                    })
                })
            } else {
                Some(RecID { 
                    file_id: record_id.file_id.clone(), 
                    block_id: BlockId::new(record_id.block_id.value()+1)
                })
            }
        });
        Ok(res)
    }

    fn previous_record( &self, record_id: RecID<LogId> ) -> Result<Option<RecID<LogId>>,ERR> {
        let result =
        if record_id.block_id.value() == 0 {
            self.offset_log_id(record_id.file_id.clone(), -1)?
            .and_then(|prev_log_id|{
                self.find_log(prev_log_id.clone()).ok()? // TODO здесь теряется информация о ошибке
                .and_then(|(_,log)|{
                    let count = log.count().ok()?; // TODO здесь теряется информация о ошибке
                    if count > 0 {
                        Some(RecID{
                            file_id: prev_log_id.clone(),
                            block_id: BlockId::new(count-1)
                        })
                    } else {
                        None
                    }
                })
            })
        } else {
            Some(RecID{
                file_id: record_id.file_id.clone(),
                block_id: BlockId::new(record_id.block_id.value()-1)
            })
        };
        Ok(result)
    }
}

impl <ERR,LogId,FILE,BUFF> LogNavigateLast<ERR,RecID<LogId>>
for dyn LogFileQueue<ERR, LogId, FILE, LogFile<BUFF>>
where
    LogId: LogQueueFileId,
    BUFF: FlatBuff,
    ERR: From<LogErr>,
{
    fn last_record( &self ) -> Result<Option<RecID<LogId>>,ERR> {
        let (file,tail) = self.tail();
        let cnt = tail.count()?;
        if cnt==0 {
            return Ok(None)
        }

        let lfile = (file,tail);
        let log_id = self.log_id_of(&lfile)?;
        
        Ok(Some(RecID {
            file_id: log_id,
            block_id: BlockId::new(cnt-1)
        }))
    }
}

pub enum LogReadingErr {
    LogNotFound
}

impl <ERR,LogId,FILE,BUFF> LogReading<ERR, RecID<LogId>, Box<Vec<u8>>, BlockOptions>
for dyn LogFileQueue<ERR, LogId, FILE, LogFile<BUFF>>
where
    LogId: LogQueueFileId,
    BUFF: FlatBuff,
    ERR: From<LogReadingErr> + From<LogErr>
{
    fn read_record( &self, record_id: RecID<LogId> ) -> Result<(Box<Vec<u8>>,BlockOptions), ERR> {
        match self.find_log(record_id.file_id.clone())? {
            None => {
                return Err( LogReadingErr::LogNotFound.into() )
            },
            Some( (_,log) ) => {
                let res = log.get_block(record_id.block_id.clone())?;
                let opts = res.head.block_options.clone();
                Ok((res.data, opts))
            }
        }
    }

    fn read_options( &self, record_id: RecID<LogId> ) -> Result<BlockOptions, ERR> {
        match self.find_log(record_id.file_id.clone())? {
            None => {
                return Err( LogReadingErr::LogNotFound.into() )
            },
            Some( (_,log) ) => {
                let res = log.get_block_header_read(record_id.block_id.clone())?;
                Ok(res.head.block_options)
            }
        }
    }
}

