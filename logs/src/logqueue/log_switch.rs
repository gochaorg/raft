use std::fmt::Debug;
#[allow(unused)]
use std::marker::PhantomData;
use crate::logfile::{LogFile, FlatBuff};

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
pub struct LogSwitcher<FILE,BUFF,LogId,FNewFile>
where
    FILE: Clone+Debug,
    BUFF: FlatBuff,
    LogId: LogQueueFileId,
    FNewFile: FnMut() -> Result<(FILE,LogFile<BUFF>),LoqErr<FILE,LogId>>,
{
    /// Создание пустого лог файла
    pub new_file: FNewFile,
}

impl<FILE,BUFF,LogId,FNewFile> LogSwitching<FILE,LogFile<BUFF>,LogId> 
for LogSwitcher<FILE,BUFF,LogId,FNewFile>
where
    FILE: Clone+Debug,
    BUFF: FlatBuff,
    LogId: LogQueueFileId,
    FNewFile: FnMut() -> Result<(FILE,LogFile<BUFF>),LoqErr<FILE,LogId>>,
{
    /// Переключение текущего лога
    fn switch<S:LogQueueState<FILE,LogFile<BUFF>,LogId>>( &mut self, log_state: &mut S ) -> Result<(),LoqErr<FILE,LogId>> {
        let mut old_file = log_state.get_current_file()?;

        let old_id = LogId::read(&old_file.0, &mut old_file.1)?;
        let new_id = LogId::new(Some(old_id.id()));

        let mut new_file = (self.new_file)()?;
        new_id.write(&new_file.0, &mut new_file.1)?;

        log_state.switch_current_file(new_file.clone())?;
        Ok(())
    }
}
