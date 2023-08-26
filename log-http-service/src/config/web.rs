use std::net::{SocketAddr, IpAddr};
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebServer {    
    /// Задает шаблон расположения каталога со статическими файлами
    pub static_files: Option<String>,

    /// Хост (ip/dns) на котором весит сервер
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

impl WebServer {
    /// Возвращает базовый адрес (URL)
    pub fn base_url( &self ) -> Result<String,String> {
        self.host.parse::<IpAddr>()
        .map_err(|e| format!("can-t parse '{}': {e:?}", self.host))
        .and_then(|addr| {
            if addr.is_multicast() || addr.is_unspecified() {
                local_ip().map_err(|e| format!("can't fetch local ip: {e:?}"))
            } else {
                Ok(addr)
            }
        })
        .and_then(|addr| {
            Ok(format!("http://{addr}:{port}", port = self.port))
        })
    }
}