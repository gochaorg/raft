use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::rc::Rc;

use parse::DecNumberParser;
use parse::HexNumberParser;
use parse::Keywords;
use parse::KeywordsBuilder;
use parse::Parser;
use parse::WhiteSpaceParser;
use parse::follow;
use range::Range;
use range::product;

/// Диапазон ip адресов
/// 
/// Синтаксис
/// 
/// ```
/// IpRange ::= Ip6Range | Ip4Range
/// Ip6Range ::= '[' HexRange16u { ( ':' | '::' ) HexRange16u } ']'
/// Ip4Range ::= DecRange8u '.' DecRange8u '.' DecRange8u '.' DecRange8u 
/// HexRange16u ::= HexNonBreakRange16u { ',' HexNonBreakRange16u }
/// DecRange8u ::= DecNonBreakRange8u { ',' DecNonBreakRange8u }
/// HexNonBreakRange16u ::= HexFromTo16u | HexSingle16u
/// DecNonBreakRange8u ::= DecFromTo8u | DecSingle8u
/// HexFromTo16u ::= Hex16u '-' Hex16u
/// HexSingle16u ::= Hex16u
/// DecFromTo8u ::= Dec8u '-' Dec8u
/// DecSingle8u ::= Dec8u
/// ```
#[derive(Debug,Clone)]
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


impl IntoIterator for &IpRange {
    type Item = IpAddr;
    type IntoIter = Box<dyn Iterator<Item = IpAddr>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            IpRange::Ip4(r1, r2, r3, r4) => {
                Box::new(iter_v4(r1.clone(), r2.clone(), r3.clone(), r4.clone()))
            }
            IpRange::Ip6(r1, r2, r3, r4, r5, r6, r7, r8) => {
                Box::new(iter_v6(r1.clone(), r2.clone(), r3.clone(), r4.clone(), r5.clone(), r6.clone(), r7.clone(), r8.clone()))
            }
        }
    }
}

impl IntoIterator for IpRange {
    type Item = IpAddr;
    type IntoIter = Box<dyn Iterator<Item = IpAddr>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            IpRange::Ip4(r1, r2, r3, r4) => {
                Box::new(iter_v4(r1.clone(), r2.clone(), r3.clone(), r4.clone()))
            }
            IpRange::Ip6(r1, r2, r3, r4, r5, r6, r7, r8) => {
                Box::new(iter_v6(r1.clone(), r2.clone(), r3.clone(), r4.clone(), r5.clone(), r6.clone(), r7.clone(), r8.clone()))
            }
        }
    }
}

#[derive(Clone)]
struct Dec8u(pub u8);
struct Dec8uParser;
impl Parser<Dec8u> for Dec8uParser {
    fn parse( &self, source: &str ) -> Option<(Dec8u, parse::CharsCount)> {
        let parser = DecNumberParser;
        parser.parse(source).and_then(
            |(n,cc)| n.try_u8().map(|n| (Dec8u(n),cc))
        )
    }
}

#[derive(Clone)]
struct Hex16u(pub u16);
struct Hex16uParser;
impl Parser<Hex16u> for Hex16uParser {    
    fn parse( &self, source: &str ) -> Option<(Hex16u, parse::CharsCount)> {
        let parser = HexNumberParser;
        parser.parse(source).and_then(
            |(n,cc)| n.try_u16().map(|n| (Hex16u(n),cc))
        )
    }
}

#[derive(Clone)]
struct HexFromTo16u(pub Hex16u,pub Hex16u);
struct HexFromTo16uParser;

#[derive(Clone)]
struct FromToDelim;

impl Parser<HexFromTo16u> for HexFromTo16uParser {
    fn parse( &self, source: &str ) -> Option<(HexFromTo16u, parse::CharsCount)> {
        let kw = KeywordsBuilder::new("-", &FromToDelim)
            .build().parser();

        let ws = WhiteSpaceParser::parser(WhiteSpaceParser);

        let from : Rc<dyn Parser<Hex16u>> = Rc::new(Hex16uParser);
        let to = from.clone();

        follow(follow(follow
            (from, ws.clone()), kw), ws.clone())
        ;
        todo!()
    }
}

