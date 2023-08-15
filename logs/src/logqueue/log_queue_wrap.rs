use core::fmt::Debug;
use std::marker::PhantomData;

use crate::logfile::FlatBuff;
use crate::logfile::block::FileOffset;
use super::super::logfile::LogFile;
use super::*;

pub trait Logging<LogId,FILE,LOG>
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
{
    fn switch( 
        &self,
        _args:(), 
        res:Result<(FILE,LogId),LoqErr<FILE,LogId>> 
    ) -> Result<(FILE,LogId),LoqErr<FILE,LogId>> { res }

    fn find_log(
        &self,
        _args:LogId,
        res:Result<Option<(FILE,LOG)>,LoqErr<FILE,LogId>>
    ) -> Result<Option<(FILE,LOG)>,LoqErr<FILE,LogId>> { res }

    fn offset_log_id( 
        &self, 
        _args: (LogId, i64),
        res: Result<Option<LogId>, LoqErr<FILE,LogId>>
    ) -> Result<Option<LogId>, LoqErr<FILE,LogId>> { res }

    fn current_log_id( &self, _args:(), res: Result<LogId, LoqErr<FILE,LogId>> )
    -> Result<LogId, LoqErr<FILE,LogId>> { res }

    fn files( &self, _args:(), res:Vec<(LogId,FILE,LOG)> ) 
    -> Vec<(LogId,FILE,LOG)> { res }

    fn tail( &self, _args:(), res:(LogId,FILE,LOG) )
    -> (LogId,FILE,LOG) { res }

    fn last_record( &self, _args:(), res:Result<Option<RecID<LogId>>,LoqErr<FILE,LogId>> )
    -> Result<Option<RecID<LogId>>,LoqErr<FILE,LogId>> { res }

    fn write( &self, _args:PreparedRecord, res:Result<RecID<LogId>,LoqErr<FILE,LogId>> )
    -> Result<RecID<LogId>,LoqErr<FILE,LogId>> { res }

    fn read( &self, _args:RecID<LogId>, res:Result<PreparedRecord,LoqErr<FILE,LogId>> )
    -> Result<PreparedRecord,LoqErr<FILE,LogId>> { res }

    fn info( &self, _args:RecID<LogId>, res:Result<RecordInfo<FILE,LogId>,LoqErr<FILE,LogId>> )
    -> Result<RecordInfo<FILE,LogId>,LoqErr<FILE,LogId>> { res }

    fn read_raw_bytes( &self, _args:(LogId,FileOffset), res:Result<u64, LoqErr<FILE,LogId>> )
    -> Result<u64, LoqErr<FILE,LogId>> { res }

    fn next_record( &self, _args:RecID<LogId>, res:Result<Option<RecID<LogId>>,LoqErr<FILE,LogId>> )
    -> Result<Option<RecID<LogId>>,LoqErr<FILE,LogId>> { res }

    fn previous_record( &self, _args:RecID<LogId>, res:Result<Option<RecID<LogId>>,LoqErr<FILE,LogId>> )
    -> Result<Option<RecID<LogId>>,LoqErr<FILE,LogId>> { res }
}

pub struct NoLog;

impl<LogId,FILE,LOG> Logging<LogId,FILE,LOG> for NoLog 
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
{    
}

#[derive(Clone,Debug)]
pub struct Wrapper<Q,L,LogId,FILE,LOG> 
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>,
    L: Logging<LogId,FILE,LOG>
{
    pub target:Q,
    pub wrap:L,
    _p:PhantomData<(LogId,FILE,LOG)>,
}

impl<'a,LogId,FILE,BUFF,FNewFile,FOpen> 
    From<LogFileQueueImpl<LogId,FILE,BUFF,FNewFile,FOpen>> 
for Wrapper<LogQueueImpl<'a,LogId,FILE,BUFF>, NoLog, LogId, FILE, LogFile<BUFF>>
where
    LogId: LogQueueFileId + 'a,
    FILE: Clone+Debug + 'a,
    BUFF: FlatBuff + 'a,
    FNewFile: NewLogFile<FILE,LogId> + 'a,
    FOpen: OpenLogFile<FILE,LogFile<BUFF>,LogId> + 'a
{
    fn from(value: LogFileQueueImpl<LogId,FILE,BUFF,FNewFile,FOpen>) -> Self {
        Self { 
            target: LogQueueImpl::new(value), 
            wrap: NoLog,
            _p: PhantomData.clone() 
        }
    }
}

impl<Q,L,LogId,FILE,LOG> LogQueue<RecID<LogId>,LogId,FILE,LOG> for Wrapper<Q,L,LogId,FILE,LOG>
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>,
    L: Logging<LogId,FILE,LOG>
{
}

impl<Q,L,LogId,FILE,LOG> LogFileQueue<LogId, FILE, LOG> for Wrapper<Q,L,LogId,FILE,LOG> 
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>,
    L: Logging<LogId,FILE,LOG>
{
    fn switch( &mut self ) -> Result<(FILE,LogId),LoqErr<FILE,LogId>> {
        self.wrap.switch((), self.target.switch())
    }

    fn find_log( &self, id:LogId ) -> Result<Option<(FILE,LOG)>,LoqErr<FILE,LogId>> {
        self.wrap.find_log(id.clone(), self.target.find_log(id))
    }

    fn offset_log_id( &self, id:LogId, offset: i64) -> Result<Option<LogId>, LoqErr<FILE,LogId>> {
        self.wrap.offset_log_id((id.clone(), offset), self.target.offset_log_id(id, offset))
    }

    fn current_log_id( &self ) -> Result<LogId, LoqErr<FILE,LogId>> {
        self.wrap.current_log_id( (), self.target.current_log_id() )
    }

    fn files( &self ) -> Vec<(LogId,FILE,LOG)> {
        self.wrap.files( (), self.target.files() )
    }

    fn tail( &self ) -> (LogId,FILE,LOG) {
        self.wrap.tail( (),self.target.tail() )
    }
}

impl<Q,L,LogId,FILE,LOG> LogNavigateLast<RecID<LogId>, FILE, LogId> for Wrapper<Q,L,LogId,FILE,LOG> 
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>,
    L: Logging<LogId,FILE,LOG>
{
    fn last_record( &self ) -> Result<Option<RecID<LogId>>,LoqErr<FILE,LogId>> {
        self.wrap.last_record((), self.target.last_record() )
    }
}

impl<Q,L,LogId,FILE,LOG> LogWriting<RecID<LogId>> for Wrapper<Q,L,LogId,FILE,LOG> 
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>,
    L: Logging<LogId,FILE,LOG>
{
    type FILE = FILE;
    type LogId = LogId;

    fn write( &self, record:&PreparedRecord ) -> Result<RecID<LogId>,LoqErr<Self::FILE,Self::LogId>>
    {
        self.wrap.write( record.clone(), 
        self.target.write(record))
    }
}

impl<Q,L,LogId,FILE,LOG> LogReading for Wrapper<Q,L,LogId,FILE,LOG> 
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>,
    L: Logging<LogId,FILE,LOG>
{
    type RecordId = RecID<LogId>;
    type FILE = FILE;
    type LogId = LogId;

    fn read( &self, record_id: Self::RecordId ) -> 
        Result<PreparedRecord, LoqErr<Self::FILE,Self::LogId>> 
    {
        self.wrap.read(record_id.clone(),self.target.read(record_id))
    }

    fn info( &self, record_id: Self::RecordId ) -> 
        Result<RecordInfo<Self::FILE,Self::LogId>, LoqErr<Self::FILE,Self::LogId>> 
    {
        self.wrap.info(record_id.clone(),self.target.info(record_id))
    }

    fn read_raw_bytes( &self, log_id: Self::LogId, pos: FileOffset, data_consumer:&mut [u8] ) ->
        Result<u64, LoqErr<Self::FILE, Self::LogId>> 
    {
        self.wrap.read_raw_bytes((log_id.clone(), pos.clone()), 
        self.target.read_raw_bytes(log_id, pos, data_consumer))
    }    
}

impl<Q,L,LogId,FILE,LOG> LogNavigationNear for Wrapper<Q,L,LogId,FILE,LOG> 
where
    LogId: LogQueueFileId,
    FILE: Clone+Debug,
    LOG: Clone+Debug,
    Q: LogQueue<RecID<LogId>,LogId,FILE,LOG>,
    L: Logging<LogId,FILE,LOG>
{
    type RecordId = RecID<LogId>;
    type FILE = FILE;
    type LogId = LogId;

    fn next_record( &self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,LoqErr<Self::FILE,Self::LogId>> {
        self.wrap.next_record(record_id.clone(), 
        self.target.next_record(record_id))
    }

    fn previous_record( &self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,LoqErr<Self::FILE,Self::LogId>> {
        self.wrap.previous_record( record_id.clone(),
        self.target.previous_record(record_id))
    }
}


