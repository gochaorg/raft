use std::marker::PhantomData;
use crate::substr::*;

enum Range<T:Sized> {
    Single(T),
    FromTo(T,T),
    Multiple(Vec<Range<T>>)
}

trait Next<T:Sized> {
    fn next(v:T) -> Option<T>;
}

