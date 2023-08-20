use std::{rc::Rc};
use crate::{CharsCount, Parser, SubString};

/// Последовательность из двух парсеров
pub struct FollowParser<'a,R1,R2> 
where 
    R1: Sized+Clone,
    R2: Sized+Clone,
{
    first: Rc<dyn Parser<R1> + 'a>,
    second: Rc<dyn Parser<R2> + 'a>,
}

impl<'a,R1,R2> FollowParser<'a,R1,R2>
where
    R1: Sized + Clone,
    R2: Sized + Clone,
{
    pub fn new( p1:Rc<dyn Parser<R1> + 'a>, p2:Rc<dyn Parser<R2> + 'a> ) -> Self {
        Self { first: p1, second: p2 }
    }

    pub fn parser( &'a self ) -> Rc<dyn Parser<(R1,R2)> + 'a> {
        let r = Self { first: self.first.clone(), second: self.second.clone() };
        Rc::new( r )
    }
}

impl<'a,R1,R2> Parser<(R1,R2)> for FollowParser<'a,R1,R2> 
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
    let parser = FollowParser::new(parser1, parser2);

    let res = parser.parse(str);
    assert!( res == Some(((Digit(1),Digit(2)),CharsCount(2))) )
}

/// Создание последовательности из двух парсеров
pub fn follow<'a,A:Sized+Clone+'a,B:Sized+Clone+'a>( left:Rc<dyn Parser<A> + 'a>, right:Rc<dyn Parser<B> + 'a> ) -> Rc<dyn Parser<(A,B)> + 'a> {
    let parser = Rc::new(FollowParser { first: left.clone(), second: right.clone() });
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

pub trait ParseAdd<P> {
    type Out;
    fn follow( self, other: P ) -> Self::Out;
}

impl<'a,A,B> ParseAdd<&Rc<dyn Parser<B> + 'a>> for Rc<dyn Parser<A> + 'a> 
where
    A:Sized+Clone + 'a,
    B:Sized+Clone + 'a,
{
    type Out = Rc<dyn Parser<(A,B)> + 'a>;
    fn follow( self, other: &Rc<dyn Parser<B> + 'a> ) -> Self::Out {
        follow(self.clone(), other.clone())
    }
}

impl<'a,A,B> ParseAdd<Rc<dyn Parser<B> + 'a>> for Rc<dyn Parser<A> + 'a> 
where
    A:Sized+Clone + 'a,
    B:Sized+Clone + 'a,
{
    type Out = Rc<dyn Parser<(A,B)> + 'a>;
    fn follow( self, other: Rc<dyn Parser<B> + 'a> ) -> Self::Out {
        follow(self.clone(), other.clone())
    }
}

#[test]
fn test_follow_3() {
    use super::*;

    let str = "123";
    
    let parser1 = DigitParser::parser();
    let parser2 = DigitParser::parser();
    let parser = parser1.follow(&parser2);

    let res = parser.parse(str);
    assert!( res == Some(((Digit(1),Digit(2)),CharsCount(2))) )
}
