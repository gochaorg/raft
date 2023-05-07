//! Основные функции
//! 
//! - просмотр лог файла
//!     - кол-во элементов
//!     - выгрузка указанных элементов
//!         - в файлы по шаблону
//!         - в stdout 
//!         - указание конкретных блоков через запятую
//! - добавление файла в лог

mod err;

use logs::{bbuff::absbuff::*, logfile::{LogFile, GetPointer}, block::DataId};
use std::{
    env, 
    path::{Path, PathBuf}, 
    fs::OpenOptions,
    io::prelude::*, sync::{RwLock, Arc}
};
use err::LogToolErr;

fn main() {
    let args:Vec<String> = env::args().collect();
    for action in parse_args(&args).into_iter() {
        match action.execute() {
            Ok(_) => {},
            Err(err) => {
                println!("execute {act:?} failed with {err:?}", act = action, err = err)
            }
        }
    }
}

fn parse_args( args:&Vec<String> ) -> Box<Vec<Action>> {
    let mut actions = Vec::<Action>::new();

    let mut itr = args.iter();
    itr.next(); // skip exe

    let mut state = "state";
    let mut append_log:Box<Option<String>> = Box::new(None);

    loop {
        let arg = itr.next();
        if arg.is_none() { break; }

        let arg = arg.unwrap();
        match state {
            "state" => {
                if arg == "a" {
                    state = "append"
                } else if arg == "v" {
                    state = "view"
                } else {
                    println!("undefined arg {arg}")
                }
            },
            "append" => {
                append_log = Box::new(Some(arg.clone()));
                state = "append_2"
            },
            "append_2" => {
                state = "state";
                actions.push(
                    Action::Append { 
                        log_file: append_log.clone().unwrap().clone(), 
                        entry_file: arg.clone()
                    }
                )
            },
            "view" => {
                state = "state";
                actions.push(
                    Action::ViewHeads { 
                        log_file: arg.clone()
                    }
                )
            }
            _ => {}
        }
    }

    Box::new( actions )
}

#[derive(Debug,Clone)]
enum Action {
    Append {
        log_file: String,
        entry_file: String,
    },
    ViewHeads {
        log_file: String
    }
}

impl Action {
    fn execute( &self ) -> Result<(), LogToolErr> {
        match self {
            Action::Append { log_file, entry_file } => {
                let mut p0 = PathBuf::new();
                p0.push(log_file);
                
                let mut p1 = PathBuf::new();
                p1.push(entry_file);
                
                append_file(
                    p0, 
                    p1 
                )
            },
            Action::ViewHeads { log_file } => {
                let mut p0 = PathBuf::new();
                p0.push(log_file);

                view_logfile(p0)
            }
        }
    }
}

fn append_file<P: AsRef<Path>, P2: AsRef<Path>>( log_file: P, entry_file: P2 ) -> Result<(), LogToolErr> {
    let buff = FileBuff::open_read_write(log_file)?;
    let mut log = LogFile::new(buff)?;

    let mut entry_file = OpenOptions::new().read(true).create(false).write(false).open(entry_file)?;

    let file_size = entry_file.metadata()?.len();
    if file_size > usize::MAX as u64 {
        return Err( LogToolErr::FileSizeToBig )
    }
    let file_size = file_size as usize;

    let mut block_data = Box::new(Vec::<u8>::new());
    block_data.resize(file_size, 0);
    let mut block_ptr = 0usize;

    entry_file.seek(std::io::SeekFrom::Start(0))?;
    let mut read_buff: [u8; 1024*8] = [0; 1024*8];    

    while block_ptr < file_size {
        let reads = entry_file.read(&mut read_buff)?;
        if reads==0 {
            break;
        } else {
            for i in 0 .. reads {
                block_data[block_ptr] = read_buff[i];
                block_ptr += 1;
            }
        }
    }

    log.append_data(DataId::user_data(), &block_data[0..block_data.len()])?;

    Ok(())
}

fn view_logfile<P: AsRef<Path>>( log_file: P ) -> Result<(), LogToolErr> {
    let buff = FileBuff::open_read_only(log_file)?;
    let log = LogFile::new(buff)?;
    let log = Arc::new(RwLock::new(log));

    let mut ptr = log.pointer_to_end()?;
    loop {
        let h = ptr.current_head();
        println!(
            "{b_id:0>6} {d_size:0>10}", 
            b_id   = h.head.block_id.value(),
            d_size = h.data_size.value()
        );

        match ptr.previous() {
            Ok(next_ptr) => {
                ptr = next_ptr
            },
            Err(_) => {
                break;
            }
        }
    }

    Ok(())
}