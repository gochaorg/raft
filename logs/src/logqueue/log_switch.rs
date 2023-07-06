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
pub struct OleNewId<'a, ID> {
    pub old_id: &'a ID,
    pub new_id: &'a ID,
}

pub trait LogSwitching<FILE,ERR,ID> : LogQueueState<FILE,ERR = ERR>
where
    ID: LogQueueFileId,
{
    fn switch( &mut self ) -> Result<(),ERR> {
        let old_file = self.get_current_file()?;
        let old_id = (self.read_id_of)(&old_file)?;

        let new_id = ID::new(Some(old_id.id()));
        let ids = OleNewId { old_id:&old_id, new_id:&new_id };

        let mut new_file = (self.new_file)()?;
        (self.write_id_to)(&mut new_file, ids)?;
        
        self.switch_current_file(new_file.clone())?;
        Ok(())
    }
}

/// Переключение лог файла
#[derive(Clone)]
pub struct LogSwitcher<FILE,ID,ERR,FReadId,FWriteId,FNewFile>
where
    FILE: Clone,
    ID: LogQueueFileId,
    FReadId: Fn(&FILE) -> Result<ID,ERR>,
    FWriteId: for <'a> Fn(&mut FILE, OleNewId<'a,ID>) -> Result<(),ERR>,
    FNewFile: FnMut() -> Result<FILE,ERR>,
{
    /// Чтение id лог файла
    pub read_id_of: FReadId,

    /// Запись id в лог файл
    pub write_id_to: FWriteId,

    /// Создание пустого лог файла
    pub new_file: FNewFile,

    pub _p : PhantomData<(FILE,ID,ERR)>
}

impl<FILE,ID,ERR,FReadId,FWriteId,FNewFile> LogSwitching<FILE,ERR,ID> 
for LogSwitcher<FILE,ID,ERR,FReadId,FWriteId,FNewFile> 
where
    Self: LogQueueState<FILE,ERR = ERR>,
    FILE: Clone,
    ID: LogQueueFileId,
    FReadId: Fn(&FILE) -> Result<ID,ERR>,
    FWriteId: for <'a> Fn(&mut FILE, OleNewId<'a,ID>) -> Result<(),ERR>,
    FNewFile: FnMut() -> Result<FILE,ERR>,
{

}
