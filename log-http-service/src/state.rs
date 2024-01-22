use std::{path::PathBuf, sync::{Mutex, Arc}};
use tokio::sync::Mutex as AMutex;

use crate::raft::RaftState;

#[derive(Clone)]
pub struct AppState {
    pub static_files: Arc<Mutex<Option<PathBuf>>>,
    pub raft: Arc<Mutex<RaftState>>
}