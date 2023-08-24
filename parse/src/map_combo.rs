use std::rc::Rc;
use crate::{CharsCount, Parser};

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
    use crate::{CharsCount, FollowParser};

    let str = "123";
    
    let parser1 = DigitParser::parser();
    let parser2 = DigitParser::parser();
    let parser = FollowParser::new(parser1, parser2);
    let parser = parser.parser();
    let parser = map(parser, |(a,b)| a.0 + b.0 );


    let res = parser.parse(str);
    assert!( res == Some((3u8,CharsCount(2))) )
}
