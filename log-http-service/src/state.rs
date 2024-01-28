use std::{path::PathBuf, sync::{Arc, Mutex}, time::Duration};
use crate::raft::RaftState;

#[derive(Clone)]
pub struct AppState {
    pub static_files: Arc<Mutex<Option<PathBuf>>>,
    pub raft: Arc<Mutex<RaftState>>,
    pub debug: Arc<Mutex<Debug>>,
}

#[derive(Clone,std::fmt::Debug)]
pub struct Debug {
    pub version_delay: Option<Duration>,
}

impl Default for Debug {
    fn default() -> Self {
        Self { 
            version_delay: None,
        }
    }
}
