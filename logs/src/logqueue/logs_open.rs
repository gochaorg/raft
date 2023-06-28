use std::marker::PhantomData;

use crate::{
    logfile::{
        FlatBuff,
        LogFile
    }
};

/// Конфигурация лог файлов, которую можно открыть
trait LogQueueConf {
    type Open;
    type OpenError;

    // Открыть конфигурацию
    fn open( &self ) -> Result<Self::Open, Self::OpenError>;
}

/// Открытые и проверенные лог файлы
trait LogQueueOpenned {    
    type LogFile;
    type LogFiles;

    /// Возвращает список лог файлов
    fn files( &self ) -> Self::LogFiles;

    /// Возвращает актуальный лог файл для записи
    fn tail( &self ) -> Self::LogFile;
}

/// Минимальная конфигурация для открытия логов
struct LogFileQueueConf<BUFF,FILE,ERR,FOpen,FFind,FValidate,FInit>
where 
    BUFF:FlatBuff,
    FOpen: Fn(FILE) -> Result<LogFile<BUFF>, ERR>,
    FFind: Fn() -> Result<Vec<FILE>, ERR>,
    FValidate: Fn(&Vec<(FILE,LogFile<BUFF>)>) -> Result<(FILE,LogFile<BUFF>),ERR>,
    FInit: Fn() -> Result<(FILE,LogFile<BUFF>), ERR>,
{
    /// Поиск лог файлов
    find_files: FFind,

    /// Открытие лог файла
    open_log_file: FOpen,

    /// Валидация открытых лог файлов
    validate: FValidate,

    /// Первичная инициализация
    init: FInit,

    _p: PhantomData<(BUFF,FILE,ERR)>
}

/// Открытые лог файлы
struct LogFilesOpenned<BUFF,FILE>
where
    BUFF:FlatBuff,
{
    /// Список открытых лог файлов
    files: Vec<(FILE,LogFile<BUFF>)>,

    /// Последний актуальный лог файл - имя файла
    tail_file: FILE,

    /// Последний актуальный лог файл
    tail_log: LogFile<BUFF>,

    _p: PhantomData<(BUFF,FILE)>
}

impl<BUFF,FILE> LogQueueOpenned for LogFilesOpenned<BUFF,FILE>
where
    BUFF:FlatBuff,
    FILE:Clone,
{
    type LogFile = (FILE,LogFile<BUFF>);
    type LogFiles = Vec<Self::LogFile>;

    fn files( &self ) -> Self::LogFiles {
        (&self.files).into_iter().map(|i| (i.0.clone(), i.1.clone())).collect()
    }

    fn tail( &self ) -> Self::LogFile {
        ( self.tail_file.clone(), self.tail_log.clone() )
    }
}

impl<BUFF,FILE,ERR,FOpen,FFind,FValidate,FInit> LogQueueConf 
for LogFileQueueConf<BUFF,FILE,ERR,FOpen,FFind,FValidate,FInit> 
where
    BUFF:FlatBuff,
    FOpen: Fn(FILE) -> Result<LogFile<BUFF>, ERR>,
    FFind: Fn() -> Result<Vec<FILE>, ERR>,
    FValidate: Fn(&Vec<(FILE,LogFile<BUFF>)>) -> Result<(FILE,LogFile<BUFF>),ERR>,
    FInit: Fn() -> Result<(FILE,LogFile<BUFF>), ERR>,
    FILE: Clone
{
    type OpenError = ERR;
    type Open = LogFilesOpenned<BUFF,FILE>;

    fn open( &self ) -> Result<Self::Open, Self::OpenError> {
        let found_files = (self.find_files)()?;
        if !found_files.is_empty() {
            let not_validated_open_files = found_files.iter().fold( 
                Ok::<Vec::<(FILE,LogFile<BUFF>)>,ERR>(Vec::<(FILE,LogFile<BUFF>)>::new()), 
                |res,file| {
                res.and_then(|mut res| {
                    let log_file = (self.open_log_file)(file.clone())?;
                    res.push((file.clone(),log_file));
                    Ok(res)
                })
            })?;

            let (tail_file, tail_log) = (self.validate)(&not_validated_open_files)?;

            Ok(LogFilesOpenned{ 
                files: not_validated_open_files, 
                tail_file: tail_file, 
                tail_log: tail_log,
                _p: PhantomData.clone(),
            })
        }else{
            let (tail_file, tail_log) =(self.init)()?;
            Ok(LogFilesOpenned{ 
                files: vec![(tail_file.clone(), tail_log.clone())], 
                tail_file: tail_file, 
                tail_log: tail_log,
                _p: PhantomData.clone(),
            })
        }
    }
}


