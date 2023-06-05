use std::{time::{Duration, Instant}, path::{PathBuf}, fs::File, rc::Rc, sync::Mutex};

use super::path_tmpl::{PathTemplate};

/// Генерация файла с уникальным именем
#[derive(Clone)]
pub struct NewFileGenerator<'a,F> 
where
    F: Fn(PathBuf) -> Result<File,std::io::Error>
{
    /// Шаблон для генерации пути файла
    pub path_template: PathTemplate<'a>,

    /// Максимальное время работы файла
    pub max_duration: Option<Duration>,

    /// Максимальное кол-во попыток открыть файл
    pub max_attemps: Option<u32>,

    /// Функция открытия файла
    pub open: F,

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

impl<'a,F> NewFileGenerator<'a,F>
where
    F: Fn(PathBuf) -> Result<File,std::io::Error>
{
    pub fn generate( &'a mut self ) -> Result<NewFile, NewFileGeneratorErr> {
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


