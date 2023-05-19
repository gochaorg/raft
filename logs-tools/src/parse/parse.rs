use std::{fmt::Debug, rc::Rc};

use crate::substr::*;

pub trait Parser<A:Sized> {
    fn parse( &self, source: &str ) -> Option<(A, CharsCount)>;
}
pub struct ParseFollow<'a,R1,R2> 
where 
    R1: Sized,
    R2: Sized,
{
    first: Rc<dyn Parser<R1> + 'a>,
    second: Rc<dyn Parser<R2> + 'a>,
}

#[allow(dead_code)]
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
    use super::digit::*;

    let str = "123";
    
    let parser1 = DigitParser::parser();
    let parser2 = DigitParser::parser();
    let parser = ParseFollow::new(parser1, parser2);

    let res = parser.parse(str);
    assert!( res == Some(((Digit(1),Digit(2)),CharsCount(2))) )
}

pub fn follow<'a,A:Sized+Clone+'a,B:Sized+Clone+'a>( left:Rc<dyn Parser<A> + 'a>, right:Rc<dyn Parser<B> + 'a> ) -> Rc<dyn Parser<(A,B)> + 'a> {
    let parser = Rc::new(ParseFollow { first: left.clone(), second: right.clone() });
    parser
}

#[test]
fn test_follow_2() {
    use super::*;

    let str = "123";
    
    let parser1 = DigitParser::parser();
    let parser2 = DigitParser::parser();
    let parser = follow(parser1, parser2);

    let res = parser.parse(str);
    assert!( res == Some(((Digit(1),Digit(2)),CharsCount(2))) )
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

