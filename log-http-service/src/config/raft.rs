use std::{time::Duration, collections::HashMap, sync::{Arc, Mutex}, net::{SocketAddr, SocketAddrV4, SocketAddrV6}};

use discovery::{DiscoveryService, DiscoverClient, UdpService, IpRangeParser, UdpClientEach, SockAddrRange};
use either::Either::Left;
use parse::{DurationParser, Parser};
use serde::{Deserialize, Serialize, Deserializer, de::Error, Serializer};
use tokio::{net::UdpSocket, sync::Mutex as AsyncMutex};
use std::net::IpAddr;
use range::*;
use super::AppConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftConfig {
    #[serde(default="raft_enabled_default")]
    pub enabled: bool,

    /// Базовый адрес URL на котором запущен WebServer
    /// 
    /// Например: `http://my-hosting.com:8080`
    #[serde(default)]
    pub base_url: Option<String>,

    #[serde(default)]
    /// Идентификатор узла кластера
    pub id: NodeId,

    /// Период с которым рассылать пинги
    #[serde(
        deserialize_with="duration_from_str", 
        serialize_with="duration_to_str",
        default="ping_period_default"
    )]
    pub ping_period: Duration,

    #[serde(
        deserialize_with="duration_from_str", 
        serialize_with="duration_to_str",
        default="heartbeat_timeout_default"
    )]
    /// Таймайут после которо переход в кандидата
    pub heartbeat_timeout: Duration,

    #[serde(
        deserialize_with="duration_from_str", 
        serialize_with="duration_to_str",
        default="nominate_min_delay_default"
    )]
    /// Минимальная задержка ответа номинанту
    pub nominate_min_delay : Duration,

    #[serde(
        deserialize_with="duration_from_str", 
        serialize_with="duration_to_str",
        default="nominate_max_delay_default"
    )]
    /// Максимальная задержка ответа номинанту
    pub nominate_max_delay: Duration,

    #[serde(
        deserialize_with="duration_from_str", 
        serialize_with="duration_to_str",
        default="renominate_min_delay_default"
    )]
    /// Минимальная задержка перед повтором самовыдвижения
    pub renominate_min_delay: Duration,

    #[serde(
        deserialize_with="duration_from_str", 
        serialize_with="duration_to_str",
        default="renominate_max_delay_default"
    )]
    /// Максимальная задержка перед повтором самовыдвижения
    pub renominate_max_delay: Duration,

    #[serde(default="votes_min_count_default")]
    /// Минимальное кол-во голосов для успеха
    pub votes_min_count: u32,

    #[serde(default="discovery_default")]
    /// Как обнаруживать сервера
    pub discovery: Option<Discovery>,
}

fn raft_enabled_default() -> bool { false }
fn ping_period_default() -> Duration { Duration::from_secs(3) }
fn heartbeat_timeout_default() -> Duration { Duration::from_secs(15) }
fn nominate_min_delay_default() -> Duration { Duration::from_millis(2) }
fn nominate_max_delay_default() -> Duration { Duration::from_millis(2000) }
fn renominate_min_delay_default() -> Duration { Duration::from_secs(6) }
fn renominate_max_delay_default() -> Duration { Duration::from_secs(10) }
fn votes_min_count_default() -> u32 { 2 }
fn discovery_default() -> Option<Discovery> { None }
// . . . . . . . . . . .

fn duration_from_str<'de, D>(deserializer: D) -> Result<Duration,D::Error> 
where D: Deserializer<'de>
{
    let s: &str = Deserialize::deserialize(deserializer)?;    
    let parser = DurationParser;
    match parser.parse(s) {
        Some( (dur,_) ) => Ok(dur),
        None => Err(D::Error::custom(format!("can't parse '{s}' as Duration, expect like '10 sec' or '12 ms' or ...")))
    }
}

fn duration_to_str<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let str = DurationParser::to_string(value.clone());
    serializer.serialize_str(&str)
}

/// Имя узла кластера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeId {
    /// Стабильное имя
    Name(String),

    /// Генерировать
    Generate
}

impl NodeId {
    /// Генерация случайного имени
    pub fn generate( len:usize ) -> String {
        use rand::*;
        let letters = "qwertyuiopasdfghjklzxcvbnm1234567890";
        let letters_count = letters.chars().count() as u8;
        let mut id = String::new();
        for _ in 0..len {
            let i = random::<u8>();
            let i = i % letters_count;

            let s : String = letters.chars().skip(i as usize).take(1).collect();
            id.push_str(&s);
        }
        id
    }
}

impl Default for RaftConfig {
    fn default() -> Self {
        Self { 
            enabled: raft_enabled_default(),
            base_url: None,
            id: NodeId::Generate,
            ping_period: ping_period_default(),
            heartbeat_timeout: heartbeat_timeout_default(),
            nominate_min_delay: nominate_min_delay_default(),
            nominate_max_delay: nominate_max_delay_default(),
            renominate_min_delay: renominate_min_delay_default(),
            renominate_max_delay: renominate_max_delay_default(),
            votes_min_count: votes_min_count_default(),
            discovery: None
        }
    }
}

impl Default for NodeId {
    fn default() -> Self {
        NodeId::Generate
    }
}

/// Как обнаруживать сервера в сети
#[derive(Debug,Serialize,Deserialize,Clone)]
pub enum Discovery {
    /// Использовать UDP для обнаружения
    UdpDiscovery {
        /// Порт на котором будет запущен UDP
        port: u16,

        /// Адрес на котором будет запущен UDP
        /// 
        /// По умолчанию будет тот же что и веб сервер
        #[serde(default="discovery_udp_bind")]
        bind: Option<IpAddr>,

        /// Адреса куда посылать рассылку
        #[serde(default="discovery_udp_targets")]
        targets: UdpDiscoveryTargets,

        /// Сколько ждать времени ответа от серверов
        #[serde(
            deserialize_with="duration_from_str", 
            serialize_with="duration_to_str",
            default="discovery_udp_recieve_timeout"
        )]    
        recieve_timeout: Duration
    }
}

fn discovery_udp_bind() -> Option<IpAddr> {
    None
}

fn discovery_udp_targets() -> UdpDiscoveryTargets {
    UdpDiscoveryTargets::IpList(vec![])
}

fn discovery_udp_recieve_timeout() -> Duration {
    Duration::from_secs(1)
}

/// Адреса по которым будет рассылка
#[derive(Debug,Serialize,Deserialize,Clone)]
pub enum UdpDiscoveryTargets {
    //IpRange(String),
    IpList(Vec<IpAddr>),
}

/// Фоновая задача обнаружения
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum DiscoveryJob {
    /// Запуститься один раз
    Once,

    /// Периодично запускать
    Regular {
        /// Как часто запускать
        #[serde(
            deserialize_with="duration_from_str", 
            serialize_with="duration_to_str",
            default="discovery_job_period"
        )]    
        period: Duration
    }
}

fn discovery_job_period() -> Duration {
    Duration::from_secs(3)
}

pub struct DiscoveryClientAndService {
    pub client:Arc<dyn DiscoverClient<String>>,
    pub service: Arc<dyn DiscoveryService>,
}

impl Discovery {
    pub async fn create_builder( &self, app_config: AppConfig ) -> Result<DiscoveryBuilder,String> {
        match self {
            Discovery::UdpDiscovery { port, bind, targets, recieve_timeout:_ } => {
                let app_web_ip = app_config.web_server.host.parse::<IpAddr>()
                    .map_err(|e| format!("can't parse app_config.web_server.host({host}), error: {e:?}",host = app_config.web_server.host))?;
                let bind_ip = bind.clone().unwrap_or(app_web_ip);

                let sock_addr = match bind_ip {
                    IpAddr::V4(ip4) => {
                        SocketAddr::V4(SocketAddrV4::new(ip4, *port))
                    }
                    IpAddr::V6(ip6) => {
                        SocketAddr::V6(SocketAddrV6::new(ip6, *port, 0,0))
                    }
                };

                let udp_socket = UdpSocket::bind(sock_addr).await
                    .map_err(|e| format!("can't bind UdpSocket to {addr}, error: {e}", addr = sock_addr.clone()))?;

                let base_url = app_config.web_server.base_url()?;

                let udp_socket = Arc::new(udp_socket);

                let targets = match targets {
                    UdpDiscoveryTargets::IpList(lst) => { 
                        let addr_range : Vec<SocketAddr> = lst
                            .iter()
                            .map(|ip| {
                                match ip {
                                    IpAddr::V4(ip4) => {
                                        SocketAddr::V4(SocketAddrV4::new(*ip4, *port))
                                    }
                                    IpAddr::V6(ip6) => {
                                        SocketAddr::V6(SocketAddrV6::new(*ip6, *port, 0,0))
                                    }
                                }
                            }).collect();

                        addr_range
                    }
                };

                Ok(DiscoveryBuilder { socket: udp_socket, client_targets: targets, base_url: base_url.clone() })
            }
        }
    }

    pub async fn create_discovery_and_client( &self, app_config: AppConfig ) -> Result<DiscoveryClientAndService,String> {        
        match self {
            Discovery::UdpDiscovery { port, bind, targets, recieve_timeout:_ } => {
                let app_web_ip = app_config.web_server.host.parse::<IpAddr>()
                    .map_err(|e| format!("can't parse app_config.web_server.host({host}), error: {e:?}",host = app_config.web_server.host))?;
                let bind_ip = bind.clone().unwrap_or(app_web_ip);

                let sock_addr = match bind_ip {
                    IpAddr::V4(ip4) => {
                        SocketAddr::V4(SocketAddrV4::new(ip4, *port))
                    }
                    IpAddr::V6(ip6) => {
                        SocketAddr::V6(SocketAddrV6::new(ip6, *port, 0,0))
                    }
                };

                let udp_socket = UdpSocket::bind(sock_addr).await
                    .map_err(|e| format!("can't bind UdpSocket to {addr}, error: {e}", addr = sock_addr.clone()))?;

                let base_url = app_config.web_server.base_url()?;

                let udp_socket = Arc::new(udp_socket);
                
                let srvc = UdpService::<String>::new(Arc::new(AsyncMutex::new(base_url.clone())), udp_socket.clone());

                // ........... client ..............
                let client = match targets {
                    UdpDiscoveryTargets::IpList(lst) => { 
                        let addr_range : Vec<SocketAddr> = lst
                            .iter()
                            .map(|ip| {
                                match ip {
                                    IpAddr::V4(ip4) => {
                                        SocketAddr::V4(SocketAddrV4::new(*ip4, *port))
                                    }
                                    IpAddr::V6(ip6) => {
                                        SocketAddr::V6(SocketAddrV6::new(*ip6, *port, 0,0))
                                    }
                                }
                            }).collect();

                        let client = UdpClientEach::new(
                            udp_socket.clone(),
                            addr_range,
                            Arc::new(AsyncMutex::new(base_url.clone())),
                            Arc::new(AsyncMutex::new(Duration::from_secs(1)))
                        );

                        client
                    }
                    // UdpDiscoveryTargets::IpRange(range) => {
                    //     let ip_range = IpRangeParser::parse(&IpRangeParser, &range).map(|(v,_)|v);
                    //     let client = match ip_range {
                    //         Some(ip_range) => {
                    //             Ok(SockAddrRange { ip_range: ip_range, port_range: range::Range::Single(*port) })
                    //         }
                    //         None => {
                    //             Err(format!(""))
                    //         }
                    //     }.and_then(|addr_range| 
                    //         Ok(UdpClientEach::new(
                    //             udp_socket.clone(),
                    //             addr_range,
                    //             Arc::new(AsyncMutex::new("".to_string())),
                    //             Arc::new(AsyncMutex::new(Duration::from_secs(1)))
                    //         ))
                    //     ).map(|client| 
                    //         Arc::new(client) as Arc<dyn DiscoverClient<String>> 
                    //     );
                    //     //let sockaddr_range = SockAddrRange { ip_range: ip_range, port_range: range::Range::Single(5000) }
                    // }
                };

                let client:Arc<dyn DiscoverClient<String>> = Arc::new(client);

                Ok( 
                    DiscoveryClientAndService { 
                        client: client,
                        service: Arc::new(srvc) 
                    }
                )
            }
        }
    }
}

pub struct DiscoveryBuilder {
    socket : Arc<UdpSocket>,
    client_targets: Vec<SocketAddr>,
    base_url: String,
}

impl DiscoveryBuilder {
    pub fn create_service( &self ) -> Arc<dyn DiscoveryService> {
        Arc::new(UdpService::<String>::new(Arc::new(AsyncMutex::new(self.base_url.clone())), self.socket.clone()))
    }

    pub fn create_client( &self ) -> Arc<dyn DiscoverClient<String>> {
        let client = UdpClientEach::new(
            self.socket.clone(),
            self.client_targets.clone(),
            Arc::new(AsyncMutex::new(self.base_url.clone())),
            Arc::new(AsyncMutex::new(Duration::from_secs(1)))
        );

        Arc::new( client )
    }
}
