use std::{time::{Duration, Instant}, rc::Rc, sync::Arc, pin::Pin};

use actix_rt::System;
use futures::{Future, executor::block_on, future::join_all};
use log::info;
use tokio::{sync::Mutex, time::sleep};

use super::*;
use super::bg_tasks::*;
use async_trait::async_trait;

type NodeID = String;

#[derive(Clone)]
struct ClusterNode
{
    /// Идентификатор
    id: NodeID,

    /// Номер эпохи
    epoch: u32,

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
    async fn ping( &self, leader:NodeID ) -> Result<(),RErr>;
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
        enum Post {
            End,
            WinNomination { votes:usize, epoch:u32 }
        }

        let self_nominate = || async {
            let mut node = self.node.lock().await;

            node.role = Role::Candidate;

            // TODO тут должна быть проверка что на текущем сроке была или нет заявка
            // Рассылаем свою кандидатуру
            let clients = join_all(node.nodes.iter().map(|nc| 
                nc.lock()
            )).await;

            let votes = join_all(
                clients.iter().map(|nc|
                    nc.nominate(
                        node.id.clone(),
                        node.epoch+1,
                    )
                )
            ).await;
            
            // Кол-во голосов для успеха должно быть больше или равно минимума
            let total_requests_count = votes.len();
            let succ_request_count = votes.iter().fold(
                0usize, |acc,it| 
                acc + match it {
                    Ok(_) => 1,
                    Err(_) => 0
                }
            );

            if node.votes_min_count as usize >= succ_request_count {
                Post::WinNomination { 
                    votes: succ_request_count,
                    epoch: node.epoch+1,
                }
            } else {
                Post::End
            }
        };
            
        let post = {
            let mut node = self.node.lock().await;
            match node.role {
                Role::Leader => {
                    // рассылка пингов
                    let send_pings_now =
                        node.last_ping_send.map(|t| 
                            Instant::now().duration_since(t) >= node.ping_period
                        ).unwrap_or(true);

                    if send_pings_now {
                        let clients = join_all(node.nodes.iter().map(|nc| 
                            nc.lock()
                        )).await;
                        
                        let pings = join_all(clients.iter().map(|nc|
                            nc.ping(node.id.clone())
                        )).await;

                        let total_requests_count = pings.len();
                        let succ_request_count = pings.iter().fold(
                            0usize, |acc,it| 
                            acc + match it {
                                Ok(_) => 1,
                                Err(_) => 0
                            }
                        );

                        info!("{n} Leader on_timer, {succ}/{tot}",
                            n = node.id,
                            succ = succ_request_count,
                            tot = total_requests_count,
                        );                    
                    };

                    Post::End
                },
                Role::Follower => {
                    match node.last_ping_recieve {
                        None => {
                            node.last_ping_recieve = Some(Instant::now());
                            Post::End
                        },
                        Some( last_ping_recieve ) => {
                            // Превышен интервал ?
                            if Instant::now().duration_since(last_ping_recieve) > node.heartbeat_timeout {
                                self_nominate().await
                                // node.role = Role::Candidate;

                                // // TODO тут должна быть проверка что на текущем сроке была или нет заявка
                                // // Рассылаем свою кандидатуру
                                // let clients = join_all(node.nodes.iter().map(|nc| 
                                //     nc.lock()
                                // )).await;

                                // let votes = join_all(
                                //     clients.iter().map(|nc|
                                //         nc.nominate(
                                //             node.id.clone(),
                                //             node.epoch+1,
                                //         )
                                //     )
                                // ).await;
                                
                                // // Кол-во голосов для успеха должно быть больше или равно минимума
                                // let total_requests_count = votes.len();
                                // let succ_request_count = votes.iter().fold(
                                //     0usize, |acc,it| 
                                //     acc + match it {
                                //         Ok(_) => 1,
                                //         Err(_) => 0
                                //     }
                                // );

                                // if node.votes_min_count as usize >= succ_request_count {
                                //     Post::WinNomination { 
                                //         votes: succ_request_count,
                                //         epoch: node.epoch+1,
                                //     }
                                // } else {
                                //     Post::End
                                // }
                            } else {
                                Post::End
                            }
                        }
                    }
                },
                Role::Candidate => { 
                    self_nominate().await
                    // Post::End 
                }
            }
        };

        match post {
            Post::End => {},
            Post::WinNomination { votes, epoch } => {
                let mut node = self.node.lock().await;
                node.role = Role::Leader;
                node.epoch = epoch;
                info!("Win in nomination with {votes} votes")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone)]
    struct NodeClientMockCheck(Arc<Mutex<ClusterNode>>);
    #[async_trait]
    impl NodeClient for NodeClientMockCheck {
        async fn ping( &self, leader:NodeID ) -> Result<(),RErr> {
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
                        let _ = nc.ping(n.id.clone()).await;
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
        async fn ping( &self, leader:NodeID ) -> Result<(),RErr> {
            let mut node = self.0.lock().await;
            node.last_ping_recieve = Some(Instant::now());
            Ok(())
        }
        async fn nominate( &self, _candidate:NodeID, _epoch:u32 ) -> Result<(),RErr> {
            Ok(())
        }
    }
}