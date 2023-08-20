use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::rc::Rc;

use either::Either::Left;
use either::Either::Right;
use parse::DecNumberParser;
use parse::HexNumberParser;
use parse::Keywords;
use parse::KeywordsBuilder;
use parse::ParseAdd;
use parse::Parser;
use parse::WhiteSpaceParser;
use parse::alternative;
use parse::follow;
use parse::map;
use parse::repeat;
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

#[derive(Clone,Debug,PartialEq)]
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

#[derive(Clone,Debug,PartialEq)]
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

#[derive(Clone,Debug,PartialEq)]
struct HexFromTo16u(pub Hex16u,pub Hex16u);
struct HexFromTo16uParser;

#[derive(Clone,Debug,PartialEq)]
struct FromToDelim;

impl Parser<HexFromTo16u> for HexFromTo16uParser {
    fn parse( &self, source: &str ) -> Option<(HexFromTo16u, parse::CharsCount)> {
        let kw = KeywordsBuilder::new("-", &FromToDelim)
            .build().parser();

        let ws1 = WhiteSpaceParser::parser(WhiteSpaceParser);
        let ws2 = ws1.clone();

        let from : Rc<dyn Parser<Hex16u>> = Rc::new(Hex16uParser);
        let to = from.clone();

        let parse1 = map(from.clone().follow(ws1.clone().follow(kw.clone().follow(ws2.clone().follow(to.clone())))), |(from,(_,(_,(_,to))))| (from.clone(), to.clone()) );
        let parse2 = map(from.clone().follow(ws1.clone().follow(kw.clone().follow(to.clone()))), |(from,(_,(_,to)))| (from.clone(), to.clone()) );
        let parse3 = map(from.clone().follow(kw.clone().follow(ws2.clone().follow(to.clone()))), |(from,(_,(_,to)))| (from.clone(), to.clone()) );
        let parse4 = map(from.clone().follow(kw.clone().follow(to.clone())), |(from,(_,to))| (from.clone(), to.clone()) );
        let parser = map(
            alternative(alternative(alternative(parse1, parse2),parse3),parse4), 
            |r| {
                let r = r.clone();
                match r {
                    Left(v) => match v {
                        Left(v) => match v {
                            Left(v) => v,
                            Right(v) => v
                        },
                        Right(v) => v
                    },
                    Right(v) => v
                }
            }
        );

        parser.parse(source).map(|((from,to),cc)| (HexFromTo16u(from,to),cc))
    }
}

#[derive(Clone,Debug,PartialEq)]
struct DecFromTo8u(pub Dec8u,pub Dec8u);
struct DecFromTo8uParser;

impl Parser<DecFromTo8u> for DecFromTo8uParser {
    fn parse( &self, source: &str ) -> Option<(DecFromTo8u, parse::CharsCount)> {
        let kw = KeywordsBuilder::new("-", &FromToDelim)
            .build().parser();

        let ws1 = WhiteSpaceParser::parser(WhiteSpaceParser);
        let ws2 = ws1.clone();

        let from : Rc<dyn Parser<Dec8u>> = Rc::new(Dec8uParser);
        let to = from.clone();

        let parse1 = map(from.clone().follow(ws1.clone().follow(kw.clone().follow(ws2.clone().follow(to.clone())))), |(from,(_,(_,(_,to))))| (from.clone(), to.clone()) );
        let parse2 = map(from.clone().follow(ws1.clone().follow(kw.clone().follow(to.clone()))), |(from,(_,(_,to)))| (from.clone(), to.clone()) );
        let parse3 = map(from.clone().follow(kw.clone().follow(ws2.clone().follow(to.clone()))), |(from,(_,(_,to)))| (from.clone(), to.clone()) );
        let parse4 = map(from.clone().follow(kw.clone().follow(to.clone())), |(from,(_,to))| (from.clone(), to.clone()) );
        let parser = map(
            alternative(alternative(alternative(parse1, parse2),parse3),parse4), 
            |r| {
                let r = r.clone();
                match r {
                    Left(v) => match v {
                        Left(v) => match v {
                            Left(v) => v,
                            Right(v) => v
                        },
                        Right(v) => v
                    },
                    Right(v) => v
                }
            }
        );

        parser.parse(source).map(|((from,to),cc)| (DecFromTo8u(from,to),cc))
    }
}

#[derive(Clone,Debug,PartialEq)]
enum HexNonBreakRange16u {
    FromTo(HexFromTo16u),
    One(Hex16u)
}

#[derive(Clone,Debug,PartialEq)]
enum DecNonBreakRange8u {
    FromTo(DecFromTo8u),
    One(Dec8u)
}

#[derive(Clone,Debug)]
struct HexNonBreakRange16uParser;
impl Parser<HexNonBreakRange16u> for HexNonBreakRange16uParser {
    fn parse( &self, source: &str ) -> Option<(HexNonBreakRange16u, parse::CharsCount)> {         
        map(alternative(Rc::new(HexFromTo16uParser), Rc::new(Hex16uParser)), |et| match et {
            Left(ft) => HexNonBreakRange16u::FromTo(ft.clone()),
            Right(v) => HexNonBreakRange16u::One(v.clone())
        }).parse(source)
    }
}

#[derive(Clone,Debug)]
struct DecNonBreakRange8uParser;
impl Parser<DecNonBreakRange8u> for DecNonBreakRange8uParser {
    fn parse( &self, source: &str ) -> Option<(DecNonBreakRange8u, parse::CharsCount)> {         
        map(alternative(Rc::new(DecFromTo8uParser), Rc::new(Dec8uParser)), |et| match et {
            Left(ft) => DecNonBreakRange8u::FromTo(ft.clone()),
            Right(v) => DecNonBreakRange8u::One(v.clone())
        }).parse(source)
    }
}

#[derive(Clone,Debug,PartialEq)]
struct HexRange16u(Vec<HexNonBreakRange16u>);
struct HexRange16uParser;
#[derive(Clone)]
struct RangeBreaker;
impl Parser<HexRange16u> for HexRange16uParser {
    fn parse( &self, source: &str ) -> Option<(HexRange16u, parse::CharsCount)> {
        let ws1 = WhiteSpaceParser::parser(WhiteSpaceParser);
        let ws2 = ws1.clone();
        let kw = KeywordsBuilder::new(",", &RangeBreaker)
        .build().parser();

        let delim1 = map(ws1.clone().follow(kw.clone().follow(ws2.clone())), |_| ());
        let delim2 = map(ws1.follow(kw.clone()), |_| ());
        let delim3 = map(kw, |_| ());
        let delim4 = map(alternative(alternative(delim1, delim2), delim3), |_| ());

        let head: Rc<dyn Parser<HexNonBreakRange16u>> = Rc::new(HexNonBreakRange16uParser);
        let tail = map(delim4.follow(head.clone()), |(_,a)| a.clone() );
        let tail = repeat(tail, None, None);

        let parser = map(head.follow(tail), |(head,tail)| {
            let mut lst : Vec<HexNonBreakRange16u> = Vec::new();
            lst.push(head.clone());
            lst.extend_from_slice(tail);
            HexRange16u(lst)
        });

        parser.parse(source)
    }
}

#[derive(Clone,Debug,PartialEq)]
struct DecRange8u(Vec<DecNonBreakRange8u>);
struct DecRange8uParser;
impl Parser<DecRange8u> for DecRange8uParser {
    fn parse( &self, source: &str ) -> Option<(DecRange8u, parse::CharsCount)> {
        let ws1 = WhiteSpaceParser::parser(WhiteSpaceParser);
        let ws2 = ws1.clone();
        let kw = KeywordsBuilder::new(",", &RangeBreaker)
        .build().parser();

        let delim1 = map(ws1.clone().follow(kw.clone().follow(ws2.clone())), |_| ());
        let delim2 = map(ws1.follow(kw.clone()), |_| ());
        let delim3 = map(kw, |_| ());
        let delim4 = map(alternative(alternative(delim1, delim2), delim3), |_| ());

        let head: Rc<dyn Parser<DecNonBreakRange8u>> = Rc::new(DecNonBreakRange8uParser);
        let tail = map(delim4.follow(head.clone()), |(_,a)| a.clone() );
        let tail = repeat(tail, None, None);

        let parser = map(head.follow(tail), |(head,tail)| {
            let mut lst : Vec<DecNonBreakRange8u> = Vec::new();
            lst.push(head.clone());
            lst.extend_from_slice(tail);
            DecRange8u(lst)
        });

        parser.parse(source)
    }
}

#[test]
fn dec_range_test() {
    let res = DecRange8uParser::parse(&DecRange8uParser, "1,3-5");
    println!("{res:?}");
    assert_eq!(res.unwrap().0, DecRange8u(vec![DecNonBreakRange8u::One(Dec8u(1))]))
}