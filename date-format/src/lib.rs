mod model;

pub use model::*;

mod model_parse;
pub use model_parse::*;

mod format;
pub use format::*;

#[test]
fn instant_test() {
    use chrono::{Local};
    let df = DateFormat::parse("str");
    let dt = Local::now();
    dt.format(df);
}