use std::{sync::Arc, path::Path, fs::OpenOptions};
use logs::{perf::{Counters, Tracker}, block::BlockOptions, bbuff::absbuff::FileBuff, LogFile};
use crate::{bytesize::ByteSize, err::LogToolErr};
use std::{
    io::prelude::*,
};
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

/// Добавляет в лог файл
///
/// Параметры
/// - `log_file_name` лог файл
/// - `entry_file_name` добавляемый файл
/// - `block_buff_size` размер буфера записи
pub fn append_file<P: AsRef<Path>, P2: AsRef<Path> + Clone>(
    log_file_name: P,
    entry_file_name: P2,
    block_buff_size: Option<ByteSize>,
    tags: &Vec<TagAction>,
) -> Result<AppendFileTiming, LogToolErr> {
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

    let mut entry_file = tracker.track("open data file", || {
        OpenOptions::new()
            .read(true)
            .create(false)
            .write(false)
            .open(entry_file_name.clone())
    })?;

    ///////////////////
    // read file

    let entry_file_meta = entry_file.metadata()?;

    let file_size = entry_file_meta.len();
    if file_size > u32::MAX as u64 {
        return Err(LogToolErr::FileSizeToBig);
    }
    let file_size = file_size as usize;

    let mut block_data = Vec::<u8>::new();
    block_data.resize(file_size, 0);
    let mut block_ptr = 0usize;

    entry_file.seek(std::io::SeekFrom::Start(0))?;
    let mut read_buff: [u8; READ_BUFF_SZIE] = [0; READ_BUFF_SZIE];

    tracker.track("read file", || {
        while block_ptr < file_size {
            match entry_file.read(&mut read_buff) {
                Err(err) => {
                    return Err(LogToolErr::from(err));
                }
                Ok(reads) => {
                    if reads == 0 {
                        break;
                    } else {
                        for i in 0..reads {
                            block_data[block_ptr] = read_buff[i];
                            block_ptr += 1;
                        }
                    }
                }
            }
        }
        Ok(())
    })?;

    ////////////////
    // tags

    let cctx = CommonContext;
    let fctx = FileContext {
        file_name: entry_file_name.clone(),
        file: &entry_file,
        metadata: &entry_file_meta,
        data: &block_data
    };

    for tag in tags {
        match cctx.apply(&mut block_opt, tag)? {
            TagApplyResult::Applied => {},
            TagApplyResult::Skipped => {
                match fctx.apply(&mut block_opt, tag)? {
                    TagApplyResult::Applied => {},
                    TagApplyResult::Skipped => {}
                }
            }
        }
    }

    //////////////////
    // append log

    tracker.track("log append", || {
        log.append_data(&block_opt, &block_data[0..block_data.len()])
    })?;

    Ok(AppendFileTiming {
        append_file: Arc::new(tracker),
        log_counters: Box::new(log.counters.clone().read()?.clone()),
        buff_tracker: buff_track,
        log_tacker: log.tracker.clone(),
    })
}

// fn append_from_stdin() {
//     let mut data
//     std::io::stdin().
// }