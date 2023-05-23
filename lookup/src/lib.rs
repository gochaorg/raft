use std::cell::RefCell;
use std::net::{IpAddr, SocketAddrV4, SocketAddrV6};
use std::{net::SocketAddr, sync::Arc};

use std::str::FromStr;
use trust_dns_client::client::{Client, SyncClient};
use trust_dns_client::op::DnsResponse;
use trust_dns_client::rr::{DNSClass, Name, RData, Record, RecordType};
use trust_dns_client::udp::UdpClientConnection;

/// Поиск серверов
pub trait ServersLookup<Addr> {
    /// Поиск серверов
    ///
    /// # Результат:
    /// Список серверов (адресов)
    fn lookup(&self) -> Arc<Vec<Addr>>;
}

//////////////////////////////////////////////////////////

/// Поиск серверов через запрос к dns серверу
pub struct DnsLookup {
    /// Список серверов к которым выполняется запрос
    /// Принимается первый позитиваный ответ от любого сервера
    pub dns_servers: Box<Vec<SocketAddr>>,

    /// Искомое имя/сервис
    pub dns_name: DnsName,
}

/// DNS Имя сервиса
pub struct DnsName(Name);

/// Парсинг DNS имени
pub struct DnsNameParse(Result<DnsName, String>);

/// Конвертор / парсинг String -> DnsNameParse
impl From<String> for DnsNameParse {
    fn from(value: String) -> Self {
        DnsNameParse(match Name::from_str(&value) {
            Ok(name) => Ok(DnsName(name)),
            Err(err) => Err(err.to_string()),
        })
    }
}

/// Конвертор / парсинг &str -> DnsNameParse
impl From<&str> for DnsNameParse {
    fn from(value: &str) -> Self {
        let res: DnsNameParse = value.to_string().into();
        res
    }
}

/// Создание клиента
pub struct DnsLookupBuilder {
    pub dns_name: DnsName,
}

impl DnsLookup {
    /// Создание клиента
    ///
    /// # Аргументы
    /// name - имя искомого сервиса
    #[allow(dead_code)]
    fn name<N>(name: N) -> Result<DnsLookupBuilder, String>
    where
        N: Into<DnsNameParse>,
    {
        let DnsNameParse(parse): DnsNameParse = name.into();
        parse.map(|name| DnsLookupBuilder { dns_name: name })
    }
}

/// Парсинг адреса ip
pub struct SocketAddrParse(Result<SocketAddr, String>);

/// Конвертор / парсинг &str -> SocketAddrParse
impl From<&str> for SocketAddrParse {
    fn from(value: &str) -> Self {
        let addr: Result<SocketAddr, _> = value.parse();
        match addr {
            Ok(addr) => SocketAddrParse(Ok(addr)),
            Err(err) => SocketAddrParse(Err(err.to_string())),
        }
    }
}

/// Конвертор / парсинг String -> SocketAddrParse
impl From<String> for SocketAddrParse {
    fn from(value: String) -> Self {
        let v: &str = &value.clone();
        let addr: Result<SocketAddr, _> = v.parse();
        match addr {
            Ok(addr) => SocketAddrParse(Ok(addr)),
            Err(err) => SocketAddrParse(Err(err.to_string())),
        }
    }
}

impl DnsLookupBuilder {
    fn dns_server<S>(self, addr: S) -> Result<DnsLookupBuilderB, String>
    where
        S: Into<SocketAddrParse>,
    {
        let SocketAddrParse(parse) = addr.into();
        parse.map(|addr| DnsLookupBuilderB {
            dns_name: self.dns_name,
            dns_servers: Box::new(vec![addr]),
        })
    }
}

pub struct DnsLookupBuilderB {
    pub dns_name: DnsName,
    pub dns_servers: Box<Vec<SocketAddr>>,
}

impl DnsLookupBuilderB {
    fn dns_server<S>(self, addr: S) -> Result<DnsLookupBuilderB, String>
    where
        S: Into<SocketAddrParse>,
    {
        let SocketAddrParse(parse) = addr.into();

        parse.map(|addr| {
            let mut servers = self.dns_servers.clone();
            servers.push(addr);

            DnsLookupBuilderB {
                dns_name: self.dns_name,
                dns_servers: servers,
            }
        })
    }
}

pub trait AddServer {
    type Out: Sized;
    fn dns_server<S>(self, addr: S) -> Result<Self::Out, String>
    where
        S: Into<SocketAddrParse>;
}

impl AddServer for Result<DnsLookupBuilder, String> {
    type Out = DnsLookupBuilderB;
    fn dns_server<S>(self, addr: S) -> Result<Self::Out, String>
    where
        S: Into<SocketAddrParse>,
    {
        self.and_then(|d| d.dns_server(addr))
    }
}

impl AddServer for Result<DnsLookupBuilderB, String> {
    type Out = DnsLookupBuilderB;

    fn dns_server<S>(self, addr: S) -> Result<Self::Out, String>
    where
        S: Into<SocketAddrParse>,
    {
        self.and_then(|d| d.dns_server(addr))
    }
}

/// Создание клиента
pub trait BuildClient {
    type Out;

    /// Создание клиента
    fn build(self) -> Result<Self::Out, String>;
}

impl BuildClient for Result<DnsLookupBuilderB, String> {
    type Out = DnsLookup;

    fn build(self) -> Result<Self::Out, String> {
        self.map(|d| DnsLookup {
            dns_name: d.dns_name,
            dns_servers: d.dns_servers,
        })
    }
}

#[test]
fn build_lookup_test() {
    let _lookup1 = DnsLookup::name("name")
        .dns_server("addr")
        .dns_server("addr")
        .build();
}

impl ServersLookup<IpAddr> for DnsLookup {
    fn lookup(&self) -> Arc<Vec<IpAddr>> {
        let dns_servers = self.dns_servers.clone();
        let mut result = RefCell::new(Vec::<IpAddr>::new());
        let dns_name = self.dns_name.0.clone();

        let lookup = |dns_server: SocketAddr| {
            let conn = UdpClientConnection::new(dns_server).unwrap();
            let client = SyncClient::new(conn);

            let response: DnsResponse = client
                .query(&dns_name, DNSClass::IN, RecordType::A)
                .unwrap();
            let answers: &[Record] = response.answers();

            for a in answers {
                let rec = Record::data(a);
                match rec {
                    Some(rec) => match rec {
                        RData::A(addr) => {
                            result.borrow_mut().push(IpAddr::V4(addr.clone()));
                        }
                        RData::AAAA(addr) => {
                            result.borrow_mut().push(IpAddr::V6(addr.clone()));
                        }
                        _ => {}
                    },
                    None => {}
                }
            }
        };

        for dns_server in dns_servers.into_iter() {
            lookup(dns_server)
        }

        Arc::new(result.get_mut().clone())
    }
}

#[test]
fn test_local() {
    let client = DnsLookup::name("cl.dev.local.")
        .dns_server("127.0.0.53:53")
        .build();
    let client = client.unwrap();
    let result = client.lookup();
    println!("{:?}", result);
}

/// Добавление адреса dns из resolv
pub trait AddServerResolv {
    type Out;

    /// Добавление адреса dns из /etc/resolv.conf
    fn from_resolv_conf(self) -> Result<Self::Out, String>;
}

impl AddServerResolv for Result<DnsLookupBuilder, String> {
    type Out = DnsLookupBuilderB;
    fn from_resolv_conf(self) -> Result<Self::Out, String> {
        let contents =
            std::fs::read_to_string("/etc/resolv.conf").expect("Failed to open resolv.conf");
        let config = resolv_conf::Config::parse(&contents).unwrap();

        let mut addrs = RefCell::new(Vec::<IpAddr>::new());

        for nameserver in config.nameservers {
            match nameserver {
                resolv_conf::ScopedIp::V4(addr) => {
                    addrs.borrow_mut().push(IpAddr::V4(addr));
                }
                resolv_conf::ScopedIp::V6(addr, _) => {
                    addrs.borrow_mut().push(IpAddr::V6(addr));
                }
            }
        }

        let addrs = addrs.get_mut().clone();
        let addrs: Vec<SocketAddr> = addrs
            .iter()
            .map(|addr| match addr {
                IpAddr::V4(addr) => SocketAddr::V4(SocketAddrV4::new(addr.clone(), 53)),
                IpAddr::V6(addr) => SocketAddr::V6(SocketAddrV6::new(addr.clone(), 53, 0, 0)),
            })
            .collect();

        if addrs.is_empty() {
            Err("nameservers not found in /etc/resolv.conf".to_string())
        } else {
            self.map(|d| DnsLookupBuilderB {
                dns_name: d.dns_name,
                dns_servers: Box::new(addrs),
            })
        }
    }
}
