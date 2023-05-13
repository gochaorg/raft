use std::fmt;

use super::BlockErr;

#[derive(Debug, Clone, Copy)]
pub struct Limit(pub u64);

impl Limit {
    pub fn check<V: Into<u64>>(self, value: V, operation: &str) -> Result<(), BlockErr> {
        let v64: u64 = value.into();
        if v64 > self.0 {
            Err(BlockErr::limit(operation, self.clone(), v64))
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
pub const LIMIT_USIZE: Limit = Limit(usize::MAX as u64);
