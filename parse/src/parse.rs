use std::rc::Rc;
use either::*;
use crate::{substr::*, FollowParser};

/// Общий интерфейс парсера
pub trait Parser<A:Sized> {
    /// Парсинг строки
    /// 
    /// # Аргументы
    /// - source - исходная строка
    /// 
    /// # Возвращает
    /// Распознаный объект и кол-во символов которых он занимамет от начала строки
    /// 
    fn parse( &self, source: &str ) -> Option<(A, CharsCount)>;
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

pub struct Repeat<'a,A>
where
    A: Sized+Clone
{
    parser: Rc<dyn Parser<A> + 'a>,
    min_match_count: Option<usize>,
    max_match_count: Option<usize>
}

impl<'a,A> Parser<Vec<A>> for Repeat<'a,A> 
where
    A:Sized+Clone
{
    fn parse( &self, source: &str ) -> Option<(Vec<A>, CharsCount)> {
        let mut res : Vec<A> = Vec::new();
        let mut src = source;
        let mut ccount = CharsCount(0);

        loop {
            if self.max_match_count.map(|c| res.len() >= c).unwrap_or(false) { break }

            match self.parser.parse(src) {
                None => break,
                Some((v,cc)) => {
                    res.push(v);
                    ccount = ccount + cc.clone();
                    match src.substring(cc) {
                        None => break,
                        Some(sub_str) => {
                            src = sub_str
                        }
                    };
                }
            }
        }

        if self.min_match_count.map(|c| res.len() < c).unwrap_or(false) { return None }

        Some( (res,ccount) )
    }
}

pub fn repeat<'a,A>( element_parser:Rc<dyn Parser<A> + 'a>, min_count:Option<usize>, max_count:Option<usize> ) -> Rc<dyn Parser<Vec<A>> + 'a> 
where 
    A: Sized+Clone,
    A: 'a
{
    let rep = Repeat {
        parser: element_parser.clone(),
        min_match_count: min_count,
        max_match_count: max_count
    };

    let rep : Rc<dyn Parser<Vec<A>> + 'a> = Rc::new(rep);
    rep
}

#[test]
fn repeat_test() {
    use super::digit::*;

    let parser1 = DigitParser::parser();
    let parser2 = repeat(parser1, None, None);

    let res = parser2.parse("123");
    println!("{res:?}");
    assert_eq!( Some(
        ( vec![
            Digit(1), Digit(2), Digit(3),
        ]
        , CharsCount(3)
        )
    ), res );
}