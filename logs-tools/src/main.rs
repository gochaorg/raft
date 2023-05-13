//! Основные функции
//!
//! - просмотр лог файла
//!     - кол-во элементов
//!     - выгрузка указанных элементов
//!         - в файлы по шаблону
//!         - в stdout
//!         - указание конкретных блоков через запятую
//! - добавление файла в лог

mod bytesize;
mod err;
use bytesize::ByteSize;

use chrono::{DateTime, Utc};
use err::LogToolErr;
use logs::{
    bbuff::absbuff::*,
    block::{BlockOptions, String16},
    logfile::{GetPointer, LogFile},
    perf::{Counters, Tracker},
};
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::OpenOptions,
    io::prelude::*,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

mod tag;
pub use tag::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    for action in parse_args(&args).into_iter() {
        match action.execute() {
            Ok(_) => {}
            Err(err) => {
                println!(
                    "execute {act:?} failed with {err:?}",
                    act = action,
                    err = err
                )
            }
        }
    }
}

fn parse_args(args: &Vec<String>) -> Box<Vec<Action>> {
    let mut actions = Vec::<Action>::new();

    let mut itr = args.iter();
    itr.next(); // skip exe

    let mut state = "state";
    let mut append_log: Box<Option<String>> = Box::new(None);
    let mut sha256 = false;
    let mut block_buff_size: Option<ByteSize> = None;
    let mut verbose: bool = false;

    loop {
        let arg = itr.next();
        if arg.is_none() {
            break;
        }

        let arg = arg.unwrap();
        match state {
            "state" => {
                if arg == "a" || arg == "append" {
                    state = "append"
                } else if arg == "v" || arg == "view" {
                    state = "view"
                } else if arg == "+sha256" {
                    sha256 = true
                } else if arg == "-sha256" {
                    sha256 = false
                } else if arg == "+v" {
                    verbose = true
                } else if arg == "-v" {
                    verbose = false
                } else if arg == "-bbsize" || arg == "-block_buffer_size" {
                    state = "-bbsize";
                } else {
                    println!("undefined arg {arg}")
                }
            }
            "-bbsize" => {
                state = "state";
                block_buff_size = Some(ByteSize::parse(arg).unwrap())
            }
            "append" => {
                append_log = Box::new(Some(arg.clone()));
                state = "append_2"
            }
            "append_2" => {
                state = "state";
                actions.push(Action::Append {
                    log_file: append_log.clone().unwrap().clone(),
                    entry_file: arg.clone(),
                    block_buff_size: block_buff_size,
                    verbose: verbose,
                })
            }
            "view" => {
                state = "state";
                actions.push(Action::ViewHeads {
                    log_file: arg.clone(),
                    sha256: sha256,
                })
            }
            _ => {}
        }
    }

    Box::new(actions)
}

/// Операции с лог файлом
#[derive(Debug, Clone)]
enum Action {
    /// Добавление файла в лог
    Append {
        log_file: String,
        entry_file: String,
        block_buff_size: Option<ByteSize>,
        verbose: bool,
    },
    /// Просмотр заголовков лог файла
    ViewHeads { log_file: String, sha256: bool },
}

impl Action {
    fn execute(&self) -> Result<(), LogToolErr> {
        match self {
            Action::Append {
                log_file,
                entry_file,
                block_buff_size,
                verbose,
            } => {
                let mut p0 = PathBuf::new();
                p0.push(log_file);

                let mut p1 = PathBuf::new();
                p1.push(entry_file);

                let timing = append_file(p0, p1, block_buff_size.clone())?;

                if *verbose {
                    println!("log counters:\n{}", timing.log_counters);
                    println!("append file tracks:\n{}", timing.append_file);
                    println!("buff tracks:\n{}", timing.buff_tracker);
                    println!("log tracks:\n{}", timing.log_tacker);
                }

                Ok(())
            }
            Action::ViewHeads { log_file, sha256 } => {
                let mut p0 = PathBuf::new();
                p0.push(log_file);

                view_logfile(p0, *sha256)
            }
        }
    }
}

const READ_BUFF_SZIE: usize = 1024 * 1024;

/// Информация о времени выполнения
struct AppendFileTiming {
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
/// - `log_file` лог файл
/// - `entry_file` добавляемый файл
/// - `block_buff_size` размер буфера записи
fn append_file<P: AsRef<Path>, P2: AsRef<Path>>(
    log_file: P,
    entry_file: P2,
    block_buff_size: Option<ByteSize>,
) -> Result<AppendFileTiming, LogToolErr> {
    let mut block_opt = BlockOptions::default();
    match entry_file.as_ref().to_str() {
        Some(file_name) => block_opt.set("file", file_name)?,
        None => {}
    }

    let tracker: Tracker = Tracker::new();

    let buff = tracker.track("open log file", || FileBuff::open_read_write(log_file))?;

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
            .open(entry_file)
    })?;

    let entry_file_meta = entry_file.metadata()?;

    let last_mod_time = entry_file_meta.modified()?;
    let last_mod_time: DateTime<Utc> = last_mod_time.into();
    block_opt.set(
        "modify_time_utc",
        last_mod_time.format("%Y-%m-%dT%H:%M:%S.%f").to_string(),
    )?;
    block_opt.set(
        "modify_time_iso8601",
        last_mod_time.format("%+").to_string(),
    )?;
    block_opt.set("modify_time_unix", last_mod_time.format("%s").to_string())?;

    ///////////////////
    // read file

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

fn view_logfile<P: AsRef<Path>>(log_file: P, sha256: bool) -> Result<(), LogToolErr> {
    let buff = FileBuff::open_read_only(log_file)?;
    let log = LogFile::new(buff)?;
    let log = Arc::new(RwLock::new(log));

    let mut ptr = log.pointer_to_end()?;
    loop {
        let h = ptr.current_head();
        print!(
            "{b_id:0>6} {d_size:0>10}",
            b_id = h.head.block_id.value(),
            d_size = h.data_size.value()
        );

        if sha256 {
            match ptr.current_data() {
                Ok(data) => {
                    let mut hasher = Sha256::new();
                    hasher.update(*data);
                    let hash = hasher.finalize();
                    let hash = hex::encode(hash);
                    print!(" {hash}");
                }
                Err(err) => {
                    print!("can't read data {err:?}")
                }
            }
        }

        //print!("{:?}", h);

        if !h.head.block_options.is_empty() {
            for (key, value) in h.head.block_options.clone() {
                print!(" {key}={value}")
            }
        }

        println!();

        match ptr.previous() {
            Ok(next_ptr) => ptr = next_ptr,
            Err(_) => {
                break;
            }
        }
    }

    Ok(())
}
