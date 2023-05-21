//! Парсинг диапазона значений
//! 
//! Синтаксис
//! 
//! range ::= [ ws ] multiple
//! 
//! multiple ::= singleOrFromTo [ ws ] [ ',' [ws] singleOrFromTo ]
//! 
//! singleOrFromTo ::=  fromTo | single
//! 
//! single ::= [ws] number
//! 
//! fromTo ::= single [ws] '-' single
//! 
use std::rc::Rc;

use crate::substr::*;
use super::super::parse::*;

#[derive(Clone,Debug,PartialEq)]
pub struct Single(pub Number);

pub struct SingleParser {
    parser: Rc<dyn Parser<Single>>
}

impl SingleParser {
    pub fn new() -> Self {        
        let parser = follow( WhiteSpaceParser.parser(), NumberParser.parser() ) ;
        let parser = map(parser, |(_ws, num)| num.clone() );

        let parser = alternative(parser, NumberParser.parser());
        let parser = map( parser, |eth| 
            match eth.is_left() {
                true => eth.clone().left().unwrap(),
                false => eth.clone().right().unwrap()
            }
        );
        let parser = map(parser, |num| Single(num.clone()));

        SingleParser {
            parser: parser
        }
    }

    pub fn parser( self ) -> Rc<dyn Parser<Single>> {
        Rc::new( self )
    }
}

impl Parser<Single> for SingleParser {
    fn parse( &self, source: &str ) -> Option<(Single, CharsCount)> {
        self.parser.parse(source)
    }
}

#[test]
fn single_parse_test() {
    let parser = SingleParser::new();
    let res = parser.parse("  12+");
    println!("{:?}", res);
    assert!(res == Some((Single(Number { digits: vec![1u8,2], base: DigitBase::Dec }), CharsCount(4))));

    let res = parser.parse("12+");
    println!("{:?}", res);
    assert!(res == Some((Single(Number { digits: vec![1u8,2], base: DigitBase::Dec }), CharsCount(2))));
}

#[derive(Clone,Debug,PartialEq)]
pub struct FromTo(pub Number,pub Number);

// fromTo ::= single [ws] '-' single
#[derive(Clone)]
pub struct FromToParser {
    parser: Rc<dyn Parser<FromTo>>
}

impl FromToParser {
    pub fn new() -> Self {
        let single_parser = SingleParser::new().parser();
        let kw_parser = KeywordsBuilder::new("-", &()).build().parser();

        let parser = follow(single_parser.clone(), WhiteSpaceParser.parser());
        let parser = follow(parser, kw_parser.clone());
        let first_num = map(parser, |((c,_),_)| c.clone() );

        let parser = 
            map(follow(single_parser.clone(), kw_parser.clone()), |(c,_)| c.clone());

        let first_num = map(alternative(first_num, parser), 
            |eth| 
            match eth.is_left() {
                true => eth.clone().left().unwrap(),
                false => eth.clone().right().unwrap()
            }
        );

        let parser = follow(first_num, single_parser.clone());
        let parser = map(parser, |(left,right)| FromTo(left.0.clone(), right.0.clone()));

        FromToParser {
            parser: parser
        }
    }

    pub fn parser( self ) -> Rc<dyn Parser<FromTo>> {
        Rc::new(self)
    }
}

impl Parser<FromTo> for FromToParser {
    fn parse( &self, source: &str ) -> Option<(FromTo, CharsCount)> {
        self.parser.parse(source)
    }
}

#[test]
fn test_parse_from_to() {
    let parser = FromToParser::new();
    
    let res = parser.parse("1-2");
    println!("{:?}",res);

    let n1 = Number { digits:vec![1u8], base: DigitBase::Dec };
    let n2 = Number { digits:vec![2u8], base: DigitBase::Dec };

    assert!( res == Some((FromTo(n1.clone(), n2.clone()),CharsCount(3))) );

    let res = parser.parse(" 1 - 2 xx");
    println!("{:?}",res);
    assert!( res == Some((FromTo(n1.clone(), n2.clone()),CharsCount(6))) );
}

#[derive(Debug,Clone)]
pub enum RangeNum {
    One( Single ),
    Range( FromTo )
}

#[derive(Debug,Clone)]
pub struct Multiple( Vec::<RangeNum> );

pub struct MultipleParse {
    delim_parser: Rc<dyn Parser<()>>,
    range_num_parser: Rc<dyn Parser<RangeNum>>,
}

impl MultipleParse {
    pub fn new() -> Self {
        let kw_parser = KeywordsBuilder::new(",", &()).build().parser();

        let delim_parser = 
            map(follow(WhiteSpaceParser.parser(), kw_parser.clone()), |(_,t)| ());

        let delim_parser =
            map( alternative(delim_parser, kw_parser.clone()), |eth| 
            ());

        let single_parser = map( SingleParser::new().parser(), |v| RangeNum::One(v.clone()) );
        let from_to_parser = map( FromToParser::new().parser(), |v| RangeNum::Range(v.clone()) );
        let range_num_parser = map(alternative(from_to_parser,single_parser), |eth| 
            match eth.is_left() {
                true => eth.clone().left().unwrap(),
                false => eth.clone().right().unwrap()
            }
        );

        MultipleParse {  
            delim_parser: delim_parser,
            range_num_parser: range_num_parser
        }
    }

    pub fn parser( self ) -> Rc<dyn Parser<Multiple>> {
        Rc::new( self )
    }
}

impl Parser<Multiple> for MultipleParse {
    fn parse( &self, source: &str ) -> Option<(Multiple, CharsCount)> {
        let mut src = source.clone();
        let mut cc = CharsCount(0);
        let mut res = Vec::<RangeNum>::new();

        loop {
            match self.range_num_parser.parse(src) {
                Some( (range, c0) ) => {
                    res.push(range.clone());
                    cc = cc + c0;

                    match src.substring(c0) {
                        Some(next_src) => {
                            src = next_src;
                            
                            match self.delim_parser.parse(src) {
                                Some( (_,c1) ) => {
                                    match src.substring(c1) {
                                        Some(next_src) => {
                                            src = next_src;
                                            cc = cc + c1
                                        },
                                        None => break
                                    }
                                },
                                None => break
                            }
                        },
                        None => {
                            break
                        }
                    }
                },
                None => {
                    break
                }
            }
        };

        if res.is_empty() {
            return None;
        }

        Some((
            Multiple(res),
            cc
        ))
    }
}

#[test]
fn multiple_parse_test() {
    let parser = MultipleParse::new();
    let res = parser.parse("1,2,4-6");
    println!("{:?}", res);
}