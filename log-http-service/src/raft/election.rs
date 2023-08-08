use std::{time::{Duration, Instant}, rc::Rc, sync::Arc, pin::Pin};
use tokio::{sync::Mutex as AsyncMutex, time::sleep};

#[cfg(test)]
mod test {
    use log::warn;
    use rand::seq::SliceRandom;
    use async_trait::async_trait;
    use log::info;
    use actix_rt::System;
    use futures::future::join_all;
    use super::*;
    use super::super::*;
    use super::super::bg_tasks::*;
    use std::sync::Mutex as SyncMutex;

    #[derive(Clone)]
    struct NodeClientMockCheck(Arc<AsyncMutex<ClusterNode>>);
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

        let node0 = NodeInstance { node: Arc::new(AsyncMutex::new(node0)) };
        let node1 = NodeInstance { node: Arc::new(AsyncMutex::new(node1)) };
        let node1c = node1.clone();

        let node2 = NodeInstance { node: Arc::new(AsyncMutex::new(node2)) };
        let node3 = NodeInstance { node: Arc::new(AsyncMutex::new(node3)) };
        let node4 = NodeInstance { node: Arc::new(AsyncMutex::new(node4)) };    

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
    enum Event {
        NodeState { node_id:NodeID, role:Role },
        CycleBegin { cycle_no:i32 },
        CycleEnd { cycle_no:i32 },
    }

    trait EventLog: Sized {
        fn push( self, e:Event );
        fn cycles( self ) -> Vec<Vec<Event>>;
        fn state_changes( self, f:impl Fn(&NodeStates) -> i32) -> Vec<i32>;
        fn state_changes_match( self, 
            f:impl Fn(&NodeStates) -> i32, 
            sample:Vec<i32> 
        ) -> bool {
            let changes = self.state_changes(f);
            let matched = changes.iter().zip(sample.iter())
                .filter(|(a,b)| **a == **b ).count();
            let matched = matched == changes.len();
            matched
        }
    }

    #[derive(Clone,Debug)]
    struct NodeStates { 
        leaders: i32,
        followers: i32,
        candidates: i32,
    }
    trait CycleEvents {
        fn node_states( self ) -> NodeStates;
    }
    impl CycleEvents for &Vec<Event> {
        fn node_states( self ) -> NodeStates {
            let mut s = NodeStates { leaders:0, followers:0, candidates:0 };
            for e in self {
                match e {
                    Event::NodeState { node_id, role } => match role {
                        Role::Leader => s.leaders += 1,
                        Role::Candidate => s.candidates += 1,
                        Role::Follower => s.followers += 1,
                    },
                    _ => {}
                }
            }
            s
        }
    }

    #[derive(Clone)]
    struct NodeClientMock( Arc<AsyncMutex<ClusterNode>>, Arc<SyncMutex<Vec<Event>>> );

    impl EventLog for &Arc<SyncMutex<Vec<Event>>> {
        fn push( self, e:Event ) {
            let mut log = self.lock().unwrap();
            log.push(e);
        }
        fn cycles( self ) -> Vec<Vec<Event>> {
            let log = self.lock().unwrap();
            let mut res: Vec<Vec<Event>> = vec![vec![]];
            let mut cur: Vec<Event> = vec![];
            for e in log.iter() {
                match e {
                    Event::CycleBegin { cycle_no } => {},
                    Event::CycleEnd { cycle_no } => {
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
        fn state_changes( self, f:impl Fn(&NodeStates) -> i32) -> Vec<i32> {
            let mut res: Vec<i32> = vec![];
            let cycles = self.cycles();
            for cycle in cycles {
                let ns = cycle.node_states();
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
    impl NodeClient for NodeClientMock {
        async fn ping( &self, leader:NodeID, epoch:EpochID, rid:RID ) -> Result<(),RErr> {
            let ni = NodeInstance { node: self.0.clone() };
            ni.ping(leader, epoch, rid).await
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

    /// что должно быть по тесту
    /// 
    /// 1. 5 участников стартуют как Follower (те 0 Leaders)
    /// 2. спустя время выбирают одного лидера
    /// 3. лидер после рассылает ping с новым номером эпохи
    /// 4. остальные участники 
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

        let node0 = NodeInstance { node: Arc::new(AsyncMutex::new(node0)) };
        let node1 = NodeInstance { node: Arc::new(AsyncMutex::new(node1)) };
        let node2 = NodeInstance { node: Arc::new(AsyncMutex::new(node2)) };
        let node3 = NodeInstance { node: Arc::new(AsyncMutex::new(node3)) };
        let node4 = NodeInstance { node: Arc::new(AsyncMutex::new(node4)) };

        let nodes = vec![
            node0.clone(),
            node1.clone(),
            node2.clone(),
            node3.clone(),
            node4.clone(),
        ];

        let cycle_no = Arc::new(AsyncMutex::new(0));
        System::new().block_on(async move {
            let log = Arc::new(SyncMutex::new(Vec::<Event>::new()));

            // link nodes
            for node in &nodes {
                for target in &nodes {                    
                    let nc = NodeClientMock(target.node.clone(), log.clone());
                    let target_id = target.node.lock().await.id.clone();
                    let mut node = node.node.lock().await;
                    if target_id != node.id {
                        println!("link {from} to {to}",
                            from = node.id,
                            to = target_id,
                        );
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
                    let mut cycle_no = cycle_no.lock().await;
                    *cycle_no += 1;
                    println!("\n== {cycle} ==", cycle=cycle_no);

                    log_bg.push(Event::CycleBegin { cycle_no: cycle_no.clone() });
                    {
                        let mut rng = rand::thread_rng();
                        nodes.shuffle(&mut rng);
                    }

                    join_all(nodes.iter_mut().map(|n| n.on_timer())).await;

                    for node in nodes {
                        let node = node.node.lock().await;
                        println!("{nid} {role:?} epoch {epoch}",
                            nid   = node.id,
                            role  = node.role,
                            epoch = node.epoch,
                        );

                        log_bg.push(Event::NodeState { 
                            node_id: node.id.clone(), 
                            role: node.role.clone() 
                        });
                    }

                    log_bg.push(Event::CycleEnd { cycle_no: cycle_no.clone() });
                    println!();
                }
            });
            bg.set_duration(Duration::from_millis(1000));

            let _ = bg.start();
            sleep(Duration::from_secs(10)).await;

            println!("");

            // Проверка результатов
            {
                let log = log.lock().unwrap();
                for e in log.iter() {
                    println!("{e:?}")
                }
            }

            // 5 участников стартуют как Follower (те 0 Leaders)
            let leaders_matched = log.state_changes_match(|n| n.leaders, vec![0,1]);
            assert!(leaders_matched,"leaders_matched");

            println!("");
        });
    }
}