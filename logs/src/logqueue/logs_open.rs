use std::marker::PhantomData;
use super::log_seq_verifier::OrderedLogs;

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
struct LogFileQueueConf<LOG,FILE,ERR,FOpen,FFind,FValidate,FInit>
where 
    LOG:Clone,
    FILE:Clone,
    FOpen: Fn(FILE) -> Result<LOG, ERR>,
    FFind: Fn() -> Result<Vec<FILE>, ERR>,
    FValidate: Fn(&Vec<(FILE,LOG)>) -> Result<OrderedLogs<(FILE,LOG)>,ERR>,
    FInit: Fn() -> Result<(FILE,LOG), ERR>,
{
    /// Поиск лог файлов
    find_files: FFind,

    /// Открытие лог файла
    open_log_file: FOpen,

    /// Валидация открытых лог файлов
    validate: FValidate,

    /// Первичная инициализация
    init: FInit,

    _p: PhantomData<(LOG,FILE,ERR)>
}

/// Открытые лог файлы
struct LogFilesOpenned<LOG,FILE>
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

impl<LOG,FILE,ERR,FOpen,FFind,FValidate,FInit> LogQueueConf 
for LogFileQueueConf<LOG,FILE,ERR,FOpen,FFind,FValidate,FInit> 
where
    LOG:Clone,
    FOpen: Fn(FILE) -> Result<LOG, ERR>,
    FFind: Fn() -> Result<Vec<FILE>, ERR>,
    FValidate: Fn(&Vec<(FILE,LOG)>) -> Result<OrderedLogs<(FILE,LOG)>,ERR>,
    FInit: Fn() -> Result<(FILE,LOG), ERR>,
    FILE: Clone
{
    type OpenError = ERR;
    type Open = LogFilesOpenned<LOG,FILE>;

    fn open( &self ) -> Result<Self::Open, Self::OpenError> {
        let found_files = (self.find_files)()?;
        if !found_files.is_empty() {
            let not_validated_open_files = found_files.iter().fold( 
                Ok::<Vec::<(FILE,LOG)>,ERR>(Vec::<(FILE,LOG)>::new()), 
                |res,file| {
                res.and_then(|mut res| {
                    let log_file = (self.open_log_file)(file.clone())?;
                    res.push((file.clone(),log_file));
                    Ok(res)
                })
            })?;

            //let (tail_file, tail_log) = (self.validate)(&not_validated_open_files)?;
            let validated_order = (self.validate)(&not_validated_open_files)?;

            Ok(LogFilesOpenned{ 
                files: not_validated_open_files, 
                tail_file: validated_order.tail.0, 
                tail_log: validated_order.tail.1,
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

        let queue_conf: LogFileQueueConf<IdTest,IdTest,String,_,_,_,_> = 
        LogFileQueueConf {
            find_files: || {
                Ok(vec![id0.clone(), id1.clone(), id2.clone(), id3.clone()])
            },
            open_log_file: |f| Ok::<IdTest,String>( f.clone() ),
            validate: |_files:&Vec<(IdTest,IdTest)>| Ok( 
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
            init: || Ok( (id0.clone(),id0.clone()) ),
            _p: PhantomData.clone()
        };

        let open_files:LogFilesOpenned<IdTest,IdTest> = queue_conf.open().unwrap();
        println!("tail {}", open_files.tail().0);
        for (f,_) in &open_files.files() {
            println!("log {}", f);
        }

    }
}
