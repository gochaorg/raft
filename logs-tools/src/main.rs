//! Основные функции
//!
//! - просмотр лог файла
//! - добавление файла в лог
//! - выгрузка файла из лога

mod bytesize;
mod err;
use bytesize::ByteSize;

use err::LogToolErr;
use logs::{
    block::{String16},
};
use std::{
    env,
    path::{PathBuf},
};

mod tag;
pub use tag::*;

mod append;
mod viewheaders;
mod extract;

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

/// Парсинг аргументов коммандной строки
fn parse_args(args: &Vec<String>) -> Box<Vec<Action>> {
    let mut actions = Vec::<Action>::new();

    let mut itr = args.iter();
    itr.next(); // skip exe

    let mut state = "state";
    let mut append_log: Box<Option<String>> = Box::new(None);
    let mut sha256 = false;
    let mut block_buff_size: Option<ByteSize> = None;
    let mut verbose: bool = false;
    let mut tags: Vec<TagAction> = vec![];

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
                } else if arg == "tag" {
                    state = "tag"
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
                    tags: tags.clone(),
                });
            }
            "view" => {
                state = "state";
                actions.push(Action::ViewHeads {
                    log_file: arg.clone(),
                    sha256: sha256,
                });
            },
            "tag" => {
                state = "state";
                match &arg[..] {
                    "clear" => {
                        tags.push(TagAction::Clear)
                    },
                    "default" => {
                        tags.push(TagAction::AddFileName { key: String16::try_from("file").unwrap() });
                        tags.push(TagAction::AddFileModifyTime { key: String16::try_from("modify_time_utc").unwrap(), format:"%Y-%m-%dT%H:%M:%S.%f".to_string() });
                        tags.push(TagAction::AddFileModifyTime { key: String16::try_from("modify_time_iso8601").unwrap(), format:"%+".to_string() });
                        tags.push(TagAction::AddFileModifyTime { key: String16::try_from("modify_time_unix").unwrap(), format:"%s".to_string() });
                    },
                    _ => {                        
                        println!("undefined tag arg: {}",arg)
                    }
                }
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
        /// Лог файл
        log_file: String,
        /// Добавляемый файл
        entry_file: String,
        /// Размер буфера записи
        block_buff_size: Option<ByteSize>,
        /// Вывести информацию о тайминге
        verbose: bool,
        /// Теги
        tags: Vec<TagAction>,
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
                tags,
            } => {
                let mut p0 = PathBuf::new();
                p0.push(log_file);

                let mut p1 = PathBuf::new();
                p1.push(entry_file);

                let timing = append::append_file(p0, p1, block_buff_size.clone(), tags)?;

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

                viewheaders::view_logfile(p0, *sha256)
            }
        }
    }
}

