use crate::*;

#[derive(Debug,Clone)]
enum WildcardToken {
    PlainText(String),
    OneAnyChar,
    MultipleAnyChar
}

#[derive(Debug,Clone)]
struct Wildcard { 
    seq: Vec<WildcardToken> 
}

#[derive(Debug,Clone)]
struct WildcardParser;

impl WildcardParser {
    pub fn new() -> Self {
        Self
    }
}

impl Parser<Wildcard> for WildcardParser {
    fn parse( &self, source: &str ) -> Option<(Wildcard, CharsCount)> {
        let mut state = "state";
        let mut cc = CharsCount(0);
        let mut res = Vec::<WildcardToken>::new();
        let mut str_buff = String::new();

        for chr in source.chars() {
            cc = cc + CharsCount(1);
            match chr {
                '?' => {
                    if !str_buff.is_empty() { 
                        res.push(WildcardToken::PlainText(str_buff.clone()));
                        str_buff.clear();
                    }
                    res.push(WildcardToken::OneAnyChar)
                },
                '*' => {
                    if !str_buff.is_empty() { 
                        res.push(WildcardToken::PlainText(str_buff.clone()));
                        str_buff.clear();
                    }
                    res.push(WildcardToken::MultipleAnyChar)
                },
                _ => {
                    str_buff.push(chr)
                }
            }
        }
        if !str_buff.is_empty() { 
            res.push(WildcardToken::PlainText(str_buff.clone()));
            str_buff.clear();
        }

        if cc.is_zero() { return None }

        Some((Wildcard { seq: res }.normalize(), cc))
    }
}

impl Wildcard {
    fn normalize( self ) -> Self {
        let tokens = self.seq.into_iter().fold( Vec::<WildcardToken>::new(), |mut acc, tok| {
            if acc.is_empty() {
                acc.push(tok);
                acc
            } else {
                let last = acc.last().unwrap().clone();
                match last {
                    WildcardToken::PlainText(str1) => {
                        match tok {
                            WildcardToken::PlainText(str2) => {
                                acc.pop();
                                let mut str3 = String::new();
                                str3.push_str(&str1);
                                str3.push_str(&str2);
                                acc.push(WildcardToken::PlainText(str3));
                            },
                            WildcardToken::OneAnyChar => {
                                acc.push(WildcardToken::OneAnyChar);
                            },
                            WildcardToken::MultipleAnyChar => {
                                acc.push(WildcardToken::MultipleAnyChar);
                            }
                        }
                    },
                    WildcardToken::OneAnyChar =>  {
                        acc.push(tok.clone());
                    },
                    WildcardToken::MultipleAnyChar =>  {
                        match tok.clone() {
                            WildcardToken::PlainText(_) => {
                                acc.push(tok);
                            },
                            WildcardToken::OneAnyChar => {
                                acc.push(WildcardToken::OneAnyChar);
                            },
                            WildcardToken::MultipleAnyChar => {}
                        }
                    }
                };
                acc
            }
        });

        Self { seq: tokens }
    }
}

#[test]
fn parse_wildcard_test() {
    let (wc,_) = WildcardParser.parse("12*a?c**d").unwrap();
    println!("{wc:?}");
}