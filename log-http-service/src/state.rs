use std::{path::PathBuf, sync::{Mutex, Arc}};

/// Состояние приложения
/// 
/// Еще состояние - очередь см [[crate::queue]]
#[derive(Clone)]
pub struct AppState {
    /// Расположение статических файлов
    pub static_files: Arc<Mutex<Option<PathBuf>>>,

    /// Состояние RAFT
    pub raft: RaftState
}

/// Состояние RAFT
#[derive(Clone)]
pub struct RaftState {
    /// ID узла
    pub id: String
}