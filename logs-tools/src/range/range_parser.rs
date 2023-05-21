//! Парсинг диапазона значений
//! 
//! Синтаксис
//! 
//! range ::= [ ws ] multiple
//! 
//! multiple ::= singleOrFromTo [ ws ] [ ',' [ws] singleOrFromTo ]
//! 
//! singleOrFromTo ::=  fromTo | single
//! 
//! single ::= number
//! 
//! fromTo ::= number [ws] '-' [ws] number
//! 
use super::range::*;
use crate::substr::*;
use super::super::parse::*;

struct Single(Number);
struct FromTo(Number,Number);

// struct SingleParser;
// impl Parser<FromTo> for SingleParser {
//     fn parse( &self, source: &str ) -> Option<(FromTo, CharsCount)> {
//         let mut src = source.clone();
//         //NumberParser.parse(source)
//     }
// }