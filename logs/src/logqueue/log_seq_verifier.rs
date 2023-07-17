use super::{log_id::*, LoqErr};
use std::fmt::Debug;

/// Извлечение и лог файла - идентификатора
pub trait IdOf<FILE,LOG,ID> 
where
    ID: Clone+Debug,
    FILE: Clone+Debug,
{
    /// получить идентификатор-ссылку
    fn id_of(a:&(FILE,LOG)) -> Result<ID,LoqErr<FILE,ID>>;
}

/// Операции с лог файлов
pub trait SeqValidateOp<FILE,LOG,ID>: IdOf<FILE,LOG,ID>
where
    ID:Clone+Debug,
    FILE: Clone+Debug,
{
    /// кол-во элементов в логе
    fn items_count(a:&(FILE,LOG)) -> Result<u32,LoqErr<FILE,ID>>;
}

/// Упорядоченный лог файлы
#[derive(Debug,Clone)]
pub struct OrderedLogs<ITEM> 
where 
    ITEM:Clone,    
{
    /// Последний лог файл в очереди
    pub tail: ITEM,

    /// Упорядоченная очередь лог файлов
    pub files: Vec<ITEM>
}

/// Валидация очереди логов
/// 
/// Проверки
/// 
/// - Должна быть одна голова - т.е. один id лога, который не ссылается на другие логи
/// - Остальный логи, из id должны ссылаться на логи
/// - ссылки должны образовывать линейную последовательность
/// 
/// Аргрументы
/// ============
/// - files - список лог файлов
/// - ITEM - тип лог файла
/// - ERR - тип ошибки
/// - ERRBuild - trait для постраения ошибок
/// - ID - тип идентификатора лог файла
/// 
/// Результат
/// ============
/// Список логов упорядоченных по времени создания
#[allow(dead_code)]
pub fn validate_sequence<FILE,LOG,ID>( files: &Vec<(FILE,LOG)> ) -> 
    Result<OrderedLogs<(FILE,LOG)>,LoqErr<FILE,ID>>
where
    FILE: Clone+Debug,
    (FILE,LOG): Clone + SeqValidateOp<FILE,LOG,ID>,
    ID: LogQueueFileId,
{
    // Выделение id файлов
    let files_with_id = files.iter().fold( 
        Ok::<Vec<((FILE,LOG),ID)>,LoqErr<FILE,ID>>(vec![]), |res,itm| {
            res.and_then(|mut res| {
                let count = <(FILE,LOG)>::items_count(itm)?;
                if count > 0u32 {
                    let id = <(FILE,LOG)>::id_of(itm)?;
                    res.push( (itm.clone(), id.clone()) );
                    Ok(res) 
                } else {
                    Ok(res) 
                }
            })
        }
    )?;

    let mut head_files: Vec<((FILE,LOG),ID)> = 
        files_with_id.iter().filter(|(_,id)| id.previous().is_none())
        .map(|(a,b)| (a.clone(), b.clone()))
        .collect();
    
    if head_files.len()>1 {        
        return Err(LoqErr::OpenTwoHeads { heads: 
            head_files.iter().map( |((file,_log),id)| (file.clone(), id.clone())).collect() 
        });
    } else if head_files.is_empty() {
        // Найти те что ссылается на не существующую id
        let mut ids = std::collections::HashSet::<ID>::new();
        for (_,id) in &files_with_id {
            ids.insert(id.clone());
        }

        // обходим список
        // ищем потенциальную голову
        let mut files_set = files_with_id.clone();        
        let mut heads = Vec::<((FILE,LOG),ID)>::new();
        while files_set.len() >= 1 {
            let (f,id) = files_set[0].clone();
            match ids.iter().find(|i| id.previous().map(|a| a == i.id()).unwrap_or(false) ) {
                Some(_) => {
                    files_set.remove(0);
                },
                None => {
                    heads.push((f.clone(), id.clone()));
                    files_set.remove(0);
                }
            }
        }

        if heads.is_empty() {
            return Err( LoqErr::OpenNoHeads );
        } else if heads.len()>1 {
            return Err( LoqErr::OpenTwoHeads { heads:  
                head_files.iter().map( |((file,_log),id)| (file.clone(), id.clone())).collect() 
            });
        }

        for (f,i) in heads {
            head_files.push((f.clone(), i.clone()));
        }
    }

    let (head,mut head_id) = head_files.iter().map(|(a,b)|(a.clone(),b.clone())).next().unwrap();

    let mut ordered_files = vec![(head)];
    let mut files_with_id: Vec<&((FILE,LOG),ID)> = files_with_id.iter().filter(|(_,id)| *id != head_id).collect();

    while ! files_with_id.is_empty() {
        match files_with_id.iter().find(|(_,id)| id.previous().map(|id| id == head_id.id()).unwrap_or(false) ) {
            Some((found,found_id)) => {
                head_id = found_id.clone();
                files_with_id = files_with_id.iter().filter(|(_,id)| id.id() != head_id.id() ).map(|x| x.clone()).collect();
                ordered_files.push(found.clone());
            }
            None => {
                break;
            }
        }
    }

    let ordered_files: Vec<(FILE,LOG)> = ordered_files.iter().map(|a|a.clone()).collect();
    if !files_with_id.is_empty() {
        return Err(LoqErr::OpenLogNotFound { 
            id:   head_id.clone(), 
            logs: head_files.iter().map( |((file,_log),id)| (file.clone(), id.clone())).collect() 
        });
    }

    let last = ordered_files.last().map(|i| i.clone()).unwrap();
    Ok(OrderedLogs{files: ordered_files, tail:last})
}

#[cfg(test)]
pub mod test {
    use uuid::Uuid;
    use crate::{logfile::block::BlockOptions, logqueue::LoqErr};

    use super::*;

    #[derive(Debug,Clone,PartialEq,Hash)]
    pub struct IdTest {
        id: Uuid,
        prev: Option<Uuid>
    }
    impl std::fmt::Display for IdTest {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f,"IdTest({}, {:?})",self.id, self.prev)
        }
    }
    impl Eq for IdTest {}
    impl Copy for IdTest {}
    impl LogQueueFileId for IdTest {
        type ID = Uuid;
        fn id( &self ) -> Self::ID {            
            self.id
        }
        fn new( prev:Option<Self::ID> ) -> Self {
            Self { id: Uuid::new_v4(), prev: prev }
        }
        fn previous( &self ) -> Option<Self::ID> {            
            self.prev
        }
    }
    impl BlockWriter for IdTest {
        fn block_write( &self, _options: &mut BlockOptions, _data: &mut Vec<u8> ) -> Result<(),LogIdReadWriteErr> {
            Ok(())
        }
    }
    impl BlockReader for IdTest {
        fn block_read( _block: &crate::logfile::block::Block ) -> Result<Self, LogIdReadWriteErr> {
            todo!()
        }
    }

    impl SeqValidateOp<IdTest,IdTest,IdTest> for (IdTest,IdTest) {
        fn items_count(_a:&(IdTest,IdTest)) -> Result<u32,LoqErr<IdTest,IdTest>> {
            Ok(1u32)
        }
    }

    impl IdOf<IdTest,IdTest,IdTest> for (IdTest,IdTest) {
        fn id_of(a:&(IdTest,IdTest)) -> Result<IdTest,LoqErr<IdTest,IdTest>> {
            Ok(a.0.clone())
        }
    }

    #[test]
    fn valid_seq() {
        let id0 = IdTest::new(None);
        let id1 = IdTest::new(Some(id0.id()));
        let id2 = IdTest::new(Some(id1.id()));
        let id3 = IdTest::new(Some(id2.id()));

        let logs = vec![
            (id3.clone(), id3.clone()),
            (id2.clone(), id2.clone()),
            (id0.clone(), id0.clone()),
            (id1.clone(), id1.clone())
        ];
        match validate_sequence::<IdTest,IdTest,IdTest>(&logs) {
            Ok(seq) => {
                println!("ok");
                println!("tail = {:?}",seq.tail);
                for itm in seq.files {
                    println!(" {itm:?}")
                }
                assert_eq!(seq.tail, (id3,id3));
            }
            Err(err) => {
                println!("err {err:?}");
                assert!(false);
            }
        }
    }

    #[test]
    fn two_heads() {
        let id0 = IdTest::new(None);
        let id1 = IdTest::new(None);
        let id2 = IdTest::new(Some(id1.id()));
        let id3 = IdTest::new(Some(id2.id()));

        let logs = vec![
            (id3.clone(),id3.clone()), 
            (id2.clone(),id2.clone()),
            (id0.clone(),id0.clone()), 
            (id1.clone(),id1.clone()),
        ];
        match validate_sequence::<IdTest,IdTest,IdTest>(&logs) {
            Ok(seq) => {
                println!("ok");
                println!("tail = {:?}",seq.tail);
                for itm in seq.files {
                    println!(" {itm:?}")
                }
                assert!(false);
            }
            Err(err) => {
                println!("err {err:?}");
                match err {
                    LoqErr::OpenTwoHeads { heads:_ } => {},
                    _ => {assert!(false);}
                }
            }
        }
    }

    #[test]
    fn two_refs() {
        let id0 = IdTest::new(None);
        let id1 = IdTest::new(Some(id0.id()));
        let id2 = IdTest::new(Some(id0.id()));
        let id3 = IdTest::new(Some(id2.id()));

        let logs = vec![
            (id3.clone(),id3.clone()), 
            (id2.clone(),id2.clone()), 
            (id0.clone(),id0.clone()), 
            (id1.clone(),id1.clone())
        ];
        match validate_sequence::<IdTest,IdTest,IdTest>(&logs) {
            Ok(seq) => {
                println!("ok");
                println!("tail = {:?}",seq.tail);
                for itm in seq.files {
                    println!(" {itm:?}")
                }
                assert!(false);
            }
            Err(err) => {
                println!("err {err:?}");
                match err {
                    LoqErr::OpenLogNotFound { id:_, logs:_ } => {},
                    _ => { assert!(false); }
                }
                assert!(true);
            }
        }
    }

    #[test]
    fn partial_queue() {
        let id0 = IdTest::new(None);
        let id1 = IdTest::new(Some(id0.id()));
        let id2 = IdTest::new(Some(id1.id()));
        let id3 = IdTest::new(Some(id2.id()));

        let logs = vec![
            (id1.clone(),id1.clone()), 
            (id2.clone(),id2.clone()), 
            (id3.clone(),id3.clone())
        ];
        match validate_sequence::<IdTest,IdTest,IdTest>(&logs) {
            Ok(seq) => {
                println!("ok");
                println!("tail = {:?}",seq.tail);
                for itm in &seq.files {
                    println!(" {itm:?}")
                }
                assert!( seq.files.len()==3 );
                assert!( seq.files[0].0.id == id1.id() );
                assert!( seq.files[1].0.id == id2.id() );
                assert!( seq.files[2].0.id == id3.id() );
            }
            Err(err) => {
                println!("err {err:?}");
                assert!(false);
            }
        }
    }
}