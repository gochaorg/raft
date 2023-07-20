use std::{rc::Rc, cell::RefCell, sync::Mutex, collections::HashMap, fmt::Debug};

use chrono::Utc;
use date_format::{DateFormat, Format};
use parse::{TemplateParser, Parser, NumberParser};
use rand::{rngs::ThreadRng, RngCore};

/// Шаблон генерируемого файла
#[allow(dead_code)]
#[derive(Clone)]
pub struct PathTemplate<'a> {
    generators: Vec<Rc<Mutex<dyn PathValue + 'a>>>
}

#[allow(dead_code)]
impl<'a> PathTemplate<'a> {
    /// Генерация имени файла
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

    pub fn clone<'r>( &'a self ) -> PathTemplate<'r> {
        PathTemplate { 
            generators: self.generators.iter()
                .map(|i| {
                    let c = i.lock().unwrap();
                    c.clone()
                })
                .collect()
        }
    }
}

/// Элемент имени файла
pub trait PathValue {
    fn generate( &mut self ) -> String;
    fn clone<'a,'r>( &'a self ) -> Rc<Mutex<dyn PathValue + 'r>>;
}

/// Обычный текст в имени файла
pub struct PlainValue( pub String );
impl PathValue for PlainValue {
    fn generate( &mut self ) -> String {
        self.0.clone()
    }
    fn clone<'a,'r>( &'a self ) -> Rc<Mutex<dyn PathValue + 'r>> {
        Rc::new(Mutex::new(PlainValue(self.0.clone())))
    }
}

/// Время в имени файла
/// DateFormat - формат времени
pub struct CurrentDateTimeValue( pub DateFormat );
impl PathValue for CurrentDateTimeValue {
    fn generate( &mut self ) -> String {
        let dt = Utc::now();
        dt.format(&self.0)
    }
    fn clone<'a,'r>( &'a self ) -> Rc<Mutex<dyn PathValue + 'r>> {
        Rc::new(Mutex::new(CurrentDateTimeValue(self.0.clone())))
    }
}

/// Случайное значение в имени файла
#[derive(Clone)]
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
                let rndi = self.rnd.next_u64() as usize;
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

    fn clone<'a,'r>( &'a self ) -> Rc<Mutex<dyn PathValue + 'r>> {
        Rc::new(Mutex::new(RandomValue {
            dic: self.dic.clone(),
            dic_char_count: self.dic_char_count.clone(),
            count: self.count.clone(),
            rnd: self.rnd.clone()
        }))
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

/// Парсер шаблона имени файла
#[derive(Clone)]
pub struct PathTemplateParser<'a> {
    pub variables: HashMap<String, Rc<Mutex<dyn PathValue + 'a>> >
}

impl<'a> Default for PathTemplateParser<'a> {
    fn default() -> Self {
        PathTemplateParser { 
            variables: HashMap::new()
        }
    }
}

impl<'a> Debug for PathTemplateParser<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        str.push_str("PathTemplateParser { variables: ");

        for (idx,key) in self.variables.keys().into_iter().enumerate() {
            if idx > 0 { str.push_str(", ") }
            str.push_str(key)
        }
        str.push_str("}");

        write!(f,"{str}")
    }
}

#[allow(dead_code)]
impl<'a> PathTemplateParser<'a> {
    /// Добавляет переменную в шаблон
    pub fn set_variable<K,V>( &mut self, name:K, value:V )
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.variables.insert(name.into(), Rc::new(Mutex::new(PlainValue(value.into()))));
    }

    /// Добавляет переменную в шаблон
    pub fn with_variable<K,V>( mut self, name:K, value:V ) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.variables.insert(name.into(), Rc::new(Mutex::new(PlainValue(value.into()))));
        self
    }

    /// Парсинг шаблона
    /// 
    /// Пример шаблона
    /// 
    /// ```
    /// "${root}/${time:local:yyyy-mm-ddThh-mi-ss}-${rnd:5}.binlog"
    /// ```
    /// 
    /// - `${....}` - некие переменные которые могут содержать значения
    /// - синтаксис шаблона описан в структуре [TemplateParser]
    /// - `${root}` - это внешняя переменная и должна быть определена явно [with_variable()]
    /// - `${time:...}` - встроенная переменаая, задает текущую дату, формат даты описан в [DateFormat]
    /// - `${rnd:5}` - случайны набор из 5 букв, число 5 - указывает на кол-во букв и может быть заменено на другое число
    /// - `${env:...}` - в качестве значения - потенциально опасно
    pub fn parse<'r>(&self, source: &str) -> Result<PathTemplate<'r>, String> {
        let p_tmpl = RefCell::new(Vec::<Rc<Mutex<dyn PathValue>>>::new());

        let tmpl = TemplateParser::default();
        match tmpl.parse(source) {
            Some((tmpl,_)) => {
                let _fold_res = tmpl.fold(&Ok(()), // TODO check result
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
        let res: Result<Vec<Rc<Mutex<dyn PathValue>>>, String> = p_tmpl.iter().fold( Ok(Vec::<Rc<Mutex<dyn PathValue + 'r>>>::new()), |res,i| {
            res.and_then(|mut res| {
                match i.lock() {
                    Ok(i) => {
                        res.push( i.clone() );
                        Ok(res)
                    },
                    Err(err) => Err(format!("can't lock item: {}", err.to_string()))
                }
            })
        });
        res.map(|res| PathTemplate { generators: res })
    }
}

#[test]
fn parse_template_test() {
    let mut parser = PathTemplateParser::default();

    let log_num = Rc::new(Mutex::new(PlainValue(format!("LOG-a"))));
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