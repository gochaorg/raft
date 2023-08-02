use std::{marker::PhantomData, time::{Instant, Duration}, rc::Rc, sync::{Arc, RwLock, Mutex, Weak}};
use actix_rt::{spawn, time::sleep, System};
use async_trait::async_trait;

use futures::{Future, future::join_all, SinkExt};
use log::{info};

/// Ошибки
enum RErr {
    /// Нет ответа
    ReponseTimeout
}

/// Идентификатор последней записи в логе
type RID = u64;

/// Роль
#[derive(Clone)]
enum Role {
    Follower,
    Candidate,
    Leader
}

/// Узел кластера
#[derive(Clone)]
struct ClusterNode 
{
    /// Идентификатор
    id: String,

    /// Номер эпохи
    epoch: u32,

    /// роль
    role: Role,

    /// Время последнего принятого пинга
    last_ping_recieve: Option<Instant>,

    /// Время последнего отправленного пинга
    last_ping_send: Option<Instant>,

    /// Период с которым рассылать пинги
    ping_period: Duration,

    /// Ссылка на лидера
    lead: Option<String>,

    /// Таймайут после которо переход в кандидата
    heartbeat_timeout: Duration,

    /// Минимальная задержка ответа номинанту
    nominate_min_delay : Duration,

    /// Минимальная задержка ответа номинанту
    nominate_max_delay: Duration,

    /// Остальные участники
    // nodes: Arc<Mutex<Vec<Weak<dyn NodeClient>>>>,
    nodes: Arc<Mutex<Vec<Box<dyn NodeClient>>>>,

    /// Минимальное кол-во голосов для успеха
    votes_min_count: u32,
}

impl ClusterNode {
    fn new(id:&str) -> ClusterNode {
        ClusterNode { 
            id: id.to_string(), 
            epoch: 0, 
            role: Role::Follower, 
            last_ping_recieve: None, 
            last_ping_send: None, 
            ping_period: Duration::from_secs(1), 
            lead: None, 
            heartbeat_timeout: Duration::from_secs(3), 
            nominate_min_delay: Duration::from_millis(50), 
            nominate_max_delay: Duration::from_millis(500), 
            nodes: Arc::new(Mutex::new(vec![])), 
            votes_min_count: 1 
        }
    }
}

#[async_trait]
trait NodeClient: Sync+Send {
    async fn ping( &self ) -> Result<RID,RErr>;
    fn clone_me( &self ) -> Box<dyn NodeClient>;
}

#[async_trait]
trait NodeInstance: Sync {
    async fn on_timer( &mut self );
}

//=======================================================================================

#[derive(Clone)]
struct NodeClientTest( Arc<Mutex<ClusterNode>> );

#[async_trait]
impl NodeClient for NodeClientTest {
    async fn ping( &self ) -> Result<RID,RErr> {        
        async {
            let mut me = self.0.lock().unwrap();
            info!("send ping to {}", me.id);

            me.last_ping_recieve = Some(Instant::now());
            Ok(1u64)
        }.await
    }

    fn clone_me( &self ) -> Box<dyn NodeClient> {
        Box::new( self.clone() )
    }
}

#[async_trait]
impl NodeInstance for ClusterNode
{
    async fn on_timer( &mut self ) {
        let r = match self.role {
            Role::Leader => {                     
                // рассылка пингов
                let send_pings_now = self.last_ping_send.map(|prev_t| Instant::now().duration_since(prev_t) >= self.ping_period ).unwrap_or(true);
                if send_pings_now {
                    let nodes:Vec<Box<dyn NodeClient>> = {
                        let nodes = self.nodes.lock().unwrap();
                        nodes.iter()
                        .map(|c| c.clone_me() )
                        .collect()
                    };

                    let t0 = Instant::now();
                    self.last_ping_send = Some(t0.clone());

                    let nodes_ping = join_all(nodes.iter().map(|c| c.ping())).await;
                    let t1 = Instant::now();

                    let succ_count = nodes_ping.iter().fold(0, |acc,it| {
                        acc + match it {
                            Ok(_) => 1usize,
                            Err(_) => 0usize,
                        }
                    });

                    info!("Leader id={id} sends ping, succ={succ}, total={tot}, time={dur:?}", 
                        id=self.id, succ=succ_count, tot=nodes.len(), dur=t1.duration_since(t0));
                }
            },
            Role::Follower => {
                // Проверка наличия ping
                // Если нет, то перейти в статус кандидата
                self.last_ping_recieve
                .map(|t| Instant::now().duration_since(t) )
                .map(|t| t > self.heartbeat_timeout );
            }
            _ => { () }
        };

        (async {}).await
    }
}

trait AddNode {
    fn add_node( self, node:Box<dyn NodeClient> );
    fn force_role( self, role:Role );
}

impl AddNode for &Arc<Mutex<ClusterNode>> {
    fn add_node( self, node:Box<dyn NodeClient> ) {
        let me = self.lock().unwrap();
        let mut nodes = me.nodes.lock().unwrap();
        nodes.push(node)
    }

    fn force_role( self, role:Role ) {
        let mut me = self.lock().unwrap();
        me.role = role;
    }
}

//=======================================================================================

#[test]
fn ping_send_test() {
    use env_logger;
    let _ = env_logger::builder().filter_level(log::LevelFilter::max()).is_test(true).try_init();

    let node0 = Arc::new(Mutex::new(ClusterNode::new("node0")));
    let node0t = NodeClientTest(node0.clone());

    let node1 = Arc::new(Mutex::new(ClusterNode::new("node1")));
    let node1t = NodeClientTest(node1.clone());

    let node2 = Arc::new(Mutex::new(ClusterNode::new("node2")));
    let node2t = NodeClientTest(node2.clone());

    node0.add_node(Box::new(node1t.clone()));
    node0.add_node(Box::new(node2t.clone()));

    node1.add_node(Box::new(node2t.clone()));
    node1.add_node(Box::new(node0t.clone()));

    node2.add_node(Box::new(node1t.clone()));
    node2.add_node(Box::new(node0t.clone()));

    node0.force_role(Role::Leader);

    let _ = System::new().block_on( async{
        info!("start bg");

        let h = spawn(async move {
            let t_start = Instant::now();        
            let mut cycle = 0;
            loop {
                cycle += 1;
                sleep(Duration::from_millis(1000)).await;

                let t_now = Instant::now();
                let t_dur = t_now.duration_since(t_start);

                if t_dur >= Duration::from_secs(15) { break; }

                info!("cycle {cycle}");

                if cycle == 3 { node0.force_role(Role::Follower) }

                // call on_timer
                {
                    let mut node = node0.lock().unwrap();
                    node.clone()
                }.on_timer().await;

                {
                    let mut node = node1.lock().unwrap();
                    node.clone()
                }.on_timer().await;

                {
                    let mut node = node2.lock().unwrap();
                    node.clone()
                }.on_timer().await
            }
        });

        h.await
    });
}