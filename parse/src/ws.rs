use std::rc::Rc;

use super::Parser;
use crate::*;

#[derive(Debug,Clone)]
pub struct WhiteSpace(pub String);

/// Парсинг пробельных символов
#[derive(Debug,Clone)]

pub struct WhiteSpaceParser;

impl WhiteSpaceParser {
    pub fn parser( self ) -> Rc<dyn Parser<WhiteSpace>> {
        Rc::new( self )
    }
}

impl Parser<WhiteSpace> for WhiteSpaceParser {
    fn parse( &self, source: &str ) -> Option<(WhiteSpace, CharsCount)> {
        let mut c_itr = source.chars();
        let mut str = String::new();
        let mut cnt = 0usize;
        loop {
            match c_itr.next() {
                Some(chr) if chr.is_whitespace() => {
                    str.push(chr);
                    cnt += 1;
                },
                _ => break
            }
        };
        if str.is_empty() {
            None
        } else {
            Some( (WhiteSpace(str), CharsCount(cnt)) )
        }
    }
}

#[test]
fn test_ws(){
    let res: (WhiteSpace,CharsCount) = WhiteSpaceParser.parse("  123").unwrap();
    assert!(res.1 == CharsCount(2))
}