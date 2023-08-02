use std::{marker::PhantomData, time::{Instant, Duration}, rc::Rc, sync::{Arc, RwLock, Mutex, Weak}};
use actix_rt::{spawn, time::sleep, System};
use async_trait::async_trait;

use futures::{Future, future::join_all, SinkExt};
use log::{info};
use std::fmt::Debug;
use rand::prelude::*;

/// Ошибки
enum RErr {
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

/// Идентификатор последней записи в логе
type RID = u64;

type NodeID = String;

/// Роль
#[derive(Clone,Debug)]
enum Role {
    Follower,
    Candidate,
    Leader
}

/// Узел кластера
#[derive(Clone,Debug)]
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

    /// За кого был отдан голос в новом цикле голосования
    vote: Option<String>
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
            votes_min_count: 1,
            vote: None
        }
    }
}

#[async_trait]
trait NodeClient: Sync+Send+Debug {
    async fn ping( &self, from:NodeID ) -> Result<RID,RErr>;
    async fn nominate( &self, epoch:u32, id:NodeID ) -> Result<(),RErr>;

    fn clone_me( &self ) -> Box<dyn NodeClient>;
}

#[async_trait]
trait NodeInstance: Sync {
    async fn on_timer( &mut self );
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

                    let nodes_ping = join_all(nodes.iter().map(|c| c.ping( self.id.clone() ))).await;
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
                let self_nominate = self.last_ping_recieve
                .map(|t| Instant::now().duration_since(t) )
                .map(|t| t > self.heartbeat_timeout )
                .unwrap_or(true);

                if self_nominate {
                    let nodes:Vec<Box<dyn NodeClient>> = {
                        let nodes = self.nodes.lock().unwrap();
                        nodes.iter()
                        .map(|c| c.clone_me() )
                        .collect()
                    };

                    let res = join_all(nodes.iter().map(|c| c.nominate(self.epoch + 1, self.id.clone()))).await;
                    let succ_cnt = res.iter().fold(0usize, |acc,it| acc + match it {
                        Ok(_) => 1usize,
                        Err(_) => 0usize
                    });

                    info!("Nominate succ {succ_cnt} total {tot}", tot=res.len());
                }
            }
            _ => { () }
        };

        (async {}).await
    }
}

//=======================================================================================

#[cfg(test)]
mod test {
    use futures::TryFutureExt;

    use super::*;

    trait AddNode {
        fn add_node( self, node:Box<dyn NodeClient> );
        fn force_role( self, role:Role );
        fn set_votes_min_count( self, cnt:u32 );
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

        fn set_votes_min_count( self, cnt:u32 ) {
            let mut me = self.lock().unwrap();
            me.votes_min_count = cnt;
        }
    }

    #[derive(Debug)]
    enum LogEntry {
        Ping { client_id: NodeID, from: NodeID },
        Nominame { client_id: NodeID, epoch:u32, candidate: NodeID, succ: bool }
    }
    
    #[derive(Clone,Debug)]
    struct NodeClientTest { 
        consumer: Arc<Mutex<ClusterNode>>,
        log: Arc<Mutex<Vec<LogEntry>>>,
        //rnd: Arc<Mutex<rand::rngs::StdRng>>,
    }
    
    #[async_trait]
    impl NodeClient for NodeClientTest {
        async fn ping( &self, from:NodeID ) -> Result<RID,RErr> {        
            // Здесь должна быть отсылка по сети
            async {
                let mut me = self.consumer.lock().unwrap();
                info!("mock: send ping to {} from {}", &me.id, &from );
    
                me.last_ping_recieve = Some(Instant::now());
                self.log.lock().unwrap().push(LogEntry::Ping { client_id: me.id.clone(), from: from.clone() });
                Ok(1u64)
            }.await
        }
    
        async fn nominate( &self, epoch:u32, id:NodeID ) -> Result<(),RErr> {
            // Здесь должна быть отсылка по сети
            let res = async {
                let me = self.consumer.lock().unwrap();

                // не совпадает срок голосования с ожидаемым
                if epoch != ( me.epoch + 1 ) {
                    info!("mock: nominate epoch {epoch} id {id0} - epoch not matched", id0=id.clone());
                    self.log.lock().unwrap().push(LogEntry::Nominame { client_id: me.id.clone(), epoch: epoch, candidate: id.clone(), succ: false });
                    return Err(RErr::EpochNotMatch { expect: epoch+1, actual: epoch })
                }

                // Уже проголосовал
                if me.vote.is_some() {
                    info!("mock: nominate epoch {epoch} id {id0} - already matched", id0=id.clone());
                    self.log.lock().unwrap().push(LogEntry::Nominame { client_id: me.id.clone(), epoch: epoch, candidate: id.clone(), succ: false });
                    return Err(RErr::AlreadVoted { 
                        nominant: me.vote.as_ref().map(|c| c.clone()).unwrap() 
                    });
                }

                // расчет задержки на ответ
                let rand_u32 = rand::random::<u32>();
                let rand_f64_0_1 : f64 = (rand_u32 as f64) / (u32::MAX as f64);

                let mic0 = me.nominate_min_delay.as_micros();
                let mic1 = me.nominate_max_delay.as_micros();
                let dur_disp = mic1.max(mic0) - mic1.min(mic0);
                let dur_disp = ((dur_disp as f64) * rand_f64_0_1) as u128;

                let dur = dur_disp + mic0.min(mic1);
                let dur = Duration::from_micros(dur as u64);

                Ok(dur)
            }.map_ok(|d| sleep(d))
            .map_ok(|sleeping| async {
                sleeping.await;
                let mut me = self.consumer.lock().unwrap();
                me.vote = Some(id.clone());

                info!("mock: nominate epoch {epoch} id {id0} - voted", id0=id.clone());
                self.log.lock().unwrap().push(LogEntry::Nominame { client_id: me.id.clone(), epoch: epoch, candidate: id.clone(), succ: true });
            })
            .await;

            let res = match res {
                Ok(r) => Ok(r.await),
                Err(e) => Err(e)
            };

            async { res }.await
        }
    
        fn clone_me( &self ) -> Box<dyn NodeClient> {
            Box::new( self.clone() )
        }
    }    

    #[test]
    fn ping_send_test() {
        use env_logger;
        let _ = env_logger::builder().filter_level(log::LevelFilter::max()).is_test(true).try_init();

        let log = Arc::new(Mutex::new(Vec::<LogEntry>::new()));

        let node0: Arc<Mutex<ClusterNode>> = Arc::new(Mutex::new(ClusterNode::new("node0")));
        let node0t = NodeClientTest { 
            consumer: node0.clone(), 
            log: log.clone()
        };

        let node1 = Arc::new(Mutex::new(ClusterNode::new("node1")));
        let node1t = NodeClientTest { consumer: node1.clone(), log: log.clone() };

        let node2 = Arc::new(Mutex::new(ClusterNode::new("node2")));
        let node2t = NodeClientTest { consumer: node2.clone(), log: log.clone() };

        node0.add_node(Box::new(node1t.clone()));
        node0.add_node(Box::new(node2t.clone()));

        node1.add_node(Box::new(node2t.clone()));
        node1.add_node(Box::new(node0t.clone()));

        node2.add_node(Box::new(node1t.clone()));
        node2.add_node(Box::new(node0t.clone()));

        node0.force_role(Role::Leader);
        node0.set_votes_min_count(2);
        node1.set_votes_min_count(2);
        node2.set_votes_min_count(2);

        let _ = System::new().block_on( async{
            info!("start bg");

            let node0c = node0.clone();
            let node1c = node1.clone();
            let node2c = node2.clone();

            let h = spawn(async move {
                let t_start = Instant::now();        
                let mut cycle = 0;
                loop {
                    cycle += 1;
                    sleep(Duration::from_millis(1000)).await;

                    let t_now = Instant::now();
                    let t_dur = t_now.duration_since(t_start);

                    if t_dur >= Duration::from_secs(10) { break; }

                    info!("cycle {cycle}");

                    if cycle == 3 { node0c.force_role(Role::Follower) }

                    // call on_timer
                    {
                        let mut node = node0c.lock().unwrap();
                        node.clone()
                    }.on_timer().await;

                    {
                        let mut node = node1c.lock().unwrap();
                        node.clone()
                    }.on_timer().await;

                    {
                        let mut node = node2c.lock().unwrap();
                        node.clone()
                    }.on_timer().await
                }
            });
            
            let _ = h.await;

            {
                let log = log.lock().unwrap();
                for e in log.iter() {
                    println!("{e:?}")
                }
            }
        });
    }
}