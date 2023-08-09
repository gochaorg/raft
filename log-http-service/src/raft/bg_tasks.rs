use std::{time::Duration, sync::{Arc, Mutex}, thread::JoinHandle};
use actix_rt::task::JoinHandle as AsyncJoinHandle;
use futures::Future;
use log;

/// Периодичная фоновая задача
pub struct BgJob<F,H> 
{
    /// Управление асинхронной задачей
    handle: Option<H>, 

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

/// Создание асинхронной фоновой периодичной задачи
#[allow(dead_code)]
pub fn bg_job_async<Fu,R>( work: Fu ) -> BgJob<Fu, AsyncJoinHandle<()>> 
    where
        Fu: Fn() -> R,        
        R: Future<Output = ()>
{    
    BgJob { 
        handle: None, 
        throttling: Duration::from_millis(100), 
        stop_signal: Arc::new(Mutex::new(false)), 
        job: work,
        name: None,
    }
}

/// Создание синхронной фоновой периодичной задачи - создается отдельный поток
#[allow(dead_code)]
pub fn bg_job_sync<Fu>( work: Fu ) -> BgJob<Fu, JoinHandle<()>> 
    where
        Fu: Fn()
{
    BgJob { 
        handle: None, 
        throttling: Duration::from_millis(100), 
        stop_signal: Arc::new(Mutex::new(false)), 
        job: work,
        name: None,
    }
}

#[allow(dead_code)]
impl<F,H> BgJob<F,H> 
{
    /// Указывает задержку перед запуском задачи
    pub fn set_duration( &mut self, value:Duration ) {
        self.throttling = value;
    }

    /// Указывает имя задачи
    pub fn set_name( &mut self, name:&str ) {
        self.name = Some(name.to_string());
    }

    /// Посылает сигнал для завершения задачи
    pub fn stop_signal( &mut self ) {
        log::info!("stop_signal");

        { 
            let mut signal = self.stop_signal.lock().unwrap(); 
            *signal = true;
        }
    }
}

pub trait StopHandle {
    fn is_finished( &self ) -> bool;
}

pub trait Starter {
    fn start( &mut self ) -> Result<(),BgErr>;
}

impl<F,H> BgJob<F,H> 
where
    H: StopHandle
{
    /// Проверяет что фоновая задача выполняется или будет запущена
    pub fn is_running( &self ) -> bool { 
        match &self.handle {
            Some(handle) => { ! handle.is_finished() },
            None => { false }
        }
    }
}

impl StopHandle for AsyncJoinHandle<()> {
    fn is_finished( &self ) -> bool {
        self.is_finished()
    }
}

impl<F,R> Starter for BgJob<F,AsyncJoinHandle<()>> 
where 
    F: Fn() -> R + Clone,
    F: 'static,
    R: Future<Output = ()>
{
    /// Запук периодичного выполнения фоновой задачи
    fn start( &mut self ) -> Result<(),BgErr> {
        use actix_rt::{spawn, time::sleep};

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

        let job = self.job.clone();

        let name = self.name.clone();

        let main_loop = spawn( async move {
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
        });

        self.handle = Some(main_loop.into());

        Ok(())
    }
}

#[test]
fn test_bg_async() {
    use actix_rt::System;
    use actix_rt::time::sleep;
    use std::sync::atomic::AtomicUsize;

    let _ = env_logger::builder().filter_level(log::LevelFilter::max()).is_test(true).try_init();

    let cnt_run = Arc::new(AtomicUsize::new(0));
    let cnt_run2 = cnt_run.clone();
    System::new().block_on( async move {
        let job = move || {
            cnt_run.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            async { 
                log::info!("do some work");
            }
        };

        //let mut bg = BgJob::fromz(job);
        let mut bg = bg_job_async(job);            

        bg.set_duration(Duration::from_millis(1000));
        bg.set_name("test");

        let _ = bg.start();
        
        sleep(Duration::from_secs(4)).await;
        bg.stop_signal();

        sleep(Duration::from_secs(2)).await;
    });

    let cnt = cnt_run2.fetch_and(0, std::sync::atomic::Ordering::SeqCst);
    println!("run count {cnt}" );
    assert!( cnt > 2usize );
}

impl StopHandle for JoinHandle<()> {
    fn is_finished( &self ) -> bool {
        self.is_finished()
    }
}

impl<F> Starter for BgJob<F,JoinHandle<()>> 
where 
    F: Fn() + Clone + Send,
    F: 'static,
{
    fn start( &mut self ) -> Result<(),BgErr> {
        use std::thread::spawn;
        use std::thread::sleep;

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

        let job = self.job.clone();
        let name = self.name.clone();

        let main_loop = spawn(move || {
            loop {
                sleep(throttling);

                {
                    let has_signal = stop_signal.lock().unwrap();
                    if *has_signal { break; }
                }

                match &name {
                    Some(name) => log::info!("started bg job {name}"),
                    None => log::info!("started bg job")
                }
        
                (job)();
            }

            match &name {
                Some(name) => log::info!("stopped bg job {name}"),
                None => log::info!("stopped bg job")
            }
        });

        self.handle = Some(main_loop.into());

        Ok(())
    }
}

impl<F,H> Drop for BgJob<F,H> {
    fn drop(&mut self) {
        self.stop_signal();
    }
}

#[test]
fn test_bg() {
    use std::thread::sleep;

    let mut bg = bg_job_sync( || {
        println!("do some work native")
    });
    bg.set_duration(Duration::from_secs(1));
    bg.set_name("test native");

    let _ = bg.start();

    sleep(Duration::from_secs(4));
    bg.stop_signal();

    sleep(Duration::from_secs(2));    
}
