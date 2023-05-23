use std::rc::Rc;

use crate::substr::*;
use super::{Parser, lookup::Lookup, lookup::LookupParser};

/// Парсинг ключевых слов
/// 
/// При использовании [KeywordsBuilder]
/// слова хранятся в порядке убывания длинны
#[derive(Clone,Debug)]
pub struct Keywords<A> 
where
    A: Clone+Sized
{
    /// Ключевое слово и его представление
    keywords: Vec<(String,A)>,

    /// Максимальная длинна слова
    max_keyword_len: usize
}

/// Создание парсера
#[derive(Clone,Debug)]
pub struct KeywordsBuilder<A> 
where
    A: Clone+Sized
{
    keywords: Vec<(String,A)>
}

impl<A> KeywordsBuilder<A> 
where
    A: Clone+Sized
{
    pub fn new( word:&str, value:&A ) -> Self {
        let mut kw = Vec::<(String,A)>::new();
        kw.push( (word.to_string(), value.clone()) );
        Self { keywords: kw }
    }

    #[allow(dead_code)]
    pub fn add( self, word:&str, value:&A ) -> Self {
        let mut kw = self.keywords.clone();
        kw.push( (word.to_string(), value.clone()) );
        Self { keywords: kw }
    }

    pub fn build( &self ) -> Keywords<A> {
        let mut kw = self.keywords.clone();
        kw.sort_by(|a,b| a.0.len().cmp(&b.0.len()));
        kw.reverse();
        let max_len = kw.iter().map(|(k,_)|k.len()).max().unwrap_or(0usize);
        Keywords { keywords: kw, max_keyword_len: max_len }
    }
}

impl<A> Parser<A> for Keywords<A>
where
    A: Clone+Sized
{
    fn parse( &self, source: &str ) -> Option<(A, CharsCount)> {
        if self.max_keyword_len==0 { return None; }

        let lookup = LookupParser { max_chars_count: self.max_keyword_len }.parse(source);

        match lookup {
            None => { return None; },
            Some((Lookup(str), _)) => {
                match self.keywords.iter().find(|(kw,_)| str.starts_with(kw)) {
                    None => None,
                    Some((kw, res)) => {
                        Some( (res.clone(), CharsCount(kw.chars().count())) )
                    }
                }
            }
        }
    }
}

impl<'a,A> Keywords<A> 
where A: Clone+Sized+'a
{
    pub fn parser( self ) -> Rc<dyn Parser<A> + 'a> {
        Rc::new( self.clone() )
    }
}

#[test]
fn test_kw_parse() {
    #[derive(Clone,Debug,PartialEq)]
    enum Op {
        Add, Sub, Add2
    }

    let kw_parser = KeywordsBuilder
        ::new("add", &Op::Add)
        .add("+", &Op::Add)
        .add("sub", &Op::Sub)
        .add("-", &Op::Sub)
        .add("++", &Op::Add2)
        .build();

    let res = kw_parser.parse("add");
    println!("{:?}", res);
    assert!(res == Some((Op::Add, CharsCount(3))));

    let res = kw_parser.parse("+");
    println!("{:?}", res);
    assert!(res == Some((Op::Add, CharsCount(1))));

    let res = kw_parser.parse("++");
    println!("{:?}", res);
    assert!(res == Some((Op::Add2, CharsCount(2))));
}
