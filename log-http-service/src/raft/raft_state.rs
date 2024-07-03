use std::time::{Duration, Instant};
use chrono::DateTime;
use chrono::Utc;
use log::info;
use log::warn;
use log::error;

use log_http_client::QueueClient;
use log_http_client::Error as ClientError;
use serde::Deserialize;
use serde::Serialize;
use crate::queue_api::ApiErr;

use super::bg_tasks::job;
use super::log_shipping::*;
use super::Role;

/// Состояние сервера
pub struct RaftState
{
    /// Идентификатор узла
    pub id: String,

    /// Фоновая задача выполняемая по таймеру
    pub bg_job : Option<Box<dyn job::Job + Send + Sync>>,

    /// Список узлов
    pub nodes : Vec<Node>,

    /// Кто текущий мастер
    pub master: Option<MasterNode>,

    /// Роль текущего сервера
    pub role: Option<Role>,
}

impl RaftState {
    pub fn new( id:String ) -> Self {
        Self {
            id: id,
            bg_job: None,
            nodes: Vec::default(),
            master: None,
            role: None,
        }
    }

    pub fn find_node( &self, node_id: &str ) -> Result<&Node, ApiErr> {
        match self.nodes.iter().find(|n| n.id == node_id ) {
            None => Err(ApiErr::BadRequest(format!("Not found {}", node_id))),
            Some(node) => Ok(node)
        }
    }
}

/// Узел
#[derive(Clone)]
pub struct Node {
    /// Идентификатор узла
    pub id: String,

    /// Базовый адрес узла
    pub base_address: String,

    /// Доступность узла
    pub hearbeat: Vec<Heartbeat>,

    /// Клиент 
    pub client: QueueClient,
    
    /// Задачи доставки логов
    pub log_shipping: LogShipping,
}

impl Node {
    pub fn new( id: String, base_address: String, client: QueueClient ) -> Self {
        Self {
            id: id,
            base_address: base_address,
            hearbeat: Vec::new(),
            client: client,
            log_shipping: LogShipping::default(),
        }
    }
}
 

/// Мастер узел
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct MasterNode {
    /// Идентификатор ведущего узла
    pub id: String,

    /// Адрес ведущего узла
    pub base_address: String,
}

/// Доступность на конкретный момент времени
#[derive(Clone)]
pub enum Heartbeat {
    /// Доступен
    Succ { 
        /// Время запроса
        started: DateTime<Utc>, 

        /// Задержка ответа
        latency: Duration 
    },

    /// Недоступен, порт закрыт
    ConnectFail { started: DateTime<Utc> },

    /// Недоступен, нет ответа
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
