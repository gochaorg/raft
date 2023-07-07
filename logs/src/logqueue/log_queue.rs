use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;

use super::log_id::LogQueueFileId;
use super::log_switch::{
    LogSwitching,
    LogQueueState
};
use super::logs_open::{
    LogQueueConf as LogOpenConf,
    LogQueueOpenned as LogOpened,
};

/// Очередь логов
pub struct LogFileQueue<ID,FILE,LOG,ERR,LOGSwitch,LOGIdOf> 
where
    LOG: Clone,
    LOGSwitch: LogSwitching<(FILE,LOG),ERR>,
    ID: LogQueueFileId,
    LOGIdOf: Fn((FILE,LOG)) -> Result<ID,ERR> + Clone,
{
    /// Список файлов
    files: Vec<(FILE,LOG)>,

    /// Актуальный лог
    tail: (FILE,LOG),

    /// Переключение лог файла
    #[allow(dead_code)]
    switching: LOGSwitch,

    /// Получение идентификатора лога
    id_of: LOGIdOf,

    /// Кеш ид - лог файл
    log_id_to_log: RefCell<Option<HashMap<ID,(FILE,LOG)>>>,

    _p: PhantomData<ERR>
}

impl<ID,FILE,LOG,ERR,LOGSwitch,LOGIdOf> LogFileQueue<ID,FILE,LOG,ERR,LOGSwitch,LOGIdOf> 
where
    ID: LogQueueFileId,
    LOG:Clone,
    FILE:Clone,
    LOGSwitch: LogSwitching<(FILE,LOG),ERR> + Clone,
    LOGIdOf: Fn((FILE,LOG)) -> Result<ID,ERR> + Clone,
{
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
            log_id_to_log: RefCell::new(None),
            id_of: id_of,
            _p: PhantomData.clone() 
        }
    }

    /// Переключение лога
    pub fn switch( &mut self ) -> Result<(),ERR> {
        let mut s = self.switching.clone();
        let _ = s.switch(self)?;
        Ok(())
    }

    /// Сброс кеша
    pub fn invalidate_cache( &self ) {
        let mut r = self.log_id_to_log.borrow_mut();
        *r = None;
    }

    pub fn find_log( &self, id:ID ) -> Result<Option<(FILE,LOG)>,ERR> {
        let cache = self.log_id_to_log.borrow_mut();
        if cache.is_none() {
            let mut res : Option<(FILE,LOG)> = None;
            // rebuild cache
            let mut cache : HashMap<ID,(FILE,LOG)> = HashMap::new();
            for file_log in &self.files() {
                let found_id = (self.id_of)(file_log.clone())?;
                if id == found_id {
                    res = Some(file_log.clone());
                }
                cache.insert(found_id, file_log.clone());
            }

            let mut x = self.log_id_to_log.borrow_mut();
            *x = Some(cache);

            return Ok(res);
        }
        let exists = cache.as_ref().and_then(|z| z.get(&id)).map(|x| x.clone());
        Ok(exists)
    }
}

impl<ID,FILE,LOG,ERR,LOGSwitch,LOGIdOf> LogOpened for LogFileQueue<ID,FILE,LOG,ERR,LOGSwitch,LOGIdOf>
where
    ID: LogQueueFileId,
    LOG:Clone,
    FILE:Clone,
    LOGSwitch: LogSwitching<(FILE,LOG),ERR>,
    LOGIdOf: Fn((FILE,LOG)) -> Result<ID,ERR> + Clone,
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

impl<ID,FILE,LOG,ERR,LOGSwitch,LOGIdOf> LogQueueState<(FILE,LOG)> for LogFileQueue<ID,FILE,LOG,ERR,LOGSwitch,LOGIdOf> 
where
    ID: LogQueueFileId,
    FILE: Clone,
    LOG: Clone,
    LOGSwitch: LogSwitching<(FILE,LOG),ERR> + Clone,
    LOGIdOf: Fn((FILE,LOG)) -> Result<ID,ERR> + Clone,
{
    type ERR = ERR;
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

/// Конфигурация логов
struct LogQueueConf<ID,FILE,LOG,ERR,LOGOpenCfg,LOGOpenRes,LOGSwitch,LOGIdOf>
where
    ID: LogQueueFileId,
    FILE: Clone,
    LOGOpenRes: LogOpened<LogFile = (FILE,LOG), LogFiles = Vec<(FILE,LOG)>>,
    LOGOpenCfg: LogOpenConf<Open = LOGOpenRes, OpenError = ERR>,
    LOGSwitch: LogSwitching<(FILE,LOG),ERR> + Clone,
    LOGIdOf: Fn((FILE,LOG)) -> Result<ID,ERR> + Clone,
{
    log_open: LOGOpenCfg,
    log_switch: LOGSwitch,
    id_of: LOGIdOf,

    _p: PhantomData<(ERR,ID)>
}

impl<ID,FILE,LOG,ERR,LOGOpenCfg,LOGOpenRes,LOGSwitch,LOGIdOf> 
    LogQueueConf<ID,FILE,LOG,ERR,LOGOpenCfg,LOGOpenRes,LOGSwitch,LOGIdOf> 
where
    ID: LogQueueFileId,
    FILE: Clone,
    LOG: Clone,
    LOGOpenRes: LogOpened<LogFile = (FILE,LOG), LogFiles = Vec<(FILE,LOG)>>,
    LOGOpenCfg: LogOpenConf<Open = LOGOpenRes, OpenError = ERR>,
    LOGSwitch: LogSwitching<(FILE,LOG),ERR> + Clone,
    LOGIdOf: Fn((FILE,LOG)) -> Result<ID,ERR> + Clone,
{
    /// Открытие логов
    pub fn open( &self ) -> Result<LogFileQueue<ID,FILE,LOG,ERR,LOGSwitch,LOGIdOf>,ERR> {
        let opened = self.log_open.open()?;
        Ok(LogFileQueue::new(
            opened.files(), 
            opened.tail(), 
            self.log_switch.clone(),
            self.id_of.clone(),
        ))
    }
}

#[test]
fn log_queue_conf_test() {
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    use super::log_seq_verifier::test::IdTest;
    use super::log_seq_verifier::OrderedLogs;
    use super::logs_open::LogFileQueueConf;
    use super::log_switch::*;
    use super::log_id::*;

    let id0 = IdTest::new(None);
    let id1 = IdTest::new(Some(id0.id()));
    let id2 = IdTest::new(Some(id1.id()));
    let id3 = IdTest::new(Some(id2.id()));

    let oldnew_id_matched = Arc::new(AtomicBool::new(false));
    let oldnew_id_matched1 = oldnew_id_matched.clone();

    let open_conf: LogFileQueueConf<IdTest,IdTest,String,_,_,_,_> = LogFileQueueConf {
        find_files: || Ok(vec![id0.clone(), id1.clone(), id2.clone(), id3.clone()]),
        open_log_file: |f| Ok::<IdTest,String>( f.clone() ),
        validate: |f| Ok(OrderedLogs {
            files: vec![
                (id1.clone(),id1.clone()), 
                (id2.clone(),id2.clone()), 
                (id3.clone(),id3.clone()),
                (id0.clone(),id0.clone()), 
            ],
            tail: (id3.clone(),id3.clone())
        }),
        init: || Ok( (id0.clone(),id0.clone()) ),
        _p: PhantomData.clone()
    };

    let log_switch : LogSwitcher<(IdTest,IdTest),IdTest,String,_,_,_> = LogSwitcher { 
        read_id_of: |f_id:&(IdTest,IdTest)| Ok( f_id.0.clone() ), 
        write_id_to: |f,ids:OleNewId<'_,IdTest>| {
            println!("old id={} new id={}", ids.old_id, ids.new_id);
            oldnew_id_matched1.store(true, std::sync::atomic::Ordering::SeqCst);
            ids.new_id.previous().map(|i| ids.old_id.id() == i );
            Ok(())
        }, 
        new_file: || {
            let id = IdTest::new(None);
            Ok( (id.clone(), id.clone()) )
        }, 
        _p: PhantomData.clone(),
    };

    let log_queue_conf : LogQueueConf<IdTest,IdTest,IdTest,String,_,_,_,_> = LogQueueConf { 
        log_open: open_conf, 
        log_switch: log_switch, 
        id_of: |f| Ok(IdTest::new(None)),
        _p: PhantomData.clone(),
    };

    let mut log_queue : LogFileQueue<IdTest,IdTest,IdTest,String,_,_> = log_queue_conf.open().unwrap();

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
