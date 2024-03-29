use std::fmt::Debug;
use crate::logfile::{block::BlockOptions, LogFile, FlatBuff};
use super::{LogWriting, RecID, LogFileQueue, LogQueueFileId, LoqErr, PreparedRecord};

impl<'a,FILE,BUFF,LogId> LogWriting<RecID<LogId>> 
for dyn LogFileQueue<LogId,FILE,LogFile<BUFF>> + 'a
where 
    FILE: Clone + Debug,
    BUFF: FlatBuff,
    LogId: LogQueueFileId
{
    type FILE = FILE;
    type LogId = LogId;

    fn write( &self, record:&PreparedRecord ) -> Result<RecID<LogId>,LoqErr<Self::FILE,Self::LogId>> 
    {
        //let prepared : PreparedRecord = record.into();
        let (_,file, mut log) = self.tail();
        let b_id = log.write_block(&record.options, &record.data)
            .map_err(|err| 
                LoqErr::LogDataWrite { 
                    file: file.clone(),
                    error: err 
                }
            )?;

        let id = LogId::read(&file, &log)?;
        Ok( RecID { log_file_id:id, block_id: b_id } )
    }
}

impl From<i32> for PreparedRecord {
    fn from(value: i32) -> Self {
        let mut data = Vec::<u8>::new();
        data.extend_from_slice(&value.to_le_bytes());
        PreparedRecord { 
            data: data, 
            options: BlockOptions::default()
        }
    }
}