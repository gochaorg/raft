use std::rc::Rc;

use crate::substr::CharsCount;

use super::Parser;

/// Цифра
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Digit( pub u8 );

/// Система счисления
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum DigitBase {
    Bin,
    Oct,
    Dec,
    Hex
}

/// Парсинг числа
#[derive(Debug,Clone,Copy)]
pub struct DigitParser {
    pub base : DigitBase
}

impl Parser<Digit> for DigitParser {
    fn parse( &self, source: &str ) -> Option<(Digit, CharsCount)> {
        if source.len()==0 { 
            None
        } else {
            match source.chars().next() {
                Some('0') if self.base == DigitBase::Bin || self.base == DigitBase::Oct || self.base == DigitBase::Dec || self.base == DigitBase::Hex => Some((Digit(0u8),CharsCount(1))),
                Some('1') if self.base == DigitBase::Bin || self.base == DigitBase::Oct || self.base == DigitBase::Dec || self.base == DigitBase::Hex=> Some((Digit(1u8),CharsCount(1))),
                Some('2') if self.base == DigitBase::Oct || self.base == DigitBase::Dec || self.base == DigitBase::Hex => Some((Digit(2u8),CharsCount(1))),
                Some('3') if self.base == DigitBase::Oct || self.base == DigitBase::Dec || self.base == DigitBase::Hex => Some((Digit(3u8),CharsCount(1))),
                Some('4') if self.base == DigitBase::Oct || self.base == DigitBase::Dec || self.base == DigitBase::Hex => Some((Digit(4u8),CharsCount(1))),
                Some('5') if self.base == DigitBase::Oct || self.base == DigitBase::Dec || self.base == DigitBase::Hex => Some((Digit(5u8),CharsCount(1))),
                Some('6') if self.base == DigitBase::Oct || self.base == DigitBase::Dec || self.base == DigitBase::Hex => Some((Digit(6u8),CharsCount(1))),
                Some('7') if self.base == DigitBase::Oct || self.base == DigitBase::Dec || self.base == DigitBase::Hex => Some((Digit(7u8),CharsCount(1))),
                Some('8') if self.base == DigitBase::Dec || self.base == DigitBase::Hex => Some((Digit(8u8),CharsCount(1))),
                Some('9') if self.base == DigitBase::Dec || self.base == DigitBase::Hex => Some((Digit(9u8),CharsCount(1))),
                Some('A') | Some('a') if self.base == DigitBase::Hex => Some((Digit(10u8),CharsCount(1))),
                Some('B') | Some('b') if self.base == DigitBase::Hex => Some((Digit(11u8),CharsCount(1))),
                Some('C') | Some('c') if self.base == DigitBase::Hex => Some((Digit(12u8),CharsCount(1))),
                Some('D') | Some('d') if self.base == DigitBase::Hex => Some((Digit(13u8),CharsCount(1))),
                Some('E') | Some('e') if self.base == DigitBase::Hex => Some((Digit(14u8),CharsCount(1))),
                Some('F') | Some('f') if self.base == DigitBase::Hex => Some((Digit(15u8),CharsCount(1))),
                _ => None
            }
        }
    }
}
impl DigitParser {
    pub fn parser() -> Rc<dyn Parser<Digit>> {
        Rc::new( DigitParser { base: DigitBase::Dec } )
    }    
}

#[test]
fn test_parse() {
    let str = "123";
    let parser = DigitParser::parser();
    let res = parser.parse(str);
    assert!( res == Some((Digit(1),CharsCount(1))) )
}
