use std::sync::Arc;

use log::info;
use tokio::sync::Mutex as AMutex;

use crate::BgJob;
use super::bg_tasks::*;

pub mod job {
    use super::super::bg_tasks::BgErr;
    use super::super::bg_tasks::BgJob;
    use super::super::bg_tasks::StopHandle;
    use super::super::bg_tasks::Starter;

    pub trait Job {
        fn is_running( &self ) -> bool;
        fn stop( &mut self );
        fn start( &mut self ) -> Result<(), BgErr>;
    }

    impl <F,H> Job for BgJob<F,H> 
    where 
        H: StopHandle,
        Self: Starter
    {
        fn is_running( &self ) -> bool {
            match &self.handle {
                None => false,
                Some(h) => ! h.is_finished()
            }
        }

        fn stop( &mut self ) {
            self.stop_signal()
        }

        fn start( &mut self ) -> Result<(), BgErr> {
            <BgJob<F,H> as Starter>::start(self)
        }
    }
}

/// Состояние сервера
pub struct RaftState
{
    pub bg_job : Option<Box<dyn job::Job + Send + Sync>>
}

impl RaftState {
    pub async fn on_timer( &self ) {
        info!("raft on timer")
    }
}

