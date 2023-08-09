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
#[derive(Debug,Clone)]
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

    // Нормер эпохи самовыдвижения
    pub epoch_of_candidate: Option<EpochID>,

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

    /// Минимальная задержка перед повтором самовыдвижения
    pub renominate_min_delay: Duration,

    /// Максимальная задержка перед повтором самовыдвижения
    pub renominate_max_delay: Duration,

    /// Минимальное кол-во голосов для успеха
    pub votes_min_count: u32,

    /// За кого был отдан голос в новом цикле голосования
    pub vote: Option<NodeID>,

    /// Остальные участники
    pub nodes: Vec<Arc<AsyncMutex<dyn NodeClient>>>,
}

/// Уведомление о иземении состояния узла
#[allow(unused_variables)]
pub trait NodeLogging:Clone {
    fn on_ping( &self, leader:NodeID, epoch:EpochID, rid:RID ) {}
    fn on_ping_leader_match( &self ) {}
    fn on_ping_epoch_greater( &self ) {}
    fn on_ping_epoch_less_or_equals( &self ) {}
    fn on_ping_leader_self( &self ) {}
    fn on_ping_leader_different( &self ) {}

    fn change_role( &self, from:Role, to:Role ) {}
    fn change_id( &self, from:Option<NodeID>, to:Option<NodeID> ) {}
    fn change_last_ping_recieve( &self, from:Option<Instant>, to:Option<Instant> ) {}
    fn change_last_ping_send( &self, from:Option<Instant>, to:Option<Instant> ) {}
    fn change_epoch( &self, from:EpochID, to:EpochID ) {}
    fn change_vote( &self, from:Option<NodeID>, to:Option<NodeID> ) {}
    fn change_leader( &self, from:Option<NodeID>, to:Option<NodeID> ) {}
}

#[derive(Debug,Clone)]
pub struct DummyNodeChanges ();

impl NodeLogging for DummyNodeChanges {
}

pub struct NodeInstance<NC: NodeLogging> {
    pub node: Arc<AsyncMutex<ClusterNode>>,
    pub changes: NC,
}

impl<NC:NodeLogging> Clone for NodeInstance<NC> {
    fn clone(&self) -> Self {
        NodeInstance { node: self.node.clone(), changes: self.changes.clone() }
    }
}