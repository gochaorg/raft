//! Основные функции
//!
//! - просмотр лог файла
//! - добавление файла в лог
//! - выгрузка файла из лога

mod bytesize;
mod err;
use actions::tag::TagAction;
use bytesize::ByteSize;

use err::LogToolErr;
use logs::{
    block::{String16, String32, BlockId},
};
use parse::Parser;
use range::{MultipleParse, Range};
use std::{
    env,
    path::{PathBuf},
};

mod actions;
use actions::*;
mod range;
mod substr;
mod parse;

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
    let mut log_file_name: Box<Option<String>> = Box::new(None);
    let mut sha256 = false;
    let mut block_buff_size: Option<ByteSize> = None;
    let mut verbose: bool = false;
    let mut tags: Vec<TagAction> = vec![];
    let mut custom_tag_name: Option<String16> = None;

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
                } else if arg == "e" || arg == "extract" {
                    state = "extract"
                } else {
                    println!("undefined arg {arg}")
                }
            }
            "-bbsize" => {
                state = "state";
                block_buff_size = Some(ByteSize::parse(arg).unwrap())
            }
            "append" => {
                log_file_name = Box::new(Some(arg.clone()));
                state = "append_2"
            }
            "append_2" => {
                state = "state";
                actions.push(Action::Append {
                    log_file: log_file_name.clone().unwrap().clone(),
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
                    "add" | "+" => {
                        state = "tag_key";
                    },
                    _ => {                        
                        println!("undefined tag arg: {}",arg)
                    }
                }
            },
            "tag_key" => {
                custom_tag_name = Some( arg.try_into().unwrap() );
                state = "tag_value"
            },
            "tag_value" => {
                state = "state";
                tags.push( TagAction::AddTag { 
                    key: custom_tag_name.clone().unwrap(), 
                    value: String32::try_from(arg).unwrap()
                })
            },
            "extract" => {
                state = "selection";
                log_file_name = Box::new(Some(arg.to_string()));                
            },
            "selection" => {
                match &arg[..] {
                    "all" => {
                        state = "state";
                        actions.push(Action::Extract { 
                            log_file: log_file_name.clone().unwrap(), 
                            selection: ExtractSelection::All
                        });
                    },
                    "range" => {
                        state = "selection_range";
                    }
                    _ => {
                        state = "state";
                        println!("undefined selection {arg}")
                    }
                }
            },
            "selection_range" => {
                let parser = MultipleParse::new();
                match parser.parse(arg) {
                    None => {
                        state = "state";
                        println!("range {arg} not parsed");
                    },
                    Some((range_ast,_)) => {
                        state = "state";
                        let range : Range<u32> = range_ast.try_into().unwrap();
                        
                        actions.push(Action::Extract { 
                            log_file: log_file_name.clone().unwrap(), 
                            selection: ExtractSelection::Range(range) 
                        });
                    }
                }
                
            }
            _ => {}
        }
    }

    Box::new(actions)
}

/// Какие блоки выгружать
#[derive(Debug, Clone)]
enum ExtractSelection {
    /// Все
    All,

    /// Конкретные диапазоны блоков выгружать
    Range( range::Range<u32> )
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

    /// Извлечение записи из лога
    #[allow(dead_code)]
    Extract {
        /// Лог файл
        log_file: String,

        /// Какие блоки выбрать
        selection: ExtractSelection
    }
}

impl Action {
    #[allow(unused_variables)]
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

                let timing = append_file(p0, p1, block_buff_size.clone(), tags)?;

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
            Action::Extract { 
                log_file,
                selection
            } => {
                let mut log_file_path = PathBuf::new();
                log_file_path.push( log_file );

                match selection {
                    ExtractSelection::All => {
                        Err(
                            LogToolErr::NotImplemented(format!("extract all entries"))
                        )
                    },
                    ExtractSelection::Range(range) => {
                        extract::extract_to_stdout(
                            log_file_path,
                            range.clone().into_iter().map(|b_id| BlockId::new(b_id))
                        )
                    }
                }
            }
        }
    }
}

