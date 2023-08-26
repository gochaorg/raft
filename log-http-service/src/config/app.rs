use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf, fs};

use super::{WebServer, QueueConfig, RaftConfig};

/// Настройки приложения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Рабочий каталог
    #[serde(skip)]
    pub work_dir: String,

    /// Настройки веб сервера
    pub web_server: WebServer,

    /// Настройки очереди
    pub queue: QueueConfig,

    /// Настройки raft
    #[serde(default)]
    pub raft: RaftConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {  
            work_dir: ".".to_string(),
            web_server: WebServer::default(),
            queue: QueueConfig::default(),
            raft: RaftConfig::default(),
        }
    }
}

impl AppConfig {
    fn work_dir() -> String {
        env::current_dir().unwrap().to_str().unwrap().to_string()
    }

    fn find_file_up( from:String, name:&str ) -> Option<PathBuf> {
        let mut dir = PathBuf::from(from);
        loop {
            let file = dir.join(&name);
            if file.exists() {
                return Some(file);
            }

            match dir.parent() {
                Some(parent) => { dir = parent.to_path_buf() },
                None => { break None; }
            }
        }
    }

    pub fn find_or_default() -> Self {
        match Self::find_file_up(Self::work_dir(), "dloghw.json").and_then(|file|{
            println!("found config file {:?}", &file);
            match fs::read_to_string(file) {
                Ok(str) => {  
                    match serde_json::from_str(&str) {
                        Ok(conf) => Some(
                            AppConfig { work_dir: Self::work_dir(), ..conf }
                        ),
                        Err(err) => {
                            println!("can't read json from config file: {}", err.to_string());
                            None
                        }
                    }
                },
                Err(err) => {
                    println!("can't read file {}",err.to_string());
                    None
                }
            }
        }) {
            Some(value) => {value},
            None => {
                println!("use default config");
                Self::default()
            }
        }
    }
}

#[test]
fn test_conf() {
    AppConfig::find_or_default();
}

#[test]
fn test_json() {
    let s = serde_json::to_string_pretty( &AppConfig::default() ).unwrap();
    println!("{}",s);
}