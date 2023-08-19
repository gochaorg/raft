use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::SocketAddrV6;
use range::Range;
use range::product;
use crate::IpRange;

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

/// # Синтаксис
/// 
/// ```
/// SockAddrRangeParser ::= SockV6Range | SockV4Range
/// SockV6Range ::= '[' NumRangeHexU16 { ( ':' | '::' ) NumRangeHexU16 } ']' ':' NumRangeDecU16
/// SockV4Range ::=  NumRangeDecU8 '.' NumRangeDecU8 '.' NumRangeDecU8 '.' NumRangeDecU8 ':' NumRangeDecU16
/// 
/// NumRangeHexU16 ::= MultipleRangeHex
/// MultipleRangeHex ::= RangeHex { ',' RangeHex }
/// RangeHex ::= HexNum [ '-' HexNum ]
/// 
/// NumRangeDecU16 ::= MultipleRangeDec
/// MultipleRangeDec ::= RangeDec { ',' RangeDec }
/// RangeDec ::= DecNum [ '-' DecNum ]
/// 
/// NumRangeDecU8 ::= MultipleRangeDec
/// 
/// DecNum ::= DecDigit { DecDigit }
/// HexNum ::= HexDigit { HexDigit }
/// 
/// DecDigit ::= '0' | '1' | '2' | '3' | '4'
///            | '5' | '6' | '7' | '8' | '9'
/// 
/// HexDigit ::= '0' | '1' | '2' | '3' | '4'
///            | '5' | '6' | '7' | '8' | '9'
///            | 'a' | 'b' | 'c' | 'd' | 'e' | 'f' 
///            | 'A' | 'B' | 'C' | 'D' | 'E' | 'F' 
/// ```
struct SockAddrRangeParser;

impl SockAddrRangeParser {
    fn new() -> Self {
        SockAddrRangeParser
    }
}

struct MultipleRangeNum(Vec<RangeNum>);

enum RangeNum {
    Single(u64),
    FromTo(u64,u64)
}

