use std::{time::{Duration, Instant}, rc::Rc, sync::Arc, pin::Pin};

use actix_rt::System;
use futures::{Future, executor::block_on, future::join_all};
use log::info;
use tokio::{sync::Mutex, time::sleep};

use super::*;
use super::bg_tasks::*;
use async_trait::async_trait;

type NodeID = String;
type EpochID = u32;
type RID = u32;

#[derive(Clone)]
struct ClusterNode
{
    /// Идентификатор
    id: NodeID,

    /// Номер эпохи
    epoch: EpochID,

    /// роль
    role: Role,

    /// Ссылка на лидера
    lead: Option<String>,

    /// Время последнего принятого пинга
    last_ping_recieve: Option<Instant>,

    /// Время последнего отправленного пинга
    last_ping_send: Option<Instant>,

    /// Период с которым рассылать пинги
    ping_period: Duration,

    /// Таймайут после которо переход в кандидата
    heartbeat_timeout: Duration,

    /// Минимальная задержка ответа номинанту
    nominate_min_delay : Duration,

    /// Максимальная задержка ответа номинанту
    nominate_max_delay: Duration,

    /// Минимальное кол-во голосов для успеха
    votes_min_count: u32,

    /// За кого был отдан голос в новом цикле голосования
    vote: Option<String>,

    /// Остальные участники
    nodes: Vec<Arc<Mutex<dyn NodeClient>>>,
}

#[async_trait]
trait NodeClient: Send+Sync {
    async fn ping( &self, leader:NodeID, epoch:EpochID, rid:RID ) -> Result<(),RErr>;
    async fn nominate( &self, candidate:NodeID, epoch:u32, ) -> Result<(),RErr>;
}

struct NodeInstance {
    node: Arc<Mutex<ClusterNode>>
}

impl Clone for NodeInstance {
    fn clone(&self) -> Self {
        NodeInstance { node: self.node.clone() }
    }
}

#[async_trait]
trait NodeWorker {
    async fn on_timer(&mut self);
}

#[async_trait]
impl NodeWorker for NodeInstance {
    async fn on_timer(&mut self) {
        enum State {
            // Start,
            End,
            WinNomination { votes:usize, epoch:u32 },
            LooseNomination { votes:usize, total: usize },
        }

        // выдвижения себя как кандидата
        let self_nominate = || async {
            info!("self_nominate calling");

            let t_0 = Instant::now();

            let (clients, nid, epoch) = {
                let mut node = self.node.lock().await;
                node.role = Role::Candidate;
                (node.nodes.clone(), node.id.clone(), node.epoch)
            };

            info!("{nid} self_nominate call clients start");

            // TODO тут должна быть проверка что на текущем сроке была или нет заявка
            // Рассылаем свою кандидатуру
            let votes = {
                let clients = join_all(
                    clients.iter().map(|nc| 
                    nc.lock()
                )).await;

                info!("{nid} self_nominate call clients locked");

                join_all(
                    clients.iter().map(|nc|
                        nc.nominate(
                            nid.clone(),
                            epoch+1,
                        )
                    )
                ).await

                // Vec::<Result<(),RErr>>::new()
            };

            let t_1 = Instant::now();

            info!("{nid} self_nominate call clients called, time {time:?}",
                time = t_1.duration_since(t_0)
            );

            // Кол-во голосов для успеха должно быть больше или равно минимума
            let total_requests_count = votes.len();
            let succ_request_count = votes.iter().fold(
                0usize, |acc,it| 
                acc + match it {
                    Ok(_) => 1,
                    Err(_) => 0
                }
            );

            {
                let node = self.node.lock().await;
                if succ_request_count >= node.votes_min_count as usize {
                    State::WinNomination { 
                        votes: succ_request_count,
                        epoch: epoch+1,
                    }
                } else {
                    State::LooseNomination { votes: succ_request_count, total: total_requests_count }
                }
            }
        };

        // Рассылка пингов
        let leader_ping = || async {
            let mut node = self.node.lock().await;

            let send_pings_now =
                node.last_ping_send.map(|t| 
                    Instant::now().duration_since(t) >= node.ping_period
                ).unwrap_or(true);

            if send_pings_now {
                let clients = join_all(node.nodes.iter().map(|nc| 
                    nc.lock()
                )).await;
                
                let pings = join_all(clients.iter().map(|nc|
                    nc.ping(node.id.clone(), node.epoch, 0)
                )).await;

                let total_requests_count = pings.len();
                let succ_request_count = pings.iter().fold(
                    0usize, |acc,it| 
                    acc + match it {
                        Ok(_) => 1,
                        Err(_) => 0
                    }
                );

                info!("{nid} Leader on_timer, {succ}/{tot}",
                    nid = node.id,
                    succ = succ_request_count,
                    tot = total_requests_count,
                );                    
            };

            State::End
        };

        let follower_state = || async {
            //let mut node = self.node.lock().await;
            let (last_ping_recieve_opt,nid) = {
                let node = self.node.lock().await;
                ( node.last_ping_recieve.clone(), node.id.clone() )
            };
            match last_ping_recieve_opt {
                None => {
                    let mut node = self.node.lock().await;
                    node.last_ping_recieve = Some(Instant::now());
                    info!("{nid} assign last_ping_recieve");
                    State::End
                },
                Some( last_ping_recieve ) => {
                    info!("{nid} assigned last_ping_recieve");
                    
                    let heartbeat_timeout = { 
                        self.node.lock().await.heartbeat_timeout.clone() 
                    };

                    // Превышен интервал ?
                    let timeout = Instant::now().duration_since(last_ping_recieve);
                    let self_nominate_now =
                        timeout >= heartbeat_timeout;

                    info!("{nid} timeout {timeout:?} self_nominate_now {self_nominate_now}");

                    if self_nominate_now {
                        self_nominate().await
                    } else {
                        State::End
                    }
                }
            }
        };

        let mut state = State::End;        
            
        loop {
            state = {
                let role = { 
                    self.node.lock().await.role.clone()
                };

                match role {
                    Role::Leader => {
                        leader_ping().await
                    },
                    Role::Follower => {
                        follower_state().await
                    },
                    Role::Candidate => { 
                        sleep(random_between(Duration::from_millis(50),Duration::from_millis(500))).await;
                        self_nominate().await
                    }
                }
            };

            match state {
                State::End => {},
                State::WinNomination { votes, epoch } => {
                    let mut node = self.node.lock().await;
                    node.role = Role::Leader;
                    node.epoch = epoch;
                    info!("{nid} Win in nomination with {votes} votes, epoch {epoch}",
                        nid = node.id
                    )
                },
                State::LooseNomination {votes, total} => {
                    let node = self.node.lock().await;
                    info!("{nid} Loose in nomination with {votes} votes in {total} total votes",
                        nid = node.id
                    )
                }
            }

            break;
        }
    }
}

#[cfg(test)]
mod test {
    use rand::seq::SliceRandom;

    use super::*;

    #[derive(Clone)]
    struct NodeClientMockCheck(Arc<Mutex<ClusterNode>>);
    #[async_trait]
    impl NodeClient for NodeClientMockCheck {
        async fn ping( &self, leader:NodeID, epoch:EpochID, rid:RID ) -> Result<(),RErr> {
            async { 
                let mut me = self.0.lock().await;
                me.epoch += 1;
                println!("{me} accept ping from {leader}", me=me.id);
                Ok(()) 
            }.await
        }
        async fn nominate( &self, _candidate:NodeID, _epoch:u32 ) -> Result<(),RErr> {
            async { Ok(()) }.await
        }
    }

    #[test]
    fn check_clone(){
        let node0 = ClusterNode {
            id: "node0".to_string(),
            epoch: 0,
            role: Role::Follower,
            lead: None,
            last_ping_recieve: None,
            last_ping_send: None,        
            ping_period: Duration::from_secs(1),
            heartbeat_timeout: Duration::from_secs(3),
            nominate_min_delay: Duration::from_millis(50),
            nominate_max_delay: Duration::from_millis(500),
            votes_min_count: 3,
            vote: None,
            nodes: vec![]
        };
        let node1 = node0.clone();
        let node2 = node0.clone();
        let node3 = node0.clone();
        let node4 = node0.clone();

        let node0 = NodeInstance { node: Arc::new(Mutex::new(node0)) };
        let node1 = NodeInstance { node: Arc::new(Mutex::new(node1)) };
        let node1c = node1.clone();

        let node2 = NodeInstance { node: Arc::new(Mutex::new(node2)) };
        let node3 = NodeInstance { node: Arc::new(Mutex::new(node3)) };
        let node4 = NodeInstance { node: Arc::new(Mutex::new(node4)) };    

        let node0c = node0.clone();

        System::new().block_on(async move {
            {
                let mut node = node0.node.lock().await;
                node.nodes.push( Arc::new(Mutex::new( NodeClientMockCheck(node1.node.clone()) )) );
            }

            let mut bg = bg_job_async( move || {
                let n = node0.clone();
                async move {
                    let mut n = n.node.lock().await;
                    
                    n.epoch += 1;
                    println!("inc epoch {}", n.epoch);

                    for nc in &n.nodes {
                        let nc = nc.lock().await;
                        let _ = nc.ping(n.id.clone(), 0, 0).await;
                    }

                    ()
                }
            });
            bg.set_duration(Duration::from_millis(500));
        
            let _= bg.start();
            sleep(Duration::from_secs(2)).await;

            let n = node0c.node.lock().await;
            println!("node0c epoch {}", n.epoch );
            assert!(n.epoch>2);

            let n = node1c.node.lock().await;
            println!("node1c epoch {}", n.epoch );
            assert!(n.epoch>2);

        })
    }

    #[derive(Clone)]
    struct NodeClientMock( Arc<Mutex<ClusterNode>> );

    #[async_trait]
    impl NodeClient for NodeClientMock {
        async fn ping( &self, leader:NodeID, epoch:EpochID, rid:RID ) -> Result<(),RErr> {
            let mut node = self.0.lock().await;
            node.last_ping_recieve = Some(Instant::now());

            info!("{n} {role:?} accept ping",
                n = node.id,
                role = node.role
            );
            Ok(())
        }
        async fn nominate( &self, candidate:NodeID, epoch:u32 ) -> Result<(),RErr> {
            let mut node = self.0.lock().await;

            info!("{n} {role:?} accept nominate, candidate={candidate}, epoch={epoch}", 
                n=node.id,
                role=node.role
            );

            // Голос уже отдан
            if node.vote.is_some() {
                let vote = node.vote.clone();
                let vote = vote.unwrap();
                return Err(RErr::AlreadVoted { nominant: vote });
            }

            sleep(random_between(node.nominate_min_delay.clone(), node.nominate_max_delay.clone())).await;

            node.vote = Some(candidate.clone());
            Ok(())
        }
    }

    #[test]
    fn main_cycle() {
        use env_logger;
        let _ = env_logger::builder().filter_level(log::LevelFilter::max()).is_test(true).try_init();

        let node0 = ClusterNode {
            id: "node0".to_string(),
            epoch: 0,
            role: Role::Follower,
            lead: None,
            last_ping_recieve: None,
            last_ping_send: None,        
            ping_period: Duration::from_secs(1),
            heartbeat_timeout: Duration::from_secs(3),
            nominate_min_delay: Duration::from_millis(50),
            nominate_max_delay: Duration::from_millis(500),
            votes_min_count: 3,
            vote: None,
            nodes: vec![]
        };
        let mut node1 = node0.clone(); 
        node1.id = "node1".to_string();

        let mut node2 = node0.clone();
        node2.id = "node2".to_string();

        let mut node3 = node0.clone();
        node3.id = "node3".to_string();

        let mut node4 = node0.clone();
        node4.id = "node4".to_string();

        let node0 = NodeInstance { node: Arc::new(Mutex::new(node0)) };
        let node1 = NodeInstance { node: Arc::new(Mutex::new(node1)) };
        let node2 = NodeInstance { node: Arc::new(Mutex::new(node2)) };
        let node3 = NodeInstance { node: Arc::new(Mutex::new(node3)) };
        let node4 = NodeInstance { node: Arc::new(Mutex::new(node4)) };

        let nodes = vec![
            node0.clone(),
            node1.clone(),
            node2.clone(),
            node3.clone(),
            node4.clone(),
        ];

        let cycle_no = Arc::new(Mutex::new(0));
        System::new().block_on(async move {
            let leaders_count_max = Arc::new(Mutex::new(0));
            let leaders_count_max2 = leaders_count_max.clone();

            // link nodes
            for node in &nodes {
                for target in &nodes {                    
                    let nc = NodeClientMock(target.node.clone());
                    let target_id = target.node.lock().await.id.clone();
                    let mut node = node.node.lock().await;
                    if target_id != node.id {
                        println!("link {from} to {to}",
                            from = node.id,
                            to = target_id,
                        );
                        node.nodes.push(Arc::new(Mutex::new(nc)));
                    }
                }
            }

            // bg job
            let mut bg = bg_job_async( move || {
                let cycle_no = cycle_no.clone();
                let mut nodes = nodes.clone();
                let leaders_count_max = leaders_count_max.clone();
                async move {
                    let mut cycle_no = cycle_no.lock().await;
                    *cycle_no += 1;
                    println!("\n== {cycle} ==", cycle=cycle_no);

                    {
                        let mut rng = rand::thread_rng();
                        nodes.shuffle(&mut rng);
                    }

                    join_all(nodes.iter_mut().map(|n| n.on_timer())).await;

                    let mut leaders_count = 0;
                    for node in nodes {
                        let node = node.node.lock().await;
                        println!("{nid} {role:?}",
                            nid = node.id,
                            role = node.role
                        );

                        if let Role::Leader = node.role {
                            leaders_count += 1;
                        }                        
                    }

                    let mut leaders_count_max = leaders_count_max.lock().await;
                    *leaders_count_max = leaders_count_max.max(leaders_count);

                    println!();
                }
            });
            bg.set_duration(Duration::from_millis(1000));

            let _ = bg.start();
            sleep(Duration::from_secs(10)).await;

            println!("");
            let leaders_count_max = leaders_count_max2.clone();
            async move {
                let leaders_count_max = leaders_count_max.lock().await;
                println!("leaders_count_max {cnt}", cnt=leaders_count_max)
            }.await;

            println!("");
        });
    }
}