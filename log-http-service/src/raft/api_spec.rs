use super::*;
use async_trait::async_trait;
use futures::future::join_all;
use log::{info, warn};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Клиент к узлу кластера
#[async_trait]
pub trait NodeClient: Send+Sync {
    async fn ping( &self, leader:NodeID, epoch:EpochID, rid:RID ) -> Result<PingResponse,RErr>;
    async fn nominate( &self, candidate:NodeID, epoch:u32, ) -> Result<(),RErr>;
}

/// Ответ на ping
#[derive(Clone,Debug)]
pub struct PingResponse {
    pub id: NodeID,
    pub epoch: EpochID,
    pub rid: RID,
}

/// Часть сервиса кластера
#[async_trait]
pub trait NodeService {
    /// Периодично вызывается сервером
    async fn on_timer( &mut self );

    /// Принимает запрос ping от клиента
    async fn ping( &self, leader:NodeID, epoch:EpochID, rid:RID ) -> Result<PingResponse,RErr>;

    /// Принимает запрос на лидера
    async fn nominate( &self, candidate:NodeID, epoch:u32 ) -> Result<(),RErr>;
}

#[async_trait]
impl<NC: NodeLogging+Sync+Send> NodeService for NodeInstance<NC> {
    async fn on_timer( &mut self ) {
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
                let mut node: tokio::sync::MutexGuard<'_, ClusterNode> = self.node.lock().await;
                
                let prev = node.role.clone();
                node.role = Role::Candidate;
                self.changes.change_role(prev, node.role.clone());

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
                let prev = node.last_ping_send.clone();
                node.last_ping_send = Some(Instant::now());
                self.changes.change_last_ping_send(prev, node.last_ping_send.clone());

                let clients = join_all(
                    node.nodes.iter().map(|nc| 
                    nc.lock()
                )).await;
                
                let pings = join_all(
                    clients.iter().map(|nc|
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

                    let prev = node.last_ping_recieve.clone();
                    node.last_ping_recieve = Some(Instant::now());
                    self.changes.change_last_ping_recieve(prev, node.last_ping_recieve.clone());

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

        loop {
            let state = {
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

                    let prev = node.role.clone();
                    node.role = Role::Leader;
                    self.changes.change_role(prev, node.role.clone());

                    let prev = node.epoch.clone();
                    node.epoch = epoch;
                    self.changes.change_epoch(prev, node.epoch.clone());

                    let prev = node.vote.clone();
                    node.vote = None;
                    self.changes.change_vote(prev, node.vote.clone());

                    let prev = node.lead.clone();
                    node.lead = None;
                    self.changes.change_leader(prev, node.lead.clone());

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
    
    async fn ping( &self, leader:NodeID, epoch:EpochID, rid:RID ) -> Result<PingResponse,RErr> {
        let mut node = self.node.lock().await;

        info!("{nid} {role:?} {n_epoch} accept ping: leader={leader} epoch={epoch}",
            nid = node.id,
            role = node.role,
            n_epoch = node.epoch,
        );

        self.changes.on_ping(leader.clone(), epoch.clone(), rid.clone());

        if node.lead == Some(leader.clone()) {
            info!("{nid} leader matched",
                nid = node.id,
            );

            self.changes.on_ping_leader_match();

            let from = node.last_ping_recieve.clone();
            node.last_ping_recieve = Some(Instant::now());
            self.changes.change_last_ping_recieve(from, node.last_ping_recieve.clone());

            let from = node.role.clone();
            node.role = Role::Follower;
            self.changes.change_role(from, node.role.clone());            
        } else if node.epoch < epoch {
            self.changes.on_ping_epoch_greater();

            if node.id != leader {
                self.changes.on_ping_leader_different();

                info!("{nid} switch leader and epoch",
                    nid = node.id
                );

                let from = node.last_ping_recieve.clone();
                node.last_ping_recieve = Some(Instant::now());
                self.changes.change_last_ping_recieve(from, node.last_ping_recieve.clone());

                let from = node.lead.clone();
                node.lead = Some(leader);
                self.changes.change_leader(from, node.lead.clone());

                let from = node.epoch.clone();
                node.epoch = epoch;
                self.changes.change_epoch(from, epoch);

                let from = node.role.clone();
                node.role = Role::Follower;
                self.changes.change_role(from, node.role.clone());

                let from = node.vote.clone();
                node.vote = None;
                self.changes.change_vote(from, node.vote.clone());
            }else{
                self.changes.on_ping_leader_self();

                warn!("{nid} accept ping from self",
                    nid = node.id,
                );
            }
        }else{            
            self.changes.on_ping_epoch_less_or_equals();
            
            warn!("{nid} epoch less or equals",
                nid = node.id,
            );
        }

        Ok(PingResponse { 
            id: node.id.clone(), 
            epoch: node.epoch, 
            rid: 0 
        })
    }

    async fn nominate( &self, candidate:NodeID, epoch:u32 ) -> Result<(),RErr> {
        let mut node = self.node.lock().await;

        info!("{n} {role:?} accept nominate, candidate={candidate}, epoch={epoch}", 
            n=node.id,
            role=node.role
        );

        // Голосовать можно за новый срок
        if node.epoch >= epoch {
            return Err(RErr::EpochNotMatch { 
                expect: node.epoch + 1, 
                actual: epoch.clone() 
            });
        }

        // Голос уже отдан
        if node.vote.is_some() {
            let vote = node.vote.clone();
            let vote = vote.unwrap();
            return Err(RErr::AlreadVoted { nominant: vote });
        }

        sleep(random_between(node.nominate_min_delay.clone(), node.nominate_max_delay.clone())).await;

        let from = node.vote.clone();
        node.vote = Some(candidate.clone());
        self.changes.change_vote(from, node.vote.clone());

        Ok(())
    }
}

