use std::{time::{Duration, Instant}, path::{PathBuf}, fs::File, rc::Rc, sync::Mutex};

use super::path_tmpl::{PathTemplate};

/// Генерация файла с уникальным именем
#[derive(Clone)]
pub struct NewFileGenerator<'a,F> 
where
    F: Fn(PathBuf) -> Result<File,std::io::Error>
{
    /// Функция открытия файла
    pub open: F,

    /// Шаблон для генерации пути файла
    pub path_template: PathTemplate<'a>,

    /// Максимальное время работы файла
    pub max_duration: Option<Duration>,

    /// Максимальное кол-во попыток открыть файл
    pub max_attemps: Option<u32>,

    /// Задержка перед новой попыткой открытия файла
    pub throttling: Option<Duration>,
}

/// Новый файл
#[derive(Clone)]
pub struct NewFile {
    /// Путь к открытому файлу
    pub path: PathBuf,

    /// Открытый файл
    pub file: Rc<Mutex<File>>
}

#[allow(dead_code)]
#[derive(Clone,Debug)]
pub enum NewFileGeneratorErr {
    /// Превышено максимальное кол-во попыток
    AttemptLimit {
        io_erros: Vec<String>,
    },

    /// Превышено максимальное продолжительность открытия
    DurationLimit {
        io_erros: Vec<String>,
    }
}

#[allow(dead_code)]
impl<'a,F> NewFileGenerator<'a,F>
where
    F: Fn(PathBuf) -> Result<File,std::io::Error>
{
    pub fn generate( &mut self ) -> Result<NewFile, NewFileGeneratorErr> {
        let mut attempt = 0u32;
        let started = Instant::now();
        let mut io_errors = Vec::<String>::new();

        loop {
            attempt += 1;
            let cur_time = Instant::now();

            if self.max_attemps.map(|c| attempt > c).unwrap_or(false) {
                return Err(NewFileGeneratorErr::AttemptLimit { io_erros: io_errors });
            }

            let cur_dur = cur_time.duration_since(started);
            if self.max_duration.map(|d| cur_dur > d).unwrap_or(false) {
                return Err(NewFileGeneratorErr::DurationLimit { io_erros: io_errors } );
            }

            let path_str = self.path_template.generate();
            let path = PathBuf::from(path_str);

            match (self.open)(path.clone()) {
                Ok(file) => {
                    return Ok(
                        NewFile {
                            path: path.clone(),
                            file: Rc::new(Mutex::new(file))
                        }
                    )
                }
                Err(err) => {
                    io_errors.push(err.to_string());
                    match self.throttling {
                        Some(dur) => {
                            std::thread::sleep(dur)
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

#[test]
fn new_file_test() {
    use crate::logqueue::path_tmpl::PathTemplateParser;
    use std::path::*;
    use std::fs::*;

    let test_dir = Path::new("./target/test/new_file");
    if ! test_dir.is_dir() {
        create_dir_all(test_dir).unwrap();
    } else {
        remove_dir_all(test_dir).unwrap();
        create_dir_all(test_dir).unwrap();
    }

    let path_tmpl_parser = PathTemplateParser::default();
    let mut path_tmpl = path_tmpl_parser.parse("./target/test/new_file/new_${time:local:yyyy-mm-ddThh-mi-ss}-${rnd:5}.log").unwrap();
    println!("{}", path_tmpl.generate());

    let mut new_file = NewFileGenerator {
        open: |path| { OpenOptions::new().create(true).read(true).write(true).open(path) },
        path_template: path_tmpl,
        max_duration: Some(Duration::from_secs(15)),
        max_attemps: Some(5),
        throttling: Some(Duration::from_millis(250))
    };
    //let mut new_file.

    let show_dir = || {
        println!("dir content");
        for rd in test_dir.read_dir().unwrap() {
            let rd = rd.unwrap();
            rd.file_name().to_str().map(|f|
                println!("{}", f)
            );
        }
    };

    show_dir();

    for _x in 0..5 {
        let file1 = new_file.generate();
        match file1 {
            Ok(_) => println!(""),
            Err(e) => println!("{e:?}")
        }
    }

    show_dir();
}