use core::fmt::Debug;

use crate::logfile::FlatBuff;
use crate::logfile::block::FileOffset;
use super::super::logfile::LogFile;
use super::*;

pub struct LogQueueImpl<LogId,FILE,BUFF> 
where
    LogId: Clone + Debug,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{
    queue: Box<dyn LogFileQueue<LogId,FILE,LogFile<BUFF>>>
}

impl<LogId,FILE,BUFF> LogQueue<RecID<LogId>, LogId, FILE, LogFile<BUFF>>
for LogQueueImpl<LogId,FILE,BUFF>
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{    
}

impl<LogId,FILE,BUFF> LogNavigateLast
for LogQueueImpl<LogId,FILE,BUFF>
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{
    type FILE = FILE;
    type LogId = LogId;
    type RecordId = RecID<LogId>;

    fn last_record( self ) -> Result<Option<Self::RecordId>,LoqErr<Self::FILE,Self::LogId>> {
        self.queue.last_record()
    }
}

impl<LogId,FILE,BUFF> LogFileQueue<LogId,FILE,LogFile<BUFF>>
for LogQueueImpl<LogId,FILE,BUFF>
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{
    fn switch( &mut self ) -> Result<(FILE,LogId),LoqErr<FILE,LogId>> {
        self.queue.switch()
    }

    fn find_log( &self, id:LogId ) -> Result<Option<(FILE,LogFile<BUFF>)>,LoqErr<FILE,LogId>> {
        self.queue.find_log(id)
    }

    fn offset_log_id( &self, id:LogId, offset: i64) -> Result<Option<LogId>, LoqErr<FILE,LogId>> {
        self.queue.offset_log_id(id, offset)
    }

    fn current_log_id( &self ) -> Result<LogId, LoqErr<FILE,LogId>> {
        self.queue.current_log_id()
    }

    fn files( &self ) -> Vec<(LogId,FILE,LogFile<BUFF>)> {
        self.queue.files()
    }

    fn tail( &self ) -> (LogId,FILE,LogFile<BUFF>) {
        self.queue.tail()
    }
}

impl<LogId,FILE,BUFF> LogNavigationNear
for LogQueueImpl<LogId,FILE,BUFF>
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{
    type FILE = FILE;
    type LogId = LogId;
    type RecordId = RecID<LogId>;

    fn next_record( self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,LoqErr<Self::FILE,Self::LogId>> {
        self.queue.next_record(record_id)
    }

    fn previous_record( self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,LoqErr<Self::FILE,Self::LogId>> {
        self.queue.previous_record(record_id)
    }

}

impl<LogId,FILE,BUFF> LogReading
for LogQueueImpl<LogId,FILE,BUFF>
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{
    type FILE = FILE;
    type LogId = LogId;
    type RecordId = RecID<LogId>;

    fn read( self, record_id: Self::RecordId ) -> 
        Result<PreparedRecord, LoqErr<Self::FILE,Self::LogId>> {
        self.queue.read(record_id)
    }

    fn info( self, record_id: Self::RecordId ) -> 
        Result<RecordInfo<Self::FILE,Self::LogId>, LoqErr<Self::FILE,Self::LogId>> {
        self.queue.info(record_id)
    }

    fn read_raw_bytes( self, log_id: Self::LogId, pos: FileOffset, data_consumer:&mut [u8] ) ->
        Result<u64, LoqErr<Self::FILE, Self::LogId>> {
        self.queue.read_raw_bytes(log_id, pos, data_consumer)
    }
}

impl<LogId,FILE,BUFF> LogWriting<RecID<LogId>>
for LogQueueImpl<LogId,FILE,BUFF>
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{
    type FILE = FILE;
    type LogId = LogId;

    fn write<Record>( self, record:Record ) -> Result<RecID<LogId>,LoqErr<Self::FILE,Self::LogId>>
    where Record: Into<PreparedRecord> {
        self.queue.write(record)
    }
}
