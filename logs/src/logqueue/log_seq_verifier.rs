use super::log_id::*;
use crate::{
    logfile::{
        FlatBuff,
        LogFile, LogErr, block::BlockId
    }
};

trait ErrThrow<FILE,BUFF,ERR,ID> 
where
    ID:LogQueueFileId,
    BUFF:FlatBuff,
{
    fn cant_get_log_count(err:LogErr) -> ERR;
    fn read_first_block(err:LogErr) -> ERR;
    fn read_file_id(err:<ID as BlockReader>::ERR) -> ERR;
    fn two_heads(heads:Vec<(FILE,LogFile<BUFF>,ID)>) -> ERR;
    fn no_heads() -> ERR;
    fn not_found_next_log( id: &ID, logs:Vec<&(FILE,LogFile<BUFF>,ID)> ) -> ERR;
}

/// Валидация очереди логов
/// 
/// Проверки
/// 
/// - Должна быть одна голова - т.е. один id лога, который не ссылается на другие логи
/// - Остальный логи, из id должны ссылаться на логи
/// - ссылки должны образовывать линейную последовательность
fn validate_sequence<FILE,BUFF,ERR,ERRBuild,ID>() -> impl Fn( &Vec<(FILE,LogFile<BUFF>)> ) -> 
    Result<
        (FILE,LogFile<BUFF>,Vec<(FILE,LogFile<BUFF>)>)
        , ERR
    >
where
    FILE:Clone,
    BUFF:FlatBuff,
    ID: LogQueueFileId,
    ERRBuild: ErrThrow<FILE,BUFF,ERR,ID>
{
    move |files| {
        let files_wits_entries = files.iter().fold( 
            Ok::<Vec<(FILE,LogFile<BUFF>,ID)>,ERR>(vec![]), |res,(file,log)| {
                res.and_then(|mut res| {
                    match log.count() {
                        Ok(count) => if count>0u32 {
                            match log.get_block(BlockId::new(0u32)) {
                                Ok(block) => {
                                    match ID::block_read(&block) {
                                        Ok(log_id) => {
                                            res.push( (file.clone(),log.clone(),log_id.clone()) );
                                            Ok(res)
                                        }
                                        Err(err) => Err(ERRBuild::read_file_id(err))
                                    }
                                }
                                Err(err) => Err(ERRBuild::read_first_block(err))
                            }
                        } else {
                            Ok(res)
                        },
                        Err(err) => Err(ERRBuild::cant_get_log_count(err))
                    }
                })
            }
        );

        let files_with_id = files_wits_entries?;

        let head_files: Vec<(FILE,LogFile<BUFF>,ID)> = 
            files_with_id.iter().filter(|(_,_,id)| id.previous().is_none())
            .map(|(a,b,c)| (a.clone(), b.clone(), c.clone()))
            .collect();
        
        if head_files.len()>1 {
            return Err(ERRBuild::two_heads(head_files));
        } else if head_files.is_empty() {
            return Err(ERRBuild::no_heads());
        }

        let (head_file,head_log,mut head_id) = head_files.iter().map(|(a,b,c)|(a.clone(), b.clone(),c.clone())).next().unwrap();

        let mut ordered_files = vec![(head_file,head_log)];
        let mut files_with_id: Vec<&(FILE,LogFile<BUFF>,ID)> = files_with_id.iter().filter(|(_,_,id)| *id != head_id).collect();

        while ! files_with_id.is_empty() {
            match files_with_id.iter().find(|(_,_,id)| id.previous().map(|id| id == head_id.id()).unwrap_or(false) ) {
                Some((found_file,found_log,found_id)) => {
                    head_id = found_id.clone();
                    files_with_id = files_with_id.iter().filter(|(_,_,id)| id.id() != head_id.id() ).map(|x| x.clone()).collect();
                    ordered_files.push((found_file.clone(), found_log.clone()));
                }
                None => {
                    break;
                }
            }
        }

        let ordered_files: Vec<(FILE,LogFile<BUFF>)> = ordered_files.iter().map(|(a,b)|(a.clone(),b.clone())).collect();
        if !files_with_id.is_empty() {
            return Err(ERRBuild::not_found_next_log(&head_id, files_with_id));
        }

        let last = ordered_files.last().map(|i| i.clone()).unwrap();
        Ok((last.0,last.1,ordered_files))
    }
}


