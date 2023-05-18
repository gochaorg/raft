use std::{fmt::Debug, rc::Rc};

use crate::substr::*;

pub trait Parser<A:Sized> {
    fn parse( &self, source: &str ) -> Option<(A, CharsCount)>;
}

/// Цифра
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Digit( u8 );

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
    fn parser() -> Rc<dyn Parser<Digit>> {
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

pub struct ParseFollow<'a,R1,R2> 
where 
    R1: Sized,
    R2: Sized,
{
    first: Rc<dyn Parser<R1> + 'a>,
    second: Rc<dyn Parser<R2> + 'a>,
}

impl<'a,R1,R2> ParseFollow<'a,R1,R2>
where
    R1: Sized + Clone,
    R2: Sized + Clone,
{
    fn new( p1:Rc<dyn Parser<R1> + 'a>, p2:Rc<dyn Parser<R2> + 'a> ) -> Self {
        Self { first: p1, second: p2 }
    }

    fn parser( &'a self ) -> Rc<dyn Parser<(R1,R2)> + 'a> {
        let r = Self { first: self.first.clone(), second: self.second.clone() };
        Rc::new( r )
    }
}

impl<'a,R1,R2> Parser<(R1,R2)> for ParseFollow<'a,R1,R2> 
where
    R1: Sized + Clone,
    R2: Sized + Clone,
{
    fn parse( &self, source: &str ) -> Option<((R1,R2), CharsCount)> {
        match self.first.parse(source) {
            Some( (t1,cc1) ) => {
                match source.substring(cc1) {
                    Some ( next_source ) => {
                        match self.second.parse(next_source) {
                            Some( (t2,cc2) ) => {
                                Some( ((t1,t2), cc1 + cc2) )
                            },
                            None => None
                        }
                    },
                    None => None
                }
            },
            None => None
        }
    }
}


#[test]
fn test_follow_1() {
    let str = "123";
    
    let parser1 = DigitParser::parser();
    let parser2 = DigitParser::parser();
    let parser = ParseFollow::new(parser1, parser2);

    let res = parser.parse(str);
    assert!( res == Some(((Digit(1),Digit(2)),CharsCount(2))) )
}

fn follow<'a,A:Sized+Clone+'a,B:Sized+Clone+'a>( left:Rc<dyn Parser<A> + 'a>, right:Rc<dyn Parser<B> + 'a> ) -> Rc<dyn Parser<(A,B)> + 'a> {
    let parser = Rc::new(ParseFollow { first: left.clone(), second: right.clone() });
    parser
}

#[test]
fn test_follow_2() {
    let str = "123";
    
    let parser1 = DigitParser::parser();
    let parser2 = DigitParser::parser();
    let parser = follow(parser1, parser2);

    let res = parser.parse(str);
    assert!( res == Some(((Digit(1),Digit(2)),CharsCount(2))) )
}

#[derive(Debug,Clone)]
pub struct Number { digits: Vec<u8>, base: DigitBase }

pub struct NumberParser { 
//    base: DigitBase, 
//    prefix:Option<String>
}

const DEC_DIGIT_PARSER : DigitParser = DigitParser { base: DigitBase::Dec };

impl Parser<Number> for NumberParser {
    fn parse( &self, source: &str ) -> Option<(Number, CharsCount)> {
        let mut src = source.clone();
        let mut digits: Vec<u8> = vec![];
        let mut chrCount = CharsCount(0);

        loop {
            match DEC_DIGIT_PARSER.parse(src) {
                Some( (d,cc) ) => {
                    digits.push(d.0);
                    match src.substring(cc) {
                        Some(substr) => {
                            src = substr;
                            chrCount = chrCount + cc;
                        },
                        None => break
                    };
                },
                None => break
            }
        };
        
        if chrCount.0 > 0 {
            Some(( Number { base: DigitBase::Dec, digits: digits }, chrCount ))
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

/// Просмотр символов
#[derive(Debug,Clone)]
pub struct Lookup( pub String );

pub struct LookupParser {
    pub max_chars_count : usize
}

impl Parser<Lookup> for LookupParser {
    fn parse( &self, source: &str ) -> Option<(Lookup, CharsCount)> {
        if self.max_chars_count == 0 { return None };
        let mut str = String::new();
        for (idx,chr) in source.char_indices() {
            str.push(chr);
            if (idx+1) >= self.max_chars_count { break; }
        }
        let cnt = str.len();
        Some( (Lookup(str), CharsCount(cnt)) )
    }
}

pub trait LookupMatch {
    type Output<A:Sized+Clone>;
    fn when_equals<A:Sized+Clone>( &self, str: &str, res:A ) -> Self::Output<A>;
}

impl LookupMatch for Option<(Lookup,CharsCount)> {
    type Output<B:Sized+Clone> = LookupContext<B>;
    fn when_equals<A:Sized+Clone>( &self, str: &str, res:A ) -> Self::Output<A> {
        let mut data:Vec<(String,A)> = vec![];
        data.push( (str.to_string(), res.clone()) );
        LookupContext {
            lookup: self.clone(),
            data: data
        }
    }
}

pub struct LookupContext<R:Sized+Clone> {
    pub lookup: Option<(Lookup,CharsCount)>,
    pub data: Vec<(String,R)>
}

impl<R:Sized+Clone> LookupContext<R> {
    fn when_equals( &mut self, str: &str, res:R ) -> &mut Self {
        self.data.push((str.to_string(), res.clone()));
        self
    }

    fn fetch( &self ) -> Option<(R,CharsCount)> {
        match &self.lookup {
            Some((Lookup(str), _)) => {
                match (&self.data).into_iter().filter(|(sample,_)| {
                    str.starts_with(sample)
                }).next().map(|c|c.clone()) {
                    Some( (str,res) ) => {
                        Some( (res,CharsCount(str.len())) )
                    },
                    None => None
                }
            },            
            None => None
        }
    }
}

#[test]
fn test_lookup() {
    let r = LookupParser { max_chars_count: 10 }.parse("source");
    println!("{r:?}");

    let r = 
        r.when_equals("src", 1)
        .when_equals("sour", 2)
        .when_equals("src", 3)
        .fetch();

    println!("{r:?}");
}

