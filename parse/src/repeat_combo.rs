use std::rc::Rc;
use crate::substr::*;
use crate::{CharsCount, Parser};

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