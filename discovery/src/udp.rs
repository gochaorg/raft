use std::future::Future;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use std::{sync::Arc, io};
use std::fmt::Debug;
use actix_rt::{spawn, System};
use derive_more::Display;
use log::{warn, info};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::time::{sleep, timeout};
use tokio::{sync::Mutex, net::UdpSocket, task::JoinHandle};
use std::sync::Mutex as SyncMutex;
use async_trait::async_trait;
use super::*;

const BUFFER_SIZE:usize = 1024*64;

#[derive(Debug,Clone,Display)]
pub enum UdpErr {
    #[display(fmt = "DecodeUtf8 {}","_0")]
    DecodeUtf8(String),

    #[display(fmt = "DecodeJson {}","_0")]
    DecodeJson(String),

    #[display(fmt = "ReadSocket {}","_0")]
    ReadSocket(String),

    #[display(fmt = "EncodeJson {}","_0")]
    EncodeJson(String),

    #[display(fmt = "WriteSocket {}","_0")]
    WriteSocket(String),
}

#[derive(Clone)]
pub struct UdpService<A> 
where
    A: Clone+Debug
{
    socket: Arc<UdpSocket>,
    join_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    pub_address: Arc<Mutex<A>>,
    stop_signal: Arc<SyncMutex<bool>>,
    buffer: Arc<Mutex<Vec<u8>>>
}

#[async_trait]
impl<A> DiscoveryService for UdpService<A>
where
    A: Clone+Debug+Send+Sync+DeserializeOwned+Serialize + 'static
{
    async fn start( &mut self ) -> Result<(),DiscoveryError> {
        let mut jh = self.join_handle.lock().await;
        if jh.is_some() {
            let jh = jh.as_ref().unwrap();
            if jh.is_finished() {            
                return Ok(())
            }
        }

        let signal = self.stop_signal.clone();

        { 
            match self.stop_signal.lock() {
                Ok(mut sign) => {
                    *sign = false;
                }
                Err(_) => {
                    return Err(DiscoveryError::UnImplemented)
                }
            }
        }

        let buff = self.buffer.clone();
        let sock = self.socket.clone();

        let pub_address = self.pub_address.clone();

        info!("Udp Service Discovery starting");

        let stop_signal = self.stop_signal.clone();

        // main cycle
        let r = spawn(async move {
            info!("Udp Service Discovery started");

            loop {
                {
                    match stop_signal.lock() {
                        Ok(sign) => {
                            if *sign { break }
                        }
                        Err(_) => {}
                    }
                }

                let mut buff = buff.lock().await;

                let request = sock.recv_from(&mut buff).await.map_err(|e| UdpErr::ReadSocket(e.to_string()))
                .and_then(|(data_size,addr)| std::str::from_utf8(&mut buff[0..data_size]).map(|r| (r,addr)).map_err(|e| UdpErr::DecodeUtf8(e.to_string())))
                .and_then(|(str_request,addr)| serde_json::from_str::<DiscoveryRequest<A>>(str_request).map(|r| (r,addr)).map_err(|e| UdpErr::DecodeJson(e.to_string())) );

                match request {
                    Ok((_request,addr_from)) => {
                        let my_addr = pub_address.lock().await.clone();
                        let response = DiscoveryResponse::Wellcome { pub_address: my_addr };
                        let response = serde_json::to_string(&response);
                        let data_to_send = response
                            .map(|str| str.into_bytes()).map_err(|e| UdpErr::EncodeJson(e.to_string()));
                        match data_to_send {
                            Ok(bytes) => {
                                match sock.send_to(&bytes, addr_from).await.map_err(|e| UdpErr::WriteSocket(e.to_string())) {
                                    Ok(_) => { info!("write response success on {addr_from}") },
                                    Err(e) => { warn!("write error {e}") }
                                }
                            },
                            Err(e) => { warn!("write error: {e}"); }
                        }
                    },
                    Err(e) => { warn!("read error: {e}"); }
                }
            }

            info!("Udp Service Discovery finished");
        });

        *jh = Some(r);

        return Ok(())
    }

    async fn stop( &mut self ) -> Result<(),DiscoveryError> {
        match self.stop_signal.lock() {
            Ok(mut sign) => {
                *sign = true;
            }
            Err(_) => {
                return Err(DiscoveryError::UnImplemented)
            }
        }

        // let mut signal = self.stop_signal.lock().await;
        // *signal = true;

        let jh = self.join_handle.lock().await;
        if jh.is_some() {
            let jh = jh.as_ref().unwrap();
            jh.abort();
        }

        Ok(())
    }

    async fn is_running( &self ) -> Result<bool,DiscoveryError> {
        let jh = self.join_handle.lock().await;
        if jh.is_some() {
            let jh = jh.as_ref().unwrap();
            return Ok(! jh.is_finished())
        }
        return Ok(false)
    }
}

impl<A> UdpService<A>
where
    A: Clone+Debug
{
    pub fn new( pub_addr:Arc<Mutex<A>>, socket:Arc<UdpSocket> ) -> Self {
        let mut buff : Vec<u8> = Vec::new();
        buff.resize(BUFFER_SIZE, 0);

        Self { 
            socket: socket, 
            join_handle: Arc::new(Mutex::new(None)), 
            pub_address: pub_addr, 
            stop_signal: Arc::new(SyncMutex::new(false)), 
            buffer: Arc::new(Mutex::new(buff))
        }
    }
}

impl<A> Drop for UdpService<A>
where
    A: Clone+Debug
{
    fn drop(&mut self) {
        { 
            match self.stop_signal.lock() {
                Ok(mut sign) => {
                    *sign = false;
                }
                Err(_) => {
                    // return Err(DiscoveryError::UnImplemented)
                }
            }
        }
    }
}

pub struct UdpClientEach<A,Targets>
where
    A:Clone+Debug,
    Targets: IntoIterator<Item=SocketAddr>,
{
    socket: Arc<UdpSocket>,
    targets: Targets,
    pub_address: Arc<Mutex<A>>,
    buffer: Arc<Mutex<Vec<u8>>>,
    recieve_timeout: Arc<Mutex<Duration>>,
}

impl<A,Targets> UdpClientEach<A,Targets> 
where
    A:Clone+Debug,
    Targets: IntoIterator<Item=SocketAddr>,
{
    pub fn new( socket: Arc<UdpSocket>, targets: Targets, pub_address: Arc<Mutex<A>> ) -> Self {
        let mut buff : Vec<u8> = Vec::new();
        buff.resize(BUFFER_SIZE, 0);

        Self {
            socket: socket,
            targets: targets,
            pub_address: pub_address,
            buffer: Arc::new(Mutex::new(buff)),
            recieve_timeout: Arc::new(Mutex::new(Duration::from_millis(1000)))
        }
    }
}

#[async_trait]
impl<A,Targets,B> DiscoverClient<A> for UdpClientEach<A,Targets> 
where
    A:Clone+Debug +Send+Sync+DeserializeOwned+Serialize + 'static,
    B:Iterator<Item = SocketAddr> + Send,
    Targets: IntoIterator<Item=SocketAddr, IntoIter = B> + Send + Sync + Clone,
{
    async fn discovery( &self ) -> Result<Vec<A>,DiscoveryError> {
        use futures::stream::*;

        let result : Arc<Mutex<Vec<A>>> = Arc::new(Mutex::new(vec![]));

        let _res = iter(self.targets.clone()).for_each_concurrent(0, |a| async move {
            let msg = DiscoveryRequest::Hello { pub_address: self.pub_address.lock().await.clone() };            
            
            match serde_json::to_string(&msg) {
                Err(e) => { warn!("can't encode json {e:?}") }
                Ok(json) => {
                    let bytes = json.into_bytes();
                    info!("send to {a:?}, {} bytes", bytes.len());

                    match self.socket.send_to(&bytes, a).await {
                        Err(e) => { warn!("send fail {e:?}") }
                        Ok(_sended_size) => {
                            let mut buff = self.buffer.lock().await;
                            let timeout1 = self.recieve_timeout.lock().await.clone();
                            let t1 = Instant::now();
                            
                            loop {
                                let t2 = Instant::now();
                                let dur = t2.duration_since(t1);
                                if dur > timeout1 { break; }

                                let res = timeout(timeout1, self.socket.recv_from(&mut buff)).await;
                                let res = match res {
                                    Err(_) => continue,
                                    Ok(res) => res
                                };

                                let request = res
                                .map_err(|e| UdpErr::ReadSocket(e.to_string()))
                                .and_then(|(data_size,addr)| std::str::from_utf8(&mut buff[0..data_size]).map(|r| (r,addr)).map_err(|e| UdpErr::DecodeUtf8(e.to_string())))
                                .and_then(|(str_request,addr)| serde_json::from_str::<DiscoveryResponse<A>>(str_request).map(|r| (r,addr)).map_err(|e| UdpErr::DecodeJson(e.to_string())) );

                                match request {
                                    Err(_e) => { warn!("recieve error {_e:?}") }
                                    Ok(respone) => {
                                        match respone.0 {
                                            DiscoveryResponse::Error { error_message } => {
                                                warn!("response error {error_message:?}")
                                            }
                                            DiscoveryResponse::Wellcome { pub_address } => {
                                                //info!("response Welcome {pub_address}");
                                                //result.lock().await.push(pub_address.clone())
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }).await;

        Err(DiscoveryError::UnImplemented)
    }
}

#[test]
fn test() {
    use env_logger;
    //env_logger::init();
    let _ = env_logger::builder().filter_level(log::LevelFilter::max()).is_test(true).try_init();

    System::new().block_on(async move {
        let servers_addr = vec![
            "127.0.0.1:5000".parse::<SocketAddr>().unwrap(),
            "127.0.0.2:5000".parse::<SocketAddr>().unwrap(),
            "127.0.0.3:5000".parse::<SocketAddr>().unwrap(),
        ];

        let socket = UdpSocket::bind(servers_addr[0].clone()).await.unwrap();
        let mut srvc = UdpService::<String>::new( 
            Arc::new(Mutex::new("http://service-1".to_string())), 
            Arc::new(socket)
        );
        let _= srvc.start().await;

        let socket = UdpSocket::bind(servers_addr[1].clone()).await.unwrap();
        let mut srvc = UdpService::<String>::new( 
            Arc::new(Mutex::new("http://service-2".to_string())), 
            Arc::new(socket)
        );
        let _= srvc.start().await;

        let socket = UdpSocket::bind(servers_addr[2].clone()).await.unwrap();
        let mut srvc = UdpService::<String>::new( 
            Arc::new(Mutex::new("http://service-3".to_string())), 
            Arc::new(socket)
        );
        let _= srvc.start().await;

        let socket = UdpSocket::bind("127.0.0.10:5000".parse::<SocketAddr>().unwrap()).await.unwrap();
        let client = UdpClientEach::new(
            Arc::new(socket), 
            servers_addr.clone(), 
            Arc::new(Mutex::new("http".to_string()))
        );
        
        let res = client.discovery().await;
        println!("discovery result");
        println!("{res:?}");        
    });
}