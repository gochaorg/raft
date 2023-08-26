use std::{time::Duration, collections::HashMap};

use parse::{DurationParser, Parser};
use serde::{Deserialize, Serialize, Deserializer, de::Error, Serializer};
use std::net::IpAddr;

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

    //pub discovery
}

fn raft_enabled_default() -> bool { false }
fn ping_period_default() -> Duration { Duration::from_secs(3) }
fn heartbeat_timeout_default() -> Duration { Duration::from_secs(15) }
fn nominate_min_delay_default() -> Duration { Duration::from_millis(2) }
fn nominate_max_delay_default() -> Duration { Duration::from_millis(2000) }
fn renominate_min_delay_default() -> Duration { Duration::from_secs(6) }
fn renominate_max_delay_default() -> Duration { Duration::from_secs(10) }
fn votes_min_count_default() -> u32 { 2 }
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

// #[allow(dead_code)]
// /// Публичный адрес
// pub struct PubAddresses {
//     /// Публичный адрес по умолчанию
//     pub address: Option<String>,

//     /// Адрес для конкретного узла - id узла
//     pub for_node: HashMap<String,String>,

//     /// Адрес для конкртеного узла - ip адрес узла
//     pub for_ip: HashMap<String,String>,
// }

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
        }
    }
}

impl Default for NodeId {
    fn default() -> Self {
        NodeId::Generate
    }
}

/// Как обнаруживать сервера в сети
pub enum Discovery {
    /// Использовать UDP для обнаружения
    UdpDiscovery {
        /// Порт на котором будет запущен UDP
        port: u16,

        /// Адрес на котором будет запущен UDP
        bind: IpAddr,

        /// Адреса куда посылать рассылку
        targets: UdpDiscoveryTargets,

        /// Сколько ждать времени ответа от серверов
        recieve_timeout: Duration
    }
}

/// Адреса по которым будет рассылка
pub enum UdpDiscoveryTargets {
    IpRange(String),
    IpList(Vec<IpAddr>),
}

/// Фоновая задача обнаружения
pub enum DiscoveryJob {
    /// Запуститься один раз
    Once,

    /// Периодично запускать
    Regular {
        /// Как часто запускать
        period: Duration
    }
}