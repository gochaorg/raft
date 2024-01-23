use std::sync::Arc;
use log::info;
use super::bg_tasks::job;

/// Состояние сервера
pub struct RaftState
{
    pub bg_job : Option<Box<dyn job::Job + Send + Sync>>,
    pub nodes : Vec<Node>,
}

impl Default for RaftState {
    fn default() -> Self {
        Self { bg_job: Default::default(), nodes: Default::default() }
    }
}

pub struct Node {
}

impl RaftState {    
    pub async fn on_timer( &self ) {
        info!("raft on timer")
    }
}
