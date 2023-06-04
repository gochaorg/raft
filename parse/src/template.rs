use crate::{Parser, CharsCount};

#[derive(Debug,Clone)]
pub struct Template {
    pub values: Vec<TemplateItem>
}

#[derive(Debug,Clone)]
pub enum TemplateItem {
    PlainText(String),
    Code(String)
}

pub struct TemplateParser {
    pub empty: bool,
    pub id: bool,
    pub dash: bool,
    pub dot: bool,
    pub underscore: bool,
    pub num_first: bool,
    pub underscore_first: bool,
    pub dot_first: bool,
    pub dash_first: bool,
}

impl Default for TemplateParser {
    fn default() -> Self {
        Self { 
            empty: true,
            id: true,
            dash: true,
            dot: true,
            underscore: true,
            num_first: true,
            underscore_first: true,
            dot_first: true,
            dash_first: true
        }
    }
}

impl Parser<Template> for TemplateParser {
    fn parse( &self, source: &str ) -> Option<(Template, CharsCount)> {
        if source.is_empty() {
            return if self.empty {
                Some(( Template {values: vec![]}, CharsCount(0) ))
            } else {
                None
            }
        }

        let mut state = "state";
        let mut level = 0u16;
        let mut cc = CharsCount(0);
        let mut buff = String::new();
        let mut res = Vec::<TemplateItem>::new();

        for chr in source.chars() {
            cc = cc + CharsCount(0);
            match state {
                "state" => {
                    match chr {
                        '\\' => {
                            state = "escape"
                        },
                        '$' => {
                            state = "begin";
                            level = 0;
                        }
                        _ => {
                            buff.push(chr)
                        }
                    }
                },
                "escape" => {
                    state = "state";
                    buff.push(chr);
                },
                "begin" => {
                    if ! buff.is_empty() {
                        res.push(TemplateItem::PlainText(buff.clone()));
                    }
                    buff.clear();
                    if ( chr.is_alphabetic() && self.id )
                    || ( self.id && self.num_first && chr.is_ascii_digit() )
                    || ( self.id && self.underscore_first && chr == '_' )
                    || ( self.id && self.dash_first && chr == '-' )
                    || ( self.id && self.dot_first && chr == '.' )
                    {
                        buff.push(chr);
                        state = "id"
                    } else if chr == '{' {
                        state = "code";
                        level += 1;
                    } else {
                        state = "state";
                    }
                },
                "id" => {
                    if chr.is_alphabetic() 
                    || chr.is_numeric() 
                    || ( chr == '_' && self.underscore )
                    || ( chr == '-' && self.dash )
                    || ( chr == '.' && self.dot )
                    {
                        buff.push(chr)
                    } else {
                        res.push(TemplateItem::Code(buff.clone()));
                        buff.clear();
                        buff.push(chr);
                        state = "state"
                    }
                },
                "code" => {
                    if chr == '{' {
                        buff.push(chr);
                        level += 1;
                    } else if chr == '}' {
                        level -= 1;
                        if level == 0 {
                            state = "state";
                            res.push(TemplateItem::Code(buff.clone()));
                            buff.clear()
                        } else {
                            buff.push(chr);
                        }
                    } else {
                        buff.push(chr);
                    }
                },
                _ => {}
            }
        }

        if ! buff.is_empty() {
            match state {
                "state" => { 
                    res.push(TemplateItem::PlainText(buff)) 
                },
                "id" => { 
                    res.push(TemplateItem::Code(buff))
                },
                "code" => { 
                    res.push(TemplateItem::Code(buff))
                },
                _ => {}
            }
        }

        Some((
            Template { values: res },
            cc
        ))
    }
}

#[test]
fn parse_test() {
    let (tmpl,_) = TemplateParser::default().parse("src $id-a.b_y x ${ aa {b} } d").unwrap();
    println!("{tmpl:?}");
}

impl Template {
    fn to_string<F>( &self, mut f:F ) -> String 
    where F: FnMut(&str) -> String
    {
        let mut buff: String = String::new();

        for itm in &self.values {
            match itm {
                TemplateItem::Code(str) => {
                    buff.push_str(&f(str));
                },
                TemplateItem::PlainText(str) => {
                    buff.push_str(str);
                }
            }
        }

        buff
    }
}

#[test]
fn to_string_test() {
    let (tmpl,_) = TemplateParser::default().parse("sample $a and $b").unwrap();
    let str = 
    tmpl.to_string(|code| 
        match code {
            "a" => format!("aa"),
            "b" => format!("bb"),
            _ => "".to_string()
        }
    );
    println!("{str}");
}