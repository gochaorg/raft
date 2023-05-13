use std::fmt;

use crate::bbuff::streambuff::{ByteBuff, ByteReader, ByteWriter};

/// Идентификатор блока в логе
///
/// Предполагается что идентификаторы растут линейно, равномерно с 0 до u32::MAX в пределах одного лог файла
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockId(u32);

impl ByteWriter<BlockId> for ByteBuff {
    fn write(&mut self, v: BlockId) {
        self.write(v.0)
    }
}

impl ByteReader<BlockId> for ByteBuff {
    fn read(&mut self, target: &mut BlockId) -> Result<(), String> {
        self.read(&mut target.0)
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockId({})", self.0)
    }
}

impl BlockId {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(self) -> u32 {
        self.0
    }
}
