use crate::logfile::{block::BlockOptions, LogFile, FlatBuff, LogErr};

use super::{LogWriting, RecID, LogFileQueue, LogQueueFileId};

/// Добавляемся запись
pub struct Record<'a> {
    /// Данные
    pub data: &'a [u8],
    /// опции ключ/значение
    pub options: BlockOptions,
}

impl <'a,ERR,LogId,FILE,BUFF> LogWriting<ERR,RecID<LogId>,Record<'a>>
for Box<dyn LogFileQueue<ERR, LogId, FILE, LogFile<BUFF>>>
where
    LogId: LogQueueFileId,
    BUFF: FlatBuff,
    ERR: From<LogErr>
{
    fn write( self, record:Record<'a> ) -> Result<RecID<LogId>,ERR> {
        let (file,mut tail) = self.tail();
        let block_id = tail.append_data(&record.options, record.data)?;
        let log_id = self.log_id_of(&(file,tail))?;
        Ok(RecID {
            file_id: log_id,
            block_id: block_id
        })
    }
}
