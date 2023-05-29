use std::path::{PathBuf};
use std::fs::{read_dir};

/// Описывает корень для обхода файлов/каталогов
/// 
/// Пример
/// 
///     let di = DirTraverse {
///         root: ".".to_string(),
///         recursive: true,
///     };
///     
///     for path in di {
///         println!("{path:?}")
///     }
#[derive(Debug,Clone)]
pub struct DirTraverse {
    /// Корень поиска
    pub root: String,

    /// Рекурсивный поиск
    pub recursive: bool,
}

/// Итератор по дереву файлов
pub struct DirIterator<F1,F2> 
where 
    F1: Fn(PathBuf,std::io::Error),
    F2: Fn(std::io::Error),
{
    work_set: Vec<(u32,PathBuf)>,
    max_depth: Option<u32>,
    log_read_dir: F1,
    log_error: F2
}

impl<F1,F2> Iterator for DirIterator<F1,F2>
where 
    F1: Fn(PathBuf,std::io::Error),
    F2: Fn(std::io::Error),
{
    type Item = PathBuf;
    fn next(&mut self) -> Option<Self::Item> {
        if self.work_set.is_empty() { return None; }

        let (depth, path) = self.work_set.pop().unwrap();
        if self.max_depth.map(|max_dep| depth >= max_dep).unwrap_or(false) {
            return Some(path);
        }

        if path.is_dir() {
            match read_dir(path.clone()) {
                Ok(rd) => {
                    for entry in rd {
                        match entry {
                            Ok(de) => {
                                self.work_set.push( (depth+1,de.path()) )
                            },
                            Err(err) => {
                                (self.log_error)(err)
                            }
                        }
                    }
                },
                Err(err) => {
                    (self.log_read_dir)(path.clone(), err)
                }
            }
        }

        Some(path)
    }
}

impl IntoIterator for &DirTraverse {
    type IntoIter = Box<dyn Iterator<Item=PathBuf>>;
    type Item = PathBuf;
    fn into_iter(self) -> Self::IntoIter {
        let root = PathBuf::new().join(self.root.clone());

        let mut di = DirIterator {
            work_set: vec![(0, root.clone())],
            max_depth: Some(0),
            log_error: |_err| {},
            log_read_dir: |_dir,_err| {},
        };

        if root.is_dir() {
            di.max_depth = Some(1)
        }

        if self.recursive {
            di.max_depth = None
        }

        Box::new(di) as Box<dyn Iterator<Item = PathBuf>>
    }
}

impl IntoIterator for DirTraverse {
    type IntoIter = Box<dyn Iterator<Item=PathBuf>>;
    type Item = PathBuf;
    fn into_iter(self) -> Self::IntoIter {
        (&self).into_iter()
    }
}

#[test]
fn dir_traverse_test() {
    let di = DirTraverse {
        root: ".".to_string(),
        recursive: true,
    };

    for path in di {
        println!("{path:?}")
    }
}