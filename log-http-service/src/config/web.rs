use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebServer {    
    pub static_files: Option<String>,
    pub host: String,
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