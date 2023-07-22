use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    pub find : QueueFind,
    pub new_file: QueueNewFile,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self { 
            find: QueueFind::default(), 
            new_file: QueueNewFile::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueFind {
    pub root: String,
    pub wildcard: String,
    pub recursive: bool
}

impl Default for QueueFind {
    fn default() -> Self {
        Self { 
            root: "${work.dir}/app_data/queue".to_string(),
            wildcard: "*.binlog".to_string(), 
            recursive: true 
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueNewFile {
    pub template: String
}

impl Default for QueueNewFile {
    fn default() -> Self {
        Self { 
            template: "${work.dir}/app_data/queue/${time:local:yyyy-mm-ddThh-mi-ss}-${rnd:5}.binlog".to_string() 
        }
    }
}
