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

