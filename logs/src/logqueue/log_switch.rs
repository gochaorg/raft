use std::fmt::Debug;
#[allow(unused)]
use std::marker::PhantomData;
use super::{log_id::*, LoqErr};

/// Состояние очереди
pub trait LogQueueState<FILE,LOG,LogId> 
where
    FILE: Clone+Debug,
    LogId: Clone+Debug,
{
    /// Возвращает текущий лог файл
    fn get_current_file( &self ) -> Result<(FILE,LOG),LoqErr<FILE,LogId>>;

    /// Указыавет новый лог файл
    fn switch_current_file( &mut self, new_file: (FILE,LOG) ) -> Result<(),LoqErr<FILE,LogId>>;
}

/// Информация о старом и новом id
pub struct OldNewId<'a, ID> {
    pub old_id: &'a ID,
    pub new_id: &'a ID,
}

/// Переключение текущего лог файла на новый
pub trait LogSwitching<FILE,LOG,LogId>
where
    FILE:  Clone+Debug,
    LogId: Clone+Debug,
{
    /// Переключение лог файла
    fn switch<S:LogQueueState<FILE,LOG,LogId>>( &mut self, log_state: &mut S ) -> Result<(),LoqErr<FILE,LogId>>;
}

/// Переключение лог файла
#[derive(Clone)]
pub struct LogSwitcher<FILE,LOG,LogId,FReadId,FWriteId,FNewFile>
where
    FILE: Clone+Debug,
    LOG: Clone,
    LogId: LogQueueFileId,
    FReadId: Fn(&(FILE,LOG)) -> Result<LogId,LoqErr<FILE,LogId>>,
    FWriteId: for <'a> Fn(&mut (FILE,LOG), OldNewId<'a,LogId>) -> Result<(),LoqErr<FILE,LogId>>,
    FNewFile: FnMut() -> Result<(FILE,LOG),LoqErr<FILE,LogId>>,
{
    /// Чтение id лог файла
    pub read_id_of: FReadId,

    /// Запись id в лог файл
    pub write_id_to: FWriteId,

    /// Создание пустого лог файла
    pub new_file: FNewFile,
}

impl<FILE,LOG,LogId,FReadId,FWriteId,FNewFile> LogSwitching<FILE,LOG,LogId> 
for LogSwitcher<FILE,LOG,LogId,FReadId,FWriteId,FNewFile>
where
    FILE: Clone+Debug,
    LOG: Clone,
    LogId: LogQueueFileId,
    FReadId: Fn(&(FILE,LOG)) -> Result<LogId,LoqErr<FILE,LogId>>,
    FWriteId: for <'a> Fn(&mut (FILE,LOG), OldNewId<'a,LogId>) -> Result<(),LoqErr<FILE,LogId>>,
    FNewFile: FnMut() -> Result<(FILE,LOG),LoqErr<FILE,LogId>>,
{
    /// Переключение текущего лога
    fn switch<S:LogQueueState<FILE,LOG,LogId>>( &mut self, log_state: &mut S ) -> Result<(),LoqErr<FILE,LogId>> {
        let old_file = log_state.get_current_file()?;
        
        let old_id = (self.read_id_of)(&old_file)?;
        let new_id = LogId::new(Some(old_id.id()));

        let ids = OldNewId { old_id:&old_id, new_id:&new_id };

        let mut new_file = (self.new_file)()?;
        (self.write_id_to)(&mut new_file, ids)?;
        
        log_state.switch_current_file(new_file.clone())?;
        Ok(())
    }
}
