use std::{time::{Duration, Instant}, sync::Arc};
use tokio::sync::Mutex;
use super::*;

/// Роль
#[derive(Clone,Debug)]
#[allow(unused)]
pub enum Role {
    Follower,
    Candidate,
    Leader
}

/// Ошибки
#[allow(dead_code)]
pub enum RErr {
    /// Нет ответа
    ReponseTimeout,

    /// Номер эпохи не совпаддает с ожидаемым
    EpochNotMatch {
        expect: u32,
        actual: u32,
    },

    /// Уже проголосовал
    AlreadVoted {
        nominant: String
    }
}

pub type NodeID = String;
pub type EpochID = u32;
pub type RID = u32;

#[derive(Clone)]
pub struct ClusterNode
{
    /// Идентификатор
    pub id: NodeID,

    /// Номер эпохи
    pub epoch: EpochID,

    /// роль
    pub role: Role,

    /// Ссылка на лидера
    pub lead: Option<String>,

    /// Время последнего принятого пинга
    pub last_ping_recieve: Option<Instant>,

    /// Время последнего отправленного пинга
    pub last_ping_send: Option<Instant>,

    /// Период с которым рассылать пинги
    pub ping_period: Duration,

    /// Таймайут после которо переход в кандидата
    pub heartbeat_timeout: Duration,

    /// Минимальная задержка ответа номинанту
    pub nominate_min_delay : Duration,

    /// Максимальная задержка ответа номинанту
    pub nominate_max_delay: Duration,

    /// Минимальное кол-во голосов для успеха
    pub votes_min_count: u32,

    /// За кого был отдан голос в новом цикле голосования
    pub vote: Option<String>,

    /// Остальные участники
    pub nodes: Vec<Arc<Mutex<dyn NodeClient>>>,
}

pub struct NodeInstance {
    pub node: Arc<Mutex<ClusterNode>>
}

impl Clone for NodeInstance {
    fn clone(&self) -> Self {
        NodeInstance { node: self.node.clone() }
    }
}