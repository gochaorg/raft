use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::SocketAddrV6;
use range::Range;
use range::product;
use super::*;

#[derive(Debug,Clone)]
pub struct SockAddrRange {
    pub ip_range: IpRange,
    pub port_range: Range<u16>
}

impl IntoIterator for &SockAddrRange {
    type Item = SocketAddr;
    type IntoIter = Box<dyn Iterator<Item = SocketAddr>>;

    fn into_iter(self) -> Self::IntoIter {
        let ip_range = self.ip_range.clone();
        let p_range = self.port_range.clone();
        let itr = product(ip_range, p_range)
        .map(|(ip,p)| {
            match ip {
                IpAddr::V4(ip) => {
                    SocketAddr::V4(SocketAddrV4::new(ip.clone(), p))
                },
                IpAddr::V6(ip) => {
                    SocketAddr::V6(SocketAddrV6::new(ip.clone(), p, 0, 0))
                }
            }
        });

        Box::new(itr)
    }
}

impl IntoIterator for SockAddrRange {
    type Item = SocketAddr;
    type IntoIter = Box<dyn Iterator<Item = SocketAddr>>;

    fn into_iter(self) -> Self::IntoIter {
        let ip_range = self.ip_range.clone();
        let p_range = self.port_range.clone();
        let itr = product(ip_range, p_range)
        .map(|(ip,p)| {
            match ip {
                IpAddr::V4(ip) => {
                    SocketAddr::V4(SocketAddrV4::new(ip.clone(), p))
                },
                IpAddr::V6(ip) => {
                    SocketAddr::V6(SocketAddrV6::new(ip.clone(), p, 0, 0))
                }
            }
        });

        Box::new(itr)
    }
}
