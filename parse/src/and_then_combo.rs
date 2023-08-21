use std::rc::Rc;
use crate::{CharsCount, Parser};

#[derive(Clone)]
pub struct ResultAndThenParser<'a,A,B,F> 
where
    F: (Fn(&A) -> Option<B>) + 'a,
    B: Clone+Sized,
{
    source: Rc<dyn Parser<A> + 'a>,
    mapper: F
}

impl<'a,A,B,F> Parser<B> for ResultAndThenParser<'a,A,B,F>
where
    F: (Fn(&A) -> Option<B>) + 'a,
    B: Clone+Sized,
{
    fn parse( &self, source: &str ) -> Option<(B, CharsCount)> {
        match self.source.parse(source) {
            Some( (src,cc) ) => {
                let dest = (self.mapper)(&src);
                dest.map(|v| (v,cc))
            },
            None => None
        }
    }    
}

pub fn and_then<'a,A,F,B>( source: Rc<dyn Parser<A> + 'a>, f:F ) -> Rc<dyn Parser<B> + 'a> 
where
    F: Fn(&A) -> Option<B> + 'a,
    B: Clone+Sized + 'a,
    A: 'a
{
    Rc::new( ResultAndThenParser {
        source: source.clone(),
        mapper: f
    })
}