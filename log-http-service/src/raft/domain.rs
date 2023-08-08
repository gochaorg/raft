use std::{time::{Duration, Instant}, sync::Arc};
use tokio::sync::Mutex as AsyncMutex;
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
    pub lead: Option<NodeID>,

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
    pub vote: Option<NodeID>,

    /// Остальные участники
    pub nodes: Vec<Arc<AsyncMutex<dyn NodeClient>>>,
}

/// Уведомление о иземении состояния узла
pub trait NodeChanges:Clone {
    fn role( &self, from:Role, to:Role ) {}
    fn id( &self, from:Option<NodeID>, to:Option<NodeID> ) {}
    fn last_ping_recieve( &self, from:Option<Instant>, to:Option<Instant> ) {}
    fn last_ping_send( &self, from:Option<Instant>, to:Option<Instant> ) {}
    fn epoch( &self, from:EpochID, to:EpochID ) {}
    fn vote( &self, from:Option<NodeID>, to:Option<NodeID> ) {}
}

#[derive(Debug,Clone)]
pub struct DummyNodeChanges ();

impl NodeChanges for DummyNodeChanges {
}

pub struct NodeInstance<NC: NodeChanges> {
    pub node: Arc<AsyncMutex<ClusterNode>>,
    pub changes: NC,
}

impl<NC:NodeChanges> Clone for NodeInstance<NC> {
    fn clone(&self) -> Self {
        NodeInstance { node: self.node.clone(), changes: self.changes.clone() }
    }
}