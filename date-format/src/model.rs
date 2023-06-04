/// Элемент значения даты-времени
#[derive(Debug,Clone, Copy,PartialEq)]
pub(crate) enum DateTimeItem {
    Year,
    Year2digit,
    Month,
    MonthNameShort,
    MonthNameFull,
    Date,
    WeekDayShort,
    WeekDayFull,
    WeekDayZero,
    WeekDayOne,
    Week,
    WeekMondayFirst,
    Hour,
    Hour12,
    AMPMLoCase,
    AMPMHiCase,
    Minute,
    Second,
    Millisec,
    Microsec,
    Nanosec,
    Zone4,
    ZoneHour,
    ZoneHourMin,
    ZoneHourMinSec,
}

impl DateTimeItem {
    pub(crate) fn chrono_fmt( self ) -> &'static str {
        match self {
            DateTimeItem::Year            => "%Y",
            DateTimeItem::Year2digit      => "%y",
            DateTimeItem::Month           => "%m",
            DateTimeItem::MonthNameShort  => "%b",
            DateTimeItem::MonthNameFull   => "%B",
            DateTimeItem::Date            => "%d",
            DateTimeItem::WeekDayShort    => "%a",
            DateTimeItem::WeekDayFull     => "%A",
            DateTimeItem::WeekDayZero     => "%w",
            DateTimeItem::WeekDayOne      => "%u",
            DateTimeItem::Week            => "%U",
            DateTimeItem::WeekMondayFirst => "%W",
            DateTimeItem::Hour            => "%H",
            DateTimeItem::Hour12          => "%I",
            DateTimeItem::AMPMLoCase      => "%P",
            DateTimeItem::AMPMHiCase      => "%p",
            DateTimeItem::Minute          => "%M",
            DateTimeItem::Second          => "%S",
            DateTimeItem::Millisec        => "%3f",
            DateTimeItem::Microsec        => "%6f",
            DateTimeItem::Nanosec         => "%9f",
            DateTimeItem::Zone4           => "%z",
            DateTimeItem::ZoneHour        => "%:::z",
            DateTimeItem::ZoneHourMin     => "%:z",
            DateTimeItem::ZoneHourMinSec  => "%::z",
        }
    }
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub(crate) struct DateValue {
    pub zone: DateTimeZone,
    pub item: DateTimeItem
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum DateTimeZone {
    Local,
    Utc,
    Offset { sign:i8, hours: u8, minutes: u8 }
}

