use chrono::{Utc, Local, TimeZone, DateTime, FixedOffset, Duration};

use crate::{DateFormat, DateTimeZone};

pub trait Format<F> {
    fn format( self, f:F ) -> String;
}

impl Format<DateFormat> for DateTime<Utc> {
    fn format( self, fmt: DateFormat ) -> String {
        match fmt.time_zone() {
            Some(tz) => {
                match tz {
                    DateTimeZone::Local => {
                        let dt_loc : DateTime<Local> = self.into();
                        DateTime::<Local>::format(&dt_loc, &fmt.chrono_fmt() ).to_string()
                    }
                    DateTimeZone::Utc => {
                        DateTime::<Utc>::format(&self, &fmt.chrono_fmt() ).to_string()
                    }
                    DateTimeZone::Offset { sign, hours, minutes } => {
                        let dt_loc : DateTime<Local> = self.into();
                        let off_sec = (sign as i32)* ( (hours as i32)*3600 + (minutes as i32)*60 );
                        match FixedOffset::east_opt(off_sec).and_then( |off| 
                            match off.from_local_datetime(&dt_loc.naive_utc()) {
                                chrono::LocalResult::Single(dt) => Some(dt),
                                _ => None
                            }
                        ) {
                            Some( dt ) => {
                                let dt = dt.checked_add_signed(Duration::seconds(off_sec as i64));
                                match dt {
                                    Some(dt) => {
                                        DateTime::<FixedOffset>::format(&dt, &fmt.chrono_fmt()).to_string()
                                    }
                                    None => {
                                        panic!("can't convert to DateTime<FixedOffset> 2!")
                                    }
                                }
                            }
                            None => panic!("can't convert to DateTime<FixedOffset>")
                        }
                    }
                }
            }
            None => {
                DateTime::<Utc>::format(&self, &fmt.chrono_fmt() ).to_string()
            }
        }
    }
}
impl Format<&DateFormat> for DateTime<Utc> {
    fn format( self, fmt: &DateFormat ) -> String {
        match fmt.time_zone() {
            Some(tz) => {
                match tz {
                    DateTimeZone::Local => {
                        let dt_loc : DateTime<Local> = self.into();
                        DateTime::<Local>::format(&dt_loc, &fmt.chrono_fmt() ).to_string()
                    }
                    DateTimeZone::Utc => {
                        DateTime::<Utc>::format(&self, &fmt.chrono_fmt() ).to_string()
                    }
                    DateTimeZone::Offset { sign, hours, minutes } => {
                        let dt_loc : DateTime<Local> = self.into();
                        let off_sec = (sign as i32)* ( (hours as i32)*3600 + (minutes as i32)*60 );
                        match FixedOffset::east_opt(off_sec).and_then( |off| 
                            match off.from_local_datetime(&dt_loc.naive_utc()) {
                                chrono::LocalResult::Single(dt) => Some(dt),
                                _ => None
                            }
                        ) {
                            Some( dt ) => {
                                let dt = dt.checked_add_signed(Duration::seconds(off_sec as i64));
                                match dt {
                                    Some(dt) => {
                                        DateTime::<FixedOffset>::format(&dt, &fmt.chrono_fmt()).to_string()
                                    }
                                    None => {
                                        panic!("can't convert to DateTime<FixedOffset> 2!")
                                    }
                                }
                            }
                            None => panic!("can't convert to DateTime<FixedOffset>")
                        }
                    }
                }
            }
            None => {
                DateTime::<Utc>::format(&self, &fmt.chrono_fmt() ).to_string()
            }
        }
    }
}
impl Format<&DateFormat> for &DateTime<Utc> {
    fn format( self, fmt: &DateFormat ) -> String {
        match fmt.time_zone() {
            Some(tz) => {
                match tz {
                    DateTimeZone::Local => {
                        let dt_loc : DateTime<Local> = self.clone().into();
                        DateTime::<Local>::format(&dt_loc, &fmt.chrono_fmt() ).to_string()
                    }
                    DateTimeZone::Utc => {
                        DateTime::<Utc>::format(&self, &fmt.chrono_fmt() ).to_string()
                    }
                    DateTimeZone::Offset { sign, hours, minutes } => {
                        let dt_loc : DateTime<Local> = self.clone().into();
                        let off_sec = (sign as i32)* ( (hours as i32)*3600 + (minutes as i32)*60 );
                        match FixedOffset::east_opt(off_sec).and_then( |off| 
                            match off.from_local_datetime(&dt_loc.naive_utc()) {
                                chrono::LocalResult::Single(dt) => Some(dt),
                                _ => None
                            }
                        ) {
                            Some( dt ) => {
                                let dt = dt.checked_add_signed(Duration::seconds(off_sec as i64));
                                match dt {
                                    Some(dt) => {
                                        DateTime::<FixedOffset>::format(&dt, &fmt.chrono_fmt()).to_string()
                                    }
                                    None => {
                                        panic!("can't convert to DateTime<FixedOffset> 2!")
                                    }
                                }
                            }
                            None => panic!("can't convert to DateTime<FixedOffset>")
                        }
                    }
                }
            }
            None => {
                DateTime::<Utc>::format(&self, &fmt.chrono_fmt() ).to_string()
            }
        }
    }
}

impl Format<DateFormat> for DateTime<Local> {
    fn format( self, fmt:DateFormat ) -> String {
        match fmt.time_zone() {
            Some(tz) => {
                match tz {
                    DateTimeZone::Local => {
                        let dt_loc : DateTime<Local> = self.into();
                        DateTime::<Local>::format(&dt_loc, &fmt.chrono_fmt() ).to_string()
                    }
                    DateTimeZone::Utc => {
                        let dt_utc : DateTime<Utc> = self.into();
                        DateTime::<Utc>::format(&dt_utc, &fmt.chrono_fmt() ).to_string()
                    }
                    DateTimeZone::Offset { sign, hours, minutes } => {
                        let dt_loc : DateTime<Local> = self.into();
                        let off_sec = (sign as i32)* ( (hours as i32)*3600 + (minutes as i32)*60 );
                        match FixedOffset::east_opt(off_sec).and_then( |off| 
                            match off.from_local_datetime(&dt_loc.naive_utc()) {
                                chrono::LocalResult::Single(dt) => Some(dt),
                                _ => None
                            }
                        ) {
                            Some( dt ) => {
                                let dt = dt.checked_add_signed(Duration::seconds(off_sec as i64));
                                match dt {
                                    Some(dt) => {
                                        DateTime::<FixedOffset>::format(&dt, &fmt.chrono_fmt()).to_string()
                                    }
                                    None => {
                                        panic!("can't convert to DateTime<FixedOffset> 2!")
                                    }
                                }
                            }
                            None => panic!("can't convert to DateTime<FixedOffset>")
                        }
                    }
                }
            }
            None => {
                DateTime::<Local>::format(&self, &fmt.chrono_fmt() ).to_string()
            }
        }
    }
}
impl Format<&DateFormat> for DateTime<Local> {
    fn format( self, fmt:&DateFormat ) -> String {
        match fmt.time_zone() {
            Some(tz) => {
                match tz {
                    DateTimeZone::Local => {
                        let dt_loc : DateTime<Local> = self.into();
                        DateTime::<Local>::format(&dt_loc, &fmt.chrono_fmt() ).to_string()
                    }
                    DateTimeZone::Utc => {
                        let dt_utc : DateTime<Utc> = self.into();
                        DateTime::<Utc>::format(&dt_utc, &fmt.chrono_fmt() ).to_string()
                    }
                    DateTimeZone::Offset { sign, hours, minutes } => {
                        let dt_loc : DateTime<Local> = self.into();
                        let off_sec = (sign as i32)* ( (hours as i32)*3600 + (minutes as i32)*60 );
                        match FixedOffset::east_opt(off_sec).and_then( |off| 
                            match off.from_local_datetime(&dt_loc.naive_utc()) {
                                chrono::LocalResult::Single(dt) => Some(dt),
                                _ => None
                            }
                        ) {
                            Some( dt ) => {
                                let dt = dt.checked_add_signed(Duration::seconds(off_sec as i64));
                                match dt {
                                    Some(dt) => {
                                        DateTime::<FixedOffset>::format(&dt, &fmt.chrono_fmt()).to_string()
                                    }
                                    None => {
                                        panic!("can't convert to DateTime<FixedOffset> 2!")
                                    }
                                }
                            }
                            None => panic!("can't convert to DateTime<FixedOffset>")
                        }
                    }
                }
            }
            None => {
                DateTime::<Local>::format(&self, &fmt.chrono_fmt() ).to_string()
            }
        }
    }
}
impl Format<&DateFormat> for &DateTime<Local> {
    fn format( self, fmt:&DateFormat ) -> String {
        match fmt.time_zone() {
            Some(tz) => {
                match tz {
                    DateTimeZone::Local => {
                        let dt_loc : DateTime<Local> = self.clone().into();
                        DateTime::<Local>::format(&dt_loc, &fmt.chrono_fmt() ).to_string()
                    }
                    DateTimeZone::Utc => {
                        let dt_utc : DateTime<Utc> = self.clone().into();
                        DateTime::<Utc>::format(&dt_utc, &fmt.chrono_fmt() ).to_string()
                    }
                    DateTimeZone::Offset { sign, hours, minutes } => {
                        let dt_loc : DateTime<Local> = self.clone().into();
                        let off_sec = (sign as i32)* ( (hours as i32)*3600 + (minutes as i32)*60 );
                        match FixedOffset::east_opt(off_sec).and_then( |off| 
                            match off.from_local_datetime(&dt_loc.naive_utc()) {
                                chrono::LocalResult::Single(dt) => Some(dt),
                                _ => None
                            }
                        ) {
                            Some( dt ) => {
                                let dt = dt.checked_add_signed(Duration::seconds(off_sec as i64));
                                match dt {
                                    Some(dt) => {
                                        DateTime::<FixedOffset>::format(&dt, &fmt.chrono_fmt()).to_string()
                                    }
                                    None => {
                                        panic!("can't convert to DateTime<FixedOffset> 2!")
                                    }
                                }
                            }
                            None => panic!("can't convert to DateTime<FixedOffset>")
                        }
                    }
                }
            }
            None => {
                DateTime::<Local>::format(&self, &fmt.chrono_fmt() ).to_string()
            }
        }
    }
}

#[test]
fn format_with_tz_test1() {
    let dt = Local::now();
    let df = DateFormat::parse("offset+0300:yyyy-mm-ddThh:mi:ss.s3zhm");
    println!("{}", dt.format(df));

    let df = DateFormat::parse("local:yyyy-mm-ddThh:mi:ss.s3zhm");
    println!("{}", dt.format(df));

    let df = DateFormat::parse("utc:yyyy-mm-ddThh:mi:ss.s3zhm");
    println!("{}", dt.format(df));
}

#[test]
fn format_with_tz_test2() {
    let dt = Utc::now();
    let df = DateFormat::parse("offset+0300:yyyy-mm-ddThh:mi:ss.s3zhm");
    println!("{}", dt.format(df));

    let df = DateFormat::parse("local:yyyy-mm-ddThh:mi:ss.s3zhm");
    println!("{}", dt.format(df));

    let df = DateFormat::parse("utc:yyyy-mm-ddThh:mi:ss.s3zhm");
    println!("{}", dt.format(df));
}