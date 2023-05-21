use crate::substr::CharsCount;

use super::Parser;

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

#[allow(dead_code)]
impl<R:Sized+Clone> LookupContext<R> {
    pub fn when_equals( &mut self, str: &str, res:R ) -> &mut Self {
        self.data.push((str.to_string(), res.clone()));
        self
    }

    pub fn fetch( &self ) -> Option<(R,CharsCount)> {
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

