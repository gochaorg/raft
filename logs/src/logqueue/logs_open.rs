use std::fs::OpenOptions;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{path::PathBuf, fmt::Debug};
use crate::{logfile::LogFile, bbuff::absbuff::FileBuff};
use super::new_file::NewFileGenerator;
use path_template::PathTemplateParser;
use super::{log_seq_verifier::OrderedLogs, find_logs::FsLogFind, LoqErr, LogQueueFileNumID, validate_sequence, SeqValidateOp, IdOf};
use super::{log_id::*, NewLogFile};
use log::error;

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
pub trait OpenLogFile<FILE,LOG,LogId> : Clone
where
    LOG: Clone,
    FILE: Clone+Debug,
    LogId: Clone+Debug,
{
    fn open_log_file( &self, file:FILE ) -> Result<LOG, LoqErr<FILE,LogId>>;
}

/// Вспомогательная структура для открытия логов
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
    fn validate( &self, log_files: &Vec<(FILE,LOG)> ) -> Result<OrderedLogs<LogId,(FILE,LOG)>,LoqErr<FILE,LogId>>;
}

/// Вспомогательная структура для валидации логов
#[derive(Clone,Debug)]
pub struct ValidateStub;
impl<FILE> ValidateLogFiles<FILE,LogFile<FileBuff>,LogQueueFileNumID> for ValidateStub 
where
    FILE: Clone+Debug
{
    fn validate( &self, log_files: &Vec<(FILE,LogFile<FileBuff>)> ) -> Result<crate::logqueue::OrderedLogs<LogQueueFileNumID,(FILE,LogFile<FileBuff>)>,LoqErr<FILE,LogQueueFileNumID>> {
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

#[derive(Clone)]
pub struct NewFileStub<F,LogId: LogQueueFileId>(pub F)
where F: FnMut() -> Result<PathBuf,LoqErr<PathBuf,LogId>>;

impl<F,LogId> NewLogFile<PathBuf,LogId> for NewFileStub<F,LogId> 
where F: FnMut() -> Result<PathBuf,LoqErr<PathBuf,LogId>> + Clone,
LogId: LogQueueFileId
{
    fn new_log_file(&mut self) -> Result<PathBuf,crate::logqueue::LoqErr<PathBuf,LogId>> {
        (self.0)()
    }
}

/// Создает функцию генерации нового файла
/// 
/// Аргументы
/// ====================
/// - `root` - переменная указыавющая на корневой каталог логов
/// - `template` - шаблон
/// 
/// Пример шаблона
/// ----------------------
/// 
/// ```
/// "${root}/${time:local:yyyy-mm-ddThh-mi-ss}-${rnd:5}.binlog"
/// ```
/// 
/// - `${....}` - некие переменные которые могут содержать значения
/// - `${root}` - это внешняя переменная
/// - `${time:...}` - встроенная переменаая, задает текущую дату, формат даты описан в [DateFormat]
/// - `${rnd:5}` - случайны набор из 5 букв, число 5 - указывает на кол-во букв и может быть заменено на другое число
/// - `${env:...}` - в качестве значения - потенциально опасно
pub fn path_template<LogId: LogQueueFileId>( root:&str, template:&str ) 
    -> Result<impl NewLogFile<PathBuf,LogId>, LoqErr<PathBuf,LogId>>
{
    let path_tmpl = PathTemplateParser::default()
        .with_variable("root", root)
        .parse(template)
        .map_err(|err| 
            LoqErr::<PathBuf,LogId>::CantParsePathTemplate { 
                error: err, 
                template: template.to_string() })?;

    let log_file_new = 
        NewFileGenerator {
            open: |path| OpenOptions::new().create(true).read(true).write(true).open(path),
            path_template: path_tmpl,
            max_duration: Some(Duration::from_secs(5)),
            max_attemps: Some(5),
            throttling: Some(Duration::from_millis(100))
        };
    let log_file_new: Arc<RwLock<NewFileGenerator<'_, _>>> = Arc::new(RwLock::new(log_file_new));

    Ok( move || {
        let mut generator = 
            log_file_new.write().map_err(|err| 
                LoqErr::<PathBuf,LogId>::CantCaptureWriteLock { error: err.to_string() }
            )?;

        let new_file = generator.generate()
            .map_err(|e| 
                {
                    error!("generate new file from {from:?} fail with {err:?}", from=&generator, err=&e);
                    LoqErr::<PathBuf,LogId>::CantGenerateNewFile { error: e, }
                }
            )?;
        let path = new_file.path.clone();
        Ok(path)
    } ).map(|r| NewFileStub(r))
}

fn path_template2impl<LogId: Clone + Debug,F>( template:&str, template_vars:F ) 
    -> Result<impl  FnMut() -> Result<PathBuf,LoqErr<PathBuf,LogId>> + Clone, LoqErr<PathBuf,LogId>>
where
    F: for <'a> Fn(PathTemplateParser<'a>) -> PathTemplateParser<'a>
{
    let path_tmpl = template_vars(PathTemplateParser::default())
        .parse(template)
        .map_err(|err| 
            LoqErr::<PathBuf,LogId>::CantParsePathTemplate { 
                error: err, 
                template: template.to_string() })?;

    let log_file_new = 
        NewFileGenerator {
            open: |path| OpenOptions::new().create(true).read(true).write(true).open(path),
            path_template: path_tmpl,
            max_duration: Some(Duration::from_secs(5)),
            max_attemps: Some(5),
            throttling: Some(Duration::from_millis(100))
        };
    let log_file_new: Arc<RwLock<NewFileGenerator<'_, _>>> = Arc::new(RwLock::new(log_file_new));

    Ok( move || {        
        let mut generator = 
            log_file_new.write().map_err(|err| 
                LoqErr::<PathBuf,LogId>::CantCaptureWriteLock { error: err.to_string() }
            )?;

        let new_file = generator.generate()
            .map_err(|e| 
                {
                    error!("generate new file from {from:?} fail with {err:?}", from=&generator, err=&e);
                    LoqErr::<PathBuf,LogId>::CantGenerateNewFile { error: e }
                }
            )?;

        let path = new_file.path.clone();
        Ok(path)
    } )
}

pub fn path_template2<LogId: LogQueueFileId,F>( template:&str, template_vars:F ) 
    -> Result<impl NewLogFile<PathBuf,LogId>, LoqErr<PathBuf,LogId>>
where
    F: for <'a> Fn(PathTemplateParser<'a>) -> PathTemplateParser<'a>
{
    path_template2impl(template, template_vars).map(|r| NewFileStub(r))
}

