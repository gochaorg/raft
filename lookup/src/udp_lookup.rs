#[allow(unused_imports)]
use derive_more::Display;
#[allow(unused_imports)]
use log::{warn, info, debug};
#[allow(unused_imports)]
use serde::{Serialize, Deserialize};
#[allow(unused_imports)]
use tokio::net::UdpSocket;
#[allow(unused_imports)]
use tokio::runtime::Handle;
#[allow(unused_imports)]
use tokio::spawn;
#[allow(unused_imports)]
use tokio::sync::mpsc;
#[allow(unused_imports)]
use tokio::task::JoinHandle;
#[allow(unused_imports)]
use std::fmt::Display;
#[allow(unused_imports)]
use std::io;
#[allow(unused_imports)]
use std::error::Error;
use std::net::SocketAddr;
#[allow(unused_imports)]
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
#[allow(unused_imports)]
use tokio::sync::Mutex as AsyncMutex;
#[allow(unused_imports)]
use tokio::time::{sleep, timeout};
use std::sync::Mutex as SyncMutex;
//use derive_more

#[allow(dead_code)]
struct UdpListener<F> 
{
    socket: Arc<UdpSocket>,
    async_thread: Option<JoinHandle<()>>,
    buffer: Arc<AsyncMutex<Vec<u8>>>,
    stop_signal: Arc<SyncMutex<bool>>,
    read_timeout: Arc<SyncMutex<Duration>>,
    responder: F
}

impl<F> Drop for UdpListener<F> {
    fn drop(&mut self) {
        match self.stop_signal.lock() {
            Ok(mut v) => { *v = true; },
            Err(_e) => {}
        }
    }
}

#[derive(Debug,Clone,Serialize,Deserialize,Display)]
enum UdpRequest {    
    /// Запрос на регистрацию
    #[display(fmt="Hello id={id} url={url}")]
    Hello {
        /// Идентификатор клиента
        id: String,

        /// Адрес клиента
        url: String,
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
enum UdpResponse {
    /// Ошибка
    Error { message:String },

    /// Клиент успешно зарегистрирован
    Registered { 
        /// Идентификатор сервера
        id: String, 

        /// Адрес api сервера
        url: String
    }
}

#[allow(dead_code)]
async fn udp_listener<Fu>( sock_addr:&str, f:Fu ) -> Result<UdpListener<Fu>,String> 
where
    Fu: Fn(UdpRequest,SocketAddr) -> UdpResponse,
    Fu: Clone + Send
{
    let sock_addr = sock_addr.parse::<SocketAddr>().map_err(|e| format!("address parse error: {e}"))?;
    let sock = UdpSocket::bind(sock_addr).await.map_err(|e| format!("bind to udp {sock_addr} error: {e}"))?;
    let mut buff: Vec<u8> = vec![];
    buff.resize(1024*64, 0);
    Ok( UdpListener { 
        socket: Arc::new(sock), 
        async_thread: None, 
        read_timeout: Arc::new(SyncMutex::new(Duration::from_secs(5))),
        buffer: Arc::new(AsyncMutex::new(buff)), 
        stop_signal: Arc::new(SyncMutex::new(false)), 
        responder: f
    })
}

#[allow(dead_code)]
fn decode_message<'de, R: Deserialize<'de> + Clone>( message: Result<Result<(usize, SocketAddr), io::Error>, tokio::time::error::Elapsed>, bytes:&'de [u8] ) 
-> Result<(R,SocketAddr),String>
{
    message
    .map_err(|e| format!("read timeout {e}"))
    .and_then(|r| r.map_err(|e| format!("read socket error {e}")))
    .and_then(|(data_size, addr_from)| 
        std::str::from_utf8(&bytes[0 .. data_size]).map(|s| (s, addr_from)
    ).map_err(|e| format!("utf8 decode error {e}")) )
    .and_then(|(s, addr_from)|{
        //let s = s.to_string();
        match serde_json::from_str::<'de,R>(&s) {
            Ok(r) => Ok((r.clone(), addr_from)),
            Err(e) => Err(format!("json decode error {e}"))
        }
    })    
}

#[allow(dead_code)]
impl<F,R> UdpListener<F> 
where
    F: Fn(UdpRequest,SocketAddr) -> R + Clone + Send + 'static,
    R: Serialize
{
    fn start( &mut self ) {
        if self.is_running() { return };

        let responder = self.responder.clone();
        let sock = self.socket.clone();
        let buff = self.buffer.clone();
        let stop_signal = self.stop_signal.clone();
        let read_timeout = self.read_timeout.clone();

        self.async_thread = Some(spawn(async move {
            loop {
                let stop_now = { 
                    match stop_signal.lock() {
                        Ok(v) => v.clone(),
                        Err(e) => {
                            warn!("can't lock: {e}");
                            false
                        }
                    }
                };

                if stop_now { break }

                let mut buff = buff.lock().await;
                let read_timeout = { 
                    match read_timeout.lock() {
                        Ok(dur) => dur.clone(),
                        Err(_) => Duration::from_secs(3)
                    }
                };

                let res = timeout( read_timeout, sock.recv_from(&mut buff) ).await;
                let res = decode_message::<UdpRequest>(res, &buff);

                let send_op = match res {
                    Ok( (req,addr) ) => {
                        Ok(((responder)(req,addr), addr))
                    }
                    Err(err) => { Err(err) }
                }.and_then(|(response,addr)| {
                    serde_json::to_string(&response).map_err(|e| format!("encode json error {e}")).map(|r| (r,addr))
                }).map(|(response,addr)| {
                    let send_bytes = response.into_bytes();
                    (send_bytes,addr)
                });

                debug!("send_op {send_op:?}");

                match send_op {
                    Err(err) => {
                        log::warn!("recieve error: {err}");
                    }
                    Ok((data,addr)) => {
                        match sock.send_to(&data, addr).await {
                            Err(e) => {
                                log::warn!("send error: {e}");
                            }
                            Ok(_) => {}
                        }
                    }
                }
            }
        }))
    }

    fn stop( &mut self ) {
        {
            match self.stop_signal.lock() {
                Ok(mut v) => { *v = true },
                Err(e) => {
                    warn!("can't lock {e}")
                }
            }
        }
        self.async_thread = None;
    }

    fn is_running( &self ) -> bool { 
        match &self.async_thread {
            Some(hdl) => ! hdl.is_finished(),
            None => false
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_listener_only() {
    let mut listener = udp_listener(
        "0.0.0.0:12000", |req,addr| {
            println!("accept request {req} from {addr}");
            UdpResponse::Registered { id: "()".to_string(), url: "()".to_string() }
        }).await.unwrap();

    listener.start();

    let sock2 = UdpSocket::bind("0.0.0.0:8084".parse::<SocketAddr>().unwrap()).await.unwrap();
    sock2.set_broadcast(true).unwrap();

    let req = UdpRequest::Hello { 
        id: "client".to_string(), 
        url: "http://client".to_string()
    };
    let req = serde_json::to_string(&req).unwrap();
    let req = req.into_bytes();
    sock2.send_to(&req, "255.255.255.255:12000".parse::<SocketAddr>().unwrap()).await.unwrap();

    sleep(Duration::from_secs(3)).await;    
}

#[allow(dead_code)]
struct UdpClient {
    socket: Arc<UdpSocket>,
    bind_address: SocketAddr,
    broadcast_address: SocketAddr,
    hello_timeout: Arc<SyncMutex<Duration>>,
    read_timeout: Arc<SyncMutex<Duration>>,
    id: String,
    url: String,
    buffer: Arc<AsyncMutex<Vec<u8>>>,    
}

#[allow(dead_code)]
#[derive(Debug,Clone)]
struct UdpSourceResponse {
    response: UdpResponse,
    address: SocketAddr,
}

#[allow(dead_code)]
impl UdpClient {
    async fn new( bind_address:&str, broadcast_addr:&str, id:&str, url:&str ) -> Result<Self,String> {
        let bind_address = bind_address.parse::<SocketAddr>().map_err(|e| format!("can't parse bind_address({bind_address}) error: {e}"))?;
        let broadcast_addr = broadcast_addr.parse::<SocketAddr>().map_err(|e| format!("can't parse broadcast_addr({broadcast_addr}) error: {e}"))?;

        let socket = UdpSocket::bind(bind_address).await.map_err(|e| format!("can't bind to {bind_address}, error {e}"))?;
        socket.set_broadcast(true).map_err(|e| format!("can't set broadcast to true for socket, error {e}"))?;

        let mut buff: Vec<u8> = vec![];
        buff.resize(1024*64, 0);
    
        Ok( 
            UdpClient { 
                socket: Arc::new(socket), 
                bind_address: bind_address, 
                broadcast_address: broadcast_addr, 
                hello_timeout: Arc::new(SyncMutex::new(Duration::from_secs(10))),
                read_timeout: Arc::new(SyncMutex::new(Duration::from_secs(2))), 
                id: id.to_string(), 
                url: url.to_string(), 
                buffer: Arc::new(AsyncMutex::new(buff))
            }
        )
    }

    async fn hello( &self ) -> Result<Vec<Result<UdpSourceResponse,String>>,String> {
        let start = Instant::now();
        let req = UdpRequest::Hello { id: self.id.clone(), url: self.url.clone() };
        let sock = self.socket.clone();
        let hello_timeout = self.hello_timeout.clone();
        let read_timeout = self.read_timeout.clone();
        let buff = self.buffer.clone();

        let send_bytes = match serde_json::to_string(&req) {
            Ok(str) => {
                str.into_bytes()
            },
            Err(e) => {return Err(format!("encode json error: {e}"));}
        };

        match sock.send_to(&send_bytes, self.broadcast_address).await {
            Ok(_sended_size) => {

            },
            Err(e) => {return Err(format!("send data to socket error: {e}"));}
        }

        let mut result: Vec<Result<(UdpResponse,SocketAddr),String>> = vec![];

        loop {
            let hello_timeout = { 
                match hello_timeout.lock() {
                    Ok(t) => t.clone(),
                    Err(_) => Duration::from_secs(10)
                }
            };

            let read_timeout = { 
                match read_timeout.lock() {
                    Ok(t) => t.clone(),
                    Err(_) => Duration::from_secs(3)
                }
            };

            let now = Instant::now();
            if now.duration_since(start) > hello_timeout { break; }

            let mut buff = buff.lock().await;

            debug!("client: try send data");
            let res = timeout(read_timeout, sock.recv_from(&mut buff)).await;
            let res = decode_message::<UdpResponse>(res, &buff);
            
            result.push(res);
        }

        let result : Vec<Result<UdpSourceResponse,String>> = 
            result.iter()
            .map(|r| 
                r.clone().map(|(a,b)| UdpSourceResponse { response: a, address: b } ))
            .collect();

        Ok(result)
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_listeners() {
    let _ = env_logger::builder().filter_level(log::LevelFilter::max()).is_test(true).try_init();

    let mut listener1 = udp_listener(
        "192.160.0.50:12000", |req,addr| {
            println!("listener1 accept request {req} from {addr}");
            UdpResponse::Registered { id: "listener1".to_string(), url: "()".to_string() }
        }).await.unwrap();

    listener1.start();

    let mut listener2 = udp_listener(
        "192.168.0.51:12000", |req,addr| {
            println!("listener2 accept request {req} from {addr}");
            UdpResponse::Registered { id: "listener2".to_string(), url: "()".to_string() }
        }).await.unwrap();
        
    listener2.start();

    let mut listener3 = udp_listener(
        "1192.168.0.52:12000", |req,addr| {
            println!("listener3 accept request {req} from {addr}");
            UdpResponse::Registered { id: "listener3".to_string(), url: "()".to_string() }
        }).await.unwrap();
        
    listener3.start();

    // let client = UdpClient::new("0.0.0.0:12001", "255.255.255.255:12000", "client", "url").await.unwrap();
    // let client = UdpClient::new("0.0.0.0:12001", "127.1.0.255:12000", "client", "url").await.unwrap();
    let client = UdpClient::new("192.168.0.53:12000", "255.255.255.255:12000", "client", "url").await.unwrap();
    //let client = UdpClient::new("127.0.0.4:12000", "127.0.0.3:12000", "client", "url").await.unwrap();
    //let client = UdpClient::new("127.0.0.4:12000", "127.1.0.1:12000", "client", "url").await.unwrap();
    let res = client.hello().await;

    println!("\nhello responses\n{res:?}");

    sleep(Duration::from_secs(2)).await;    
}

