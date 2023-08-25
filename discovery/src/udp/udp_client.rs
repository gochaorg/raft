use std::net::SocketAddr;
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::fmt::Debug;
use log::{warn, info};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::time::timeout;
use tokio::{sync::Mutex, net::UdpSocket};
use async_trait::async_trait;
use super::*;

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
        let result : Arc<Mutex<Vec<A>>> = Arc::new(Mutex::new(vec![]));
        let res = result.clone();
        let send_to = |a:SocketAddr| async move {
            let msg = DiscoveryRequest::Hello { pub_address: self.pub_address.lock().await.clone() };            
            match serde_json::to_string(&msg) {
                Err(e) => { Err(UdpErr::EncodeJson(e.to_string())) }
                Ok(json) => {
                    let bytes = json.into_bytes();
                    info!("send to {a:?}, {} bytes", bytes.len());

                    match self.socket.send_to(&bytes, a).await {
                        Err(e) => { Err(UdpErr::WriteSocket(e.to_string())) }
                        Ok(_sended_size) => { Ok(()) }
                    }
                }
            }
        };

        for a in self.targets.clone() {
            match (send_to.clone())(a.clone()).await {
                Ok(_) => {},
                Err(e) => { warn!("send error {e}") }
            }
        }

        let wait_for_results = || async move {
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
                                result.lock().await.push(pub_address.clone())
                            }
                        }
                    }
                }
            }
        };

        wait_for_results().await;
        let res = res.lock().await.clone();

        Ok(res)
    }
}