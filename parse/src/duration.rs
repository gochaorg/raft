use std::{time::Duration, rc::Rc};

use crate::{Parser, NumberParser, WhiteSpaceParser, KeywordsBuilder, Keywords, follow};
use lazy_static::lazy_static;

/// Парсинг временной продолжительности (секунда/минута/...)
pub struct DurationParser;

#[derive(Clone)]
enum DurationSuff {
    Days,
    Hours,
    Minutes,
    Seconds,
    MilliSeconds,
    MicroSeconds,
    NanoSeconds,
}

impl DurationSuff {
    fn from( &self, num:u64 ) -> Duration {
        match self {
            DurationSuff::Days => Duration::from_secs(num * 60 * 60 * 24),
            DurationSuff::Hours => Duration::from_secs(num * 60 * 60),
            DurationSuff::Minutes => Duration::from_secs(num * 60),
            DurationSuff::Seconds => Duration::from_secs(num),
            DurationSuff::MilliSeconds => Duration::from_millis(num),
            DurationSuff::MicroSeconds => Duration::from_micros(num),
            DurationSuff::NanoSeconds => Duration::from_nanos(num),
        }
    }
}

lazy_static! {
    static ref SUFF_PARSER : Keywords<DurationSuff> = {
        KeywordsBuilder
            ::new("seconds", &DurationSuff::Seconds)
            .add("second", &DurationSuff::Seconds)
            .add("sec", &DurationSuff::Seconds)
            .add("milliseconds", &DurationSuff::MilliSeconds)
            .add("millisecond", &DurationSuff::MilliSeconds)
            .add("millisec", &DurationSuff::MilliSeconds)
            .add("msec", &DurationSuff::MilliSeconds)
            .add("ms", &DurationSuff::MilliSeconds)
            .add("microseconds", &DurationSuff::MicroSeconds)
            .add("microsecond", &DurationSuff::MicroSeconds)
            .add("microsec", &DurationSuff::MicroSeconds)
            .add("mrsec", &DurationSuff::MicroSeconds)
            .add("mrs", &DurationSuff::MicroSeconds)
            .add("nanoseconds", &DurationSuff::NanoSeconds)
            .add("nanosecond", &DurationSuff::NanoSeconds)
            .add("nanosec", &DurationSuff::NanoSeconds)
            .add("ns", &DurationSuff::NanoSeconds)
            .add("minutes", &DurationSuff::Minutes)
            .add("minute", &DurationSuff::Minutes)
            .add("min", &DurationSuff::Minutes)
            .add("hours", &DurationSuff::Hours)
            .add("hour", &DurationSuff::Hours)
            .add("hs", &DurationSuff::Hours)
            .add("days", &DurationSuff::Days)
            .add("day", &DurationSuff::Days)
            .add("ds", &DurationSuff::Days)
            .build()
    };
}

impl Parser<Duration> for DurationParser {
    fn parse( &self, source: &str ) -> Option<(Duration, crate::CharsCount)> {
        let num_parser = NumberParser::parser(NumberParser{});
        let ws_parser = WhiteSpaceParser::parser(WhiteSpaceParser);
        let suff_parser: Rc<dyn Parser<DurationSuff>> = Rc::new( SUFF_PARSER.clone() );
        
        let parser1: Rc<dyn Parser<((crate::Number, crate::WhiteSpace), DurationSuff)>> = 
            follow(follow(num_parser, ws_parser), suff_parser);

        let parser2 = 
            follow(NumberParser::parser(NumberParser{}), Rc::new( SUFF_PARSER.clone() ));

        let x = 
        parser1.parse(source).and_then(|(((num, _), suff),cc)| {
            num.try_u64().map(|num| (num,suff,cc))
        }).map(|(num,suff,cc)|{
            (suff.from(num), cc)
        });

        let y =
        parser2.parse(source).and_then(|((num,suff),cc)|{
            num.try_u64().map(|num| (suff.from(num) ,cc))
        });

        x.or(y)
    }
}

#[test]
fn test1() {
    let parser = DurationParser;
    assert!( parser.parse("10 sec").unwrap().0 == Duration::from_secs(10) );
    assert!( parser.parse("10sec").unwrap().0 == Duration::from_secs(10) );
    assert!( parser.parse("10 ms").unwrap().0 == Duration::from_millis(10) );
}

impl DurationParser {
    pub fn to_string( duration:Duration ) -> String {
        let nanos = duration.as_nanos();
        if nanos % 1000 > 0 {
            return format!("{nanos} ns");
        }

        let micros = duration.as_micros();
        if micros % 1000 > 0 {
            return format!("{micros} microsec");
        }

        let millisec = duration.as_millis();
        if millisec % 1000 > 0 {
            return format!("{millisec} ms");
        }

        let seconds = duration.as_secs();
        if seconds % 60 > 0 {
            return format!("{seconds} sec");
        }

        let minutes = seconds / 60;
        if minutes % 60 > 0 {
            return format!("{seconds} minutes");
        }

        let hours = minutes / 60;
        if hours % 60 > 0 {
            return format!("{hours} hours");
        }

        let days = hours / 24;
        format!("{days} days")
    }
}