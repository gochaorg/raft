use crate::*;

#[derive(Debug,Clone,PartialEq)]
pub enum WildcardToken {
    PlainText(String),
    OneAnyChar,
    MultipleAnyChar
}

#[derive(Debug,Clone,PartialEq)]
pub struct Wildcard { 
    pub seq: Vec<WildcardToken> 
}

#[derive(Debug,Clone)]
struct WildcardParser;

impl Parser<Wildcard> for WildcardParser {
    fn parse( &self, source: &str ) -> Option<(Wildcard, CharsCount)> {
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
    /// Удаляет дублирующие записи
    /// 
    ///     [ ... , WildcardToken::MultipleAnyChar, WildcardToken::MultipleAnyChar, ... ]
    /// 
    /// заменяется на 
    /// 
    ///     [ ... , WildcardToken::MultipleAnyChar, ... ]
    /// 
    /// запись
    /// 
    ///     [ ..., WildcardToken::PlainText(str1), WildcardToken::PlainText(str2), ... ]
    /// 
    /// заменяется на 
    /// 
    ///     [ ..., WildcardToken::PlainText(str1 + str2), ... ]
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
    println!("aaa");
    println!("{wc:?}");
    assert!( wc == Wildcard { seq: vec![
        WildcardToken::PlainText("12".to_string()),
        WildcardToken::MultipleAnyChar,
        WildcardToken::PlainText("a".to_string()),
        WildcardToken::OneAnyChar,
        WildcardToken::PlainText("c".to_string()),
        WildcardToken::MultipleAnyChar,
        WildcardToken::PlainText("d".to_string()),
    ]})
}

impl Wildcard {
    /// Проверка текста на совпадение
    pub fn test( &self, text:&str ) -> bool {
        let work_set = &self.seq;

        if work_set.is_empty() && text.is_empty() {
            return true; // matched
        }

        // Сначала захватываем plain text позиции
        #[derive(Debug,Clone)]
        enum Capture {
            Plain {
                from: usize,
                to: usize,
            },
            Several {
                chars_counts: usize,
                upper_unlimited: bool,
            }
        }        

        let update_several = | captures: &mut Vec<Capture>, inc_several: usize, unlimited: bool | {
            match captures.pop() {
                Some( Capture::Several { chars_counts, upper_unlimited: upper_limited } ) => {
                    captures.push( Capture::Several { chars_counts: chars_counts + inc_several, upper_unlimited: upper_limited || unlimited } )
                },
                Some( c @ Capture::Plain { from:_, to:_ } ) => {
                    captures.push( c );
                    captures.push( Capture::Several { chars_counts: inc_several, upper_unlimited: unlimited } )
                },
                None => {
                    captures.push( Capture::Several { chars_counts: inc_several, upper_unlimited: unlimited } )
                }
            }
        };

        let (cap_succ, _, captures) = work_set.into_iter().fold( 
            (true, 0usize, Vec::<Capture>::new() ) , 
            |(succ, from, mut captures),it| {
                if succ {
                    match it {
                        WildcardToken::PlainText(sample) => {
                            match text.get(from..).and_then(|sub_text| sub_text.find(sample).map(|i| from+i)) {
                                Some(sub_match_start) => {
                                    let sub_match_end = sample.len() + sub_match_start;
                                    captures.push( Capture::Plain { from: sub_match_start, to: sub_match_end } );
                                    ( true, sub_match_end, captures )
                                },
                                None => {
                                    ( false, from, captures )
                                }
                            }
                        },
                        WildcardToken::OneAnyChar => {
                            update_several( &mut captures, 1, false );
                            ( true, from, captures )
                        },
                        WildcardToken::MultipleAnyChar => {
                            update_several( &mut captures, 0, true );
                            ( true, from, captures )
                        }
                    }
                } else {
                    (succ, from, captures)
                }
            }
        );

        if ! cap_succ || captures.is_empty() {
            return false; // not matched
        }

        let captures1 = &captures;

        let (_,cmin,cmax) = captures1.into_iter().fold(
            (0,0,0),
            |(n,vmin,vmax),it| {
            match it {
                Capture::Plain { from:_, to:_ } => {
                    let v = n+1;
                    (v, vmin.min(v.min(n)), vmax.max(v.max(n)))
                },
                Capture::Several { chars_counts:_, upper_unlimited:_ } => {
                    let v = n - 1;
                    (v, vmin.min(v.min(n)), vmax.max(v.max(n)))
                }
            }
        });

        if cmin < -1 { todo!("bug!"); } // bug
        if cmax >  1 { todo!("bug!"); } // bug

        let captures2 = &captures;

        let (mut _c_from,mut c_to) = 
        match captures2.into_iter().next() {
            Some( Capture::Plain { from, to }) => {
                if *from>0 { 
                    return false; // not matched
                }
                (*from, *to)
            },
            Some( Capture::Several { chars_counts:_, upper_unlimited:_ } ) => {
                if captures.len()>1 {
                    match &captures[1] {
                        Capture::Plain { from, to } => (*from, *to),
                        Capture::Several { chars_counts:_, upper_unlimited:_ } => {
                            todo!()
                        }
                    }
                }else{
                    (0usize, text.len())
                }
            }
            _ => todo!()
        };

        #[derive(Debug,Clone)]
        enum CapPos {
            Plain,
            Several {
                from: usize,
                to: usize,
                chars_counts: usize,
                upper_unlimited: bool,
            }
        }

        let mut cap_pos = Vec::<CapPos>::new();

        for idx in 0..captures.len() {
            let it = &captures[idx];
            match it {
                Capture::Plain { from, to } => {
                    _c_from = *from;
                    c_to = *to;
                    cap_pos.push( CapPos::Plain );
                },
                Capture::Several { chars_counts, upper_unlimited: upper_limited } => {
                    let start_pos = c_to;
                    let end_pos = 
                        if idx >= (captures.len()-1) {
                            text.len()
                        } else {
                            match &captures[idx+1] {
                                Capture::Plain { from, to:_ } => { *from },
                                Capture::Several { chars_counts:_, upper_unlimited:_ } => { todo!() }
                            }
                        };
                    _c_from = start_pos;
                    c_to = end_pos;
                    cap_pos.push( CapPos::Several { from: start_pos, to: end_pos, chars_counts: *chars_counts, upper_unlimited: *upper_limited } );
                }
            }
        }

        let succ = &cap_pos.into_iter().filter_map(|it| match &it {
            CapPos::Plain => None,
            CapPos::Several { from, to, chars_counts, upper_unlimited: upper_limited } => Some((*from, *to, *chars_counts, *upper_limited))
        }).map( |(from,to,chars_counts,upper_unlimited)| {
            let cnt = text[from..to].chars().count();
            //println!("  {}");
            if upper_unlimited {
                cnt >= chars_counts
            } else {
                cnt == chars_counts
            }
        }).fold( true, |acc,it| acc && it );
        
        *succ
    }
}

#[test]
fn wildcard_test() {

    #[derive(Debug,Clone, Copy)]
    struct Sample {
        pattern: &'static str,
        sample: &'static str,
        expect: bool
    }    

    let samples = vec![
        Sample { pattern: "ab*cd",   sample: "abXXXcd", expect:true },
        Sample { pattern: "ab*??cd", sample: "abXXXcd", expect:true },
        Sample { pattern: "ab???cd", sample: "abXXXcd", expect:true },
        Sample { pattern: "ab*cd",   sample: "abXXrd",  expect:false },
        Sample { pattern: "*",       sample: "abXXrd",  expect:true  },
    ];

    for sample in samples {        
        let (wc, _) = WildcardParser.parse(sample.pattern).unwrap();
        let actual = wc.test(sample.sample);
        println!( "pattern \"{ptrn}\" sample \"{sample}\" expect {expect} actual {actual}", 
            ptrn = sample.pattern,
            sample = sample.sample,
            expect = sample.expect
        );
        assert!( actual == sample.expect )
    }
}