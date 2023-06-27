#[allow(unused_imports)]
use std::{fs::File, path::PathBuf, sync::{Arc, RwLock, Mutex, PoisonError, MutexGuard, RwLockWriteGuard}, rc::Rc};

#[allow(unused_imports)]
use crate::{
    perf::{Tracker, Counters}, 
    logfile::{LogFile, FlatBuff, block::BlockId, LogErr}, 
    logqueue::new_file::NewFile, bbuff::absbuff::FileBuff
};

use super::{new_file::{NewFileGenerator, NewFileGeneratorErr}, find_logs::FsLogFind};

#[allow(dead_code)]
pub struct FsLogQueueConf<'a, FNewFile> 
where
    FNewFile: Fn(PathBuf) -> Result<File,std::io::Error>
{
    new_file_generator: Arc<RwLock<NewFileGenerator<'a, FNewFile>>>,
    find_files: FsLogFind,
    tracker: Option<Arc<Tracker>>,
}

impl<'a,FNewFile> FsLogQueueConf<'a,FNewFile> 
where
    FNewFile: Fn(PathBuf) -> Result<File,std::io::Error>
{
    /// Первичная инициализация
    fn open_first_log( &self, file: Rc<Mutex<File>> ) -> Result<LogFile<FileBuff>, FsLogOpenError> {
        let f = file.lock()?;
        let f_clone = f.try_clone()?;
        let f_buff = FileBuff {
            file: Arc::new(RwLock::new(f_clone)),
            tracker: Arc::new(Tracker::new())
        };
        let lf = LogFile::new(f_buff)?;
        Ok(lf)
    }

    /// Открытие ранее существовавшего лог файла
    fn open_log( &self, file: &PathBuf ) -> Result<LogFile<FileBuff>, FsLogOpenError> {
        todo!()
    }

    /// Открытие лог файлов
    /// Аргументы
    ///   - self - ссылка на конфигурацию
    ///   - init - первичная инициализация логов
    ///   - check - валидация логов
    pub fn open<FInit, FCheck>(
        &'a self,
        mut init: FInit,
        mut check: FCheck
    ) -> Result<FsLogQueueOpenned<'a,FNewFile>, FsLogOpenError>
    where
        FInit: FnMut(&mut LogFile<FileBuff>) -> Result<BlockId,FsLogOpenError>,
        FCheck: FnMut(&Vec<Arc<RwLock<LogQueueFile>>>) -> 
            Result<Arc<RwLock<LogQueueFile>>, FsLogOpenError>
    {
        // поиск логов
        let mut log_files: Vec<Arc<RwLock<LogQueueFile>>> = vec![];
        
        for found in &self.find_files {
            match FsLogQueueConf::<'a,FNewFile>::open_log(self, &found) {
                Ok(log_file) => {
                    log_files.push(
                        Arc::new(RwLock::new(
                            LogQueueFile {
                                log_file: log_file,
                                path: found.clone()
                            }
                        ))
                    );
                },
                Err(err) => { return Err(err) }
            }
        }

        if ! log_files.is_empty() {
            match check( &log_files ) {
                Err(err) => { return Err(err) }
                Ok(tail_log) => {
                    return Ok( FsLogQueueOpenned {
                        new_file_generator: self.new_file_generator.clone(),
                        log_files: log_files,
                        tail_log: tail_log
                    })
                }
            }
        } else {
            let mut nf_generator = self.new_file_generator.write()?;
            let NewFile { path, file }  = nf_generator.generate()?;
            let mut log_file = self.open_first_log(file)?;

            init(&mut log_file)?;

            let log_file_first = Arc::new(RwLock::new(LogQueueFile { log_file: log_file, path: path }));

            log_files.push(log_file_first.clone());
            Ok( FsLogQueueOpenned { 
                new_file_generator: self.new_file_generator.clone(), 
                log_files: log_files,
                tail_log: log_file_first
            })
        }        
    }
}

/// Описание ошибок при открытии лога
#[derive(Debug,Clone)]
pub enum FsLogOpenError {
    IOError { message:String },
    LogError( LogErr ),
    MutexError { message:String },
    LockWriteError { message:String },
    NewFileGeneratorErr( NewFileGeneratorErr )
}

impl From<NewFileGeneratorErr> for FsLogOpenError {
    fn from(value: NewFileGeneratorErr) -> Self {
        FsLogOpenError::NewFileGeneratorErr(value.clone())
    }
}

impl<A> From<PoisonError<RwLockWriteGuard<'_, A>>> for FsLogOpenError {
    fn from(value: PoisonError<RwLockWriteGuard<'_, A>>) -> Self {
        FsLogOpenError::LockWriteError { message: value.to_string() }
    }
}

impl<A> From<PoisonError<MutexGuard<'_, A>>> for FsLogOpenError {
    fn from(value: PoisonError<MutexGuard<'_, A>>) -> Self {
        FsLogOpenError::MutexError { message: value.to_string() }
    }
}

impl From<std::io::Error> for FsLogOpenError {
    fn from(value: std::io::Error) -> Self {
        FsLogOpenError::IOError { message: value.to_string() }
    }
}

impl From<LogErr> for FsLogOpenError {
    fn from(value: LogErr) -> Self {
        FsLogOpenError::LogError(value)
    }
}

/// Описывает очередь открытых лог файлов
pub struct FsLogQueueOpenned<'a,FNewFile>
where
    FNewFile: Fn(PathBuf) -> Result<File,std::io::Error>
{
    new_file_generator: Arc<RwLock<NewFileGenerator<'a, FNewFile>>>,
    log_files: Vec<Arc<RwLock<LogQueueFile>>>,
    tail_log: Arc<RwLock<LogQueueFile>>,
}

/// Описывает лог файл в очереди
pub struct LogQueueFile {
    /// лог файл
    pub log_file: LogFile<FileBuff>,

    /// Путь к лог файлу
    pub path: PathBuf
}

