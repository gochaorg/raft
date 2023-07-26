use std::env;

use super::AppConfig;

/// Параметры коммандной строки
#[derive(Debug,Clone,Default)]
pub struct CmdLineParams {
    /// `-host` _ip/dns_ - Переопределить хост
    pub web_server_host: Option<String>,

    /// `-port` _u16_ - Переопределить порт
    pub web_server_port: Option<u16>,

    /// `-wd` _dir_ - Переопределить текущий каталог
    pub work_dir: Option<String>,
}

impl CmdLineParams {
    /// Парсинг коммандной строки
    pub fn from_cmd_line() -> Self {
        let cmdl = CmdLineParams::default();
        env::args().fold((cmdl, "state"), |(cmdl, state), arg| {
            match state {
                "state" => {
                    match arg.as_str() {
                        "-host" => (cmdl,"-host"),
                        "-port" => (cmdl,"-port"),
                        "-work.dir" | "-wd" => (cmdl,"-wd"),
                        _ => (cmdl,state)
                    }
                },
                "-host" => ( CmdLineParams { web_server_host:Some(arg.clone()), ..cmdl }, "state" ),
                "-port" => ( CmdLineParams { web_server_port:Some(u16::from_str_radix(&arg, 10).unwrap()), ..cmdl }, "state" ),
                "-wd" => ( CmdLineParams { work_dir:Some(arg.clone()) , ..cmdl }, "state" ),
                _ => (cmdl,state)
                
            }
        }).0
    }

    /// Переопределить параметры
    pub fn apply(&self, conf:AppConfig) -> AppConfig {
        AppConfig { 
            work_dir: self.work_dir.clone().unwrap_or(conf.work_dir.clone()), 
            web_server: super::WebServer { 
                static_files: conf.web_server.static_files.clone(), 
                host: self.web_server_host.clone().unwrap_or(conf.web_server.host.clone()), 
                port: self.web_server_port.clone().unwrap_or(conf.web_server.port.clone()) 
            }, 
            queue: conf.queue.clone()
        }
    }
}