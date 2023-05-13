use crate::bbuff::streambuff::{ByteBuff, ByteReader, ByteWriter};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct DataId(u32);

impl DataId {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(self) -> u32 {
        self.0
    }

    pub fn user_data() -> Self {
        Self::new(1024)
    }
}

impl ByteWriter<DataId> for ByteBuff {
    fn write(&mut self, v: DataId) {
        self.write(v.0)
    }
}

impl ByteReader<DataId> for ByteBuff {
    fn read(&mut self, target: &mut DataId) -> Result<(), String> {
        self.read(&mut target.0)
    }
}
