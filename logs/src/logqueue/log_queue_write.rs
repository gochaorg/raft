use std::path::PathBuf;

use crate::{logfile::{block::BlockOptions, LogFile, FlatBuff, LogErr}, bbuff::absbuff::FileBuff};

use super::{LogWriting, RecID, LogFileQueue, LogQueueFileId, LoqErr, LogFileQueueImpl, LogQueueFileNumID, LogSwitching};

impl<LOGSwitch,LOGIdOf> LogWriting<LoqErr,RecID<LogQueueFileNumID>> 
for &LogFileQueueImpl<LogQueueFileNumID,PathBuf,LogFile<FileBuff>,LoqErr,LOGSwitch,LOGIdOf>
where
    LOGSwitch: LogSwitching<(PathBuf,LogFile<FileBuff>),LoqErr>,
    LOGIdOf: Fn((PathBuf,LogFile<FileBuff>)) -> Result<LogQueueFileNumID,LoqErr> + Clone,
{
    fn write<Record>( self, record:Record ) -> Result<RecID<LogQueueFileNumID>,LoqErr> {
        todo!()
    }
}

impl LogWriting<LoqErr,RecID<LogQueueFileNumID>> 
for Box<dyn LogFileQueue<LoqErr,LogQueueFileNumID,PathBuf,LogFile<FileBuff>>>
{
    fn write<Record>( self, record:Record ) -> Result<RecID<LogQueueFileNumID>,LoqErr> {
        todo!()
    }
}