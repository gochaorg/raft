use crate::*;

#[derive(Debug,Clone,PartialEq)]
pub enum WildcardToken {
    PlainText(String),
    OneAnyChar,
    MultipleAnyChar
}

/// Шаблон для сопостовления текст
/// 
/// Пример
/// 
///     // создание шаблона
///     let sample = "шаблон";
///     let (wc, _) = WildcardParser::new().parse(sample).unwrap();
/// 
///     // проверка совпадения текста
///     if wc.test("текст") {
///     }
/// 
/// Исходный шаблон может содержать специальные символы и обычный текст
/// 
/// - `?` - любой символ, одна штука
/// - `*` - либой символ, 0 или более штук
/// 
/// Примеры
/// 
/// | Шаблон     | Проверяемая строка | Результат |
/// |------------|--------------------|-----------|
/// | "ab*cd"    | "abXXXcd"          | true      |
/// | "ab*??cd"  | "abXXXcd"          | true      |
/// | "ab???cd"  | "abXXXcd"          | true      |
/// | "ab*cd"    | "abXXrd"           | false     |
/// | "*"        | "abXXrd"           | true      |
/// | ""         | "abXXrd"           | false     |
/// | ""         | ""                 | true      |
#[derive(Debug,Clone,PartialEq)]
pub struct Wildcard { 
    pub seq: Vec<WildcardToken> 
}

/// Парсер шаблона
#[derive(Debug,Clone)]
pub struct WildcardParser {
    /// Допускается пустой шаблон
    pub allow_empty: bool
}

impl WildcardParser {
    /// Создает парсер, по умолчанию допускается пустой шаблон
    pub fn new() -> Self {
        Self { allow_empty: true }
    }

    /// Создание шаблона
    pub fn create() -> WildcardParserBuilder {
        WildcardParserBuilder { allow_empty: true }
    }
}

/// Builder для [WildcardParser]
pub struct WildcardParserBuilder {
    pub allow_empty: bool
}

impl WildcardParserBuilder {
    /// Допускать или нет пустой шаблон
    pub fn allow_empty( self, allow: bool ) -> Self {
        Self { allow_empty: allow }
    }

    /// Создание [WildcardParser]
    pub fn build( self ) -> WildcardParser {
        WildcardParser { allow_empty: self.allow_empty }
    }
}

/// Парсер шаблона
impl Parser<Wildcard> for WildcardParser {
    fn parse( &self, source: &str ) -> Option<(Wildcard, CharsCount)> {
        if self.allow_empty && source.is_empty() {
            return Some((
                Wildcard { seq: vec![] },
                CharsCount(0)
            ))
        }

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
    let (wc,_) = WildcardParser::new().parse("12*a?c**d").unwrap();
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

const WILDCARD_DEBUG : bool = false;

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
                // Минимальное кол-во любых символов (?)
                chars_counts: usize,

                // Ограниченна ли верхнаяя граница или нет (*)
                upper_unlimited: bool,
            }
        }        

        // Обновление/добавление последнего значение Capture::Several
        // 
        // - inc_several - какое значение добавить chars_counts
        // - unlimited - установить ли upper_unlimited
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

        // Поиск совпадения plain text
        // все записи WildcardToken::PlainText должны быть найдены в строке, в той же последованности что заданы в шаблоне
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
            // не найдены WildcardToken::PlainText
            // или расположены в не той последовательности
            return false;
        }

        // элементы captures должны чередоваться 
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

        // если чередования нет, то будет превышение
        if cmin < -1 { todo!("bug!"); } // bug
        if cmax >  1 { todo!("bug!"); } // bug

        let captures2 = &captures;
        if WILDCARD_DEBUG { println!("\n{:?}", captures2.clone()) }

        let (mut _c_from, mut _c_to) = 
        match captures2.into_iter().next() {
            Some( Capture::Plain { from, to }) => {
                if *from>0 { 
                    // если начинается не с начало строки, то уже не совпадение
                    return false; 
                }
                (0usize, *to)
            },
            Some( Capture::Several { chars_counts:_, upper_unlimited:_ } ) => {
                if captures.len()>1 {
                    match &captures[1] {
                        Capture::Plain { from, to:_ } => (0usize, *from),
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

        // Последний сapture должен совпадать с концом текста
        match captures2.last() {
            Some( Capture::Plain { from:_, to } ) => {
                if *to < text.len() {
                    return false;
                }
            },
            _ => {}
        }

        // Все Capture::Plain - проверели
        // теперь проверяем Capture::Several
        // сначала полчим для каждого Capture::Several вычислим расположение в строке

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

        // for idx in 0..captures.len() {
        //     let it = &captures[idx];
        //     match it {
        //         Capture::Plain { from, to } => {
        //             c_from = *from;
        //             c_to = *to;
        //             cap_pos.push( CapPos::Plain );
        //         },
        //         Capture::Several { chars_counts, upper_unlimited: upper_limited } => {
        //             let start_pos = c_to;
        //             let end_pos = 
        //                 if idx >= (captures.len()-1) {
        //                     text.len()
        //                 } else {
        //                     match &captures[idx+1] {
        //                         Capture::Plain { from, to:_ } => { *from },
        //                         Capture::Several { chars_counts:_, upper_unlimited:_ } => { todo!() }
        //                     }
        //                 };
        //             c_from = start_pos;
        //             c_to = end_pos;
        //             cap_pos.push( CapPos::Several { from: start_pos, to: end_pos, chars_counts: *chars_counts, upper_unlimited: *upper_limited } );
        //         }
        //     }
        // }

        if captures.len()>1 {
            for idx in 0..captures.len() {
                if idx == captures.len()-1 {
                    // last
                    let prev = &captures[idx-1];
                    let cur = &captures[idx];
                    match cur {
                        Capture::Several { chars_counts, upper_unlimited } => {
                            match prev {
                                Capture::Plain { from, to } => {
                                    let (_prev_from, prev_to) = (*from, *to);
                                    cap_pos.push( CapPos::Several { from: prev_to, to: text.len(), chars_counts: *chars_counts, upper_unlimited: *upper_unlimited } )
                                },
                                _ => { todo!("bug") }
                            }
                        },
                        _ => { cap_pos.push( CapPos::Plain ) }
                    }
                } else if idx == 0 {
                    // first
                    let cur = &captures[idx];
                    let next = &captures[idx+1];
                    match cur {
                        Capture::Several { chars_counts, upper_unlimited } => {
                            match next {
                                Capture::Plain { from, to } => {
                                    let (next_from, _next_to) = (*from, *to);
                                    cap_pos.push( CapPos::Several { from: 0usize, to: next_from, chars_counts: *chars_counts, upper_unlimited: *upper_unlimited } )
                                },
                                _ => { todo!("bug") }
                            }
                        },
                        _ => { cap_pos.push( CapPos::Plain ) }
                    }
                } else {
                    // middle
                    let prev = &captures[idx-1];
                    let cur = &captures[idx];
                    let next = &captures[idx+1];
                    match cur {
                        Capture::Several { chars_counts, upper_unlimited } => {
                            match next {
                                Capture::Plain { from, to } => {
                                    let (next_from, _next_to) = (*from, *to);
                                    match prev {
                                        Capture::Plain { from, to } => {
                                            let (_prev_from, prev_to) = (*from, *to);
                                            cap_pos.push( CapPos::Several { from: prev_to, to: next_from, chars_counts: *chars_counts, upper_unlimited: *upper_unlimited } )
                                        },
                                        _ => { todo!("bug") }
                                    }
                                },
                                _ => { todo!("bug") }
                            }
                        },
                        _ => { cap_pos.push( CapPos::Plain ) }
                    }
                }
            }
        } else if captures.len() == 1 {
            let first = &captures[0];
            match first {
                Capture::Plain { from:_, to:_ } => {
                    cap_pos.push( CapPos::Plain )
                },
                Capture::Several { chars_counts, upper_unlimited } => {
                    cap_pos.push( CapPos::Several { from: 0usize, to: text.len(), chars_counts: *chars_counts, upper_unlimited: *upper_unlimited } )
                }
            }
        }

        // Все CapPos::Several должны совпадать с требованием chars_counts и upper_unlimited
        let cap_pos1 = &cap_pos;
        if WILDCARD_DEBUG { println!("{cap_pos1:?}") }

        let succ = {
            let succ = cap_pos1.into_iter().filter_map(|it| match &it {
                CapPos::Plain => None,
                CapPos::Several { from, to, chars_counts, upper_unlimited: upper_limited } => Some((*from, *to, *chars_counts, *upper_limited))
            }).map( |(from,to,chars_counts,upper_unlimited)| {
                let cnt = text[from..to].chars().count();
                if upper_unlimited {
                    cnt >= chars_counts
                } else {
                    cnt == chars_counts
                }
            }).fold( true, |acc,it| acc && it );
            succ
        };

        // Последний сapture должен совпадать с концом текста
        match cap_pos1.last() {
            Some(CapPos::Several { from:_, to, chars_counts:_, upper_unlimited:_ }) => {
                if *to < text.len() {
                    return false;
                }
            },
            _ => {}
        }

        succ
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
        Sample { pattern: "*",       sample: "abXXrd",  expect:true  },
        Sample { 
            pattern: "*log*rs",   
            sample: "./target/doc/logs_tools/actions/viewheaders", 
            expect: true
        },
        Sample { pattern: "ab???cd", sample: "abXXXcd", expect:true },
        Sample { 
            pattern: "*log*rs",   
            sample: "./target/debug/incremental/logs-172hjs3xscjc0/s-glbyyu88xb-1rdcwv3-1q7l0skrhptjq/2e8lrs8qt1qb18xz.o", 
            expect: false
        },
        Sample { pattern: "ab*cd",   sample: "abXXXcd", expect:true },
        Sample { pattern: "ab*??cd", sample: "abXXXcd", expect:true },
        Sample { pattern: "ab*cd",   sample: "abXXrd",  expect:false },
        Sample { pattern: "",        sample: "abXXrd",  expect:false  },
        Sample { pattern: "",        sample: "",        expect:true  },
    ];

    for sample in samples {        
        let (wc, _) = WildcardParser::new().parse(sample.pattern).unwrap();
        print!( "pattern \"{ptrn}\" sample \"{sample}\" expect {expect}", 
            ptrn = sample.pattern,
            sample = sample.sample,
            expect = sample.expect,
        );
        let actual = wc.test(sample.sample);
        println!(" actual {actual}");
        assert!( actual == sample.expect )
    }
}