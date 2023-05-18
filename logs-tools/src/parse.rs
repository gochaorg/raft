use std::{fmt::Debug, rc::Rc};

use crate::substr::*;

pub trait Parser<A:Sized> {
    fn parse( &self, source: &str ) -> Option<(A, CharsCount)>;
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Digit( u8 );



#[derive(Debug,Clone,Copy)]
pub struct DigitParser {

}

impl Parser<Digit> for DigitParser {
    fn parse( &self, source: &str ) -> Option<(Digit, CharsCount)> {
        if source.len()==0 { 
            None
        } else {
            match source.chars().next() {
                Some('0') => Some((Digit(0u8),CharsCount(1))),
                Some('1') => Some((Digit(1u8),CharsCount(1))),
                Some('2') => Some((Digit(2u8),CharsCount(1))),
                Some('3') => Some((Digit(3u8),CharsCount(1))),
                Some('4') => Some((Digit(4u8),CharsCount(1))),
                Some('5') => Some((Digit(5u8),CharsCount(1))),
                Some('6') => Some((Digit(6u8),CharsCount(1))),
                Some('7') => Some((Digit(7u8),CharsCount(1))),
                Some('8') => Some((Digit(8u8),CharsCount(1))),
                Some('9') => Some((Digit(9u8),CharsCount(1))),
                Some('A') | Some('a') => Some((Digit(10u8),CharsCount(1))),
                Some('B') | Some('b') => Some((Digit(11u8),CharsCount(1))),
                Some('C') | Some('c') => Some((Digit(12u8),CharsCount(1))),
                Some('D') | Some('d') => Some((Digit(13u8),CharsCount(1))),
                Some('E') | Some('e') => Some((Digit(14u8),CharsCount(1))),
                Some('F') | Some('f') => Some((Digit(15u8),CharsCount(1))),
                _ => None
            }
        }
    }
}
impl DigitParser {
    fn parser() -> Rc<dyn Parser<Digit>> {
        Rc::new( DigitParser {} )
    }    
}

#[test]
fn test_parse() {
    let str = "123";
    let parser = DigitParser::parser();
    let res = parser.parse(str);
    assert!( res == Some((Digit(1),CharsCount(1))) )
}

struct ParseFollow<'a,R1,R2> 
where 
    R1: Sized,
    R2: Sized,
{
    first: Rc<dyn Parser<R1> + 'a>,
    second: Rc<dyn Parser<R2> + 'a>,
}

impl<'a,R1,R2> ParseFollow<'a,R1,R2>
where
    R1: Sized + Clone,
    R2: Sized + Clone,
{
    fn new( p1:Rc<dyn Parser<R1> + 'a>, p2:Rc<dyn Parser<R2> + 'a> ) -> Self {
        Self { first: p1, second: p2 }
    }

    fn parser( &'a self ) -> Rc<dyn Parser<(R1,R2)> + 'a> {
        let r = Self { first: self.first.clone(), second: self.second.clone() };
        Rc::new( r )
    }
}

impl<'a,R1,R2> Parser<(R1,R2)> for ParseFollow<'a,R1,R2> 
where
    R1: Sized + Clone,
    R2: Sized + Clone,
{
    fn parse( &self, source: &str ) -> Option<((R1,R2), CharsCount)> {
        match self.first.parse(source) {
            Some( (t1,cc1) ) => {
                match source.substring(cc1) {
                    Some ( next_source ) => {
                        match self.second.parse(next_source) {
                            Some( (t2,cc2) ) => {
                                Some( ((t1,t2), cc1 + cc2) )
                            },
                            None => None
                        }
                    },
                    None => None
                }
            },
            None => None
        }
    }
}


#[test]
fn test_follow_1() {
    let str = "123";
    
    let parser1 = DigitParser::parser();
    let parser2 = DigitParser::parser();
    let parser = ParseFollow::new(parser1, parser2);

    let res = parser.parse(str);
    assert!( res == Some(((Digit(1),Digit(2)),CharsCount(2))) )
}

fn follow<'a,A:Sized+Clone+'a,B:Sized+Clone+'a>( left:Rc<dyn Parser<A> + 'a>, right:Rc<dyn Parser<B> + 'a> ) -> Rc<dyn Parser<(A,B)> + 'a> {
    let parser = Rc::new(ParseFollow { first: left.clone(), second: right.clone() });
    parser
}

#[test]
fn test_follow_2() {
    let str = "123";
    
    let parser1 = DigitParser::parser();
    let parser2 = DigitParser::parser();
    let parser = follow(parser1, parser2);

    let res = parser.parse(str);
    assert!( res == Some(((Digit(1),Digit(2)),CharsCount(2))) )
}
