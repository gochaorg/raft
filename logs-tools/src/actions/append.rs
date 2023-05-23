use std::{sync::Arc, path::Path, fs::OpenOptions};
use std::io;
use std::io::prelude::*;
use logs::{perf::{Counters, Tracker}, block::BlockOptions, bbuff::absbuff::FileBuff, LogFile};

use crate::{bytesize::ByteSize, err::LogToolErr};
use crate::actions::tag::*;

const READ_BUFF_SZIE: usize = 1024 * 1024;

/// Информация о времени выполнения
pub struct AppendFileTiming {
    /// Счетчик операций
    pub log_counters: Box<Counters>,

    /// Время выполнения операции append_file
    pub append_file: Arc<Tracker>,

    /// Время выполнения оперций над логом буфера
    pub buff_tracker: Arc<Tracker>,

    /// Время выполнения оперций на логом
    pub log_tacker: Arc<Tracker>,
}

/// Применение текущего контекста к опциям блока
pub trait ApplyContext {
    fn apply(&self, b_opt: &mut BlockOptions, tags: &Vec<TagAction> ) -> Result<(),LogToolErr>;
}

impl<P2> ApplyContext for (P2, &std::fs::Metadata, &Vec<u8>) 
where
    P2: AsRef<Path> + Clone
{
    fn apply(&self, block_opt: &mut BlockOptions, tags: &Vec<TagAction> ) -> Result<(),LogToolErr> {
        let cctx = CommonContext;
        let fctx = FileContext {
            file_name: self.0.clone(),
            metadata: self.1,
            data: self.2
        };

        for tag in tags {
            match cctx.apply(block_opt, tag)? {
                TagApplyResult::Applied => {},
                TagApplyResult::Skipped => {
                    match fctx.apply(block_opt, tag)? {
                        TagApplyResult::Applied => {},
                        TagApplyResult::Skipped => {}
                    }
                }
            }
        }

        Ok(())
    }
}

impl ApplyContext for &Vec<u8>
{
    fn apply(&self, block_opt: &mut BlockOptions, tags: &Vec<TagAction> ) -> Result<(),LogToolErr> {
        let cctx = CommonContext;
        for tag in tags {
            match cctx.apply(block_opt, tag)? {
                TagApplyResult::Applied => {},
                TagApplyResult::Skipped => {}
            }
        }

        Ok(())
    }
}

/// Возвращает добавляемые данные в лог
pub trait EntryData {
    fn entry_data( self ) -> Result<Vec<u8>,LogToolErr>;
}

/// Добавляет в лог файл
///
/// Параметры
/// - `log_file_name` лог файл
/// - `entry_data` добавляемый файл / данные
/// - `block_buff_size` размер буфера записи
pub fn append_entry<P: AsRef<Path>, F, D>
(
    log_file_name: P,
    entry_data: F,
    block_buff_size: Option<ByteSize>,
    tags: &Vec<TagAction>,
) -> Result<AppendFileTiming, LogToolErr> 
where
    F: FnOnce(&Tracker) -> Result<D,LogToolErr>,
    D: EntryData + ApplyContext
{
    let mut block_opt = BlockOptions::default();

    let tracker: Tracker = Tracker::new();
    let buff = tracker.track("open log file", || FileBuff::open_read_write(log_file_name))?;
    let buff_track = buff.tracker.clone();

    let mut log = LogFile::new(buff)?;
    if block_buff_size.is_some() {
        tracker.track("resize log block buffer", || {
            log.resize_block_buffer(block_buff_size.unwrap().0)
        })
    }

    let data = entry_data(&tracker)?;
    let _ = &data.apply(&mut block_opt, tags)?;

    // append log
    let data_bytes = data.entry_data()?;
    tracker.track("log append", || {
        log.append_data(&block_opt, &data_bytes[..])
    })?;

    Ok(AppendFileTiming {
        append_file: Arc::new(tracker),
        log_counters: Box::new(log.counters.clone().read()?.clone()),
        buff_tracker: buff_track,
        log_tacker: log.tracker.clone(),
    })
}

/// Источник данных для лога - файл
pub struct EntryFile<P> 
where
    P: AsRef<Path> + Clone
{
    file_data: Vec<u8>,
    file_name: P,
    file_meta_data: std::fs::Metadata,
}

impl<P: AsRef<Path> + Clone> EntryFile<P> {
    pub fn read_file( file:P, tracker: &Tracker ) -> Result<Self,LogToolErr> {
        let (meta_data, data) = read_file(file.clone(), tracker)?;
        Ok(Self { file_data: data, file_name: file.clone(), file_meta_data: meta_data })
    }
}

impl<P: AsRef<Path> + Clone> EntryData for EntryFile<P> {
    fn entry_data( self ) -> Result<Vec<u8>,LogToolErr> {
        Ok(self.file_data)
    }
}

impl<P: AsRef<Path> + Clone> ApplyContext for EntryFile<P> {
    fn apply(&self, b_opt: &mut BlockOptions, tags: &Vec<TagAction> ) -> Result<(),LogToolErr> {
        (self.file_name.clone(),&self.file_meta_data,&self.file_data).apply(b_opt, tags)
    }
}

/// Источник данных для лога - stdin
pub struct EntryStdin {
    data: Vec<u8>
}

impl EntryStdin {
    pub fn read_stdin() -> Result<Self,LogToolErr> {
        let data = read_stdin()?;
        Ok( Self { data: data } )
    }
}

impl EntryData for EntryStdin {
    fn entry_data( self ) -> Result<Vec<u8>,LogToolErr> {
        Ok(self.data)
    }
}

impl ApplyContext for EntryStdin {
    fn apply(&self, b_opt: &mut BlockOptions, tags: &Vec<TagAction> ) -> Result<(),LogToolErr> {
        (&self.data).apply(b_opt, tags)
    }
}

fn read_stdin() -> Result<Vec<u8>,LogToolErr> {
    let mut data = Vec::<u8>::new();
    let mut stdin = io::stdin();
    
    let mut buff: [u8;1024*16] = [0;1024*16];
    loop {
        let reads = stdin.read(&mut buff)?;
        if reads == 0 {
            break;
        }else{
            data.extend_from_slice(&buff[0..reads]);
        }
    };

    Ok(data)
}

fn read_file<P: AsRef<Path>>( entry_file_name:P, tracker: &Tracker ) -> Result<(std::fs::Metadata,Vec<u8>), LogToolErr> {
    let mut entry_file = tracker.track("open data file", || {
        OpenOptions::new()
            .read(true)
            .create(false)
            .write(false)
            .open(entry_file_name)
    })?;

    let entry_file_meta = entry_file.metadata()?;

    let file_size = entry_file_meta.len();
    if file_size > u32::MAX as u64 {
        return Err(LogToolErr::FileSizeToBig);
    }

    let mut block_data = Vec::<u8>::new();
    let mut read_buff: [u8; READ_BUFF_SZIE] = [0; READ_BUFF_SZIE];

    tracker.track("read file", || {
        loop {
            match entry_file.read(&mut read_buff) {
                Err(err) => {
                    return Err(LogToolErr::from(err));
                }
                Ok(reads) => {
                    if reads == 0 {
                        break;
                    } else {
                        block_data.extend_from_slice(&read_buff[0..reads]);
                    }
                }
            }
        }
        Ok(())
    })?;

    Ok((entry_file_meta, block_data))
}
