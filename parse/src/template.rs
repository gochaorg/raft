use either::Either;

use crate::{Parser, CharsCount};

/// Шаблон - содержит список элементов шаблона [TemplateItem]
#[derive(Debug,Clone)]
pub struct Template {
    /// Элементы шаблона
    pub values: Vec<TemplateItem>
}

/// Элемент шаблона
#[derive(Debug,Clone)]
pub enum TemplateItem {
    /// Обычный текст
    PlainText(String),

    /// Некий код который надо интерпретировать
    Code(String)
}

/// Парсер шаблона
/// 
/// Синтаксис
/// ============
/// 
/// ```
/// // Шаблон
/// template ::= code 
///            | id      // правило регулируется флагом self.id
///            | escape 
///            | plain
/// 
/// // некий код
/// code ::= '${' inner '}'
/// 
/// // любые символы с учетом сбалансированных фигруных скобок
/// inner
/// 
/// // некий идентификатор (например переменная)
/// id ::= '$' id_start { id_cont }
/// 
/// id_start ::= буквы
///            | цифры // регулируется self.num_first
///            | '_'   // регулируется self.underscore_first
///            | '-'   // регулируется self.dash_first
///            | '.'   // регулируется self.dot_first
/// 
/// id_cont ::= буквы
///           | цифры
///           | '_'    // регулируется self.underscore
///           | '-'    // регулируется self.dash
///           | '.'    // регулируется self.dot
/// 
/// // экранирование
/// escape ::= '\\' any_char // одиночная обратная касая черта
/// ```
pub struct TemplateParser {
    /// Пустая строка интерпретируется как шаблон с 0 элементами, иначе None, а не шаблон
    pub empty: bool,

    /// дозволено ссылаться на id в шаблоне    
    pub id: bool,

    /// использование '-' в шаблоне id
    pub dash: bool,

    /// использование '.' в шаблоне id
    pub dot: bool,

    /// использование '_' в шаблоне id
    pub underscore: bool,

    /// использование цифр в качестве первого символа в шаблоне id
    pub num_first: bool,

    /// использование '_' в качестве первого символа в шаблоне id
    pub underscore_first: bool,

    /// использование '.' в качестве первого символа в шаблоне id
    pub dot_first: bool,

    /// использование '-' в качестве первого символа в шаблоне id
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
    pub fn to_string<F>( &self, mut f:F ) -> String 
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

impl Template {
    pub fn map<F1, A, F2, B>( &self, mut code_map:F1, mut text_map:F2 ) -> Vec<Either<A,B>> 
    where
        F1: FnMut(&str) -> A,
        A: Sized + Clone,
        F2: FnMut(&str) -> B,
        B: Sized + Clone,
    {
        let mut result : Vec<Either<A,B>> = vec![];
        for itm in &self.values {
            match itm {
                TemplateItem::Code(str) => {
                    result.push( Either::Left(code_map(str)) )
                },
                TemplateItem::PlainText(str) => {
                    result.push( Either::Right(text_map(str)) )
                }
            }
        }
        result
    }

    pub fn fold<A, F1, F2>( &self, initial:&A,  mut code_map:F1, mut text_map:F2 ) -> A
    where
        A: Sized + Clone,
        F1: FnMut(&A, &str) -> A,
        F2: FnMut(&A, &str) -> A,
    {
        let mut result = initial.clone();
        for itm in &self.values {
            match itm {
                TemplateItem::Code(str) => {
                    result = code_map(&result,str);
                },
                TemplateItem::PlainText(str) => {
                    result = text_map(&result, str);
                }
            }
        }
        result
    }
}