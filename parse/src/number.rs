use std::rc::Rc;

use crate::substr::*;

use super::*;

/// Целое число см [NumberParser]
#[derive(Debug,Clone,PartialEq)]
pub struct Number { pub digits: Vec<u8>, pub base: DigitBase }

impl Number {
    pub fn try_u128( &self ) -> Option<u128> {
        let res : Result<u128,String> = self.clone().try_into();
        match res {
            Ok(res) => Some(res),
            Err(_) => None
        }
    }

    pub fn try_u64( &self ) -> Option<u64> {
        self.try_u128().and_then(|n| 
            if n > u64::MAX as u128 {
                None
            } else {
                Some(n as u64)
            }
        )
    }

    pub fn try_u32( &self ) -> Option<u32> {
        self.try_u128().and_then(|n| 
            if n > u32::MAX as u128 {
                None
            } else {
                Some(n as u32)
            }
        )
    }
}

impl TryFrom<Number> for u128 {
    type Error = String;
    fn try_from(value: Number) -> Result<Self, Self::Error> {
        let mut digits = value.digits.clone();
        digits.reverse();

        let base = match value.base {
            DigitBase::Bin => 2u128,
            DigitBase::Oct => 8u128,
            DigitBase::Dec => 10u128,
            DigitBase::Hex => 16u128,
        };

        let mut kof = 1u128;
        let mut sum = 0u128;

        for digit in digits {
            match (digit as u128).checked_mul(kof) {
                Some(succ_mul) => {
                    sum += succ_mul;
                    match kof.checked_mul(base) {
                        Some(succ_kof) => kof = succ_kof,
                        None => return Err(format!("overflow parse number: {kof} * {base}"))
                    }
                },
                None => return Err(format!("overflow parse number: {digit} * {kof}"))
            }
        }
        
        Ok(sum)
    }
}

impl TryFrom<Number> for u64 {
    type Error = String;
    fn try_from(value: Number) -> Result<Self, Self::Error> {
        let v:u128 = value.try_into()?;
        if v > u64::MAX as u128 { return Err(format!("can't convert {v} to u64 from u128: overflow u64::MAX")); }
        Ok(v as u64)
    }
}

impl TryFrom<Number> for u32 {
    type Error = String;
    fn try_from(value: Number) -> Result<Self, Self::Error> {
        let v:u128 = value.try_into()?;
        if v > u32::MAX as u128 { return Err(format!("can't convert {v} to u32 from u128: overflow u32::MAX")); }
        Ok(v as u32)
    }
}

impl TryFrom<Number> for u16 {
    type Error = String;
    fn try_from(value: Number) -> Result<Self, Self::Error> {
        let v:u128 = value.try_into()?;
        if v > u16::MAX as u128 { return Err(format!("can't convert {v} to u16 from u128: overflow u16::MAX")); }
        Ok(v as u16)
    }
}

impl TryFrom<Number> for u8 {
    type Error = String;
    fn try_from(value: Number) -> Result<Self, Self::Error> {
        let v:u128 = value.try_into()?;
        if v > u8::MAX as u128 { return Err(format!("can't convert {v} to u8 from u128: overflow u8::MAX")); }
        Ok(v as u8)
    }
}

/// Парсер числа
/// 
/// Синтаксис
/// 
///     Number ::= hex_number | oct_number | bin_number | dec_number
///     hex_number ::= '0x' hex_digit { hex_digit }
///     hex_digit  ::= '0' | '1' | '2' | '3' | '4'
///                  | '5' | '6' | '7' | '8' | '9'
///                  | 'a' | 'b' | 'c' | 'd' | 'e' | 'f'
///                  | 'A' | 'B' | 'C' | 'D' | 'E' | 'F'
/// 
///     oct_number ::= '0o' oct_digit { oct_digit }
///     oct_digit  ::= '0' | '1' | '2' | '3' | '4'
///                  | '5' | '6' | '7'
/// 
///     bin_number ::= '0b' bin_digit { bin_digit }
///     bin_digit  ::= '0' | '1'
/// 
///     dec_number ::= dec_digit dec_digit
///     dec_digit  ::= '0' | '1' | '2' | '3' | '4'
///                  | '5' | '6' | '7' | '8' | '9'
/// 
pub struct NumberParser;

impl NumberParser {
    pub fn parser( self ) -> Rc<dyn Parser<Number>> {
        Rc::new( self )
    }
}

const BIN_DIGIT_PARSER : DigitParser = DigitParser { base: DigitBase::Bin };
const OCT_DIGIT_PARSER : DigitParser = DigitParser { base: DigitBase::Oct };
const DEC_DIGIT_PARSER : DigitParser = DigitParser { base: DigitBase::Dec };
const HEX_DIGIT_PARSER : DigitParser = DigitParser { base: DigitBase::Hex };

impl Parser<Number> for NumberParser {
    fn parse( &self, source: &str ) -> Option<(Number, CharsCount)> {
        let mut src = source.clone();
        let (num_parser,prefix_cc) = LookupParser { max_chars_count: 2 }.parse(source)
            .when_equals("0x", HEX_DIGIT_PARSER)
            .when_equals("0o", OCT_DIGIT_PARSER)
            .when_equals("0b", BIN_DIGIT_PARSER)
            .fetch()
            .unwrap_or((DEC_DIGIT_PARSER, CharsCount(0)))
            ;

        if prefix_cc.0 > 0 {
            match src.substring(prefix_cc) {
                Some(substr) => {
                    src = substr;
                },
                None => {}
            }
        }

        let mut digits: Vec<u8> = vec![];
        let mut chr_count = prefix_cc;

        loop {
            match num_parser.parse(src) {
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
            Some(( Number { base: num_parser.base, digits: digits }, chr_count ))
        } else {
            None
        }
    }
}

#[test]
fn test_number() {
    let parser = NumberParser {};
    let res = parser.parse("123 as");
    println!("{res:?}");

    let (num,_) = res.unwrap();
    println!("{:?}", num.try_u128());
    assert!( num.try_u32() == Some(123u32) );

    let res = parser.parse("0xFe");
    let (num,_) = res.unwrap();
    println!("{:?}", num.try_u128());
    assert!( num.try_u32() == Some(254u32) );
}

pub struct BaseNumberParser( pub DigitBase );
impl Parser<Number> for BaseNumberParser {
    fn parse( &self, source: &str ) -> Option<(Number, CharsCount)> {
        let digit_parser = DigitParser { base: self.0 };

        let mut digits: Vec<u8> = vec![];
        let mut chr_count = CharsCount(0);
        let mut src = source;

        loop {
            match digit_parser.parse(src) {
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
            Some(( Number { base: digit_parser.base, digits: digits }, chr_count ))
        } else {
            None
        }
    }
}

pub struct HexNumberParser;

impl Parser<Number> for HexNumberParser {
    fn parse( &self, source: &str ) -> Option<(Number, CharsCount)> {
        BaseNumberParser(DigitBase::Hex).parse(source)
    }
}

pub struct BinNumberParser;

impl Parser<Number> for BinNumberParser {
    fn parse( &self, source: &str ) -> Option<(Number, CharsCount)> {
        BaseNumberParser(DigitBase::Bin).parse(source)
    }
}

pub struct OctNumberParser;

impl Parser<Number> for OctNumberParser {
    fn parse( &self, source: &str ) -> Option<(Number, CharsCount)> {
        BaseNumberParser(DigitBase::Oct).parse(source)
    }
}

pub struct DecNumberParser;

impl Parser<Number> for DecNumberParser {
    fn parse( &self, source: &str ) -> Option<(Number, CharsCount)> {
        BaseNumberParser(DigitBase::Dec).parse(source)
    }
}