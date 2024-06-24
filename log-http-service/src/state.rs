use std::{path::PathBuf, sync::{Arc, Mutex}, time::Duration};
use crate::raft::RaftState;

#[derive(Clone)]
pub struct AppState {    
    pub static_files: Arc<Mutex<Option<PathBuf>>>,
    pub raft: Arc<Mutex<RaftState>>,
    pub debug: Arc<Mutex<Debug>>,
}

/// Используется для отладки - только для разработки
#[derive(Clone,std::fmt::Debug)]
pub struct Debug {
    /// Задержка ответа на запрос GET http://localhost:8080/queue/version
    pub version_delay: Option<Duration>,
}

impl Default for Debug {
    fn default() -> Self {
        Self { 
            version_delay: None,
        }
    }
}
