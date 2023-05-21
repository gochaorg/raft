use std::{fmt::Debug, rc::Rc};

use crate::substr::*;

pub trait Parser<A:Sized> {
    fn parse( &self, source: &str ) -> Option<(A, CharsCount)>;
}
pub struct ParseFollow<'a,R1,R2> 
where 
    R1: Sized,
    R2: Sized,
{
    first: Rc<dyn Parser<R1> + 'a>,
    second: Rc<dyn Parser<R2> + 'a>,
}

#[allow(dead_code)]
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
    use super::digit::*;

    let str = "123";
    
    let parser1 = DigitParser::parser();
    let parser2 = DigitParser::parser();
    let parser = ParseFollow::new(parser1, parser2);

    let res = parser.parse(str);
    assert!( res == Some(((Digit(1),Digit(2)),CharsCount(2))) )
}

#[allow(dead_code)]
pub fn follow<'a,A:Sized+Clone+'a,B:Sized+Clone+'a>( left:Rc<dyn Parser<A> + 'a>, right:Rc<dyn Parser<B> + 'a> ) -> Rc<dyn Parser<(A,B)> + 'a> {
    let parser = Rc::new(ParseFollow { first: left.clone(), second: right.clone() });
    parser
}

#[test]
fn test_follow_2() {
    use super::*;

    let str = "123";
    
    let parser1 = DigitParser::parser();
    let parser2 = DigitParser::parser();
    let parser = follow(parser1, parser2);

    let res = parser.parse(str);
    assert!( res == Some(((Digit(1),Digit(2)),CharsCount(2))) )
}

