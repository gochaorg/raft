use std::path::PathBuf;

use crate::{logfile::{block::BlockOptions, LogFile, FlatBuff, LogErr}, bbuff::absbuff::FileBuff, logqueue::LogWriteErr};

use super::{LogWriting, RecID, LogFileQueue, LogQueueFileId, LoqErr, LogFileQueueImpl, LogQueueFileNumID, LogSwitching, PreparedRecord};

impl<LOGSwitch,LOGIdOf> LogWriting<LoqErr,RecID<LogQueueFileNumID>> 
for &LogFileQueueImpl<LogQueueFileNumID,PathBuf,LogFile<FileBuff>,LoqErr,LOGSwitch,LOGIdOf>
where
    LOGSwitch: LogSwitching<(PathBuf,LogFile<FileBuff>),LoqErr>,
    LOGIdOf: Fn((PathBuf,LogFile<FileBuff>)) -> Result<LogQueueFileNumID,LoqErr> + Clone,
{
    fn write<Record>( self, record:Record ) -> Result<RecID<LogQueueFileNumID>,LoqErr> 
    where Record: Into<PreparedRecord>
    {
        let prepared : PreparedRecord = record.into();
        let (file, mut log) = self.tail.clone();
        let b_id = log.append_data(&prepared.options, &prepared.data).map_err(|err| LogWriteErr(err))?;
        let id = (self.id_of)( (file,log).clone() )?;
        Ok( RecID { file_id:id, block_id: b_id } )
    }
}

impl LogWriting<LoqErr,RecID<LogQueueFileNumID>> 
for Box<dyn LogFileQueue<LoqErr,LogQueueFileNumID,PathBuf,LogFile<FileBuff>>>
{
    fn write<Record>( self, record:Record ) -> Result<RecID<LogQueueFileNumID>,LoqErr> 
    where Record: Into<PreparedRecord>
    {
        let prepared : PreparedRecord = record.into();
        let (file, mut log) = self.tail();
        let b_id = log.append_data(&prepared.options, &prepared.data).map_err(|err| LogWriteErr(err))?;
        let id = self.log_id_of(&(file,log))?;
        Ok( RecID { file_id:id, block_id: b_id } )
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