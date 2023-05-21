//! Парсинг диапазона значений
//! 
//! Синтаксис
//! 
//! range ::= [ ws ] multiple
//! 
//! multiple ::= singleOrFromTo [ ws ] [ ',' [ws] singleOrFromTo ]
//! 
//! singleOrFromTo ::=  fromTo | single
//! 
//! single ::= number
//! 
//! fromTo ::= number [ws] '-' [ws] number
//! 
use super::range::*;
use crate::substr::*;