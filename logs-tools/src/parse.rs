use crate::substr::*;

trait Parser<A:Sized> {
    fn parse( &self, source: &str ) -> Option<(A, CharsCount)>;
}

///////////////////////////

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
struct Digit( u8 );

#[derive(Debug,Clone,Copy)]
struct DigitParser {}

impl Parser<Digit> for DigitParser {
    fn parse( &self, source: &str ) -> Option<(Digit, CharsCount)> {
        if source.len()==0 { 
            None
        } else {
            match source.chars().next() {
                Some('0') => Some((Digit(0u8),CharsCount(1))),
                Some('1') => Some((Digit(1u8),CharsCount(1))),
                Some('2') => Some((Digit(2u8),CharsCount(1))),
                Some('3') => Some((Digit(3u8),CharsCount(1))),
                Some('4') => Some((Digit(4u8),CharsCount(1))),
                Some('5') => Some((Digit(5u8),CharsCount(1))),
                Some('6') => Some((Digit(6u8),CharsCount(1))),
                Some('7') => Some((Digit(7u8),CharsCount(1))),
                Some('8') => Some((Digit(8u8),CharsCount(1))),
                Some('9') => Some((Digit(9u8),CharsCount(1))),
                _ => None
            }
        }
    }
}
impl DigitParser {
    fn parser() -> Box<dyn Parser<Digit>> {
        Box::new( DigitParser {} )
    }    
}

#[test]
fn test_parse() {
    let str = "123";
    let parser = DigitParser::parser();
    let res = parser.parse(str);
    assert!( res == Some((Digit(1),CharsCount(1))) )
}

struct ParseFollow<R1,R2> 
where 
    R1: Sized,
    R2: Sized,
{
    first: Box<dyn Parser<R1>>,
    second: Box<dyn Parser<R2>>,
}

impl<R1,R2> ParseFollow<R1,R2>
where
    R1: Sized,
    R2: Sized,
{
    fn new( p1:Box<dyn Parser<R1>>, p2:Box<dyn Parser<R2>> ) -> Self {
        Self { first: p1, second: p2 }
    }
}

impl<R1,R2> Parser<(R1,R2)> for ParseFollow<R1,R2> 
where
    R1: Sized,
    R2: Sized,
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
fn test_follow() {
    let str = "123";
    
    let parser1 = DigitParser::parser();
    let parser2 = DigitParser::parser();
    let parser = ParseFollow::new(parser1, parser2);

    let res = parser.parse(str);
    assert!( res == Some(((Digit(1),Digit(2)),CharsCount(2))) )
}
