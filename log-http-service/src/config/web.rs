use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebServer {    
    /// Задает шаблон расположения каталога со статическими файлами
    pub static_files: Option<String>,

    /// Хост на котором весит сервер
    pub host: String,

    /// Порт на котором весит сервер
    pub port: u16,
}

impl Default for WebServer {
    fn default() -> Self {
        Self { 
            static_files: None,
            host: "127.0.0.1".to_string(), 
            port: 8080 
        }
    }
}