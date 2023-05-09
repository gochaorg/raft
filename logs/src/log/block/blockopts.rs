use crate::bbuff::streambuff::{ByteWriter, ByteBuff, ByteReader};

/// Опции блока
#[derive(Copy, Clone, Debug)]
pub struct BlockOptions {
}

impl Default for BlockOptions {
  fn default() -> Self {
    BlockOptions {
    }
  }
}

impl ByteWriter<BlockOptions> for ByteBuff {
  fn write( &mut self, _v:BlockOptions ) {

  }
}
impl ByteReader<BlockOptions> for ByteBuff {
  fn read( &mut self, _target:&mut BlockOptions ) -> Result<(),String> {
    Ok(())
  }
}
