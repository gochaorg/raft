use std::rc::Rc;
use crate::substr::*;

/// Общий интерфейс парсера
pub trait Parser<A:Sized> {
    /// Парсинг строки
    /// 
    /// # Аргументы
    /// - source - исходная строка
    /// 
    /// # Возвращает
    /// Распознаный объект и кол-во символов которых он занимамет от начала строки
    /// 
    fn parse( &self, source: &str ) -> Option<(A, CharsCount)>;
}

