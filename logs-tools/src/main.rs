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
use parse::*;
use range::{MultipleParse, Range};
use std::{
    env,
    path::{PathBuf},
};

mod actions;
use actions::*;
mod range;
mod buildinfo;

fn main() {
    //println!("{}", buildinfo::CRATE_NAME);
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
/// 
/// Синтаксис
/// 
/// ```
/// args ::= exe_name {command_or_option} 
/// ```
/// 
/// exe_name - первый аргумент - имя exe файла
/// 
/// ```
/// command_or_option ::= option | command
/// ```
/// 
/// Опции влияют на поведение комманд.
/// Команды выполняют конкретные действия.
/// 
/// ```
/// option ::= verbose | compute_sha256_entry | block_buffer_size | tag
/// 
/// verbose ::= '-v' | '+v'
/// compute_sha256_entry ::= '-sha256' | '+sha256'
/// block_buffer_size ::= ( '-block_buffer_size' | '-bbsize' ) ByteSize
/// tag ::= 'tag' tag_action
/// tag_action ::= 'clear' | 'default' | tag_add
/// tag_add ::= ( 'add' | '+' ) tag_key tag_value
/// 
/// ByteSize ::= dec_number {white_space} [size_suffix]
/// size_suffix ::= kb | mb | gb
/// kb ::= ( 'K' | 'k' ) b
/// mb ::= ( 'M' | 'm' ) b
/// gb ::= ( 'G' | 'g' ) b
/// b = 'B' | 'b'
/// 
/// command ::= append_cmd | view_cmd | extract_cmd
/// ```
/// 
/// Комманды
/// - append_cmd - добавляет запись в лог
/// - view_cmd - просмотр заголовков записей в логе
/// - extract_cmd - извлечение записи из лога
/// 
/// ```
/// append_cmd ::= ( 'a' | 'append' ) log_file_name append_what
/// append_what ::= append_file | append_stdin
/// append_file ::= 'file' append_file_name
/// append_stdin ::= 'stdin'
/// ```
/// 
/// - log_file_name - имя лог файла
/// 
/// ```
/// view_cmd ::= ( 'v' | 'view' ) log_file_name
/// 
/// extract_cmd ::= ( 'e' | 'extract' ) log_file_name extract_selection
/// ```
/// 
/// - extract_selection - Указывает какие записи необходимо получить
/// 
/// ```
/// extract_selection ::= 'all' | 'range' range_select
/// 
/// range_select ::= range_string_arg
/// ```
/// 
/// - range_string_arg - это параметр коммандной строки, один параметр
///  если параметр должен содержать пробел, тогда параметр должен быть в кавычках
/// 
/// ```
/// range_string_arg ::= multiple 
/// 
/// multiple ::= RangeNum { delim RangeNum }
/// RangeNum ::= FromTo | Single
/// delim ::= [ WhiteSpace ] ','
/// 
/// FromTo ::= Single [ WhiteSpace ] '-' Single
/// 
/// Single ::= [ WhiteSpace ] Number
/// 
/// Number ::= hex_number | oct_number | bin_number | dec_number
/// hex_number ::= '0x' hex_digit { hex_digit }
/// hex_digit  ::= '0' | '1' | '2' | '3' | '4'
///              | '5' | '6' | '7' | '8' | '9'
///              | 'a' | 'b' | 'c' | 'd' | 'e' | 'f'
///              | 'A' | 'B' | 'C' | 'D' | 'E' | 'F'
/// 
/// oct_number ::= '0o' oct_digit { oct_digit }
/// oct_digit  ::= '0' | '1' | '2' | '3' | '4'
///              | '5' | '6' | '7'
/// 
/// bin_number ::= '0b' bin_digit { bin_digit }
/// bin_digit  ::= '0' | '1'
/// 
/// dec_number ::= dec_digit dec_digit
/// dec_digit  ::= '0' | '1' | '2' | '3' | '4'
///              | '5' | '6' | '7' | '8' | '9'
/// ```
/// 
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
                state = "append_what"
            },
            "append_what" => {
                match &arg[..] {
                    "file" => {
                        state = "append_file"
                    },
                    "stdin" => {
                        state = "state";
                        actions.push(Action::Append {
                            log_file: log_file_name.clone().unwrap().clone(),
                            entry: EntryDataSource::Stdin,
                            block_buff_size: block_buff_size,
                            verbose: verbose,
                            tags: tags.clone(),
                        });
                    },
                    _ => {
                        state = "state";
                        println!("undefined input arg {arg}, expect file <file_name> or stdin")
                    }
                }
            },
            "append_file" => {
                state = "state";
                actions.push(Action::Append {
                    log_file: log_file_name.clone().unwrap().clone(),
                    entry: EntryDataSource::File(arg.clone()),
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

#[derive(Debug, Clone)]
enum EntryDataSource {
    Stdin,
    File(String)
}

/// Операции с лог файлом
#[derive(Debug, Clone)]
enum Action {
    /// Добавление файла в лог
    Append {
        /// Лог файл
        log_file: String,

        /// Добавляемый файл/данные
        entry: EntryDataSource,

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
                entry,
                block_buff_size,
                verbose,
                tags,
            } => {
                let mut log_file_path = PathBuf::new();
                log_file_path.push(log_file);

                let timing = match entry {
                    EntryDataSource::File(file_name) => {
                        let mut entry_file_path = PathBuf::new();
                        entry_file_path.push(file_name);

                        append_entry(
                            log_file_path, 
                            |tr| EntryFile::read_file(entry_file_path, tr), 
                            *block_buff_size, 
                            tags)
                    },
                    EntryDataSource::Stdin => {
                        append_entry(
                            log_file_path, 
                            |_| EntryStdin::read_stdin(), 
                            *block_buff_size, 
                            tags)
                    }
                }?;

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

