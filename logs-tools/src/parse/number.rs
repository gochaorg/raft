use crate::substr::*;

use super::{DigitBase, DigitParser, Parser};

#[derive(Debug,Clone)]
pub struct Number { pub digits: Vec<u8>, pub base: DigitBase }

pub struct NumberParser { 
//    base: DigitBase, 
//    prefix:Option<String>
}

const DEC_DIGIT_PARSER : DigitParser = DigitParser { base: DigitBase::Dec };

impl Parser<Number> for NumberParser {
    fn parse( &self, source: &str ) -> Option<(Number, CharsCount)> {
        let mut src = source.clone();
        let mut digits: Vec<u8> = vec![];
        let mut chr_count = CharsCount(0);

        loop {
            match DEC_DIGIT_PARSER.parse(src) {
                Some( (d,cc) ) => {
                    digits.push(d.0);
                    match src.substring(cc) {
                        Some(substr) => {
                            src = substr;
                            chr_count = chr_count + cc;
                        },
                        None => break
                    };
                },
                None => break
            }
        };
        
        if chr_count.0 > 0 {
            Some(( Number { base: DigitBase::Dec, digits: digits }, chr_count ))
        } else {
            None
        }
    }
}

#[test]
fn test_number() {
    let parser = NumberParser {};
    let res = parser.parse("123 as");
    println!("{res:?}")
}
