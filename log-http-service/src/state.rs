use std::{path::PathBuf, sync::{Mutex, Arc, RwLock}};

use logs::{logqueue::{LogFileQueue, LogQueueFileNumID}, logfile::LogFile, bbuff::absbuff::FileBuff};

#[derive(Clone)]
pub struct AppState {
    pub static_files: Arc<Mutex<Option<PathBuf>>>,
}