use std::fmt;

use crate::bbuff::streambuff::{ByteBuff, ByteReader, ByteWriter};

/// Смещение относительно начала файла в байтах
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct FileOffset(u64);

impl fmt::Display for FileOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileOffset({})", self.0)
    }
}

impl FileOffset {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

impl From<u64> for FileOffset {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<u32> for FileOffset {
    fn from(value: u32) -> Self {
        Self::new(value as u64)
    }
}

impl From<u16> for FileOffset {
    fn from(value: u16) -> Self {
        Self::new(value as u64)
    }
}

impl From<usize> for FileOffset {
    fn from(value: usize) -> Self {
        Self::new(value as u64)
    }
}

impl ByteReader<FileOffset> for ByteBuff {
    fn read(&mut self, target: &mut FileOffset) -> Result<(), String> {
        let mut off: u64 = 0;
        self.read(&mut off)?;

        *target = FileOffset::from(off);
        Ok(())
    }
}

impl ByteWriter<FileOffset> for ByteBuff {
    fn write(&mut self, v: FileOffset) {
        self.write(v.value() as u64)
    }
}
