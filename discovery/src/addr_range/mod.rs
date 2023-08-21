//! Диапазон ip адресов
//! 
//! Синтаксис диапазона адресов
//! 
//! ```
//! IpRange ::= Ip6Range | Ip4Range
//! Ip6Range ::= '[' ( Ip6RangeA | Ip6RangeB ) ']'
//! Ip6RangeLst ::= HexRange16u Ip6Tail
//! Ip6RangeA ::= '::' IpRangeLst 
//! Ip6RangeB ::= IpRangeLst [ Ip6RangeA ] 
//! Ip6Tail ::= { ':' HexRange16u }
//! 
//! Ip4Range ::= DecRange8u '.' DecRange8u '.' DecRange8u '.' DecRange8u 
//! HexRange16u ::= HexNonBreakRange16u { ',' HexNonBreakRange16u }
//! DecRange8u ::= DecNonBreakRange8u { ',' DecNonBreakRange8u }
//! HexNonBreakRange16u ::= HexFromTo16u | HexSingle16u
//! DecNonBreakRange8u ::= DecFromTo8u | DecSingle8u
//! HexFromTo16u ::= Hex16u '-' Hex16u
//! HexSingle16u ::= Hex16u
//! DecFromTo8u ::= Dec8u '-' Dec8u
//! DecSingle8u ::= Dec8u
//! ```


mod ip_range;
pub use ip_range::*;

mod sockaddr_range;
pub use sockaddr_range::*;

mod parse;
pub use self::parse::*;