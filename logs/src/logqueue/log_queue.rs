use std::cell::RefCell;
use std::collections::HashMap;

use core::fmt::Debug;

#[allow(unused)]
use crate::logfile::{LogFile, FlatBuff, LogErr};

#[allow(unused)]
use crate::logfile::block::{BlockId, BlockOptions};

#[allow(unused)]
use super::find_logs::FsLogFind;
use super::{log_id::*, LoqErr};
use super::log_switch::{
    LogSwitching,
    LogQueueState
};
use super::logs_open::{
    LogQueueOpenConf,
    LogQueueOpenned as LogOpened,
};

#[allow(unused)]
use super::log_api::*;

/// Очередь логов
pub trait LogFileQueue<ID,FILE,LOG>: LogOpened<LogFiles = Vec<(FILE,LOG)>, LogFile = (FILE,LOG)> 
where ID: Clone + Debug, FILE: Clone + Debug 
{
    /// Переключение лога
    fn switch( &mut self ) -> Result<(),LoqErr<FILE,ID>>;

    /// Поиск лог файла по его ID
    /// 
    /// Аргументы
    /// ==============
    /// - `id` идентификатор
    /// 
    /// Результат
    /// =============
    /// лог
    fn find_log( &self, id:ID ) -> Result<Option<(FILE,LOG)>,LoqErr<FILE,ID>>;

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
    fn offset_log_id( &self, id:ID, offset: i64) -> Result<Option<ID>, LoqErr<FILE,ID>>;

    /// Получение идентификатора лог файла
    fn log_id_of( &self, log_file: &(FILE,LOG) ) -> Result<ID,LoqErr<FILE,ID>>;
}

/// Очередь логов
pub struct LogFileQueueImpl<ID,FILE,LOG,LOGSwitch,LOGIdOf> 
where
    LOG: Clone + Debug,
    FILE: Clone + Debug,
    LOGSwitch: LogSwitching<(FILE,LOG),LoqErr<FILE,ID>>,
    ID: LogQueueFileId,
    LOGIdOf: Fn((FILE,LOG)) -> Result<ID,LoqErr<FILE,ID>> + Clone,
{
    /// Список файлов
    files: Vec<(FILE,LOG)>,

    /// Актуальный лог
    pub tail: (FILE,LOG),

    /// Переключение лог файла
    #[allow(dead_code)]
    switching: LOGSwitch,

    /// Получение идентификатора лога
    pub id_of: LOGIdOf,

    /// текущий id лога
    current_log_id: RefCell<Option<ID>>,

    /// Кеш ид - лог файл
    log_id_to_log: RefCell<Option<HashMap<ID,(FILE,LOG)>>>,

    /// Очередность id логов
    log_id_order: RefCell<Option<Vec<ID>>>,
}

impl<ID,FILE,LOG,LOGSwitch,LOGIdOf> LogFileQueueImpl<ID,FILE,LOG,LOGSwitch,LOGIdOf> 
where
    ID: LogQueueFileId,
    LOG:Clone+Debug,
    FILE:Clone+Debug,
    LOGSwitch: LogSwitching<(FILE,LOG),LoqErr<FILE,ID>> + Clone,
    LOGIdOf: Fn((FILE,LOG)) -> Result<ID,LoqErr<FILE,ID>> + Clone,
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
        files: Vec<(FILE,LOG)>,
        tail: (FILE,LOG),
        switching: LOGSwitch,
        id_of: LOGIdOf
    ) -> Self {
        Self { 
            files: files, 
            tail: tail, 
            switching: switching, 
            current_log_id: RefCell::new(None),
            log_id_to_log: RefCell::new(None),
            log_id_order: RefCell::new(None),            
            id_of: id_of,
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
    fn log_id_map_cache_read<R,F>( &self, default:R, consume:F ) -> Result<R,LoqErr<FILE,ID>>
    where
        R: Sized,
        F: for <'a> Fn(&'a HashMap<ID,(FILE,LOG)>) -> R,
    {
        let mut cache_opt = self.log_id_to_log.borrow_mut();
        if cache_opt.is_none() {
            let mut cache : HashMap<ID,(FILE,LOG)> = HashMap::new();
            for file_log in &self.files() {
                let found_id = (self.id_of)(file_log.clone())?;
                cache.insert(found_id, file_log.clone());
            }
            *cache_opt = Some(cache);
        }

        Ok(cache_opt.as_ref().map(|x| {  
            consume(x)
        }).unwrap_or(default))
    }

    // пересоздание кеша, если необходимо и обход кеша
    fn log_order_cache_read<R,F>( &self, default:R, consume:F ) -> Result<R,LoqErr<FILE,ID>>
    where
        R: Sized,
        F: for <'a> Fn(&'a Vec<ID>) -> R
    {
        let mut cache_opt = self.log_id_order.borrow_mut();
        if cache_opt.is_none() {
            let mut cache: Vec<ID> = Vec::new();
            for file_log in &self.files() {
                let id = (self.id_of)(file_log.clone())?;
                cache.push(id);
            }
            *cache_opt = Some(cache);
        }

        Ok(cache_opt.as_ref().map(|x| consume(x)).unwrap_or(default))
    }

    #[allow(unused)]
    fn current_log_id_read<R,F>( &self, default:R, consume:F ) -> Result<R,LoqErr<FILE,ID>>
    where
        R: Sized,
        F: Fn(ID) -> R 
    {
        let mut cache_opt = self.current_log_id.borrow_mut();
        if cache_opt.is_none() {            
            let id = (self.id_of)(self.tail.clone())?;
            *cache_opt = Some(id);
        }
        Ok(consume(cache_opt.unwrap()))
    }
}

impl<ID,FILE,LOG,LOGSwitch,LOGIdOf> LogOpened 
for LogFileQueueImpl<ID,FILE,LOG,LOGSwitch,LOGIdOf>
where
    ID: LogQueueFileId,
    LOG:Clone + Debug,
    FILE:Clone + Debug,
    LOGSwitch: LogSwitching<(FILE,LOG),LoqErr<FILE,ID>>,
    LOGIdOf: Fn((FILE,LOG)) -> Result<ID,LoqErr<FILE,ID>> + Clone,
{
    type LogFile = (FILE,LOG);
    type LogFiles = Vec<(FILE,LOG)>;

    fn files( &self ) -> Self::LogFiles {
        let res : Vec<(FILE,LOG)> = 
            self.files.iter().map(|c|c.clone()).collect();
        res
    }

    fn tail( &self ) -> Self::LogFile {
        self.tail.clone()
    }
}

impl<ID,FILE,LOG,LOGSwitch,LOGIdOf> LogQueueState<(FILE,LOG)> 
for LogFileQueueImpl<ID,FILE,LOG,LOGSwitch,LOGIdOf> 
where
    ID: LogQueueFileId,
    FILE: Clone + Debug,
    LOG: Clone + Debug,
    LOGSwitch: LogSwitching<(FILE,LOG),LoqErr<FILE,ID>> + Clone,
    LOGIdOf: Fn((FILE,LOG)) -> Result<ID,LoqErr<FILE,ID>> + Clone,
{
    type ERR = LoqErr<FILE,ID>;
    fn get_current_file( &self ) -> Result<(FILE,LOG),Self::ERR> {
        Ok( self.tail.clone() )
    }
    fn switch_current_file( &mut self, new_file: (FILE,LOG) ) -> Result<(),Self::ERR> {
        self.invalidate_cache();
        self.files.push(new_file.clone());
        self.tail = new_file;
        Ok(())
    }
}

impl<ID,FILE,LOG,LOGSwitch,LOGIdOf> LogFileQueue<ID,FILE,LOG>
for LogFileQueueImpl<ID,FILE,LOG,LOGSwitch,LOGIdOf>
where
    ID: LogQueueFileId,
    FILE: Clone + Debug,
    LOG: Clone + Debug,
    LOGSwitch: LogSwitching<(FILE,LOG),LoqErr<FILE,ID>> + Clone,
    LOGIdOf: Fn((FILE,LOG)) -> Result<ID,LoqErr<FILE,ID>> + Clone,
{
    fn switch( &mut self ) -> Result<(),LoqErr<FILE,ID>> {
        let mut s = self.switching.clone();
        let _ = s.switch(self)?;
        Ok(())
    }

    fn find_log( &self, id:ID ) -> Result<Option<(FILE,LOG)>,LoqErr<FILE,ID>> {
        self.log_id_map_cache_read(
            None, 
            |cache| {
                cache.get(&id).map(|i|i.clone())
            }
        )
    }

    fn offset_log_id( &self, id:ID, offset: i64) -> Result<Option<ID>, LoqErr<FILE,ID>> {
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

    fn log_id_of( &self, log_file: &(FILE,LOG) ) -> Result<ID,LoqErr<FILE,ID>> {
        (self.id_of)( log_file.clone() )
    }
}

/// Конфигурация логов
pub struct LogQueueConf<ID,FILE,LOG,LOGOpenCfg,LOGOpenRes,LOGSwitch,LOGIdOf>
where
    ID: LogQueueFileId,
    FILE: Clone + Debug,
    LOGOpenRes: LogOpened<LogFile = (FILE,LOG), LogFiles = Vec<(FILE,LOG)>>,
    LOGOpenCfg: LogQueueOpenConf<Open = LOGOpenRes, OpenError = LoqErr<FILE,ID>>,
    LOGSwitch: LogSwitching<(FILE,LOG),LoqErr<FILE,ID>> + Clone,
    LOGIdOf: Fn((FILE,LOG)) -> Result<ID,LoqErr<FILE,ID>> + Clone,
{
    pub log_open: LOGOpenCfg,
    pub log_switch: LOGSwitch,
    pub id_of: LOGIdOf,
}

impl<ID,FILE,LOG,LOGOpenCfg,LOGOpenRes,LOGSwitch,LOGIdOf> 
    LogQueueConf<ID,FILE,LOG,LOGOpenCfg,LOGOpenRes,LOGSwitch,LOGIdOf> 
where
    ID: LogQueueFileId,
    FILE: Clone + Debug,
    LOG: Clone + Debug,
    LOGOpenRes: LogOpened<LogFile = (FILE,LOG), LogFiles = Vec<(FILE,LOG)>>,
    LOGOpenCfg: LogQueueOpenConf<Open = LOGOpenRes, OpenError = LoqErr<FILE,ID>>,
    LOGSwitch: LogSwitching<(FILE,LOG),LoqErr<FILE,ID>> + Clone,
    LOGIdOf: Fn((FILE,LOG)) -> Result<ID,LoqErr<FILE,ID>> + Clone,
{
    /// Открытие логов
    pub fn open( &self ) -> Result<LogFileQueueImpl<ID,FILE,LOG,LOGSwitch,LOGIdOf>,LoqErr<FILE,ID>> {
        let opened = self.log_open.open()?;
        Ok(LogFileQueueImpl::new(
            opened.files(), 
            opened.tail(), 
            self.log_switch.clone(),
            self.id_of.clone(),
        ))
    }
}

#[cfg(test)]
mod test {
    use std::marker::PhantomData;

    use crate::logqueue::{FindFiles, OpenLogFile, ValidateLogFiles, InitializeFirstLog};

    #[test]
    fn log_queue_conf_test() {
        use std::sync::Arc;
        use std::sync::atomic::AtomicBool;

        use super::*;

        use super::super::log_seq_verifier::test::IdTest;
        use super::super::log_seq_verifier::OrderedLogs;
        use super::super::logs_open::LogFileQueueConf;
        use super::super::log_switch::*;
        #[allow(unused)]
        use super::super::log_id::*;

        let id0 = IdTest::new(None);
        let id1 = IdTest::new(Some(id0.id()));
        let id2 = IdTest::new(Some(id1.id()));
        let id3 = IdTest::new(Some(id2.id()));

        let oldnew_id_matched = Arc::new(AtomicBool::new(false));
        let oldnew_id_matched1 = oldnew_id_matched.clone();

        struct FindFilesStub(Vec<IdTest>);
        impl FindFiles<IdTest,LoqErr<IdTest,IdTest>> for FindFilesStub {
            fn find_files( &self ) -> Result<Vec<IdTest>,LoqErr<IdTest,IdTest>> {
                Ok(self.0.clone())
            }
        }

        struct OpenFileStub;
        impl OpenLogFile<IdTest,IdTest,LoqErr<IdTest,IdTest>> for OpenFileStub {
            fn open_log_file( &self, file:IdTest ) -> Result<IdTest, LoqErr<IdTest,IdTest>> {
                Ok(file.clone())
            }
        }

        struct ValidateStub(OrderedLogs<(IdTest,IdTest)>);
        impl ValidateLogFiles<(IdTest,IdTest),LoqErr<IdTest,IdTest>> for ValidateStub {
            fn validate( &self, _log_files: &Vec<(IdTest,IdTest)> ) -> Result<OrderedLogs<(IdTest,IdTest)>,LoqErr<IdTest,IdTest>> {
                Ok( self.0.clone() )
            }
        }

        struct InitStub(IdTest);
        impl InitializeFirstLog<(IdTest,IdTest),LoqErr<IdTest,IdTest>> for InitStub {
            fn initialize_first_log( &self ) -> Result<(IdTest,IdTest), LoqErr<IdTest,IdTest>> {
                Ok((self.0.clone(), self.0.clone()))
            }
        }

        let open_conf = LogFileQueueConf {
            find_files: FindFilesStub(vec![id0.clone(), id1.clone(), id2.clone(), id3.clone()]),
            open_log_file: OpenFileStub,
            validate: ValidateStub(OrderedLogs {
                files: vec![
                    (id1.clone(),id1.clone()), 
                    (id2.clone(),id2.clone()), 
                    (id3.clone(),id3.clone()),
                    (id0.clone(),id0.clone()), 
                ],
                tail: (id3.clone(),id3.clone())
            }),
            init: InitStub(id0.clone()),
            _p: PhantomData.clone(),
        };

        let log_switch = LogSwitcher { 
            read_id_of: |f_id:&(IdTest,IdTest)| Ok( f_id.0.clone() ), 
            write_id_to: |_,ids:OldNewId<'_,IdTest>| {
                println!("old id={} new id={}", ids.old_id, ids.new_id);
                oldnew_id_matched1.store(true, std::sync::atomic::Ordering::SeqCst);
                ids.new_id.previous().map(|i| ids.old_id.id() == i );
                Ok(())
            }, 
            new_file: || {
                let id = IdTest::new(None);
                Ok( (id.clone(), id.clone()) )
            }, 
        };

        let log_queue_conf = LogQueueConf { 
            log_open: open_conf, 
            log_switch: log_switch, 
            id_of: |_f| Ok(IdTest::new(None)),
        };

        let mut log_queue : LogFileQueueImpl<IdTest,IdTest,IdTest,_,_> = log_queue_conf.open().unwrap();

        println!("before");

        let count0 = log_queue.files().len();
        for (a,_) in log_queue.files() {
            println!("log {a}");
        }

        println!("after");

        log_queue.switch().unwrap();
        let count1 = log_queue.files().len();
        for (a,_) in log_queue.files() {
            println!("log {a}");
        }

        assert!(count1 > count0);
        assert!(oldnew_id_matched.load(std::sync::atomic::Ordering::SeqCst));
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
    use crate::logfile::block::{BlockId, BlockOptions};
    use crate::logqueue::new_file::NewFileGenerator;
    use crate::logqueue::path_tmpl::PathTemplateParser;

    #[allow(unused)]
    use crate::logqueue::{log_id::*, LogFileQueueConf, LoqErr, validate_sequence, SeqValidateOp, IdOf, 
        LogQueueConf, LogSwitcher, OldNewId, LogFileQueue, OpenLogFile, ValidateLogFiles, InitializeFirstLog, LogWriting, LogNavigateLast
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

        struct OpenLogFileStub;
        impl OpenLogFile<PathBuf,LogFile<FileBuff>,LoqErr<PathBuf,LogQueueFileNumID>> for OpenLogFileStub {
            fn open_log_file( &self, file:PathBuf ) -> Result<LogFile<FileBuff>, LoqErr<PathBuf,LogQueueFileNumID>> {
                open_file(file)
            }
        }

        struct ValidateStub;
        impl ValidateLogFiles<(PathBuf,LogFile<FileBuff>),LoqErr<PathBuf,LogQueueFileNumID>> for ValidateStub {
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

        struct InitStub<'a,F>( Arc<RwLock<NewFileGenerator<'a,F>>> )
        where F: Fn(PathBuf) -> Result<File,std::io::Error>;

        impl<'a,F> InitializeFirstLog<(PathBuf,LogFile<FileBuff>),LoqErr<PathBuf,LogQueueFileNumID>> for InitStub<'a,F> 
        where F: Fn(PathBuf) -> Result<File,std::io::Error>
        {
            fn initialize_first_log( &self ) -> Result<(PathBuf,LogFile<FileBuff>), LoqErr<PathBuf,LogQueueFileNumID>> {
                let mut generator = self.0.write().unwrap();
                let new_file = generator.generate().unwrap();
                let path = new_file.path.clone();
                let mut log = open_file(new_file.path.clone())?;

                let id = LogQueueFileNumID::new(None);
                id.write(&path, &mut log)?;

                Ok((path,log))
            }
        }

        let log_file_queue_conf: 
        LogFileQueueConf<
            LogFile<FileBuff>, 
            PathBuf, 
            LoqErr<PathBuf,LogQueueFileNumID>,
            _, _, _, _>
         = LogFileQueueConf {
            find_files: fs_log_find,
            open_log_file: OpenLogFileStub,
            validate: ValidateStub,
            init: InitStub(log_file_new.clone()),
            _p: PhantomData.clone()
        };

        let log_switch : LogSwitcher<(PathBuf,LogFile<FileBuff>), LogQueueFileNumID, LoqErr<PathBuf,LogQueueFileNumID>, _, _, _> =
        LogSwitcher { 
            read_id_of: |log_file_pair: &(PathBuf,LogFile<FileBuff>)| {
                id_of(log_file_pair)
            }, 
            write_id_to: 
                |log_file_pair: &mut (PathBuf,LogFile<FileBuff>),
                 ids: OldNewId<LogQueueFileNumID>
                | 
            { 
                let (filename, log) = log_file_pair;
                let mut options = BlockOptions::default();
                let mut data = Vec::<u8>::new();
                ids.new_id.block_write(&mut options, &mut data)
                    .map_err(|err| LoqErr::LogIdWrite { 
                        file: filename.clone(), 
                        error: err
                })?;
                log.append_data(&options, &data)
                    .map_err(|err| LoqErr::LogIdWrite2 { 
                        file: filename.clone(), 
                        error: err 
                    })?;
                Ok(())
            },
            new_file: move || {
                let mut generator = log_file_new.write().unwrap();
                let new_file = generator.generate().unwrap();
                let path = new_file.path.clone();
                let log = open_file(new_file.path.clone());
                log.map(|log| (path,log))
            } 
        };

        let log_queue_conf = LogQueueConf {
            log_open:   log_file_queue_conf,
            log_switch: log_switch,
            id_of:      |log_file_pair:(PathBuf,LogFile<FileBuff>)| {
                id_of(&log_file_pair)
            }
        };

        let log_queue = log_queue_conf.open().unwrap();
        println!("log_queue openned");

        let mut log_queue: Box<dyn LogFileQueue<LogQueueFileNumID,PathBuf,LogFile<FileBuff>> + '_>
            = Box::new(log_queue);

        let rec = log_queue.write(20).unwrap();
        println!("log_queue writed, rec id = {:?}",rec);

        log_queue.switch().unwrap();
        println!("log_queue switched");

        let rec = log_queue.write(30).unwrap();
        println!("log_queue writed, rec id = {:?}",rec);

        let rec = log_queue.write(32).unwrap();
        println!("log_queue writed, rec id = {:?}",rec);

        let rec = log_queue.write(34).unwrap();
        println!("log_queue writed, rec id = {:?}",rec);

        let rec = log_queue.last_record().unwrap();
        println!("last rec = {:?}",rec);
        //let rec_id = log_queue.last

    }
}
