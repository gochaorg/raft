use std::rc::Rc;
use either::*;
use crate::substr::*;

/// Общий интерфейс парсера
pub trait Parser<A:Sized> {
    fn parse( &self, source: &str ) -> Option<(A, CharsCount)>;
}

pub struct FollowParser<'a,R1,R2> 
where 
    R1: Sized+Clone,
    R2: Sized+Clone,
{
    first: Rc<dyn Parser<R1> + 'a>,
    second: Rc<dyn Parser<R2> + 'a>,
}

#[allow(dead_code)]
impl<'a,R1,R2> FollowParser<'a,R1,R2>
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

#[allow(dead_code)]
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

pub struct AlternativeParser<'a,A,B>
where
    A: Sized+Clone,
    B: Sized+Clone
{
    first:  Rc<dyn Parser<A> + 'a>,
    second: Rc<dyn Parser<B> + 'a>,
}

impl<'a,A,B> Parser<Either<A,B>> for AlternativeParser<'a,A,B> 
where
    A: Sized+Clone,
    B: Sized+Clone
{
    fn parse( &self, source: &str ) -> Option<(Either<A,B>, CharsCount)> {
        match self.first.parse(source) {
            Some((res,cc)) => Some( (Left(res), cc) ),
            None => {
                match self.second.parse(source) {
                    Some((res,cc)) => Some( (Right(res),cc) ),
                    None => None
                }
            }
        }
    }
}

pub fn alternative<'a,A:Sized+Clone+'a,B:Sized+Clone+'a>( left:Rc<dyn Parser<A> + 'a>, right:Rc<dyn Parser<B> + 'a> ) -> Rc<dyn Parser<Either<A,B>> + 'a> {
    Rc::new( AlternativeParser { first: left.clone(), second: right.clone() } )
}


#[derive(Clone)]
pub struct ResultMapperParser<'a,A,B,F> 
where
    F: (Fn(&A) -> B) + 'a,
    B: Clone+Sized,
{
    source: Rc<dyn Parser<A> + 'a>,
    mapper: F
}

impl<'a,A,B,F> Parser<B> for ResultMapperParser<'a,A,B,F>
where
    F: (Fn(&A) -> B) + 'a,
    B: Clone+Sized,
{
    fn parse( &self, source: &str ) -> Option<(B, CharsCount)> {
        match self.source.parse(source) {
            Some( (src,cc) ) => {
                let dest = (self.mapper)(&src);
                Some((dest,cc))
            },
            None => None
        }
    }    
}

pub fn map<'a,A,F,B>( source: Rc<dyn Parser<A> + 'a>, f:F ) -> Rc<dyn Parser<B> + 'a> 
where
    F: Fn(&A) -> B + 'a,
    B: Clone+Sized + 'a,
    A: 'a
{
    Rc::new( ResultMapperParser {
        source: source.clone(),
        mapper: f
    })
}

// pub struct Repeat<'a,A>
// where
//     A: Sized+Clone
// {
//     parser: Rc<dyn Parser<A> + 'a>,
//     min_match_count: Option<usize>,
//     max_match_count: Option<usize>
// }

#[test]
fn test_map_1() {
    use super::digit::*;

    let str = "123";
    
    let parser1 = DigitParser::parser();
    let parser2 = DigitParser::parser();
    let parser = FollowParser::new(parser1, parser2);
    let parser = parser.parser();
    let parser = map(parser, |(a,b)| a.0 + b.0 );


    let res = parser.parse(str);
    assert!( res == Some((3u8,CharsCount(2))) )
}
