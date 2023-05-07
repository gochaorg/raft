//! Основные функции
//! 
//! - просмотр лог файла
//!     - кол-во элементов
//!     - выгрузка указанных элементов
//!         - в файлы по шаблону
//!         - в stdout 
//!         - указание конкретных блоков через запятую
//! - добавление файла в лог

use logs::{bbuff::absbuff::*, logfile::{LogFile, LogErr}};
use std::{env, path::Path, fs::OpenOptions};

fn main() {
    //println!("Hello, world! {}", logs::add(1, 2));
    //logs::log::block

    // let args = env::args();
    // let x:Vec<String> = args.collect();
}

#[derive(Debug,Clone)]
enum LogToolErr {
    Log(LogErr),
    BuffErr(ABuffError),
    IOError { message:String, os_error:Option<i32> }
}

impl From<LogErr> for LogToolErr {
    fn from(value: LogErr) -> Self {
        Self::Log(value.clone())
    }
}

impl From<ABuffError> for LogToolErr {
    fn from(value: ABuffError) -> Self {
        Self::BuffErr(value.clone())
    }
}

impl From<std::io::Error> for LogToolErr {
    fn from(value: std::io::Error) -> Self {
        Self::IOError { message: value.to_string(), os_error: value.raw_os_error() }
    }
}

fn append_file( log_file: Box<Path>, entry_file: Box<Path> ) -> Result<(), LogToolErr> {
    let buff = FileBuff::open_read_write(log_file)?;
    let mut log = LogFile::new(buff)?;

    let entry_file = OpenOptions::new().read(true).create(false).write(false).open(entry_file)?;

    Ok(())
}
