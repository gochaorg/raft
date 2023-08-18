use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use range::Range;
use range::product;

/// Диапазон ip адресов
pub enum IpRange {
    Ip4( Range<u8>, Range<u8>, Range<u8>, Range<u8> ),
    Ip6( Range<u16>, Range<u16>, Range<u16>, Range<u16>, Range<u16>, Range<u16>, Range<u16>, Range<u16> )
}

fn iter_v4( r1:Range<u8>, r2:Range<u8>, r3:Range<u8>, r4:Range<u8> ) -> impl Iterator<Item = IpAddr> {
    product(product(product(r1, r2),r3),r4)
        .map(|(((v1,v2),v3),v4)| (v1,v2,v3,v4))
        .map(|(v1,v2,v3,v4)| IpAddr::V4(Ipv4Addr::new(v1, v2, v3, v4)))
}

fn iter_v6( r1:Range<u16>, r2:Range<u16>, r3:Range<u16>, r4:Range<u16>, r5:Range<u16>, r6:Range<u16>, r7:Range<u16>, r8:Range<u16> ) -> impl Iterator<Item = IpAddr> {
    product(product(product(product(product(product(product(r1, r2),r3),r4),r5),r6),r7),r8)
        .map(|(((((((v1,v2),v3),v4),v5),v6),v7),v8)| (v1,v2,v3,v4,v5,v6,v7,v8))
        .map(|(v1,v2,v3,v4,v5,v6,v7,v8)| IpAddr::V6(Ipv6Addr::new(v1, v2, v3, v4,v5,v6,v7,v8)))
}


impl IntoIterator for IpRange {
    type Item = IpAddr;
    type IntoIter = Box<dyn Iterator<Item = IpAddr>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            IpRange::Ip4(r1, r2, r3, r4) => {
                Box::new(iter_v4(r1, r2, r3, r4))
            }
            IpRange::Ip6(r1, r2, r3, r4, r5, r6, r7, r8) => {
                Box::new(iter_v6(r1, r2, r3, r4, r5, r6, r7, r8))
            }
        }
    }
}

#[test]
fn itreee() {
    let r1 : Range<u8> = todo!();
    let r2 : Range<u8> = todo!();
    let r3 : Range<u8> = todo!();
    let r4 : Range<u8> = todo!();
    let it = 
        product(product(product(r1, r2),r3),r4)
        .map(|(((v1,v2),v3),v4)| (v1,v2,v3,v4))
        .map(|(v1,v2,v3,v4)| Ipv4Addr::new(v1, v2, v3, v4))
        ;
}