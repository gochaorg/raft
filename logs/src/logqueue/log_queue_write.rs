use std::fmt::Debug;

use crate::{logfile::{block::BlockOptions, LogFile, FlatBuff, LogErr}, bbuff::absbuff::FileBuff, logqueue::LogWriteErr};

use super::{LogWriting, RecID, LogFileQueue, LogQueueFileId, LoqErr, LogFileQueueImpl, LogQueueFileNumID, LogSwitching, PreparedRecord};

impl<FILE,BUFF,LogId> LogWriting<RecID<LogId>> 
for & dyn LogFileQueue<LoqErr<FILE,LogId>,LogId,FILE,LogFile<BUFF>>
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
        let b_id = log.append_data(&prepared.options, &prepared.data)
            .map_err(|err| 
                LoqErr::LogDataWrite { error: err }
            )?;
        let id = self.log_id_of(&(file,log))?;
        Ok( RecID { log_file_id:id, block_id: b_id } )
    }
}

impl From<i32> for PreparedRecord {
    fn from(value: i32) -> Self {
        PreparedRecord { 
            data: Box::new(value.to_le_bytes()), 
            options: BlockOptions::default()
        }
    }
}