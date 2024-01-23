use std::env;

use super::{AppConfig, NodeId};

/// Параметры коммандной строки
#[derive(Debug,Clone,Default)]
pub struct CmdLineParams {
    /// `-host` _ip/dns_ - Переопределить хост
    pub web_server_host: Option<String>,

    /// `-port` _u16_ - Переопределить порт
    pub web_server_port: Option<u16>,

    /// `-wd` _dir_ - Переопределить текущий каталог
    pub work_dir: Option<String>,

    /// - `-raft.id` _str_
    /// - `-id` _str_
    /// 
    /// Указывает идентификатор raft узла
    pub raft_id: Option<String>,

    /// `-queue.new.template` _str_ - шаблон имени нового лог файла
    pub queue_new_file_template: Option<String>,

    /// `-queue.find.root` _dir_ - каталог в котором производиться поиск ранее созданых лог файлов
    pub queue_find_root: Option<String>,

    /// `-queue.find.wildcard` _str_ - шаблон искомых лог файолов
    pub queue_find_wildcard: Option<String>,

    /// `-queue.find.recursive` _bool_ - искать файлы рекурсивно
    pub queue_find_recursive: Option<bool>,

    /// `-conf` _file_ - файл конфигурации
    pub conf_file: Option<String>,
}

impl CmdLineParams {
    /// Парсинг коммандной строки
    pub fn from_cmd_line() -> Self {
        let cmdl = CmdLineParams::default();
        env::args().fold((cmdl, "state"), |(cmdl, state), arg| {
            match state {
                "state" => {
                    match arg.as_str() {
                        "-conf" => (cmdl,"-conf"),
                        "-host" => (cmdl,"-host"),
                        "-port" => (cmdl,"-port"),
                        "-work.dir" | "-wd" => (cmdl,"-wd"),
                        "-raft.id" | "-id" => (cmdl, "-raft.id"),
                        "-queue.new.template" => (cmdl,"-queue.new.template"),
                        "-queue.find.root" => (cmdl,"-queue.find.root"),
                        "-queue.find.wildcard" => (cmdl,"-queue.find.wildcard"),
                        "-queue.find.recursive" => (cmdl,"-queue.find.recursive"),
                        _ => (cmdl,state)
                    }
                },
                "-conf" => ( CmdLineParams { conf_file:Some(arg.clone()), ..cmdl }, "state" ),
                "-host" => ( CmdLineParams { web_server_host:Some(arg.clone()), ..cmdl }, "state" ),
                "-port" => ( CmdLineParams { web_server_port:Some(u16::from_str_radix(&arg, 10).unwrap()), ..cmdl }, "state" ),
                "-wd" => ( CmdLineParams { work_dir:Some(arg.clone()) , ..cmdl }, "state" ),   
                "-raft.id" => ( CmdLineParams { raft_id:Some(arg.clone()), .. cmdl}, "state" ),
                "-queue.new.template" => ( CmdLineParams { queue_new_file_template:Some(arg.clone()), .. cmdl} ,"state"),
                "-queue.find.root" => ( CmdLineParams { queue_find_root:Some(arg.clone()), .. cmdl} ,"state" ),
                "-queue.find.wildcard" => ( CmdLineParams { queue_find_wildcard:Some(arg.clone()), .. cmdl} ,"state" ),
                "-queue.find.recursive" => ( CmdLineParams { queue_find_recursive:match arg.as_str() { "true" => Some(true), "false" => Some(false), _ => None}, .. cmdl} ,"state" ),
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
                port: self.web_server_port.clone().unwrap_or(conf.web_server.port.clone()),
            }, 
            queue: super::QueueConfig { 
                find: super::QueueFind { 
                    root: self.queue_find_root.clone().unwrap_or(conf.queue.find.root.clone()), 
                    wildcard: self.queue_find_wildcard.clone().unwrap_or(conf.queue.find.wildcard.clone()), 
                    recursive: self.queue_find_recursive.clone().unwrap_or(conf.queue.find.recursive.clone()), 
                }, 
                new_file: super::QueueNewFile { 
                    template: self.queue_new_file_template.clone().unwrap_or(conf.queue.new_file.template.clone()),
                },
            },
            raft: super::RaftConfig {
                id: self.raft_id.clone().map(|c| NodeId::Name(c)).unwrap_or(conf.raft.id.clone()),
                .. conf.raft
            },
        }
    }
}