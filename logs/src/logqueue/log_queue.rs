use std::cell::RefCell;
use std::collections::HashMap;

use core::fmt::Debug;
use std::marker::PhantomData;

use crate::logfile::{LogFile, FlatBuff};

use super::{log_id::*, LoqErr, FindFiles, OpenLogFile, ValidateLogFiles};

/// Очередь логов
pub trait LogFileQueue<LogId,FILE,LOG>
where 
    LogId: Clone + Debug, 
    FILE: Clone + Debug 
{
    /// Переключение лога
    fn switch( &mut self ) -> Result<(),LoqErr<FILE,LogId>>;

    /// Поиск лог файла по его ID
    /// 
    /// Аргументы
    /// ==============
    /// - `id` идентификатор
    /// 
    /// Результат
    /// =============
    /// лог
    fn find_log( &self, id:LogId ) -> Result<Option<(FILE,LOG)>,LoqErr<FILE,LogId>>;

    /// Получение ID лога, относительно указаного
    /// 
    /// Аргументы
    /// ==============
    /// - `id` идентификатор
    /// - `offset` смещение
    ///    - `0` - возвращает сам аргумент `id`
    ///    - `-1` - предшедствующий указаному
    ///    - `1` - следующий за указаным
    /// 
    /// Результат
    /// =============
    /// идентификатор относительно указанного
    fn offset_log_id( &self, id:LogId, offset: i64) -> Result<Option<LogId>, LoqErr<FILE,LogId>>;

    /// Чтение идентификатора текущего лога
    fn current_log_id( &self ) -> Result<LogId, LoqErr<FILE,LogId>>;

    /// Чтение лог файлов
    fn files( &self ) -> Vec<(FILE,LOG)>;

    /// Работа с актуальным лог файлом
    fn tail( &self ) -> (FILE,LOG);

}

/// Очередь логов
pub struct LogFileQueueImpl<LogId,FILE,BUFF,FNewFile,FOpen> 
where
    BUFF: FlatBuff,
    FILE: Clone + Debug,
    LogId: LogQueueFileId,
    FNewFile: FnMut() -> Result<FILE,LoqErr<FILE,LogId>> + Clone,
    FOpen: OpenLogFile<FILE,LogFile<BUFF>,LogId>
{
    /// Список файлов
    files: Vec<(FILE,LogFile<BUFF>)>,

    /// Актуальный лог
    tail: (FILE,LogFile<BUFF>),

    /// Генерация нового пустого файла
    new_file: FNewFile,

    /// Открытие файла
    open_file: FOpen,

    /// текущий id лога
    current_log_id: RefCell<Option<LogId>>,

    /// Кеш ид - лог файл
    log_id_to_log: RefCell<Option<HashMap<LogId,(FILE,LogFile<BUFF>)>>>,

    /// Очередность id логов
    log_id_order: RefCell<Option<Vec<LogId>>>,
}

impl<LogId,FILE,BUFF,FNewFile,FOpen> LogFileQueueImpl<LogId,FILE,BUFF,FNewFile,FOpen> 
where
    BUFF: FlatBuff,
    FILE: Clone + Debug,
    LogId: LogQueueFileId,
    FNewFile: FnMut() -> Result<FILE,LoqErr<FILE,LogId>> + Clone,
    FOpen: OpenLogFile<FILE,LogFile<BUFF>,LogId>
{
    /// Конструктор
    /// 
    /// Аргументы
    /// ===========
    /// - `files` - упорядоченная последовательность (должны быть) логов
    /// - `tail` - актуальный лог файл
    /// - `switching` - переключение лог файла
    /// - `id_of` - получение идентификатора лог файла
    pub fn new(
        files: Vec<(FILE,LogFile<BUFF>)>,
        tail: (FILE,LogFile<BUFF>),
        new_file: FNewFile,
        open_file: FOpen,
    ) -> Self {
        Self { 
            files: files, 
            tail: tail, 
            new_file: new_file, 
            open_file: open_file,
            current_log_id: RefCell::new(None),
            log_id_to_log: RefCell::new(None),
            log_id_order: RefCell::new(None),            
        }
    }

    /// Сброс кеша
    pub fn invalidate_cache( &self ) {
        let mut r = self.log_id_to_log.borrow_mut();
        *r = None;

        let mut r = self.log_id_order.borrow_mut();
        *r = None;

        let mut r = self.current_log_id.borrow_mut();
        *r = None;
    }

    // пересоздание кеша, если необходимо и обход кеша
    fn log_id_map_cache_read<R,F>( &self, default:R, consume:F ) -> Result<R,LoqErr<FILE,LogId>>
    where
        R: Sized,
        F: for <'a> Fn(&'a HashMap<LogId,(FILE,LogFile<BUFF>)>) -> R,
    {
        let mut cache_opt = self.log_id_to_log.borrow_mut();
        if cache_opt.is_none() {
            let mut cache : HashMap<LogId,(FILE,LogFile<BUFF>)> = HashMap::new();
            for file_log in &self.files {
                let found_id = LogId::read(&file_log.0, &file_log.1)?;
                cache.insert(found_id, file_log.clone());
            }
            *cache_opt = Some(cache);
        }

        Ok(cache_opt.as_ref().map(|x| {  
            consume(x)
        }).unwrap_or(default))
    }

    // пересоздание кеша, если необходимо и обход кеша
    fn log_order_cache_read<R,F>( &self, default:R, consume:F ) -> Result<R,LoqErr<FILE,LogId>>
    where
        R: Sized,
        F: for <'a> Fn(&'a Vec<LogId>) -> R
    {
        let mut cache_opt = self.log_id_order.borrow_mut();
        if cache_opt.is_none() {
            let mut cache: Vec<LogId> = Vec::new();
            for file_log in &self.files {
                let id = LogId::read(&file_log.0, &file_log.1)?;
                cache.push(id);
            }
            *cache_opt = Some(cache);
        }

        Ok(cache_opt.as_ref().map(|x| consume(x)).unwrap_or(default))
    }

    /// Чтение id текущего лог файла
    #[allow(unused)]
    fn current_log_id_read<R,F>( &self, consume:F ) -> Result<R,LoqErr<FILE,LogId>>
    where
        R: Sized,
        F: Fn(LogId) -> R 
    {
        let mut cache_opt = self.current_log_id.borrow_mut();
        if cache_opt.is_none() {            
            let id = LogId::read(&self.tail.0, &self.tail.1)?;
            *cache_opt = Some(id);
        }
        Ok(consume(cache_opt.unwrap()))
    }
}

impl<LogId,FILE,BUFF,FNewFile,FOpen> LogFileQueue<LogId,FILE,LogFile<BUFF>>
for LogFileQueueImpl<LogId,FILE,BUFF,FNewFile,FOpen>
where
    BUFF: FlatBuff,
    FILE: Clone + Debug,
    LogId: LogQueueFileId,
    FNewFile: FnMut() -> Result<FILE,LoqErr<FILE,LogId>> + Clone,
    FOpen: OpenLogFile<FILE,LogFile<BUFF>,LogId>
{
    fn switch( &mut self ) -> Result<(),LoqErr<FILE,LogId>> {
        let file_name = (self.new_file)()?;
        let mut log_file = self.open_file.open_log_file(file_name.clone())?;
        let new_log_id = self.current_log_id_read(|id| LogId::new(Some(id.id())))?;
        new_log_id.write(&file_name, &mut log_file)?;
        self.invalidate_cache();
        self.files.push( self.tail.clone() );
        self.tail = (file_name,log_file);
        (*self.current_log_id.borrow_mut()) = Some(new_log_id);
        Ok(())
    }

    fn find_log( &self, id:LogId ) -> Result<Option<(FILE,LogFile<BUFF>)>,LoqErr<FILE,LogId>> {
        self.log_id_map_cache_read(
            None, 
            |cache| {
                cache.get(&id).map(|i|i.clone())
            }
        )
    }

    fn current_log_id( &self ) -> Result<LogId, LoqErr<FILE,LogId>> {
        self.current_log_id_read(|id| id.clone())
    }

    fn offset_log_id( &self, id:LogId, offset: i64) -> Result<Option<LogId>, LoqErr<FILE,LogId>> {
        if offset == 0i64 { return Ok(Some(id.clone())); }

        let idx = self.log_order_cache_read(None, |ids| {
            ids.iter().enumerate()
                .find(|(_,found_id)| id == **found_id )
                .map(|(idx,_)| idx)
        })?;

        if idx.is_none() { return Ok(None); }
        let idx = idx.unwrap();

        let target = (idx as i64) + offset;
        if target < 0 { return Ok(None); }
        let target = target as usize;

        self.log_order_cache_read(None, |ids| {
            if target >= ids.len() {
                None
            } else {
                Some(ids[target].clone())
            }
        })
    }

    fn files( &self ) -> Vec<(FILE,LogFile<BUFF>)> {
        self.files.clone()
    }

    fn tail( &self ) -> (FILE,LogFile<BUFF>) {
        self.tail.clone()
    }
}

/// Конфигурация логов
pub struct LogQueueConf<
    LogId, FILE, BUFF,
    FFind, FOpen, FValidate, FNewFile,
    >
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff,
    FFind: FindFiles<FILE,LogId>,
    FOpen: OpenLogFile<FILE,LogFile<BUFF>,LogId> + Clone,
    FValidate: ValidateLogFiles<FILE,LogFile<BUFF>,LogId>,
    FNewFile: FnMut() -> Result<FILE,LoqErr<FILE,LogId>> + Clone,
{
    /// Поиск лог файлов
    pub find_files: FFind,

    /// Открытие лог файла
    pub open_log_file: FOpen,

    /// Валидация открытых лог файлов
    pub validate: FValidate,

    /// Создание пустого лог файла
    pub new_file: FNewFile,

    pub _p: PhantomData<BUFF>
}


impl<LogId,FILE,BUFF,FFind,FOpen,FValidate,FNewFile> 
    LogQueueConf<LogId,FILE,BUFF,FFind,FOpen,FValidate,FNewFile> 
where
    LogId: LogQueueFileId,
    FILE: Clone + Debug,
    BUFF: FlatBuff,
    FFind: FindFiles<FILE,LogId>,
    FOpen: OpenLogFile<FILE,LogFile<BUFF>,LogId> + Clone,
    FValidate: ValidateLogFiles<FILE,LogFile<BUFF>,LogId>,
    FNewFile: FnMut() -> Result<FILE,LoqErr<FILE,LogId>> + Clone,
{
    /// Открытие логов
    pub fn open( &self ) -> 
    Result<LogFileQueueImpl<LogId,FILE,BUFF,FNewFile,FOpen>,LoqErr<FILE,LogId>> 
    {
        let found_files = self.find_files.find_files()?;
        if !found_files.is_empty() {
            let not_validated_open_files = found_files.iter().fold( 
                Ok::<Vec::<(FILE,LogFile<BUFF>)>,LoqErr<FILE,LogId>>(Vec::<(FILE,LogFile<BUFF>)>::new()), 
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

            let queue = 
            LogFileQueueImpl::new(
                not_validated_open_files, 
                validated_order.tail, 
                self.new_file.clone(), 
                self.open_log_file.clone()
            );

            Ok(queue)
        } else {
            let file_name = (self.new_file.clone())()?;
            let mut log_file = 
                self.open_log_file.open_log_file(file_name.clone())?;
            let id = LogId::new(None);
            id.write(&file_name, &mut log_file)?;

            let queue = 
            LogFileQueueImpl::new(
                vec![(file_name.clone(), log_file.clone())], 
                (file_name,log_file), 
                self.new_file.clone(), 
                self.open_log_file.clone()
            );

            Ok(queue)
        }
    }
}

#[cfg(test)]
mod full_test {
    #[allow(unused)]
    use std::any::{TypeId, type_name};
    use std::fs::*;
    use std::marker::PhantomData;
    use std::path::PathBuf;
    use std::env::*;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    struct Prepared {
        log_dir_root: PathBuf,
    }

    fn prepare() -> Prepared {
        let target = current_dir().unwrap().join("target");
        if ! target.is_dir() { panic!("target dir not found!") };

        let full_test = target.join("test").join("full_test");
        if full_test.exists() {
            remove_dir_all(full_test.clone()).expect("can't remove full_test dir");
        }
        create_dir_all(full_test.clone()).expect("can't create full_test dir");

        println!("test preprared");

        Prepared {
            log_dir_root: full_test.clone()
        }
    }

    use crate::bbuff::absbuff::FileBuff;
    use crate::logfile::LogFile;
    use crate::logqueue::new_file::NewFileGenerator;
    use crate::logqueue::path_tmpl::PathTemplateParser;

    #[allow(unused)]
    use crate::logqueue::{log_id::*, LogFileQueueConf, LoqErr, validate_sequence, SeqValidateOp, IdOf, 
        LogQueueConf, LogFileQueue, OpenLogFile, ValidateLogFiles, InitializeFirstLog, LogWriting, LogNavigateLast
    };
    use crate::logqueue::find_logs::FsLogFind;

    fn open_file( path:PathBuf ) -> Result<LogFile<FileBuff>,LoqErr<PathBuf,LogQueueFileNumID>> {
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

    fn id_of( a:&(PathBuf,LogFile<FileBuff>) ) -> Result<LogQueueFileNumID,LoqErr<PathBuf,LogQueueFileNumID>> {
        let (filename,log) = a;
        Ok(LogQueueFileNumID::read(filename, log)?)
    }

    #[test]
    fn do_test() {
        let prepared = prepare();

        println!("run test");

        let fs_log_find = 
            FsLogFind::new( 
                prepared.log_dir_root.to_str().unwrap(), 
                "*.binlog", 
                true ).unwrap();

        let path_tmpl_parser = PathTemplateParser::default();
        let path_tmpl = path_tmpl_parser.parse(
            &format!("{root}/{name}",
            root = prepared.log_dir_root.to_str().unwrap(),
            name = "${time:local:yyyy-mm-ddThh-mi-ss}-${rnd:5}.binlog"
        )).unwrap();

        let log_file_new = 
            NewFileGenerator {
                open: |path| OpenOptions::new().create(true).read(true).write(true).open(path),
                path_template: path_tmpl,
                max_duration: Some(Duration::from_secs(5)),
                max_attemps: Some(5),
                throttling: Some(Duration::from_millis(100))
            };
        let log_file_new = Arc::new(RwLock::new(log_file_new));

        #[derive(Clone,Debug)]
        struct OpenLogFileStub;
        impl OpenLogFile<PathBuf,LogFile<FileBuff>,LogQueueFileNumID> for OpenLogFileStub {
            fn open_log_file( &self, file:PathBuf ) -> Result<LogFile<FileBuff>, LoqErr<PathBuf,LogQueueFileNumID>> {
                open_file(file)
            }
        }

        #[derive(Clone,Debug)]
        struct ValidateStub;
        impl ValidateLogFiles<PathBuf,LogFile<FileBuff>,LogQueueFileNumID> for ValidateStub {
            fn validate( &self, log_files: &Vec<(PathBuf,LogFile<FileBuff>)> ) -> Result<crate::logqueue::OrderedLogs<(PathBuf,LogFile<FileBuff>)>,LoqErr<PathBuf,LogQueueFileNumID>> {
                validate_sequence::<PathBuf,LogFile<FileBuff>,LogQueueFileNumID>(log_files)
            }
        }

        impl SeqValidateOp<PathBuf, LogFile<FileBuff>, LogQueueFileNumID> for (PathBuf, LogFile<FileBuff>) {
            fn items_count(a:&(PathBuf,LogFile<FileBuff>)) -> Result<u32,LoqErr<PathBuf,LogQueueFileNumID>> {
                a.1.count().map_err(|e| LoqErr::LogCountFail { file: a.0.clone(), error: e })
            }
        }

        impl IdOf<PathBuf, LogFile<FileBuff>, LogQueueFileNumID> for (PathBuf, LogFile<FileBuff>) {
            fn id_of(a:&(PathBuf,LogFile<FileBuff>)) -> Result<LogQueueFileNumID,LoqErr<PathBuf,LogQueueFileNumID>> {
                id_of(a)
            }
        }

        let log_queue_conf: LogQueueConf<LogQueueFileNumID, PathBuf, FileBuff, _, _, _, _> = LogQueueConf {
            find_files: fs_log_find,
            open_log_file: OpenLogFileStub,
            validate: ValidateStub,
            new_file: move || {
                let mut generator = log_file_new.write().unwrap();
                let new_file = generator.generate().unwrap();
                let path = new_file.path.clone();
                Ok(path)
            },
            _p: PhantomData.clone(),
        };    

        let log_queue = log_queue_conf.open().unwrap();
        println!("log_queue openned");

        let mut log_queue: Box<dyn LogFileQueue<LogQueueFileNumID,PathBuf,LogFile<FileBuff>> + '_>
            = Box::new(log_queue);

        let rec = log_queue.write(20).unwrap();
        println!("log_queue writed, rec id = {:?}",rec);

        log_queue.switch().unwrap();
        println!("log_queue switched");

        let rec1 = log_queue.write(30).unwrap();
        println!("log_queue writed, rec id = {:?}",rec1);

        let rec2 = log_queue.write(32).unwrap();
        println!("log_queue writed, rec id = {:?}",rec2);
        assert!(rec2.block_id.value() > rec1.block_id.value());

        let rec3 = log_queue.write(34).unwrap();
        println!("log_queue writed, rec id = {:?}",rec3);
        assert!(rec3.block_id.value() > rec2.block_id.value());

        let rec4 = log_queue.last_record().unwrap();
        println!("last rec = {:?}",rec4);
        //let rec_id = log_queue.last

    }
}
