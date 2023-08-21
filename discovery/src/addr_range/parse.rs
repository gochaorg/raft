use std::rc::Rc;

use either::Either::Left;
use either::Either::Right;
use parse::DecNumberParser;
use parse::HexNumberParser;
use parse::KeywordsBuilder;
use parse::ParseAdd;
use parse::Parser;
use parse::WhiteSpaceParser;
use parse::alternative;
use parse::and_then;
//use parse::follow;
use parse::map;
use parse::repeat;
use range::Range;

use crate::IpRange;
//use range::Range;
//use range::product;

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

#[test]
fn dec8u_test() {
    assert_eq!( Dec8uParser::parse(&Dec8uParser, "12").map(|(v,_)| v), Some(Dec8u(12)) );
}

/////////////////////////////////////

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

#[test]
fn hex16u_test() {
    assert_eq!( Hex16uParser::parse(&Hex16uParser, "12").map(|(v,_)| v), Some(Hex16u(16+2)) );
}

/////////////////////////////////////

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

#[test]
fn from_to_hex_test() {
    assert_eq!( HexFromTo16uParser::parse(&HexFromTo16uParser, "8-12").map(|(v,_)| v), Some(HexFromTo16u(Hex16u(8),Hex16u(16+2))) );
}
/////////////////////////////////////

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

#[test]
fn from_to_dec_test() {
    assert_eq!( DecFromTo8uParser::parse(&DecFromTo8uParser, "8-12").map(|(v,_)| v), Some(DecFromTo8u(Dec8u(8),Dec8u(12))) );
}
/////////////////////////////////////

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

#[test]
fn hex_nonbr_range_test() {
    assert_eq!( HexNonBreakRange16uParser::parse(&HexNonBreakRange16uParser, "8-12").map(|(v,_)| v), Some(HexNonBreakRange16u::FromTo( HexFromTo16u(Hex16u(8),Hex16u(16+2))) ) );
}

/////////////////////////////////////

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

impl From<HexRange16u> for Range<u16> {
    fn from(value: HexRange16u) -> Self {
        Range::Multiple(
            value.0.iter().map(|r| match r {
                HexNonBreakRange16u::FromTo(from_to) => Range::FromToInc(from_to.0.0, from_to.1.0),
                HexNonBreakRange16u::One(one) => Range::Single(one.0)
            }).collect()
        )
    }
}

#[test]
fn hex_range_test() {
    assert_eq!( HexRange16uParser::parse(&HexRange16uParser, "1,2,10-12").map(|(v,_)| v), 
        Some(HexRange16u(
            vec![
                HexNonBreakRange16u::One(Hex16u(1)),
                HexNonBreakRange16u::One(Hex16u(2)),
                HexNonBreakRange16u::FromTo(HexFromTo16u(Hex16u(16),Hex16u(16+2)))
            ]
        )) 
    );
}

/////////////////////////////////////

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

impl From<DecRange8u> for Range<u8> {
    fn from(value: DecRange8u) -> Self {
        Range::Multiple(
            value.0.iter().map(|a| 
                match a {
                    DecNonBreakRange8u::FromTo(from_to) => {
                        Range::FromToInc(from_to.0.0, from_to.1.0)
                    },
                    DecNonBreakRange8u::One(a) => {
                        Range::Single(a.0)
                    }
                }
            ).collect()
        )
    }
}

#[test]
fn dec_range_test() {
    let res = DecRange8uParser::parse(&DecRange8uParser, "1,3-5");
    println!("{res:?}");
    assert_eq!(res.unwrap().0, DecRange8u(vec![
        DecNonBreakRange8u::One(Dec8u(1)),
        DecNonBreakRange8u::FromTo(DecFromTo8u(Dec8u(3),Dec8u(5))),
    ]))
}

/////////////////////////////////////

#[derive(Clone,Debug)]
struct Ip4Delim;

#[derive(Clone,Debug,PartialEq)]
struct Ip4Range(pub DecRange8u, pub DecRange8u, pub DecRange8u, pub DecRange8u);

struct Ip4RangeParser;
impl Parser<Ip4Range> for Ip4RangeParser {
    fn parse( &self, source: &str ) -> Option<(Ip4Range, parse::CharsCount)> {
        let ip: Rc<dyn Parser<DecRange8u>> = Rc::new(DecRange8uParser);

        let kw = KeywordsBuilder::new(".", &Ip4Delim)
            .build().parser();

        let ws = WhiteSpaceParser::parser(WhiteSpaceParser);
        let delim1 = 
            map(ws.clone().follow(kw.clone().follow(ws.clone())),|_| ());
        let delim2 = 
            map(ws.clone().follow(kw.clone()),|_| ());
        let delim3 = 
            map(kw.clone(),|_| ());
        let delim = 
        map(
            alternative(
            alternative(delim1, delim2), delim3),
            |_| ()
        );

        let parser =
        ip.clone().follow(
            delim.clone().follow(
                ip.clone().follow(
                    delim.clone().follow(
                        ip.clone().follow(
                            delim.clone().follow(ip.clone())))))
        );

        let parser = 
        map(parser,
            |(b1,(_,(b2,(_,(b3,(_,b4))))))|
            (b1.clone(),b2.clone(),b3.clone(),b4.clone())
        );

        let parser =
        map(parser, |(b1,b2,b3,b4)| 
            Ip4Range(b1.clone(), b2.clone(), b3.clone(), b4.clone())
        );

        parser.parse(source)
    }
}

#[test]
fn ip4_range_test() {
    let res = Ip4RangeParser::parse(&Ip4RangeParser, 
        "127.0-4.2,3.8,9-10"
    ).map(|(v,_)|v);

    assert_eq!(res, Some(
        Ip4Range(
            DecRange8u(vec![
                DecNonBreakRange8u::One(Dec8u(127))
            ]),
            DecRange8u(vec![
                DecNonBreakRange8u::FromTo(DecFromTo8u(Dec8u(0), Dec8u(4)))
            ]),
            DecRange8u(vec![
                DecNonBreakRange8u::One(Dec8u(2)),
                DecNonBreakRange8u::One(Dec8u(3)),
            ]),
            DecRange8u(vec![
                DecNonBreakRange8u::One(Dec8u(8)),
                DecNonBreakRange8u::FromTo(DecFromTo8u(Dec8u(9), Dec8u(10)))
            ])
        )
    ))
}

/////////////////////////////////////

#[derive(Clone,Debug)]
struct Ip6Range(pub Vec<Ip6ByteAddr>);
struct Ip6RangeParser;

#[derive(Clone,Debug)]
enum Ip6Delim { One, Two }

#[derive(Clone,Debug)]
enum Ip6ByteAddr { One(HexRange16u), Two(HexRange16u) }

#[derive(Clone,Debug)]
enum Ip6Brace { Open, Close }

impl Parser<Ip6Range> for Ip6RangeParser {
    fn parse( &self, source: &str ) -> Option<(Ip6Range, parse::CharsCount)> {
        let kw_one = 
        KeywordsBuilder::new(":", &Ip6Delim::One)
        .build().parser();

        let kw_two = 
        KeywordsBuilder::new("::", &Ip6Delim::Two)
        .build().parser();

        let kw_open = 
        KeywordsBuilder::new("[", &Ip6Brace::Open)
        .build().parser();

        let kw_close = 
        KeywordsBuilder::new("]", &Ip6Brace::Close)
        .build().parser();

        let ws = WhiteSpaceParser::parser(WhiteSpaceParser);

        let ip6b: Rc<dyn Parser<HexRange16u>> = Rc::new(HexRange16uParser);
        let ip6b = 
        map(alternative(
            map(ws.clone().follow(ip6b.clone().follow(ws.clone())), 
            |(_,(a,_))| a.clone()
            ),
            map(
                alternative(
                map(ws.clone().follow(ip6b.clone()), |(_,a)| a.clone()), 
                ip6b.clone()                
                ), |v| match v {
                    Left(v) => v.clone(),
                    Right(v) => v.clone()
                }
            )
        ), |v| match v {
            Left(v) => v.clone(),
            Right(v) => v.clone()
        });

        let tail_parser = 
        repeat(map(
            kw_one.clone().follow(ip6b.clone()),
            |(_,a)| a.clone()
        ), None, None);

        let lst_parser = 
        map(ip6b.clone().follow(tail_parser.clone()),
        |(h,t)| {
            let mut lst = t.clone();
            lst.insert(0, h.clone());
            lst
        });

        let lst_parser_a = 
        map(kw_two.clone().follow(lst_parser.clone()), |(_,lst)| {
            let lst: Vec<Ip6ByteAddr> = lst.iter().enumerate()
            .map(|(i,a)| 
                match i {
                    0 => Ip6ByteAddr::Two(a.clone()),
                    _ => Ip6ByteAddr::One(a.clone())
                }
            )
            .collect();
            lst
        });

        let lst_parser_b =
        map(alternative(
            lst_parser.clone().follow(lst_parser_a.clone()),
            lst_parser.clone()
        ), |et| match et {
            Left((h,t)) => {
                let mut lst: Vec<Ip6ByteAddr> = 
                h.iter().map(|a| Ip6ByteAddr::One(a.clone())).collect();

                lst.extend(t.clone());

                lst
            },
            Right(v) => {
                let lst: Vec<Ip6ByteAddr> = 
                v.iter().map(|a| Ip6ByteAddr::One(a.clone())).collect();

                lst
            }
        });

        let lst_parser = 
        map(kw_open.clone().follow(
            map(alternative(lst_parser_a.clone(), lst_parser_b.clone()),
            |et| match et {
                Left(v) => v.clone(),
                Right(v) => v.clone()
            })
        ).follow(kw_close.clone()),
        |((_,a),_)| 
        Ip6Range(a.clone())
        );

        lst_parser.parse(source)
    }
}

impl TryFrom<Ip6Range> for IpRange {
    type Error = String;
    fn try_from(value: Ip6Range) -> Result<Self, Self::Error> {
        let values = value.0;
        if values.len() == 0 { return Err("no values".to_string()); }

        let split_count = values.iter().filter(|e| match e {
            Ip6ByteAddr::Two(_) => true,
            _ => false
        }).count();

        if split_count > 1usize { return Err("split count > 1".to_string()); }
        
        if split_count == 0usize {
            if values.len() != 8 { return Err("expect 8 groups of byte ranges".to_string()); }
            let hex_range = |a:Ip6ByteAddr| {
                match a {
                    Ip6ByteAddr::One(a) => Some(a),
                    _ => None
                }
            };

            let res = 
            hex_range(values[0].clone()).and_then(|r0|
                hex_range(values[1].clone()).and_then(|r1|
                    hex_range(values[2].clone()).and_then(|r2|
                        hex_range(values[3].clone()).and_then(|r3|
                            hex_range(values[4].clone()).and_then(|r4|
                                hex_range(values[5].clone()).and_then(|r5|
                                    hex_range(values[6].clone()).and_then(|r6|
                                        hex_range(values[7].clone()).map(|r7|
                                            (r0.clone(),r1.clone(),r2.clone(),r3.clone(),r4.clone(),r5.clone(),r6.clone(),r7.clone())
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            ).map(|(r0,r1,r2,r3,r4,r5,r6,r7)| 
                IpRange::Ip6( 
                    r0.into(),r1.into(),r2.into(),r3.into(),r4.into(),r5.into(),r6.into(),r7.into()
                )
            );

            return res.ok_or("bug".to_string());
        }

        let (left,right) = values.split_at(
            values.iter().enumerate().filter(
                |(_,a)| match a {
                    Ip6ByteAddr::Two(_) => true,
                    _ => false
                }
            ).map(|(i,_)| i).next().unwrap()
        );
        let (left,right) = (
            left.iter().map(|a| match a {
                Ip6ByteAddr::One(a) => a.clone(),
                Ip6ByteAddr::Two(a) => a.clone(),
            }),
            right.iter().map(|a| match a {
                Ip6ByteAddr::One(a) => a.clone(),
                Ip6ByteAddr::Two(a) => a.clone(),
            })
        );
        let left:  Vec<HexRange16u> = left.collect();
        let right: Vec<HexRange16u> = right.collect();

        let c_diff = 8usize - (left.len() + right.len());
        if c_diff == 0 {
            let mut lst : Vec<HexRange16u> = Vec::new();
            lst.extend(left.clone());
            lst.extend(right.clone());
            return Ok(IpRange::Ip6(
                lst[0].clone().into(), lst[1].clone().into(), lst[2].clone().into(), lst[3].clone().into(), 
                lst[4].clone().into(), lst[5].clone().into(), lst[6].clone().into(), lst[7].clone().into()
            ));
        }

        let left_append = true; //left.len() < right.len();

        let mut lst : Vec<HexRange16u> = Vec::new();
        lst.extend(left.clone());
        if left_append {
            for _i in 0..c_diff {
                lst.push(HexRange16u(vec![HexNonBreakRange16u::One(Hex16u(0))]));
            }
        }
        lst.extend(right.clone());
        if ! left_append {
            for _i in 0..c_diff {
                lst.push(HexRange16u(vec![HexNonBreakRange16u::One(Hex16u(0))]));
            }
        }

        return Ok(IpRange::Ip6(
            lst[0].clone().into(), lst[1].clone().into(), lst[2].clone().into(), lst[3].clone().into(), 
            lst[4].clone().into(), lst[5].clone().into(), lst[6].clone().into(), lst[7].clone().into()
        ));
    }
}

impl From<Ip4Range> for IpRange {
    fn from(value: Ip4Range) -> Self {
        IpRange::Ip4(
            value.0.into(), 
            value.1.into(), 
            value.2.into(), 
            value.3.into()
        )
    }
}

pub struct IpRangeParser;
impl Parser<IpRange> for IpRangeParser {
    fn parse( &self, source: &str ) -> Option<(IpRange, parse::CharsCount)> {
        let ip6parse : Rc<dyn Parser<Ip6Range>> = Rc::new( Ip6RangeParser );
        let ip6parse = and_then(ip6parse, |ip6r|{ 
            let ip_range = match IpRange::try_from(ip6r.clone()) {
                Ok(v) => Some(v),
                Err(_) => None
            };
            ip_range
        });

        let ip4parse : Rc<dyn Parser<Ip4Range>> = Rc::new( Ip4RangeParser );
        let ip4parse = map(ip4parse, |ip4r| IpRange::from(ip4r.clone()));

        let parse = map(
            alternative(ip6parse, ip4parse),
            |et| match et {
                Left(v) => v.clone(),
                Right(v) => v.clone()
            }
        );
        
        parse.parse(source)
    }
}

#[test]
fn ip_range_test() {
    let res = IpRangeParser::parse(&IpRangeParser, 
        "127.0-4.2,3.8,9-10"
    ).map(|(v,_)|v);

    assert_eq!(res, Some(
        IpRange::Ip4(
            Range::Multiple(vec![
                Range::Single(127u8)
            ]),
            Range::Multiple(vec![
                Range::FromToInc(0, 4)
            ]),
            Range::Multiple(vec![
                Range::Single(2),
                Range::Single(3),
            ]),
            Range::Multiple(vec![
                Range::Single(8),
                Range::FromToInc(9, 10)
            ])
        )
    ));

    let res = IpRangeParser::parse(&IpRangeParser, 
        "127 . 0-4 . 2,3 . 8,9-10"
    ).map(|(v,_)|v);

    assert_eq!(res, Some(
        IpRange::Ip4(
            Range::Multiple(vec![
                Range::Single(127u8)
            ]),
            Range::Multiple(vec![
                Range::FromToInc(0, 4)
            ]),
            Range::Multiple(vec![
                Range::Single(2),
                Range::Single(3),
            ]),
            Range::Multiple(vec![
                Range::Single(8),
                Range::FromToInc(9, 10)
            ])
        )
    ));

    let res = IpRangeParser::parse(&IpRangeParser, 
        "127  .  0 - 4  .  2,3  .  8,9 - 10"
    ).map(|(v,_)|v);

    assert_eq!(res, Some(
        IpRange::Ip4(
            Range::Multiple(vec![
                Range::Single(127u8)
            ]),
            Range::Multiple(vec![
                Range::FromToInc(0, 4)
            ]),
            Range::Multiple(vec![
                Range::Single(2),
                Range::Single(3),
            ]),
            Range::Multiple(vec![
                Range::Single(8),
                Range::FromToInc(9, 10)
            ])
        )
    ));
}
