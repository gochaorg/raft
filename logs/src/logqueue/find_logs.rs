use std::path::PathBuf;
use parse::{WildcardParser, Parser, Wildcard};
use crate::fs::DirTraverse;

/// Описывает где искать логи
#[derive(Debug,Clone)]
pub struct FsLogFind {
    /// Шаблон искомого файла
    wildcard: Wildcard,

    /// Корень поиска
    root: String,

    /// Рекурсивный поиск
    recursive: bool,
}

impl IntoIterator for &FsLogFind {
    type IntoIter = Box<dyn Iterator<Item = PathBuf>>;
    type Item = PathBuf;
    fn into_iter(self) -> Self::IntoIter {
        let di = DirTraverse {
            root: self.root.clone(),
            recursive: self.recursive
        };

        let wc = self.wildcard.clone();

        let itr = 
            di.into_iter().filter(
                move |path| path.to_str().map(|str| wc.test(str)).unwrap_or(false)
            )
            ;

        let itr = Box::new(itr) as Box<dyn Iterator<Item = PathBuf>>;
        itr
    }
}

impl IntoIterator for FsLogFind {
    type IntoIter = Box<dyn Iterator<Item = PathBuf>>;
    type Item = PathBuf;
    fn into_iter(self) -> Self::IntoIter {
        (&self).into_iter()
    }
}

impl FsLogFind {    
    pub fn new( root:&str, wildcard:&str, recursive:bool ) -> Result<Self, String> {
        match WildcardParser::new().parse(wildcard) {
            Some((wc,_)) => {
                Ok( Self { wildcard: wc, root: root.to_string(), recursive: recursive } )
            },
            None => Err(format!("can't parse wildcard: \"{wildcard}\""))
        }
    }

    pub fn to_conf<ERR>( &self ) -> impl Fn() -> Result<Vec<PathBuf>,ERR> {
        let conf = self.clone();
        move || {
            let mut res : Vec<PathBuf> = vec![];
            for x in &conf {
                res.push(x)
            }
            Ok(res)
        }
    }
}

#[test]
fn log_find_test() {
    let fs_log = FsLogFind::new(".", "*log*.rs", true).unwrap();

    for path in fs_log {
        println!("{path:?}")
    }
}


