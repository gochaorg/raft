use core::fmt::Debug;
use std::marker::PhantomData;

use crate::logfile::FlatBuff;
use crate::logfile::block::FileOffset;
use super::super::logfile::LogFile;
use super::*;

pub struct Wrapper<Q,LogId,FILE,LOG> 
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>
{
    pub target:Q,
    _p:PhantomData<(LogId,FILE,LOG)>
}

impl<'a,LogId,FILE,BUFF,FNewFile,FOpen> 
    From<LogFileQueueImpl<LogId,FILE,BUFF,FNewFile,FOpen>> 
for Wrapper<LogQueueImpl<'a,LogId,FILE,BUFF>, LogId, FILE, LogFile<BUFF>>
where
    LogId: LogQueueFileId + 'a,
    FILE: Clone+Debug + 'a,
    BUFF: FlatBuff + 'a,
    FNewFile: NewLogFile<FILE,LogId> + 'a,
    FOpen: OpenLogFile<FILE,LogFile<BUFF>,LogId> + 'a
{
    fn from(value: LogFileQueueImpl<LogId,FILE,BUFF,FNewFile,FOpen>) -> Self {
        Self { target: LogQueueImpl::new(value), _p: PhantomData.clone() }
    }
}

// impl<'a,Q,LogId,FILE,LOG> Wrapper<Q,LogId,FILE,LOG> 
// where
//     LogId: LogQueueFileId + 'a,
//     FILE: Clone+Debug + 'a,
//     LOG: Clone+Debug + 'a,
//     Q: LogQueue<RecID<LogId>,LogId,FILE,LOG> 
// {
//     pub fn from_1<BUFF, FNewFile, FOpen>( value: &'a LogFileQueueImpl<LogId,FILE,BUFF,FNewFile,FOpen> ) -> Self
//     where
//     BUFF: FlatBuff + 'a,
//     FNewFile: NewLogFile<FILE,LogId> + 'a,
//     FOpen: OpenLogFile<FILE,LogFile<BUFF>,LogId> + 'a
//     {
//         value.clone().into()
//     }
// }

impl<Q,LogId,FILE,LOG> LogQueue<RecID<LogId>,LogId,FILE,LOG> for Wrapper<Q,LogId,FILE,LOG>
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>
{
}

impl<Q,LogId,FILE,LOG> LogFileQueue<LogId, FILE, LOG> for Wrapper<Q,LogId,FILE,LOG> 
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>
{
    fn switch( &mut self ) -> Result<(FILE,LogId),LoqErr<FILE,LogId>> {
        self.target.switch()
    }

    fn find_log( &self, id:LogId ) -> Result<Option<(FILE,LOG)>,LoqErr<FILE,LogId>> {
        self.target.find_log(id)
    }

    fn offset_log_id( &self, id:LogId, offset: i64) -> Result<Option<LogId>, LoqErr<FILE,LogId>> {
        self.target.offset_log_id(id, offset)
    }

    fn current_log_id( &self ) -> Result<LogId, LoqErr<FILE,LogId>> {
        self.target.current_log_id()
    }

    fn files( &self ) -> Vec<(LogId,FILE,LOG)> {
        self.target.files()
    }

    fn tail( &self ) -> (LogId,FILE,LOG) {
        self.target.tail()
    }
}

impl<Q,LogId,FILE,LOG> LogNavigateLast<RecID<LogId>, FILE, LogId> for Wrapper<Q,LogId,FILE,LOG> 
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>
{
    fn last_record( &self ) -> Result<Option<RecID<LogId>>,LoqErr<FILE,LogId>> {
        self.target.last_record()
    }
}

impl<Q,LogId,FILE,LOG> LogWriting<RecID<LogId>> for Wrapper<Q,LogId,FILE,LOG> 
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>
{
    type FILE = FILE;
    type LogId = LogId;

    fn write<Record>( &self, record:Record ) -> Result<RecID<LogId>,LoqErr<Self::FILE,Self::LogId>>
    where Record: Into<PreparedRecord> {
        self.target.write(record)
    }
}

impl<Q,LogId,FILE,LOG> LogReading for Wrapper<Q,LogId,FILE,LOG> 
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>
{
    type RecordId = RecID<LogId>;
    type FILE = FILE;
    type LogId = LogId;

    fn read( &self, record_id: Self::RecordId ) -> 
        Result<PreparedRecord, LoqErr<Self::FILE,Self::LogId>> {
        self.target.read(record_id)
    }

    fn info( &self, record_id: Self::RecordId ) -> 
        Result<RecordInfo<Self::FILE,Self::LogId>, LoqErr<Self::FILE,Self::LogId>> {
        self.target.info(record_id)
    }

    fn read_raw_bytes( &self, log_id: Self::LogId, pos: FileOffset, data_consumer:&mut [u8] ) ->
        Result<u64, LoqErr<Self::FILE, Self::LogId>> {
        self.target.read_raw_bytes(log_id, pos, data_consumer)
    }    
}

impl<Q,LogId,FILE,LOG> LogNavigationNear for Wrapper<Q,LogId,FILE,LOG> 
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>
{
    type RecordId = RecID<LogId>;
    type FILE = FILE;
    type LogId = LogId;

    fn next_record( &self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,LoqErr<Self::FILE,Self::LogId>> {
        self.target.next_record(record_id)
    }

    fn previous_record( &self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,LoqErr<Self::FILE,Self::LogId>> {
        self.target.previous_record(record_id)
    }
}


