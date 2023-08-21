use std::rc::Rc;
use either::*;
use crate::{CharsCount, Parser};

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
