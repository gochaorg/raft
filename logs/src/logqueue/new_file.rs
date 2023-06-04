use std::{rc::Rc};

#[allow(dead_code)]
#[derive(Clone)]
struct PathTemplate<'a> {
    generators: Vec<Rc<dyn PathValue + 'a>>
}

trait PathValue {
    fn generate( &mut self ) -> String;
}
