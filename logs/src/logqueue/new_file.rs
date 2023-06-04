use std::{rc::Rc, cell::RefCell, sync::Mutex, collections::HashMap};

use chrono::Utc;
use date_format::{DateFormat, Format};
use parse::{TemplateParser, Parser, NumberParser};
use rand::{rngs::ThreadRng, RngCore};

#[allow(dead_code)]
#[derive(Clone)]
pub struct PathTemplate<'a> {
    generators: Vec<Rc<Mutex<dyn PathValue + 'a>>>
}

impl<'a> PathTemplate<'a> {
    pub fn generate( &mut self ) -> String {
        let mut str = String::new();
        for generator in &mut self.generators {
            match generator.lock() {
                Ok(mut generator) => {
                    let value = generator.generate();
                    str.push_str(&value);
                },
                _ => {
                    panic!("can't lock")
                }
            }
        }
        str
    }
}

pub trait PathValue {
    fn generate( &mut self ) -> String;
}

pub struct PlainValue( pub String );
impl PathValue for PlainValue {
    fn generate( &mut self ) -> String {
        self.0.clone()
    }
}

pub struct CurrentDateTimeValue( pub DateFormat );
impl PathValue for CurrentDateTimeValue {
    fn generate( &mut self ) -> String {
        let dt = Utc::now();
        dt.format(&self.0)
    }
}

pub struct RandomValue {
    dic: String,
    dic_char_count: usize,
    count: u32,
    rnd: ThreadRng,
}

impl PathValue for RandomValue {
    fn generate( &mut self ) -> String {
        let mut str = String::new();
        if self.dic.len()>0 {
            for _x in 0..self.count {
                let rndi = (self.rnd.next_u64() as usize);
                let rndi = rndi % self.dic_char_count;
                match &self.dic.chars().skip(rndi).next() {
                    Some(c) => { 
                        str.push(*c) 
                    }
                    _ => {}
                }
            }
        }
        str
    }
}

impl Default for RandomValue {
    fn default() -> Self {
        let dic = "0123456789abcdefghijklmnopqrstuvwxyz";
        RandomValue { 
            dic: dic.to_string(), 
            dic_char_count: dic.chars().count(), 
            count: 1, 
            rnd: rand::thread_rng()
        }
    }
}

#[derive(Clone)]
pub struct PathTemplateParser {
    pub variables: HashMap<String, Rc<Mutex<dyn PathValue>> >
}

impl Default for PathTemplateParser {
    fn default() -> Self {
        PathTemplateParser { 
            variables: HashMap::new()
        }
    }
}

impl PathTemplateParser {
    pub fn parse<'a>(&self, source: &'a str) -> Result<Box<PathTemplate>, String> {
        let mut p_tmpl = RefCell::new(Vec::<Rc<Mutex<dyn PathValue>>>::new());

        let mut eval_code_err : Option<String> = None;
        let tmpl = TemplateParser::default();
        match tmpl.parse(source) {
            Some((tmpl,_)) => {
                let fold_res = tmpl.fold(&Ok(()), 
                |res,code| { 
                    if res.is_err() {
                        return res.clone();
                    }

                    if code.starts_with("time:") && code.len() > "time:".len() {
                        let sub_code = &code["time:".len()..];
                        p_tmpl.borrow_mut().push(
                            Rc::new(Mutex::new(
                                CurrentDateTimeValue( DateFormat::parse(sub_code) )
                            ))
                        );
                        Ok(())
                    } else if code.starts_with("env:") && code.len() > "env:".len() {
                        let sub_code = &code["env:".len()..];
                        match std::env::var(sub_code) {
                            Ok(value) => {
                                p_tmpl.borrow_mut().push(
                                    Rc::new(Mutex::new(
                                        PlainValue(value)
                                    ))
                                );
                                Ok(())
                            },
                            Err(err) => { Err(format!("env variable err: {err}")) }
                        }
                    } else if code.starts_with("rnd:") && code.len() > "rnd:".len() {
                        let sub_code = &code["rnd:".len()..];
                        match NumberParser.parse(sub_code) {
                            Some((num,_)) => {
                                match num.try_u32() {
                                    Some(num) => {
                                        let mut rnd = RandomValue::default();
                                        rnd.count = num;
                                        p_tmpl.borrow_mut().push(
                                            Rc::new(Mutex::new(
                                                rnd
                                            ))
                                        );
                                        Ok(())
                                    }
                                    None => { Err(format!("can't convert from {sub_code} to u32")) }
                                }
                            }
                            None => { Err(format!("can't parse {sub_code} as number")) }
                        }
                    } else { 
                        match self.variables.get(&code.to_string()) {
                            Some(path_var) => {
                                let path = path_var.clone();
                                p_tmpl.borrow_mut().push( path );

                                Ok(())
                            },
                            _ => {
                                Err(format!("can't read internal variable {code}"))
                            }
                        }
                    }
                }, 
                |_,text| {
                    p_tmpl.borrow_mut().push(
                        Rc::new(Mutex::new(
                            PlainValue(text.to_string())
                        ))
                    );
                    Ok(())
                });
            },
            None => {}
        };

        let p_tmpl = p_tmpl.borrow().clone();
        Ok(Box::new( PathTemplate { generators: p_tmpl } ))
    }
}

#[test]
fn parse_template_test() {
    let mut parser = PathTemplateParser::default();

    let mut log_num = Rc::new(Mutex::new(PlainValue(format!("LOG-a"))));
    parser.variables.insert(format!("logn"), log_num.clone());

    let tmpl = parser.parse("/home/${env:USER}/${logn}/${time:yyyy-mm-dd}/${time:hh-mi-ss}-rnd${rnd:5}.log");
    let tmpl = tmpl.clone();
    match tmpl {
        Ok(mut path_tmpl) => {
            let generated = path_tmpl.generate();
            println!("{generated}");

            {
                log_num.lock().unwrap().0 = "LOG-b".to_string()
            }

            let generated = path_tmpl.generate();
            println!("{generated}");
        },
        _ => {}
    }
}