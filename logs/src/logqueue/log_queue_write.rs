use std::fmt::Debug;
use crate::logfile::{block::BlockOptions, LogFile, FlatBuff};
use super::{LogWriting, RecID, LogFileQueue, LogQueueFileId, LoqErr, PreparedRecord};

impl<FILE,BUFF,LogId> LogWriting<RecID<LogId>> 
for & dyn LogFileQueue<LogId,FILE,LogFile<BUFF>>
where 
    FILE: Clone + Debug,
    BUFF: FlatBuff,
    LogId: LogQueueFileId
{
    type FILE = FILE;
    type LogId = LogId;

    fn write<Record>( self, record:Record ) -> Result<RecID<LogId>,LoqErr<Self::FILE,Self::LogId>> 
    where Record: Into<PreparedRecord>
    {
        let prepared : PreparedRecord = record.into();
        let (file, mut log) = self.tail();
        let b_id = log.write_block(&prepared.options, &prepared.data)
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