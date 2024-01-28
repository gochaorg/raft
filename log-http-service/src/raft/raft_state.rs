use std::time::{Duration, Instant};
use chrono::DateTime;
use chrono::Utc;
use log::info;
use log::warn;
use log::error;

use log_http_client::QueueClient;
use log_http_client::Error as ClientError;
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

#[derive(Clone)]
pub struct Node {
    pub id: String,
    pub base_address: String,
    pub hearbeat: Vec<Heartbeat>,
    pub client: QueueClient,
}

#[derive(Clone)]
pub enum Heartbeat {
    Succ { started: DateTime<Utc>, latency: Duration },
    ConnectFail { started: DateTime<Utc> },
    Timeout { started: DateTime<Utc> },
}

impl RaftState {    
    pub async fn on_timer( &mut self ) {
        info!("raft on timer");

        for node in self.nodes.iter_mut() {
            let t0 = Instant::now();
            let t0d = Utc::now();
            let _node_id: &str = &node.id;

            match node.client.version().await {
                Err(err) => match err {
                    ClientError::Timeout(err) => {
                        node.hearbeat.push(Heartbeat::Timeout {started:t0d});
                        warn!("timeout to node {_node_id} error: {err}");
                    },
                    ClientError::Connect(err) => {
                        node.hearbeat.push(Heartbeat::ConnectFail {started:t0d});
                        warn!("connect to node {_node_id} error: {err}");
                    },
                    _ => {
                        error!("some error with node {_node_id}");
                    }
                },
                Ok(_) => {
                    let t1 = Instant::now();
                    node.hearbeat.push(Heartbeat::Succ { started: t0d, latency: t1.duration_since(t0) });

                    info!("hearbeat ok, node {_node_id}");
                }
            }    

            if node.hearbeat.len() > 50 {
                let remove_count = node.hearbeat.len() - 50;
                for _ in 0..remove_count {
                    if !node.hearbeat.is_empty() {
                        node.hearbeat.remove(0);
                    }
                }
            }
        }
    }
}
