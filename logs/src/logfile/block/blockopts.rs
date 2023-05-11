use std::collections::HashMap;

use crate::bbuff::streambuff::{ByteWriter, ByteBuff, ByteReader, ByteArrayRead};

/// Опции блока
#[derive(Clone, Debug)]
pub struct BlockOptions {
  pub values: Box<HashMap<String,String>>
}

impl Default for BlockOptions {
  fn default() -> Self {
    BlockOptions {
      values: Box::new(HashMap::<String,String>::new())
    }
  }
}

impl ByteWriter<String> for ByteBuff {
  fn write( &mut self, v:String ) {
    self.write(v.as_bytes().len());
    self.write(v.as_bytes());
  }
}

impl ByteReader<String> for ByteBuff {
  fn read( &mut self, target:&mut String ) -> Result<(),String> {
    let mut size  : usize = 0;
    self.read(&mut size)?;

    if size > u32::MAX as usize {
      return Err(format!("can't reasd string more than u32::MAX"))
    }

    let mut bytes = ByteArrayRead { data: Box::new(Vec::<u8>::new()), expect_size: size as u32 };
    self.read(&mut bytes)?;

    let string = String::from_utf8_lossy(&bytes.data);
    target.clear();
    target.push_str(&string);

    Ok(())
  }
}

impl ByteWriter<&BlockOptions> for ByteBuff {
  fn write( &mut self, options:&BlockOptions ) {
    self.write(options.values.len());
    for (key,value) in options.values.clone().into_iter() {
      self.write(key);
      self.write(value);
    }
  }
}

/// Минимальное кол-во байт для блока опций
pub const BLOCK_OPTION_MIN_SIZE : u32 = 8;

impl ByteReader<BlockOptions> for ByteBuff {
  fn read( &mut self, target:&mut BlockOptions ) -> Result<(),String> {
    let mut count : usize = 0;
    self.read(&mut count)?;

    for _i in 0..count {
      let mut key = String::new();
      let mut value = String::new();
      self.read(&mut key)?;
      self.read(&mut value)?;
      target.values.insert(key, value);
    }

    Ok(())
  }
}
