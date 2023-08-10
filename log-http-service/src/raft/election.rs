#[allow(unused_imports)]
use std::{time::{Duration, Instant}, rc::Rc, sync::Arc, pin::Pin};
#[allow(unused_imports)]
use tokio::{sync::Mutex as AsyncMutex, time::sleep};

#[cfg(test)]
mod test {    
    use async_trait::async_trait;
    use actix_rt::System;
    use futures::future::join_all;
    use super::*;
    use super::super::*;
    use super::super::bg_tasks::*;
    use std::collections::{HashMap, HashSet};
    use std::marker::PhantomData;
    use std::sync::Mutex as SyncMutex;

    #[derive(Clone)]
    struct NodeClientMockCheck<RID>(Arc<AsyncMutex<ClusterNode<RID>>>);

    #[async_trait]
    impl<RID:Sync+Send> NodeClient<RID> for NodeClientMockCheck<RID> {
        async fn ping( &self, leader:NodeID, _epoch:EpochID, _rid:RID ) -> Result<PingResponse,RErr> {
            async { 
                let mut me = self.0.lock().await;
                me.epoch += 1;
                println!("{me} accept ping from {leader}", me=me.id);
                Ok(
                    PingResponse {
                        id: me.id.clone(),
                        epoch: me.epoch.clone(),
                        rid: 0
                    }
                ) 
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
            epoch_of_candidate: None,
            role: Role::Follower,
            lead: None,
            last_ping_recieve: None,
            last_ping_send: None,        
            ping_period: Duration::from_secs(1),
            heartbeat_timeout: Duration::from_secs(3),
            nominate_min_delay: Duration::from_millis(50),
            nominate_max_delay: Duration::from_millis(500),
            renominate_min_delay: Duration::from_millis(50),
            renominate_max_delay: Duration::from_millis(500),
            votes_min_count: 3,
            vote: None,
            nodes: vec![]
        };
        let node1 = node0.clone();
        let node2 = node0.clone();
        let node3 = node0.clone();
        let node4 = node0.clone();

        let node0 = NodeInstance { 
            node: Arc::new(AsyncMutex::new(node0)),
            changes: DummyNodeChanges(),
            _p: PhantomData.clone()
        };
        let node1 = NodeInstance { 
            node: Arc::new(AsyncMutex::new(node1)),
            changes: DummyNodeChanges(),
            _p: PhantomData.clone()
        };
        let node1c = node1.clone();

        let _node2 = NodeInstance { 
            node: Arc::new(AsyncMutex::new(node2)),
            changes: DummyNodeChanges(),
            _p: PhantomData.clone()
        };
        let _node3 = NodeInstance { 
            node: Arc::new(AsyncMutex::new(node3)),
            changes: DummyNodeChanges(),
            _p: PhantomData.clone()
        };
        let _node4 = NodeInstance { 
            node: Arc::new(AsyncMutex::new(node4)),
            changes: DummyNodeChanges(),
            _p: PhantomData.clone()
        };

        let node0c = node0.clone();

        System::new().block_on(async move {
            {
                let mut node = node0.node.lock().await;
                node.nodes.push( Arc::new(AsyncMutex::new( NodeClientMockCheck(node1.node.clone()) )) );
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

    //////////////////////////////
    #[derive(Debug,Clone)]
    #[allow(dead_code)]
    enum Event<RID> {
        NodeState { node_id:NodeID, role:Role, epoch: EpochID, rid:RID },
        CycleBegin { cycle_no:i32 },
        CycleEnd { cycle_no:i32 },
        RoleChanged { node_id:NodeID, from:Role, to:Role },
        EpochChanged { node_id:NodeID, from:EpochID, to:EpochID },
        VoteChanged { node_id:NodeID, from:Option<NodeID>, to:Option<NodeID> },
        PingRequest  { cycle_no: i32, node_id:NodeID, leader: NodeID, epoch:EpochID, rid:RID },
        PingResponse { cycle_no: i32, node_id:NodeID, leader: NodeID, epoch:EpochID, rid:RID, 
            response: Result<PingResponse,RErr> 
        },
        NominateRequest {
            cycle_no: i32,
            node_id:NodeID,
            candidate:NodeID, 
            epoch:u32
        },
        NominateResponse {
            cycle_no: i32,
            node_id:NodeID,
            candidate:NodeID, 
            epoch:u32,
            response: Result<(),RErr>
        }
    }

    trait EventLog<RID>: Sized+Sync+Send {
        fn push( &self, e:Event<RID> );
        fn cycles( &self ) -> Vec<Vec<Event<RID>>>;
        fn state_changes<E:Sized+Eq>( &self, f:impl Fn(&NodeStates) -> E) -> Vec<E>;
        fn state_changes_match<E:Sized+Eq>( &self, 
            f:impl Fn(&NodeStates) -> E, 
            sample:Vec<E> 
        ) -> bool {
            let changes = self.state_changes(f);
            let matched = changes.iter().zip(sample.iter())
                .filter(|(a,b)| **a == **b ).count();
            let matched = matched == changes.len();
            matched
        }
    }

    #[derive(Clone,Debug)]
    #[allow(dead_code)]
    struct NodeStates { 
        cycle_no: i32,
        leaders: i32,
        followers: i32,
        candidates: i32,
        epoch_min_max: Option<(EpochID,EpochID)>,
        nominate_epoch_min_max: Option<(EpochID,EpochID)>
    }
    trait CycleEvents {
        fn node_states( self, cycle_no:i32 ) -> NodeStates;
    }
    impl<RID> CycleEvents for &Vec<Event<RID>> {
        fn node_states( self, cycle_no:i32 ) -> NodeStates {
            let mut s = NodeStates { 
                cycle_no:cycle_no,
                leaders:0, 
                followers:0, 
                candidates:0, 
                epoch_min_max:None,
                nominate_epoch_min_max:None,
            };
            for e in self {
                match e {
                    Event::NominateRequest { cycle_no:_, node_id:_, candidate:_, epoch } => {
                        s.nominate_epoch_min_max = Some(
                            s.nominate_epoch_min_max.map(|(min_e,max_e)|
                                (min_e.min(*epoch), max_e.max(*epoch))
                            ).unwrap_or(
                                (*epoch, *epoch)
                            )
                        );
                    }
                    Event::NodeState { node_id:_, role, epoch, rid:_ } => {
                        match role {
                            Role::Leader => s.leaders += 1,
                            Role::Candidate => s.candidates += 1,
                            Role::Follower => s.followers += 1,                        
                        };
                        s.epoch_min_max = Some(
                            s.epoch_min_max
                            .map(|(min_e,max_e)| {
                                ( min_e.min(*epoch)
                                , max_e.max(*epoch)
                                )
                            })
                            .unwrap_or((*epoch,*epoch))
                        );
                    },
                    _ => {}
                }
            }
            s
        }
    }

    #[derive(Clone)]
    struct NodeClientMock<RID, NC:NodeLogging<RID>, Log:EventLog<RID>> { 
        node: NodeInstance<RID,NC>,
        cycle_no: Arc<AsyncMutex<i32>>,
        log: Log
    }

    impl<RID:Sync+Send+Clone> EventLog<RID> for Arc<SyncMutex<Vec<Event<RID>>>> {
        fn push( &self, e:Event<RID> ) {
            let mut log = self.lock().unwrap();
            log.push(e);
        }
        fn cycles( &self ) -> Vec<Vec<Event<RID>>> {
            let log = self.lock().unwrap();
            let mut res: Vec<Vec<Event<RID>>> = vec![vec![]];
            let mut cur: Vec<Event<RID>> = vec![];
            for e in log.iter() {
                match e {
                    Event::CycleBegin { cycle_no:_ } => {},
                    Event::CycleEnd { cycle_no:_ } => {
                        res.push(cur);
                        cur = vec![];
                    },
                    _ => {
                        cur.push(e.clone());
                    }
                }
            }
            res
        }
        fn state_changes<E:Sized+Eq>( &self, f:impl Fn(&NodeStates) -> E) -> Vec<E> {
            let mut res: Vec<E> = vec![];
            let cycles = self.cycles();
            for (c,cycle) in cycles.iter().enumerate() {
                let ns = cycle.node_states(c as i32 + 1);
                let n = f(&ns);
                match res.last() {
                    None => {
                        res.push(n);
                    }
                    Some(last) => {
                        if (*last) != n {
                            res.push(n);
                        }
                    }
                }
            }
            res
        }
    }

    #[async_trait]
    impl<RID:Sync+Send+Clone+Default, NC:NodeLogging<RID>+Send+Sync, Log:EventLog<RID>> NodeClient<RID> for NodeClientMock<RID,NC,Log> {
        async fn ping( &self, leader:NodeID, epoch:EpochID, rid:RID ) -> Result<PingResponse,RErr> {
            let cycle_no = { self.cycle_no.lock().await.clone() };

            self.log.push(Event::PingRequest { 
                cycle_no: cycle_no, 
                node_id: self.node.node.lock().await.id.clone(), 
                leader: leader.clone(), 
                epoch: epoch.clone(), 
                rid: rid.clone() 
            });
            
            let resp = 
            if cycle_no >= 8  && cycle_no < 12 // 6. спустя время лидер перестает посылать ping
            || cycle_no >= 15 && cycle_no < 22 // 12. спустя время лидер перестает посылать ping
            {
                Err(RErr::ReponseTimeout)
            } else {            
                self.node.ping(leader.clone(), epoch, rid.clone()).await
            };

            self.log.push(Event::PingResponse { 
                cycle_no: cycle_no, 
                node_id: self.node.node.lock().await.id.clone(), 
                leader: leader.clone(), 
                epoch: epoch.clone(), 
                rid: rid.clone(),
                response: resp.clone(),
            });

            resp
        }

        async fn nominate( &self, candidate:NodeID, epoch:u32 ) -> Result<(),RErr> {
            let cycle_no = { self.cycle_no.lock().await.clone() };

            self.log.push(Event::NominateRequest { 
                cycle_no: cycle_no, 
                node_id: self.node.node.lock().await.id.clone(), 
                candidate: candidate.clone(), 
                epoch: epoch.clone(), 
            });

            // 14. голоса разделяются поровну - не добирается минимальный проходной минимум
            let response = if cycle_no >= 17 && cycle_no <= 19 {
                Err(RErr::ReponseTimeout)
            } else {
                self.node.nominate(candidate.clone(), epoch).await
            };

            self.log.push(Event::NominateResponse { 
                cycle_no: cycle_no, 
                node_id: self.node.node.lock().await.id.clone(), 
                candidate: candidate.clone(), 
                epoch: epoch.clone(), 
                response: response.clone()
            });

            response.clone()
        }
    }

    #[derive(Clone)]
    struct NodeLog<RID> { node_id:NodeID, log: Arc<SyncMutex<Vec<Event<RID>>>> }

    impl<RID:Clone> NodeLogging<RID> for NodeLog<RID> {
        fn change_role( &self, from:Role, to:Role ) {
            let mut log = self.log.lock().unwrap();
            log.push(Event::RoleChanged { 
                node_id: self.node_id.clone(), 
                from: from, 
                to: to 
            });
        }
        fn change_id( &self, _from:Option<NodeID>, _to:Option<NodeID> ) {}
        fn change_last_ping_recieve( &self, _from:Option<Instant>, _to:Option<Instant> ) {}
        fn change_last_ping_send( &self, _from:Option<Instant>, _to:Option<Instant> ) {}
        fn change_epoch( &self, from:EpochID, to:EpochID ) {
            let mut log = self.log.lock().unwrap();
            log.push(Event::EpochChanged { 
                node_id: self.node_id.clone(), 
                from: from, 
                to: to 
            });
        }
        fn change_vote( &self, from:Option<NodeID>, to:Option<NodeID> ) {
            let mut log = self.log.lock().unwrap();
            log.push(Event::VoteChanged { 
                node_id: self.node_id.clone(), 
                from: from, 
                to: to 
            });
        }
    }

    /// что должно быть по тесту
    /// 
    /// 1. 5 участников стартуют как Follower (те 0 Leaders) - эпоха 0
    /// 2. спустя время выбирают одного лидера
    /// 3. лидер после рассылает ping с новым номером эпохи
    /// 4. остальные участники меняют номер эпохи с 0 на 1
    /// 5. остальные участники меняют лидера на выбронного
    /// 6. спустя время лидер перестает посылать ping
    /// 7. начинается выбор нового лидера
    /// 8. выбран новый лидер с новым сроком
    /// 9. лидер после рассылает ping с новым номером эпохи - 2
    /// 10. остальные участники меняют номер эпохи с 1 на 2
    /// 11. остальные участники меняют лидера на выбронного
    /// 12. спустя время лидер перестает посылать ping
    /// 13. начинается выбор нового лидера - эпоха 3
    /// 14. голоса разделяются поровну - не добирается минимальный проходной минимум
    /// 15. начинается выбор нового лидера с новой эпохи - эпоха 4
    /// 16. выбран новый лидер с новым сроком  - эпоха 4
    /// 17. лидер после рассылает ping с новым номером эпохи  - эпоха 4
    /// 
    /// требования
    /// А) выдвинуть кандидатуру можно только один раз на один срок
    /// Б) после успешных выборов, кол-во лидеров должно быть 1
    #[test]
    fn main_cycle() {
        use env_logger;
        let _ = env_logger::builder().filter_level(log::LevelFilter::max()).is_test(true).try_init();
        let log = Arc::new(SyncMutex::new(Vec::<Event<u32>>::new()));

        // Создание узлов кластера
        let node0 = ClusterNode {
            id: "node0".to_string(),
            epoch: 0,
            epoch_of_candidate: None,
            role: Role::Follower,
            lead: None,
            last_ping_recieve: None,
            last_ping_send: None,        
            ping_period: Duration::from_secs(1),
            heartbeat_timeout: Duration::from_secs(3),
            nominate_min_delay: Duration::from_millis(50),
            nominate_max_delay: Duration::from_millis(500),
            renominate_min_delay: Duration::from_millis(50),
            renominate_max_delay: Duration::from_millis(500),
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

        let node0 = NodeInstance { 
            node: Arc::new(AsyncMutex::new(node0)),
            changes: NodeLog { node_id: "node0".to_string(), log: log.clone() },
            _p: PhantomData.clone()
        };
        let node1 = NodeInstance { 
            node: Arc::new(AsyncMutex::new(node1)),
            changes: NodeLog { node_id: "node1".to_string(), log: log.clone() },
            _p: PhantomData.clone()
        };
        let node2 = NodeInstance { 
            node: Arc::new(AsyncMutex::new(node2)),
            changes: NodeLog { node_id: "node2".to_string(), log: log.clone() },
            _p: PhantomData.clone()
        };
        let node3 = NodeInstance { 
            node: Arc::new(AsyncMutex::new(node3)),
            changes: NodeLog { node_id: "node3".to_string(), log: log.clone() },
            _p: PhantomData.clone()
        };
        let node4 = NodeInstance { 
            node: Arc::new(AsyncMutex::new(node4)),
            changes: NodeLog { node_id: "node4".to_string(), log: log.clone() },
            _p: PhantomData.clone()
        };

        let nodes = vec![
            node0.clone(),
            node1.clone(),
            node2.clone(),
            node3.clone(),
            node4.clone(),
        ];

        let cycle_no = Arc::new(AsyncMutex::new(0));
        
        System::new().block_on(async move {
            // link nodes
            for node in &nodes {
                for target in &nodes {                    
                    let nc = NodeClientMock { 
                        node: target.clone(), 
                        cycle_no: cycle_no.clone(),
                        log: log.clone(),
                    };
                    let target_id = target.node.lock().await.id.clone();
                    let mut node = node.node.lock().await;
                    if target_id != node.id {
                        node.nodes.push(Arc::new(AsyncMutex::new(nc)));
                    }
                }
            }

            // bg job
            // Лог событий
            let log_bg = log.clone();
            let mut bg = bg_job_async( move || {
                let cycle_no = cycle_no.clone();
                let mut nodes = nodes.clone();
                let log_bg = log_bg.clone();
                async move {
                    let cycle_no = {
                        let mut cycle_no = cycle_no.lock().await;
                        *cycle_no += 1;
                        cycle_no.clone()
                    };

                    println!("\n== {cycle} ==", cycle=cycle_no);

                    log_bg.push(Event::CycleBegin { cycle_no: cycle_no.clone() });

                    join_all(
                        nodes.iter_mut().map(|n| 
                            n.on_timer()
                        )
                    ).await;

                    for node in nodes {
                        let node = node.node.lock().await;
                        println!("{nid} {role:?} epoch {epoch}",
                            nid   = node.id,
                            role  = node.role,
                            epoch = node.epoch,
                        );

                        log_bg.push(Event::NodeState { 
                            node_id: node.id.clone(), 
                            role: node.role.clone(),
                            epoch: node.epoch,
                            rid: 0
                        });
                    }

                    log_bg.push(Event::CycleEnd { cycle_no: cycle_no.clone() });
                    println!();
                }
            });

            // Периодичность задач
            bg.set_duration(Duration::from_millis(1000));

            let _ = bg.start();
            sleep(Duration::from_secs(30)).await;

            println!("log");
            {
                let log = log.lock().unwrap();
                for e in log.iter() {
                    println!("{e:?}")
                }
            }

            // Проверка результатов
            println!("\nstates");
            {
                let log = log.cycles();
                for (c,es) in log.iter().enumerate() {
                    println!("{:?}", es.node_states(c as i32 + 1))
                }
            }

            #[derive(Debug,Clone,PartialEq)]
            struct Nomination {
                cycle: i32,
                epoch: EpochID,                
            }
            let mut nominations : HashMap<NodeID,Vec<Nomination>> = HashMap::new();
            {
                let events = log.lock().unwrap();
                for ev in events.iter() {
                    match ev {
                        Event::NominateRequest { cycle_no, node_id:_, candidate, epoch } => {
                            let nom = Nomination { cycle:*cycle_no, epoch:*epoch };
                            match nominations.get_mut(candidate) {
                                Some(lst) => {
                                    if ! lst.contains(&nom) { lst.push(nom) }
                                }
                                None => {
                                    nominations.insert(candidate.clone(), vec![nom]);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                for (_, noms) in nominations.iter_mut() {
                    noms.sort_by_key(|n| n.cycle);
                }
            }
            // А) выдвинуть кандидатуру можно только один раз на один срок
            println!("\nnominations");
            for (nid,noms) in nominations.iter() {
                let mut epoch_dup : HashSet<EpochID> = HashSet::new();
                for nom in noms {
                    assert!( !epoch_dup.contains(&nom.epoch) );
                    epoch_dup.insert(nom.epoch);
                }
            }

            // 5 участников стартуют как Follower (те 0 Leaders)
            // спустя время выбирают одного лидера
            let leaders_matched = log.state_changes_match(
                |n| n.leaders, vec![0,1,2,1,2,1]);
            // assert!(leaders_matched,"leaders_matched");

            // 3. лидер после рассылает ping с новым номером эпохи
            
            // 4. остальные участники меняют номер эпохи
            let epoch_matches =
            log.state_changes_match(|n| n.epoch_min_max, 
                vec![None,
                Some((0,0)),
                Some((0,1)),
                Some((1,1)),
                Some((1,2)),
                Some((2,2)),
                Some((2,3)),
                Some((3,3)),
                ]
            );
            // assert!(epoch_matches,"epoch_matches");
        });
    }
}