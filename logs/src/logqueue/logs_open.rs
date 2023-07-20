use std::{path::PathBuf, fmt::Debug};
use crate::{logfile::LogFile, bbuff::absbuff::FileBuff};
use super::{log_seq_verifier::OrderedLogs, find_logs::FsLogFind, LoqErr, LogQueueFileNumID, validate_sequence, SeqValidateOp, IdOf};
use super::log_id::*;

/// Поиск файлов логов
pub trait FindFiles<FILE,LogId>
where
    FILE: Clone+Debug,
    LogId: Clone+Debug,
{
    fn find_files( &self ) -> Result<Vec<FILE>,LoqErr<FILE,LogId>>;
}

impl<LogId> FindFiles<PathBuf,LogId> for FsLogFind 
where
    LogId: Clone+Debug,
{
    fn find_files( &self ) -> Result<Vec<PathBuf>,LoqErr<PathBuf,LogId>> {
        self.to_conf::<LoqErr<PathBuf,LogId>>()()
    }
}
/// Открытие лог файла
pub trait OpenLogFile<FILE,LOG,LogId> 
where
    LOG: Clone,
    FILE: Clone+Debug,
    LogId: Clone+Debug,
{
    fn open_log_file( &self, file:FILE ) -> Result<LOG, LoqErr<FILE,LogId>>;
}

#[derive(Clone,Debug)]
pub struct LogQueueFileNumIDOpen;

impl OpenLogFile<PathBuf,LogFile<FileBuff>,LogQueueFileNumID> for LogQueueFileNumIDOpen {
    fn open_log_file( &self, path:PathBuf ) -> Result<LogFile<FileBuff>, LoqErr<PathBuf,LogQueueFileNumID>> {
        let buff = 
        FileBuff::open_read_write(path.clone()).map_err(|err| LoqErr::OpenFileBuff { 
            file: path.clone(), 
            error: err
        })?;

        let log = LogFile::new(buff)
        .map_err(|err| LoqErr::OpenLog { 
            file: path.clone(), 
            error: err
        })?;

        Ok(log)
    }
}

/// Валидация логов
pub trait ValidateLogFiles<FILE,LOG,LogId> 
where 
    FILE: Clone + Debug,
    LogId: Clone + Debug,
    LOG: Clone,
{
    fn validate( &self, log_files: &Vec<(FILE,LOG)> ) -> Result<OrderedLogs<(FILE,LOG)>,LoqErr<FILE,LogId>>;
}

#[derive(Clone,Debug)]
pub struct ValidateStub;
impl<FILE> ValidateLogFiles<FILE,LogFile<FileBuff>,LogQueueFileNumID> for ValidateStub 
where
    FILE: Clone+Debug
{
    fn validate( &self, log_files: &Vec<(FILE,LogFile<FileBuff>)> ) -> Result<crate::logqueue::OrderedLogs<(FILE,LogFile<FileBuff>)>,LoqErr<FILE,LogQueueFileNumID>> {
        validate_sequence::<FILE,LogFile<FileBuff>,LogQueueFileNumID>(log_files)
    }
}

impl<FILE> SeqValidateOp<FILE, LogFile<FileBuff>, LogQueueFileNumID> for (FILE, LogFile<FileBuff>) 
where
    FILE: Clone+Debug
{
    fn items_count(a:&(FILE,LogFile<FileBuff>)) -> Result<u32,LoqErr<FILE,LogQueueFileNumID>> {
        a.1.count().map_err(|e| LoqErr::LogCountFail { file: a.0.clone(), error: e })
    }
}

impl<FILE> IdOf<FILE, LogFile<FileBuff>, LogQueueFileNumID> for (FILE, LogFile<FileBuff>) 
where
    FILE: Clone+Debug
{
    fn id_of(a:&(FILE,LogFile<FileBuff>)) -> Result<LogQueueFileNumID,LoqErr<FILE,LogQueueFileNumID>> {
        let (filename,log) = a;
        Ok(LogQueueFileNumID::read(filename, log)?)
    }
}
