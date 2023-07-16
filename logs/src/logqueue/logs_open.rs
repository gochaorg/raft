use std::{marker::PhantomData, path::PathBuf};
use super::{log_seq_verifier::OrderedLogs, find_logs::FsLogFind};

/// Конфигурация лог файлов, которую можно открыть
pub trait LogQueueOpenConf {
    type Open;
    type OpenError;

    // Открыть конфигурацию
    fn open( &self ) -> Result<Self::Open, Self::OpenError>;
}

/// Открытые и проверенные лог файлы
pub trait LogQueueOpenned {    
    type LogFile;
    type LogFiles;

    /// Возвращает список лог файлов
    fn files( &self ) -> Self::LogFiles;

    /// Возвращает актуальный лог файл для записи
    fn tail( &self ) -> Self::LogFile;
}

/// Поиск файлов логов
pub trait FindFiles<FOUND,ERR> {
    fn find_files( &self ) -> Result<Vec<FOUND>,ERR>;
}

/// Открытие лог файла
pub trait OpenLogFile<FILE,LOG,ERR> {
    fn open_log_file( &self, file:FILE ) -> Result<LOG, ERR>;
}

/// Валидация логов
pub trait ValidateLogFiles<LogFile,ERR> 
where 
    LogFile: Clone
{
    fn validate( &self, log_files: &Vec<LogFile> ) -> Result<OrderedLogs<LogFile>,ERR>;
}

/// Инициализация первого лог файла
pub trait InitializeFirstLog<LogFile,ERR>
where
    LogFile: Clone
{
    fn initialize_first_log( &self ) -> Result<LogFile, ERR>;
}

/// Минимальная конфигурация для открытия логов
pub struct LogFileQueueConf<LOG,FILE,ERR,FOpen,FFind,FValidate,FInit>
where 
    LOG:Clone,
    FILE:Clone,
    FOpen: OpenLogFile<FILE,LOG,ERR>,
    FFind: FindFiles<FILE,ERR>,
    FValidate: ValidateLogFiles<(FILE,LOG),ERR>,
    FInit: InitializeFirstLog<(FILE,LOG),ERR>,
{
    /// Поиск лог файлов
    pub find_files: FFind,

    /// Открытие лог файла
    pub open_log_file: FOpen,

    /// Валидация открытых лог файлов
    pub validate: FValidate,

    /// Первичная инициализация
    pub init: FInit,

    pub _p : PhantomData<(LOG,FILE,ERR)>
}

/// Открытые лог файлы
pub struct LogFilesOpenned<LOG,FILE>
where
    LOG:Clone,
{
    /// Список открытых лог файлов
    files: Vec<(FILE,LOG)>,

    /// Последний актуальный лог файл - имя файла
    tail_file: FILE,

    /// Последний актуальный лог файл
    tail_log: LOG,

    _p: PhantomData<(LOG,FILE)>
}

impl<LOG,FILE> LogQueueOpenned for LogFilesOpenned<LOG,FILE>
where
    LOG:Clone,
    FILE:Clone,
{
    type LogFile = (FILE,LOG);
    type LogFiles = Vec<Self::LogFile>;

    fn files( &self ) -> Self::LogFiles {
        (&self.files).into_iter().map(|i| (i.0.clone(), i.1.clone())).collect()
    }

    fn tail( &self ) -> Self::LogFile {
        ( self.tail_file.clone(), self.tail_log.clone() )
    }
}

impl<LOG,FILE,ERR,FOpen,FFind,FValidate,FInit> LogQueueOpenConf 
for LogFileQueueConf<LOG,FILE,ERR,FOpen,FFind,FValidate,FInit> 
where
    FILE: Clone,
    LOG:Clone,
    FOpen: OpenLogFile<FILE,LOG,ERR>,
    FFind: FindFiles<FILE,ERR>,
    FValidate: ValidateLogFiles<(FILE,LOG),ERR>,
    FInit: InitializeFirstLog<(FILE,LOG),ERR>,
{
    type OpenError = ERR;
    type Open = LogFilesOpenned<LOG,FILE>;

    fn open( &self ) -> Result<Self::Open, Self::OpenError> {
        let found_files = self.find_files.find_files()?;
        if !found_files.is_empty() {
            let not_validated_open_files = found_files.iter().fold( 
                Ok::<Vec::<(FILE,LOG)>,ERR>(Vec::<(FILE,LOG)>::new()), 
                |res,file| {
                res.and_then(|mut res| {
                    let log_file = 
                        self.open_log_file.open_log_file(file.clone())?;
                    res.push((file.clone(),log_file));
                    Ok(res)
                })
            })?;

            let validated_order = 
                self.validate.validate(&not_validated_open_files)?;

            Ok(LogFilesOpenned{ 
                files: not_validated_open_files, 
                tail_file: validated_order.tail.0, 
                tail_log: validated_order.tail.1,
                _p: PhantomData.clone(),
            })
        }else{
            let (tail_file, tail_log) = self.init.initialize_first_log()?;  // (self.init)()?;
            Ok(LogFilesOpenned{ 
                files: vec![(tail_file.clone(), tail_log.clone())], 
                tail_file: tail_file, 
                tail_log: tail_log,
                _p: PhantomData.clone(),
            })
        }
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use std::marker::PhantomData;
    #[allow(unused_imports)]
    use super::super::log_id::*;
    #[allow(unused_imports)]
    use super::super::log_seq_verifier::*;
    #[allow(unused_imports)]
    use super::super::log_seq_verifier::test::*;
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn open_logs() {
        let id0 = IdTest::new(None);
        let id1 = IdTest::new(Some(id0.id()));
        let id2 = IdTest::new(Some(id1.id()));
        let id3 = IdTest::new(Some(id2.id()));

        struct FindFilesStub(Vec<IdTest>);
        impl FindFiles<IdTest,String> for FindFilesStub {
            fn find_files( &self ) -> Result<Vec<IdTest>,String> {
                Ok(self.0.clone())
            }
        }

        struct OpenLogFileStub;
        impl OpenLogFile<IdTest,IdTest,String> for OpenLogFileStub {
            fn open_log_file( &self, file:IdTest ) -> Result<IdTest, String> {
                Ok(file.clone())
            }
        }

        struct ValidateStub( OrderedLogs<(IdTest,IdTest)> );
        impl ValidateLogFiles<(IdTest,IdTest),String> for ValidateStub {
            fn validate( &self, _log_files: &Vec<(IdTest,IdTest)> ) -> Result<OrderedLogs<(IdTest,IdTest)>,String> {
                Ok( self.0.clone() )
            }
        }

        struct InitializeStub(IdTest);
        impl InitializeFirstLog<(IdTest,IdTest),String> for InitializeStub {
            fn initialize_first_log( &self ) -> Result<(IdTest,IdTest), String> {
                Ok( (self.0.clone(), self.0.clone()) )
            }
        }

        let queue_conf: LogFileQueueConf<IdTest,IdTest,String,_,_,_,_> = 
        LogFileQueueConf {
            find_files: FindFilesStub(vec![id0.clone(), id1.clone(), id2.clone(), id3.clone()]),
            open_log_file: OpenLogFileStub,
            validate: ValidateStub(
                OrderedLogs {
                    files: vec![
                    (id1.clone(),id1.clone()), 
                    (id2.clone(),id2.clone()), 
                    (id3.clone(),id3.clone()),
                    (id0.clone(),id0.clone()), 
                    ],
                    tail: (id3.clone(),id3.clone())
                }
            ),
            init: InitializeStub(id0.clone()),
            _p: PhantomData.clone()
        };

        let open_files:LogFilesOpenned<IdTest,IdTest> = queue_conf.open().unwrap();
        println!("tail {}", open_files.tail().0);
        for (f,_) in &open_files.files() {
            println!("log {}", f);
        }

    }
}

impl<ERR> FindFiles<PathBuf,ERR> for FsLogFind {
    fn find_files( &self ) -> Result<Vec<PathBuf>,ERR> {
        self.to_conf::<ERR>()()
    }
}