use std::fmt::Debug;
use derive_more::Display;
use super::*;

mod udp_client;
pub use udp_client::*;

mod udp_service;
pub use udp_service::*;

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

#[test]
fn udp_each_test() {
    use parse::Parser;
    use tokio::{sync::Mutex, net::UdpSocket};
    use actix_rt::System;
    use std::sync::Arc;
    use std::net::SocketAddr;

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

        let ip_range = IpRangeParser::parse(&IpRangeParser, 
            "127  .  0  .  0  .  1-3"
        ).map(|(v,_)|v).unwrap();
        let addr_range = SockAddrRange { ip_range: ip_range, port_range: range::Range::Single(5000) };     
        let addr_range: Vec<SocketAddr> = addr_range.into_iter().collect();   

        let socket = UdpSocket::bind("127.0.0.10:5000".parse::<SocketAddr>().unwrap()).await.unwrap();
        let client = UdpClientEach::new(
            Arc::new(socket), 
            //servers_addr.clone(), 
            addr_range,
            Arc::new(Mutex::new("http".to_string()))
        );
        
        let res = client.discovery().await;
        println!("discovery result");
        println!("{res:?}");        
    });
}