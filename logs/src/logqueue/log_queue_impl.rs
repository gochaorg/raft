use core::fmt::Debug;
use std::sync::{Arc, RwLock};

use crate::logfile::FlatBuff;
use crate::logfile::block::FileOffset;
use super::super::logfile::LogFile;
use super::*;

pub struct LogQueueImpl<'a,LogId,FILE,BUFF> 
where
    LogId: Clone + Debug,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{
    pub queue: Arc<RwLock<dyn LogFileQueue<LogId,FILE,LogFile<BUFF>> + 'a>>
}

impl<'a,LogId,FILE,BUFF> LogQueueImpl<'a,LogId,FILE,BUFF>
where
    LogId: LogQueueFileId + 'a,
    FILE: Clone + Debug + 'a,
    BUFF: FlatBuff + 'a
{
    pub fn new<FNewFile,FOpen>( queue: LogFileQueueImpl<LogId,FILE,BUFF,FNewFile,FOpen> ) -> Self 
    where
        FNewFile: NewLogFile<FILE,LogId> + 'a,
        FOpen: OpenLogFile<FILE,LogFile<BUFF>,LogId> + 'a
    {
        Self { queue: Arc::new(RwLock::new(queue)) }
    }
}

impl<'a,LogId,FILE,BUFF> LogQueue<RecID<LogId>, LogId, FILE, LogFile<BUFF>>
for LogQueueImpl<'a,LogId,FILE,BUFF>
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{    
}

impl<'a,LogId,FILE,BUFF> LogNavigateLast<RecID<LogId>,FILE,LogId>
for LogQueueImpl<'a,LogId,FILE,BUFF>
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{
    fn last_record( &self ) -> Result<Option<RecID<LogId>>,LoqErr<FILE,LogId>> {
        self.queue.read()?.last_record()
    }
}

impl<'a,LogId,FILE,BUFF> LogFileQueue<LogId,FILE,LogFile<BUFF>>
for LogQueueImpl<'a,LogId,FILE,BUFF>
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{
    fn switch( &mut self ) -> Result<(FILE,LogId),LoqErr<FILE,LogId>> {
        self.queue.write().unwrap().switch()
    }

    fn find_log( &self, id:LogId ) -> Result<Option<(FILE,LogFile<BUFF>)>,LoqErr<FILE,LogId>> {
        self.queue.read()?.find_log(id)
    }

    fn offset_log_id( &self, id:LogId, offset: i64) -> Result<Option<LogId>, LoqErr<FILE,LogId>> {
        self.queue.read()?.offset_log_id(id, offset)
    }

    fn current_log_id( &self ) -> Result<LogId, LoqErr<FILE,LogId>> {
        self.queue.read()?.current_log_id()
    }

    fn files( &self ) -> Vec<(LogId,FILE,LogFile<BUFF>)> {
        self.queue.read().unwrap().files()
    }

    fn tail( &self ) -> (LogId,FILE,LogFile<BUFF>) {
        self.queue.read().unwrap().tail()
    }
}

impl<'a,LogId,FILE,BUFF> LogNavigationNear
for LogQueueImpl<'a,LogId,FILE,BUFF>
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{
    type FILE = FILE;
    type LogId = LogId;
    type RecordId = RecID<LogId>;

    fn next_record( &self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,LoqErr<Self::FILE,Self::LogId>> {
        self.queue.read()?.next_record(record_id)
    }

    fn previous_record( &self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,LoqErr<Self::FILE,Self::LogId>> {
        self.queue.read()?.previous_record(record_id)
    }
}

impl<'a,LogId,FILE,BUFF> LogReading
for LogQueueImpl<'a,LogId,FILE,BUFF>
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{
    type FILE = FILE;
    type LogId = LogId;
    type RecordId = RecID<LogId>;

    fn read( &self, record_id: Self::RecordId ) -> Result<PreparedRecord, LoqErr<Self::FILE,Self::LogId>> 
    {
        self.queue.read()?.read(record_id)
    }

    fn info( &self, record_id: Self::RecordId ) -> Result<RecordInfo<Self::FILE,Self::LogId>, LoqErr<Self::FILE,Self::LogId>> 
    {
        self.queue.read()?.info(record_id)
    }

    fn read_raw_bytes( &self, log_id: Self::LogId, pos: FileOffset, data_consumer:&mut [u8] ) -> Result<u64, LoqErr<Self::FILE, Self::LogId>> 
    {
        self.queue.read()?.read_raw_bytes(log_id, pos, data_consumer)
    }
}

impl<'a,LogId,FILE,BUFF> LogWriting<RecID<LogId>>
for LogQueueImpl<'a,LogId,FILE,BUFF>
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff
{
    type FILE = FILE;
    type LogId = LogId;

    fn write( &self, record:&PreparedRecord ) -> Result<RecID<LogId>,LoqErr<Self::FILE,Self::LogId>>
    {
        self.queue.read()?.write(record)
    }
}
