use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    pub find : QueueFind,
    pub new_file: QueueNewFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueFind {
    pub root: String,
    pub wildcard: String,
    pub recursive: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueNewFile {
    pub template: String
}

