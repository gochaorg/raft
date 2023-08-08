use super::*;
use async_trait::async_trait;
use futures::future::join_all;
use log::{info, warn};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Клиент к узлу кластера
#[async_trait]
pub trait NodeClient: Send+Sync {
    async fn ping( &self, leader:NodeID, epoch:EpochID, rid:RID ) -> Result<(),RErr>;
    async fn nominate( &self, candidate:NodeID, epoch:u32, ) -> Result<(),RErr>;
}

/// Часть сервиса кластера
#[async_trait]
pub trait NodeService {
    /// Периодично вызывается сервером
    async fn on_timer(&mut self);

    /// Принимает запрос ping от коиента
    async fn ping( &self, leader:NodeID, epoch:EpochID, rid:RID ) -> Result<(),RErr>;
}

#[async_trait]
impl NodeService for NodeInstance {
    async fn on_timer(&mut self) {
        enum State {
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

        // Проверка когда был пинг, и выдвижения себя как кандидата
        let follower_state = || async {
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
                    let now = Instant::now();
                    let timeout = now.duration_since(last_ping_recieve);
                    let self_nominate_now =
                        timeout >= heartbeat_timeout;

                    if self_nominate_now {
                        info!("{nid} timeout {timeout:?} heartbeat_timeout {heartbeat_timeout:?} self_nominate_now {self_nominate_now}");
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
                        follower_state().await
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
    
    async fn ping( &self, leader:NodeID, epoch:EpochID, rid:RID ) -> Result<(),RErr> {
        let mut node = self.node.lock().await;

        info!("{nid} {role:?} {n_epoch} accept ping: leader={leader} epoch={epoch}",
            nid = node.id,
            role = node.role,
            n_epoch = node.epoch,
        );

        if node.lead == Some(leader.clone()) {
            info!("{nid} leader matched",
                nid = node.id,
            );
            node.last_ping_recieve = Some(Instant::now());
            node.role = Role::Follower;
        } else if node.epoch < epoch {
            if node.id != leader {
                info!("{nid} switch leader and epoch",
                    nid = node.id
                );
                node.last_ping_recieve = Some(Instant::now());
                node.lead = Some(leader);
                node.epoch = epoch;
                node.role = Role::Follower;
                node.vote = None;
            }else{
                warn!("{nid} accept ping from self",
                    nid = node.id,
                );
            }
        }else{
            warn!("{nid} epoch less or equals",
                nid = node.id,
            );
        }

        Ok(())
    }
}

