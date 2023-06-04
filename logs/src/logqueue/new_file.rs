use core::fmt;
use std::{rc::Rc, cell::{Cell, RefCell}};
use chrono::{Utc, Local, TimeZone, DateTime};

#[allow(dead_code)]
#[derive(Clone)]
struct PathTemplate<'a> {
    generators: Vec<Rc<dyn PathValue + 'a>>
}

trait PathValue {
    fn generate( &mut self ) -> String;
}
