use std::{time::Duration, sync::{Arc, Mutex}};
use actix_rt::{task::JoinHandle, spawn, time::sleep, System};
use futures::Future;
use log;

/// Периодичная фоновая задача
pub struct BgJob<F> 
{
    /// Управление асинхронной задачей
    handle: Option<JoinHandle<()>>,

    /// Задержка между повторным выполнением
    throttling: Duration,

    /// Сигнал остановки
    stop_signal: Arc<Mutex<bool>>,

    /// Задача
    job: F,

    name: Option<String>,
}

pub enum BgErr {
    AlreadyRunning
}

pub fn bg_job<Fu,R>( work: Fu ) -> BgJob<Fu> 
    where
        Fu: Fn() -> R
{
    BgJob { 
        handle: None, 
        throttling: Duration::from_millis(100), 
        stop_signal: Arc::new(Mutex::new(false)), 
        job: work,
        name: None,
    }
}

impl<F> BgJob<F> 
{
    pub fn duration( self, value:Duration ) -> Self {
        Self { throttling:value, ..self }
    }

    pub fn name( self, name:&str ) -> Self {
        Self { name: Some(name.to_string()), ..self }
    }

    pub fn is_running( &self ) -> bool { 
        match &self.handle {
            Some(handle) => { ! handle.is_finished() },
            None => { false }
        }
    }

    pub fn stop_signal( &mut self ) -> Result<(),BgErr> {
        if !self.is_running() { return Ok(()) }

        log::info!("stop_signal");

        { 
            let mut signal = self.stop_signal.lock().unwrap(); 
            *signal = true;
        }

        Ok(())
    }

    pub fn stop_force( &mut self ) -> Result<(),BgErr> {
        if !self.is_running() { return Ok(()) }

        log::info!("stop_force");

        match &self.handle {
            Some(handle) => {
                if handle.is_finished() {
                    Ok(())
                } else {
                    handle.abort();
                    Ok(())
                }
            },
            None => { Ok(()) }
        }
}}

impl<F,R> BgJob<F> 
where 
    F: Fn() -> R + Clone,
    F: 'static,
    R: Future<Output = ()>
{
    pub fn start( &mut self ) -> Result<(),BgErr> {
        if self.is_running() { return Err(BgErr::AlreadyRunning) }

        match &self.name {
            Some(name) => log::info!("starting bg job {name}"),
            None => log::info!("starting bg job")
        }

        let throttling = self.throttling.clone();
        let stop_signal = self.stop_signal.clone();
        {
            let mut signal = stop_signal.lock().unwrap();
            *signal = false;
        }
        //let jobs = self.jobs.clone();
        let job = self.job.clone();

        let name = self.name.clone();

        self.handle = Some(spawn( async move {
            loop {
                sleep(throttling).await;
                
                {
                    let has_signal = stop_signal.lock().unwrap();
                    if *has_signal { break; }
                }

                match &name {
                    Some(name) => log::info!("started bg job {name}"),
                    None => log::info!("started bg job")
                }
        
                    let res = (job)();
                res.await
            }

            match &name {
                Some(name) => log::info!("stopped bg job {name}"),
                None => log::info!("stopped bg job")
            }
        }));

        Ok(())
    }
}

#[test]
fn test_bg() {
    let _ = env_logger::builder().filter_level(log::LevelFilter::max()).is_test(true).try_init();

    System::new().block_on( async{
        let mut bg = bg_job(|| async { 
            log::info!("do some work");
        }).duration(Duration::from_millis(1000)).name("test");

        let _ = bg.start();
        
        sleep(Duration::from_secs(4)).await;
        let _ = bg.stop_force();

        sleep(Duration::from_secs(2)).await;
    })
}