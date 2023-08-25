use std::sync::Arc;
use std::fmt::Debug;
use actix_rt::spawn;
use log::{warn, info, error};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::{sync::Mutex, net::UdpSocket, task::JoinHandle};
use std::sync::Mutex as SyncMutex;
use async_trait::async_trait;
use super::*;

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
                Err(e) => {
                    error!("error in Drop: {e:?}")
                }
            }
        }
    }
}

