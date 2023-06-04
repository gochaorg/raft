use std::{ cell::{Cell, RefCell} };
use parse::Parser;

use crate::{DateTimeItem, DateValue, DateTimeZone};

/// Парсинг формата времени
/// 
/// | Переменная | Значение       | DateTimeItem | Пример |
/// |------------|----------------|--------------|----|
/// | yyyy       | год - 4 цифры                                | Year           | 1998 | 
/// | yy         | год - 2 цифры                                | Year2digit     | 98 |
/// | mm         | месяц 01..12                                 | Month          |
/// | mmm        | месяц 3 буквы                                | MonthNameShort |
/// | mmmm       | месяц полное название                        | MonthNameFull  |
/// | dd         | дата 01..31                                  | Date           |
/// | wd         | день недели 3 буквы                          | WeekDayShort   |
/// | wd0        | день недели Воскресенье=0 ... Суббота=6      | WeekDayZero    |
/// | wd1        | день недели Понедельник=1 ... Воскресенье=7  | WeekDayOne     |
/// | ww         | неделя 00..53                                | Week
/// | wwd        | день недели - полное имя                     | WeekDayFull
/// | w1         | неделя 00..53 - неделя начинается с ПН       | WeekMondayFirst
/// | hh         | час 0-23                                     | Hour
/// | hp         | час 0-12                                     | Hour12
/// | ha         | час 0-12                                     | Hour12
/// | am         | am или pm                                    | AMPMLoCase
/// | AM         | AM или PM                                    | AMPMHiCase
/// | pm         | am или pm                                    | AMPMLoCase
/// | PM         | AM или PM                                    | AMPMHiCase
/// | mi         | минуты                                       | Minute |
/// | ss         | секунды                                      | Second |
/// | s3         | миллисек                                     | Millisec | 026 |
/// | s6         | микросек                                     | Microsec | 026490 |
/// | s9         | наносек                                      | Nanosec | 026490000 |
/// | ms         | миллисек                                     | Millisec | 026490 |
/// | ns         | наносек                                      | Nanosec | 026490000 |
/// | z4         | смещение UTC                                 | Zone4 | +0930 |
/// | zh         | смещение UTC                                 | ZoneHour | +09 |
/// | zhm        | смещение UTC                                 | ZoneHourMin | +09:30 |
/// | zhms       | смещение UTC                                 | ZoneHourMinSec | +09:30:00 |
pub struct DateFormatParser {
    pub(crate) default_time_zone : DateTimeZone
}

impl Default for DateFormatParser {
    fn default() -> Self {
        Self { default_time_zone: DateTimeZone::Local }
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct DateFormat {
    pub(crate) format: Vec<DateFormatItem>
}

#[derive(Debug,Clone,PartialEq)]
pub(crate) enum DateFormatItem {
    Value(DateValue),
    PlainText(String)
}

impl Parser<DateFormat> for DateFormatParser {
    fn parse( &self, source: &str ) -> Option<(DateFormat, parse::CharsCount)> {
        let mut res = RefCell::new(Vec::<DateFormatItem>::new());

        let buff = RefCell::new(String::new());
        let buff_push = | chr:char | {
            buff.borrow_mut().push(chr)
        };
        let buff_flush = || {
            let mut buf = buff.borrow_mut();
            if ! buf.is_empty() {
                res.borrow_mut().push( DateFormatItem::PlainText(buf.clone()) );
            }
            buf.clear();
        };

        let state = Cell::new("init");
        let state_get  = || { state.get() };
        let state_set = | st: &'static str | {
            state.set(st)
        };

        let mut tz = self.default_time_zone;

        let state_init = |chr: char| {
            match chr {
                '\'' => state_set("quote"),
                'y' => state_set("y"),
                'm' => state_set("m"),
                'd' => state_set("d"),
                'w' => state_set("w"),
                'h' => state_set("h"),
                'a' => state_set("a"),
                'A' => state_set("A"),
                'p' => state_set("p"),
                'P' => state_set("P"),
                's' => state_set("s"),
                'n' => state_set("n"),
                'z' => state_set("z"),
                'u' => state_set("u"),
                'l' => state_set("l"),
                'o' => state_set("o"),
                _ => buff.borrow_mut().push(chr)
            }
        };

        let state_accpet = |chr: char| {
            match chr {
                '\'' => state_set("quote"),
                'y' => state_set("y"),
                'm' => state_set("m"),
                'd' => state_set("d"),
                'w' => state_set("w"),
                'h' => state_set("h"),
                'a' => state_set("a"),
                'A' => state_set("A"),
                'p' => state_set("p"),
                'P' => state_set("P"),
                's' => state_set("s"),
                'n' => state_set("n"),
                'z' => state_set("z"),
                _ => buff.borrow_mut().push(chr)
            }
        };

        let mut cc = parse::CharsCount(0);
        let mut offset_sign : i8 = 0;
        let mut offset_digits = Vec::<char>::new();
        
        for chr in source.chars() {
            cc = cc + parse::CharsCount(1);
            match state_get() {
                "init" => { state_init(chr) },
                "state" => { state_accpet(chr) },
                "quote" => {
                    match chr {
                        '\'' => {
                            buff_push('\'');
                            state_set("state");
                        },
                        _ => {
                            state_set("quoted");
                            buff_push(chr)
                        }
                    }
                }
                "quoted" => {
                    match chr {
                        '\'' => {
                            state_set("quoted_try_end")
                        },
                        _ => {
                            buff_push(chr)
                        }
                    }
                }
                "quoted_try_end" => {
                    match chr {
                        '\'' => {
                            state_set("quoted");
                            buff_push('\'');
                        },
                        _ => {
                            state_set("state");
                            state_accpet(chr)
                        }
                    }
                }
                "y" => {
                    match chr {
                        'y' => state_set("yy"),
                        _ => {
                            buff_push('y');
                            state_set("state");
                            state_accpet(chr)
                        }
                    }
                }
                "yy" => {
                    match chr {
                        'y' => state_set("yyy"),
                        _ => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Year2digit 
                                    }
                                )
                            );
                            state_accpet(chr)
                        }
                    }
                }
                "yyy" => {
                    match chr {
                        'y' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Year 
                                    }
                                )
                            )
                        },
                        _ => {
                            state_set("state");
                            buff_push('y');
                            buff_push('y');
                            state_accpet(chr)
                        }
                    }
                }
                "m" => {
                    match chr {
                        'm' => state_set("mm"),
                        'i' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Minute 
                                    }
                                )
                            );
                        },
                        's' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Millisec 
                                    }
                                )
                            );
                        },
                        _ => {
                            state_set("state");
                            buff_push('m');
                            state_accpet(chr)
                        }
                    }
                }
                "mm" => {
                    match chr {
                        'm' => state_set("mmm"),
                        _ => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Month 
                                    }
                                )
                            );
                            state_accpet(chr)
                        }
                    }
                }
                "mmm" => {
                    match chr {
                        'm' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::MonthNameFull 
                                    }
                                )
                            );
                        },
                        _ => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::MonthNameShort 
                                    }
                                )
                            );
                            state_accpet(chr)
                        }
                    }
                }
                "d" => {
                    match chr {
                        'd' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Date 
                                    }
                                )
                            );
                        }
                        _ => {
                            state_set("state");
                            buff_push('d');
                            state_accpet(chr)
                        }
                    }
                }
                "w" => {
                    match chr {
                        'd' => {
                            state_set("wd");
                        }
                        'w' => {
                            state_set("ww");
                        }
                        '1' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::WeekMondayFirst 
                                    }
                                )
                            );
                        }
                        _ => {
                            state_set("state");
                            buff_push('w');
                            state_accpet(chr)
                        }
                    }
                }
                "wd" => {
                    match chr {
                        '0' => {
                            buff_flush();
                            state_set("state");
                            DateFormatItem::Value(
                                DateValue { 
                                    zone: tz, 
                                    item: DateTimeItem::WeekDayZero 
                                }
                            );
                        }
                        '1' => {
                            buff_flush();
                            state_set("state");
                            DateFormatItem::Value(
                                DateValue { 
                                    zone: tz, 
                                    item: DateTimeItem::WeekDayOne 
                                }
                            );
                        }
                        _ => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::WeekDayShort 
                                    }
                                )
                            );
                            state_accpet(chr)
                        }
                    }
                }
                "ww" => {
                    match chr {
                        'd' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::WeekDayFull 
                                    }
                                )
                            );
                        }
                        _ => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Week 
                                    }
                                )
                            );
                            state_accpet(chr)
                        }
                    }
                }
                "h" => {
                    match chr {
                        'h' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Hour 
                                    }
                                )
                            );
                        }
                        'a' | 'p' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Hour12 
                                    }
                                )
                            );
                        }
                        _ => {
                            state_set("state");
                            buff_push('h');
                            state_accpet(chr)
                        }
                    }
                }
                "a" => {
                    match chr {
                        'm' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::AMPMLoCase 
                                    }
                                )
                            );
                        }
                        _ => {
                            state_set("state");
                            buff_push('a');
                            state_accpet(chr)
                        }
                    }
                }
                "A" => {
                    match chr {
                        'M' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::AMPMHiCase 
                                    }
                                )
                            );
                        }
                        _ => {
                            state_set("state");
                            buff_push('A');
                            state_accpet(chr)
                        }
                    }
                }
                "p" => {
                    match chr {
                        'm' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::AMPMLoCase 
                                    }
                                )
                            );
                        }
                        _ => {
                            state_set("state");
                            buff_push('p');
                            state_accpet(chr)
                        }
                    }
                }
                "P" => {
                    match chr {
                        'M' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::AMPMHiCase 
                                    }
                                )
                            );
                        }
                        _ => {
                            state_set("state");
                            buff_push('P');
                            state_accpet(chr)
                        }
                    }
                }
                "s" => {
                    match chr {
                        's' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Second 
                                    }
                                )
                            );
                        }
                        '3' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Millisec 
                                    }
                                )
                            );
                        }
                        '6' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Microsec 
                                    }
                                )
                            );
                        }
                        '9' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Nanosec 
                                    }
                                )
                            );
                        }
                        _ => {
                            state_set("state");
                            buff_push('s');
                            state_accpet(chr)
                        }
                    }
                }
                "n" => {
                    match chr {
                        's' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Nanosec 
                                    }
                                )
                            );
                        }
                        _ => {
                            state_set("state");
                            buff_push('n');
                            state_accpet(chr)
                        }
                    }
                }
                "z" => {
                    match chr {
                        '4' => {
                            buff_flush();
                            state_set("state");
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::Zone4 
                                    }
                                )
                            );
                        }
                        'h' => { state_set("zh"); }
                        _ => {
                            state_set("state");
                            buff_push('z');
                            state_accpet(chr)
                        }
                    }
                }
                "zh" => {
                    match chr {
                        'm' => { state_set("zhm"); }
                        _ => {
                            buff_flush();
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::ZoneHour 
                                    }
                                )
                            );

                            state_set("state");
                            buff_push('z');
                            state_accpet(chr)
                        }
                    }
                }
                "zhm" => {
                    match chr {
                        's' => { 
                            buff_flush();
                            state_set("state"); 

                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::ZoneHourMinSec 
                                    }
                                )
                            );
                        }
                        _ => {
                            buff_flush();
                            res.borrow_mut().push(
                                DateFormatItem::Value(
                                    DateValue { 
                                        zone: tz, 
                                        item: DateTimeItem::ZoneHourMin 
                                    }
                                )
                            );

                            state_set("state");
                            buff_push('z');
                            state_accpet(chr)
                        }
                    }
                }
                "u" => {
                    match chr {
                        't' => state_set("ut"),
                        _ => {
                            state_set("state");
                            buff_push('u');
                            state_accpet(chr)
                        }
                    }
                }
                "ut" => {
                    match chr {
                        'c' => state_set("utc"),
                        _ => {
                            state_set("state");
                            buff_push('u');
                            buff_push('t');
                            state_accpet(chr)
                        }
                    }
                }
                "utc" => {
                    match chr {
                        ':' =>{ 
                            state_set("state");
                            tz = DateTimeZone::Utc;
                        }
                        _ => {
                            state_set("state");
                            buff_push('u');
                            buff_push('t');
                            buff_push('c');
                            state_accpet(chr)
                        }
                    }
                }
                "l" => {
                    match chr {
                        'o' => state_set("lo"),
                        _ => {
                            state_set("state");
                            buff_push('l');
                            state_accpet(chr)
                        }
                    }
                }
                "lo" => {
                    match chr {
                        'c' => state_set("loc"),
                        _ => {
                            state_set("state");
                            buff_push('l');
                            buff_push('o');
                            state_accpet(chr)
                        }
                    }
                }
                "loc" => {
                    match chr {
                        'a' => state_set("loca"),
                        _ => {
                            state_set("state");
                            buff_push('l');
                            buff_push('o');
                            buff_push('c');
                            state_accpet(chr)
                        }
                    }
                }
                "loca" => {
                    match chr {
                        'l' => state_set("local"),
                        _ => {
                            state_set("state");
                            buff_push('l');
                            buff_push('o');
                            buff_push('c');
                            buff_push('a');
                            state_accpet(chr)
                        }
                    }
                }
                "local" => {
                    match chr {
                        ':' => {
                            state_set("state");
                            tz = DateTimeZone::Local;
                        }
                        _ => {
                            state_set("state");
                            buff_push('l');
                            buff_push('o');
                            buff_push('c');
                            buff_push('a');
                            buff_push('l');
                            state_accpet(chr)
                        }
                    }
                }
                "o" => {
                    match chr {
                        'f' => { state_set("of") },
                        _ => { 
                            buff_push('o');
                            state_accpet(chr)
                        }
                    }
                }
                "of" => {
                    match chr {
                        'f' => { state_set("off") },
                        _ => { 
                            buff_push('o');
                            buff_push('f');
                            state_accpet(chr)
                        }
                    }
                }
                "off" => {
                    match chr {
                        's' => { state_set("offs") },
                        _ => { 
                            buff_push('o');
                            buff_push('f');
                            buff_push('f');
                            state_accpet(chr)
                        }
                    }
                }
                "offs" => {
                    match chr {
                        'e' => { state_set("offse") },
                        _ => { 
                            buff_push('o');
                            buff_push('f');
                            buff_push('f');
                            buff_push('s');
                            state_accpet(chr)
                        }
                    }
                }
                "offse" => {
                    match chr {
                        't' => { state_set("offset") },
                        _ => { 
                            buff_push('o');
                            buff_push('f');
                            buff_push('f');
                            buff_push('s');
                            buff_push('e');
                            state_accpet(chr)
                        }
                    }
                }
                "offset" => {
                    match chr {
                        '+' => { 
                            state_set("offset_d1");
                            offset_sign = 1;
                        },
                        '-' => { 
                            state_set("offset_d1");
                            offset_sign = -1;
                        },
                        _ => { 
                            buff_push('o');
                            buff_push('f');
                            buff_push('f');
                            buff_push('s');
                            buff_push('e');
                            buff_push('t');
                            state_accpet(chr)
                        }
                    }
                }
                "offset_d1" => {
                    match chr {
                        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                            state_set("offset_d2");
                            offset_digits.push(chr);
                        }
                        _ => {
                            state_set("state");
                            buff_push('o');buff_push('f');buff_push('f');buff_push('s');buff_push('e');buff_push('t');
                            match offset_sign {
                                1  => { buff_push('+') }
                                -1 => { buff_push('-') }
                                _ => {}
                            }
                            state_accpet(chr)
                        }
                    }
                }
                "offset_d2" => {
                    match chr {
                        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                            state_set("offset_d3");
                            offset_digits.push(chr)
                        }
                        _ => {
                            state_set("state");
                            buff_push('o');buff_push('f');buff_push('f');buff_push('s');buff_push('e');buff_push('t');
                            match offset_sign {
                                1  => { buff_push('+') }
                                -1 => { buff_push('-') }
                                _ => {}
                            }
                            for x in &offset_digits { buff_push(*x) }
                            state_accpet(chr)
                        }
                    }
                }
                "offset_d3" => {
                    match chr {
                        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                            state_set("offset_d4");
                            offset_digits.push(chr)
                        }
                        _ => {
                            state_set("state");
                            buff_push('o');buff_push('f');buff_push('f');buff_push('s');buff_push('e');buff_push('t');
                            match offset_sign {
                                1  => { buff_push('+') }
                                -1 => { buff_push('-') }
                                _ => {}
                            }
                            for x in &offset_digits { buff_push(*x) }
                            state_accpet(chr)
                        }
                    }
                }
                "offset_d4" => {
                    match chr {
                        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                            state_set("offset_end");
                            offset_digits.push(chr)
                        }
                        _ => {
                            state_set("state");
                            buff_push('o');buff_push('f');buff_push('f');buff_push('s');buff_push('e');buff_push('t');
                            match offset_sign {
                                1  => { buff_push('+') }
                                -1 => { buff_push('-') }
                                _ => {}
                            }
                            for x in &offset_digits { buff_push(*x) }
                            state_accpet(chr)
                        }
                    }
                }
                "offset_end" => {
                    match chr {
                        ':' => {
                            state_set("state");
                            if offset_digits.len() == 4 {
                                fn digit(c:char) -> u8 {
                                    match c {
                                        '0' => 0u8, '1' => 1u8, '2' => 2u8, '3' => 3u8, '4' => 4u8,
                                        '5' => 5u8, '6' => 6u8, '7' => 7u8, '8' => 8u8, '9' => 9u8,
                                        _ => 0u8
                                    }
                                }
                                let hour = digit(offset_digits[0]) * 10u8 + digit(offset_digits[1]);
                                let minute = digit(offset_digits[2]) * 10u8 + digit(offset_digits[3]);
                                tz = DateTimeZone::Offset { sign: offset_sign, hours: hour, minutes: minute };
                            }
                        }
                        _ => {
                            state_set("state");
                            buff_push('o');buff_push('f');buff_push('f');buff_push('s');buff_push('e');buff_push('t');
                            match offset_sign {
                                1  => { buff_push('+') }
                                -1 => { buff_push('-') }
                                _ => {}
                            }
                            for x in &offset_digits { buff_push(*x) }
                            state_accpet(chr)
                        }
                    }
                }
                _ => {}
            }
        };

        buff_flush();

        match state_get() {
            "yy" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Year2digit 
                        }
                    )
                )
            }
            "yyyy" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Year 
                        }
                    )
                )
            }
            "mm" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Month 
                        }
                    )
                )
            }
            "mmm" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::MonthNameShort
                        }
                    )
                )
            }
            "mmmm" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::MonthNameFull
                        }
                    )
                )
            }
            "dd" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Date
                        }
                    )
                )
            }
            "wd" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::WeekDayShort
                        }
                    )
                )
            }
            "wd0" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::WeekDayZero
                        }
                    )
                )
            }
            "wd1" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::WeekDayOne
                        }
                    )
                )
            }
            "ww" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Week
                        }
                    )
                )
            }
            "wwd" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::WeekDayFull
                        }
                    )
                )
            }
            "w1" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::WeekMondayFirst
                        }
                    )
                )
            }
            "hh" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Hour
                        }
                    )
                )
            }
            "hp" | "ha" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Hour12
                        }
                    )
                )
            }
            "am" | "pm" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::AMPMLoCase
                        }
                    )
                )
            }
            "AM" | "PM" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::AMPMHiCase
                        }
                    )
                )
            }
            "mi" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Minute
                        }
                    )
                )
            }
            "ss" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Second
                        }
                    )
                )
            }
            "s3" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Millisec
                        }
                    )
                )
            }
            "s6" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Microsec
                        }
                    )
                )
            }
            "s9" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Nanosec
                        }
                    )
                )
            }
            "ms" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Millisec
                        }
                    )
                )
            }
            "ns" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Nanosec
                        }
                    )
                )
            }
            "z4" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::Zone4
                        }
                    )
                )
            }
            "zh" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::ZoneHour
                        }
                    )
                )
            }
            "zhm" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::ZoneHourMin
                        }
                    )
                )
            }
            "zhms" => {
                res.borrow_mut().push(
                    DateFormatItem::Value(
                        DateValue { 
                            zone: tz, 
                            item: DateTimeItem::ZoneHourMinSec
                        }
                    )
                )
            }
            _ => {}
        }

        Some((DateFormat {format: (res.get_mut()).clone()}, cc))
    }
}

#[test]
fn parse_test() {
    use crate::*;
    use chrono::{Local};
    
    let (df,_) = DateFormatParser::default().parse("utc:yyyy:mm").unwrap();
    println!("{df:?}");
    assert!( df == DateFormat { format: vec![
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Utc, item: DateTimeItem::Year }),
        DateFormatItem::PlainText(format!(":")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Utc, item: DateTimeItem::Month }),
    ]} );

    let (df,_) = DateFormatParser::default().parse("yyyy:mm").unwrap();
    println!("{df:?}");
    assert!( df == DateFormat { format: vec![
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Local, item: DateTimeItem::Year }),
        DateFormatItem::PlainText(format!(":")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Local, item: DateTimeItem::Month }),
    ]} );

    let (df,_) = DateFormatParser::default().parse("yyyy:mm:ddThh:mi:ss.s9zhm").unwrap();
    println!("{df:?}");
    assert!( df == DateFormat { format: vec![
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Local, item: DateTimeItem::Year }),
        DateFormatItem::PlainText(format!(":")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Local, item: DateTimeItem::Month }),
        DateFormatItem::PlainText(format!(":")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Local, item: DateTimeItem::Date }),
        DateFormatItem::PlainText(format!("T")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Local, item: DateTimeItem::Hour }),
        DateFormatItem::PlainText(format!(":")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Local, item: DateTimeItem::Minute }),
        DateFormatItem::PlainText(format!(":")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Local, item: DateTimeItem::Second }),
        DateFormatItem::PlainText(format!(".")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Local, item: DateTimeItem::Nanosec }),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Local, item: DateTimeItem::ZoneHourMin }),
    ]} );

    let dt = Local::now();
    println!("{}", dt.format( &df ) );

    let (df,_) = DateFormatParser::default().parse("y:yyy''T' a''bc 'ss").unwrap();
    println!("{df:?}");
    assert!( df == DateFormat { format: vec![
        DateFormatItem::PlainText(format!("y:yy'T a'bc ")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Local, item: DateTimeItem::Second }),
    ]} );

}

fn default_date_format() -> DateFormat {
    DateFormat {  format: vec![
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Utc, item: DateTimeItem::Year }),
        DateFormatItem::PlainText(format!(":")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Utc, item: DateTimeItem::Month }),
        DateFormatItem::PlainText(format!(":")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Utc, item: DateTimeItem::Date }),
        DateFormatItem::PlainText(format!("T")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Utc, item: DateTimeItem::Hour }),
        DateFormatItem::PlainText(format!(":")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Utc, item: DateTimeItem::Minute }),
        DateFormatItem::PlainText(format!(":")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Utc, item: DateTimeItem::Second }),
        DateFormatItem::PlainText(format!(".")),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Utc, item: DateTimeItem::Nanosec }),
        DateFormatItem::Value(DateValue { zone: DateTimeZone::Utc, item: DateTimeItem::ZoneHourMin }),
    ]}
}

impl DateFormat {
    pub(crate) fn chrono_fmt( &self ) -> String {
        let mut fmt = String::new();
        for itm in &self.format {
            match itm {
                DateFormatItem::PlainText(str) => {
                    fmt.push_str(str)
                }
                DateFormatItem::Value(dv) => {
                    fmt.push_str(dv.item.chrono_fmt())
                }
            }
        }
        fmt
    }

    pub(crate) fn time_zone( &self ) -> Option<DateTimeZone> {
        (&self.format).into_iter().filter_map(|s| match s {
            DateFormatItem::Value(dv) => Some(dv.zone),
            DateFormatItem::PlainText(_) => None
        }).next()
    }

    /// Парсинг даты времени см [DateFormatParser]
    pub fn parse(str: &str) -> DateFormat {
        DateFormatParser::default().parse(str).map(|(df,_)| df).unwrap_or(default_date_format())
    }
}

#[test]
fn test_conv() {
    use chrono::{Utc, Local, TimeZone, DateTime, FixedOffset};
    use crate::*;

    let df = default_date_format();

    let dt_loc = Local::now();
    println!("{}", dt_loc.format( &df ) );

    let dt_utc : DateTime<Utc> = dt_loc.into();
    println!("{}", dt_utc.format( &df ) );

    let dt_off = FixedOffset::east_opt(-3 * 3600).unwrap().from_local_datetime(&dt_loc.naive_local()).unwrap();
    println!("{}", dt_off.format( &df.chrono_fmt() ));

    let dt_loc : DateTime<Local> = dt_off.into();
    println!("{}", dt_loc.format( &df ) );
}
