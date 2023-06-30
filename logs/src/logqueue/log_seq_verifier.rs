use super::log_id::*;
use crate::{
    logfile::{
        FlatBuff,
        LogFile, LogErr, block::BlockId
    }
};

trait ErrThrow<ITEM,ERR,ID> 
where
    ID:LogQueueFileId,
{
    fn two_heads(heads:Vec<(ITEM,ID)>) -> ERR;
    fn no_heads() -> ERR;
    fn not_found_next_log( id: &ID, logs:Vec<&(ITEM,ID)> ) -> ERR;
}

/// Операции с лог файлов
trait SeqValidateOp<A,ERR,ID> 
where
    ID:LogQueueFileId
{
    /// кол-во элементов в логе
    fn items_count(a:&A) -> Result<u32,ERR>;

    /// получить идентификатор-ссылку
    fn id_of(a:&A) -> Result<ID,ERR>;
}

#[derive(Debug)]
struct OrderedLogs<ITEM> {
    tail: ITEM,
    files: Vec<ITEM>
}

/// Валидация очереди логов
/// 
/// Проверки
/// 
/// - Должна быть одна голова - т.е. один id лога, который не ссылается на другие логи
/// - Остальный логи, из id должны ссылаться на логи
/// - ссылки должны образовывать линейную последовательность
fn validate_sequence<ITEM,ERR,ERRBuild,ID>( files: &Vec<ITEM> ) -> 
    Result<OrderedLogs<ITEM>,ERR>
where
    ITEM: Clone + SeqValidateOp<ITEM,ERR,ID>,
    ID: LogQueueFileId,
    ERRBuild: ErrThrow<ITEM,ERR,ID>
{
    let files_with_id = files.iter().fold( 
        Ok::<Vec<(ITEM,ID)>,ERR>(vec![]), |res,itm| {
            res.and_then(|mut res| {
                let count = ITEM::items_count(itm)?;
                if count > 0u32 {
                    let id = ITEM::id_of(itm)?;
                    res.push( (itm.clone(), id.clone()) );
                    Ok(res) 
                } else {
                    Ok(res) 
                }
            })
        }
    )?;

    let head_files: Vec<(ITEM,ID)> = 
        files_with_id.iter().filter(|(_,id)| id.previous().is_none())
        .map(|(a,b)| (a.clone(), b.clone()))
        .collect();
    
    if head_files.len()>1 {
        return Err(ERRBuild::two_heads(head_files));
    } else if head_files.is_empty() {
        // Найти те что ссылается на не существующую id
        // let mut ids = std::collections::HashSet::<ID>::new();
        // for (_,id) in head_files {
        //     ids
        // }

        return Err(ERRBuild::no_heads());
    }

    let (head,mut head_id) = head_files.iter().map(|(a,b)|(a.clone(),b.clone())).next().unwrap();

    let mut ordered_files = vec![(head)];
    let mut files_with_id: Vec<&(ITEM,ID)> = files_with_id.iter().filter(|(_,id)| *id != head_id).collect();

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

    let ordered_files: Vec<ITEM> = ordered_files.iter().map(|(a)|a.clone()).collect();
    if !files_with_id.is_empty() {
        return Err(ERRBuild::not_found_next_log(&head_id, files_with_id));
    }

    let last = ordered_files.last().map(|i| i.clone()).unwrap();
    Ok(OrderedLogs{files: ordered_files, tail:last})
}

mod test {
    use uuid::Uuid;
    use super::*;

    #[derive(Debug,Clone,PartialEq,Hash)]
    struct IdTest {
        id: Uuid,
        prev: Option<Uuid>
    }
    impl std::fmt::Display for IdTest {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f,"IdTest({}, {:?})",self.id, self.prev)
        }
    }
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
        type ERR = String;
        fn block_write( &self, block: &mut crate::logfile::block::Block ) -> Result<(),Self::ERR> {
            Ok(())
        }
    }
    impl BlockReader for IdTest {
        type ERR = String;
        fn block_read( block: &crate::logfile::block::Block ) -> Result<Self, Self::ERR> {
            todo!()
        }
    }

    impl SeqValidateOp<IdTest,String,IdTest> for IdTest {
        fn id_of(a:&IdTest) -> Result<IdTest,String> {            
            Ok(a.clone())
        }
        fn items_count(a:&IdTest) -> Result<u32,String> {
            Ok(1u32)
        }
    }

    impl ErrThrow<IdTest,String,IdTest> for IdTest {
        fn two_heads(heads:Vec<(IdTest,IdTest)>) -> String {
            "two_heads".to_string()
        }

        fn no_heads() -> String {
            "no_heads".to_string()
        }

        fn not_found_next_log( id: &IdTest, logs:Vec<&(IdTest,IdTest)> ) -> String {
            format!("not_found_next_log id={id}")
        }
    }

    #[test]
    fn valid_seq() {
        let id0 = IdTest::new(None);
        let id1 = IdTest::new(Some(id0.id()));
        let id2 = IdTest::new(Some(id1.id()));
        let id3 = IdTest::new(Some(id2.id()));

        let logs = vec![id3.clone(), id2.clone(), id0.clone(), id1.clone()];
        match validate_sequence::<IdTest,String,IdTest,IdTest>(&logs) {
            Ok(seq) => {
                println!("ok");
                println!("tail = {}",seq.tail);
                for itm in seq.files {
                    println!(" {itm}")
                }
                assert_eq!(seq.tail, id3);
            }
            Err(err) => {
                println!("err {err}");
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

        let logs = vec![id3.clone(), id2.clone(), id0.clone(), id1.clone()];
        match validate_sequence::<IdTest,String,IdTest,IdTest>(&logs) {
            Ok(seq) => {
                println!("ok");
                println!("tail = {}",seq.tail);
                for itm in seq.files {
                    println!(" {itm}")
                }
                assert!(false);
            }
            Err(err) => {
                println!("err {err}");
                assert_eq!(err,"two_heads".to_string());
                assert!(true);
            }
        }
    }

    #[test]
    fn two_refs() {
        let id0 = IdTest::new(None);
        let id1 = IdTest::new(Some(id0.id()));
        let id2 = IdTest::new(Some(id0.id()));
        let id3 = IdTest::new(Some(id2.id()));

        let logs = vec![id3.clone(), id2.clone(), id0.clone(), id1.clone()];
        match validate_sequence::<IdTest,String,IdTest,IdTest>(&logs) {
            Ok(seq) => {
                println!("ok");
                println!("tail = {}",seq.tail);
                for itm in seq.files {
                    println!(" {itm}")
                }
                assert!(false);
            }
            Err(err) => {
                println!("err {err}");
                //assert_eq!(err,"two_heads".to_string());
                assert!(err.starts_with("not_found_next_log"));
                assert!(true);
            }
        }
    }


    #[test]
    fn no_head() {
        let id0 = IdTest::new(None);
        let id1 = IdTest::new(Some(id0.id()));
        let id2 = IdTest::new(Some(id0.id()));
        let id3 = IdTest::new(Some(id2.id()));

        let logs = vec![id1.clone(), id2.clone(), id3.clone()];
        match validate_sequence::<IdTest,String,IdTest,IdTest>(&logs) {
            Ok(seq) => {
                println!("ok");
                println!("tail = {}",seq.tail);
                for itm in seq.files {
                    println!(" {itm}")
                }
            }
            Err(err) => {
                println!("err {err}");
                assert!(false);
            }
        }
    }
}