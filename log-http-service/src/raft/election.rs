use std::{marker::PhantomData, time::{Instant, Duration}, rc::Rc};

use futures::Future;

/// Ошибки
enum RErr {
    /// Нет ответа
    ReponseTimeout
}

/// Идентификатор последней записи в логе
type RID = u64;

/// Протокол взаимодействия Лидера/Leader
trait LeaderProto 
{
    type Node;
    type RPing: Future<Output = Result<RID,RErr>>;

    /// Посылка heartbeat к Follower
    fn ping( &self, follower_node:&Self::Node ) -> Self::RPing;
}

/// Протокол взаимодействия Сторонника/Follower
trait FollowerProto
{
    type RePing: Future<Output = Result<RID,RErr>>;
    fn on_ping( &self ) -> Self::RePing;
}

/// Протокол взаимодействия Кандидата/Candidate
trait CandidateProto
{
    type Node;
    type RNomination: Future<Output = Result<(),RErr>>;

    /// Выдвижение своей кандидатуры
    fn self_nominate( &self, node:&Self::Node ) -> Self::RNomination;
}

/// Заявка на лидира
struct Nomination<NodeID> {
    /// Идентификатор узла
    pub node_id: NodeID
}

/// Протокол для всех участников
trait NodeProto
{
    type NodeID;
    type ReNomination: Future<Output = Result<(),RErr>>;

    /// Получение заявки в лидеры
    fn on_nominate( &self, nomination:Nomination<Self::NodeID> ) -> Self::ReNomination;

    fn on_timer( &self );
}

/// Общий протокол
trait CommonProto
    : NodeProto
    + CandidateProto<Node = Self::CNode>
    + FollowerProto
    + LeaderProto<Node = Self::CNode>
{
    type CNode;
}

/// Роль
enum Role {
    Follower,
    Candidate,
    Leader
}

/// Узел кластера
struct ClusterNode<Node> {
    /// Идентификатор
    pub id: String,

    /// Номер эпохи
    pub epoch: u32,

    /// роль
    pub role: Role,

    /// Время последнего пинга
    pub last_ping: Option<Instant>,

    /// Ссылка на лидера
    pub lead: Option<Node>,

    /// Таймайут после которо переход в кандидата
    pub heartbeat_timeout: Duration,

    /// Минимальная задержка ответа номинанту
    pub nominate_min_delay : Duration,

    /// Минимальная задержка ответа номинанту
    pub nominate_max_delay: Duration,

    /// Остальные участники
    pub nodes: Vec<Node>,

    /// Минимальное кол-во голосов для успеха
    pub votes_min_count: u32,
}

impl FollowerProto
for ClusterNode<Rc<dyn CommonProto>>
{
}