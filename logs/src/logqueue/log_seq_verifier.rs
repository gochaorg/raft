use super::log_id::*;

/// Генерация ошибки
pub trait ErrThrow<ITEM,ERR,ID> 
where
    ID:LogQueueFileId,
{
    /// Найдено 2 или более файла, которые могут быть началом (головой) очереди логов
    /// Должен быть один
    fn two_heads(heads:Vec<(ITEM,ID)>) -> ERR;

    /// Не найдено начала очереди
    fn no_heads() -> ERR;

    /// Битая ссылка, голова лог файла ссылается на не существующий лог файл
    fn not_found_next_log( id: &ID, logs:Vec<&(ITEM,ID)> ) -> ERR;
}

/// Извлечение и лог файла - идентификатора
pub trait IdOf<A,ID,ERR> {
    /// получить идентификатор-ссылку
    fn id_of(a:&A) -> Result<ID,ERR>;
}

/// Операции с лог файлов
pub trait SeqValidateOp<A,ERR,ID>: IdOf<A,ID,ERR>
where
    ID:LogQueueFileId
{
    /// кол-во элементов в логе
    fn items_count(a:&A) -> Result<u32,ERR>;

    // получить идентификатор-ссылку
    //fn id_of(a:&A) -> Result<ID,ERR>;
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
pub fn validate_sequence<ITEM,ERR,ERRBuild,ID>( files: &Vec<ITEM> ) -> 
    Result<OrderedLogs<ITEM>,ERR>
where
    ITEM: Clone + SeqValidateOp<ITEM,ERR,ID>,
    ID: LogQueueFileId,
    ERRBuild: ErrThrow<ITEM,ERR,ID>
{
    // Выделение id файлов
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

    let mut head_files: Vec<(ITEM,ID)> = 
        files_with_id.iter().filter(|(_,id)| id.previous().is_none())
        .map(|(a,b)| (a.clone(), b.clone()))
        .collect();
    
    if head_files.len()>1 {
        return Err(ERRBuild::two_heads(head_files));
    } else if head_files.is_empty() {
        // Найти те что ссылается на не существующую id
        let mut ids = std::collections::HashSet::<ID>::new();
        for (_,id) in &files_with_id {
            ids.insert(id.clone());
        }

        // обходим список
        // ищем потенциальную голову
        let mut files_set = files_with_id.clone();        
        let mut heads = Vec::<(ITEM,ID)>::new();
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
            return Err(ERRBuild::no_heads());
        } else if heads.len()>1 {
            return Err(ERRBuild::two_heads(heads));
        }

        for (f,i) in heads {
            head_files.push((f.clone(), i.clone()));
        }
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

    let ordered_files: Vec<ITEM> = ordered_files.iter().map(|a|a.clone()).collect();
    if !files_with_id.is_empty() {
        return Err(ERRBuild::not_found_next_log(&head_id, files_with_id));
    }

    let last = ordered_files.last().map(|i| i.clone()).unwrap();
    Ok(OrderedLogs{files: ordered_files, tail:last})
}

pub mod test {
    use uuid::Uuid;
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
        fn block_write( &self, _block: &mut crate::logfile::block::Block ) -> Result<(),Self::ERR> {
            Ok(())
        }
    }
    impl BlockReader for IdTest {
        type ERR = String;
        fn block_read( _block: &crate::logfile::block::Block ) -> Result<Self, Self::ERR> {
            todo!()
        }
    }

    impl SeqValidateOp<IdTest,String,IdTest> for IdTest {
        fn items_count(_a:&IdTest) -> Result<u32,String> {
            Ok(1u32)
        }
    }

    impl IdOf<IdTest,IdTest,String> for IdTest {
        fn id_of(a:&IdTest) -> Result<IdTest,String> {
            Ok(a.clone())
        }
    }

    impl ErrThrow<IdTest,String,IdTest> for IdTest {
        fn two_heads(_heads:Vec<(IdTest,IdTest)>) -> String {
            "two_heads".to_string()
        }

        fn no_heads() -> String {
            "no_heads".to_string()
        }

        fn not_found_next_log( id: &IdTest, _logs:Vec<&(IdTest,IdTest)> ) -> String {
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
    fn partial_queue() {
        let id0 = IdTest::new(None);
        let id1 = IdTest::new(Some(id0.id()));
        let id2 = IdTest::new(Some(id1.id()));
        let id3 = IdTest::new(Some(id2.id()));

        let logs = vec![id1.clone(), id2.clone(), id3.clone()];
        match validate_sequence::<IdTest,String,IdTest,IdTest>(&logs) {
            Ok(seq) => {
                println!("ok");
                println!("tail = {}",seq.tail);
                for itm in &seq.files {
                    println!(" {itm}")
                }
                assert!( seq.files.len()==3 );
                assert!( seq.files[0].id == id1.id() );
                assert!( seq.files[1].id == id2.id() );
                assert!( seq.files[2].id == id3.id() );
            }
            Err(err) => {
                println!("err {err}");
                assert!(false);
            }
        }
    }
}