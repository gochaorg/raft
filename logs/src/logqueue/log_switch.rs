#[allow(unused)]
use std::marker::PhantomData;
use super::log_id::*;

/// Состояние очереди
pub trait LogQueueState<FILE> {
    type ERR;

    /// Возвращает текущий лог файл
    fn get_current_file( &self ) -> Result<FILE,Self::ERR>;

    /// Указыавет новый лог файл
    fn switch_current_file( &mut self, new_file: FILE ) -> Result<(),Self::ERR>;
}

/// Информация о старом и новом id
pub struct OldNewId<'a, ID> {
    pub old_id: &'a ID,
    pub new_id: &'a ID,
}

/// Переключение текущего лог файла на новый
pub trait LogSwitching<FILE,ERR> 
{
    /// Переключение лог файла
    fn switch<S:LogQueueState<FILE,ERR = ERR>>( &mut self, log_state: &mut S ) -> Result<(),ERR>;
}

/// Переключение лог файла
#[derive(Clone)]
pub struct LogSwitcher<FILE,ID,ERR,FReadId,FWriteId,FNewFile>
where
    FILE: Clone,
    ID: LogQueueFileId,
    FReadId: Fn(&FILE) -> Result<ID,ERR>,
    FWriteId: for <'a> Fn(&mut FILE, OldNewId<'a,ID>) -> Result<(),ERR>,
    FNewFile: FnMut() -> Result<FILE,ERR>,
{
    /// Чтение id лог файла
    pub read_id_of: FReadId,

    /// Запись id в лог файл
    pub write_id_to: FWriteId,

    /// Создание пустого лог файла
    pub new_file: FNewFile,
}

impl<FILE,ERR,ID,FReadId,FWriteId,FNewFile> LogSwitching<FILE,ERR> 
for LogSwitcher<FILE,ID,ERR,FReadId,FWriteId,FNewFile>
where
    FILE: Clone,
    ID: LogQueueFileId,
    FReadId: Fn(&FILE) -> Result<ID,ERR>,
    FWriteId: for <'a> Fn(&mut FILE, OldNewId<'a,ID>) -> Result<(),ERR>,
    FNewFile: FnMut() -> Result<FILE,ERR>,
{
    /// Переключение текущего лога
    fn switch<S:LogQueueState<FILE,ERR = ERR>>( &mut self, log_state: &mut S ) -> Result<(),ERR> {
        let old_file = log_state.get_current_file()?;
        let old_id = (self.read_id_of)(&old_file)?;
        let new_id = ID::new(Some(old_id.id()));
        let ids = OldNewId { old_id:&old_id, new_id:&new_id };

        let mut new_file = (self.new_file)()?;
        (self.write_id_to)(&mut new_file, ids)?;
        
        log_state.switch_current_file(new_file.clone())?;
        Ok(())
    }
}
