use std::{path::PathBuf, sync::{Mutex, Arc}};

#[derive(Debug,Clone)]
pub struct AppState {
    pub static_files: Arc<Mutex<Option<PathBuf>>>
}