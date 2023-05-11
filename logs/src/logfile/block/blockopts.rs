use std::collections::HashMap;

use crate::bbuff::streambuff::{ByteWriter, ByteBuff, ByteReader, ByteArrayRead};

use super::{BlockErr, String16, String32};

/// Опции блока
#[derive(Clone, Debug)]
pub struct BlockOptions {
  pub values: Box<HashMap<String16,String32>>
}

impl BlockOptions {
    pub fn set<K: TryInto<String16, Error=BlockErr>, V:TryInto<String32, Error=BlockErr>>( &mut self, key:K, value:V ) -> Result<(), BlockErr> {
      let key: String16 = key.try_into()?;
      let value: String32 = value.try_into()?;
      self.values.insert(key, value);
      Ok(())
    }
}

impl Default for BlockOptions {
  fn default() -> Self {
    BlockOptions {
      values: Box::new(HashMap::<String16,String32>::new())
    }
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
      let mut key = String16::default();
      let mut value = String32::default();
      self.read(&mut key)?;
      self.read(&mut value)?;
      target.values.insert(key, value);
    }

    Ok(())
  }
}

#[test]
fn test_store_restore() {
  let mut bbuf = ByteBuff::new();

  let mut block_opt = BlockOptions::default();
  block_opt.set("key", "value").unwrap();

  bbuf.write(&block_opt);
  assert!( bbuf.buff.len()>0 );

  println!("{:x?}", bbuf.buff);

  let mut block_opt2 = BlockOptions::default();
  bbuf.position = 0;
  bbuf.read(&mut block_opt2).unwrap();

  assert!( block_opt2.values.len() == block_opt.values.len() );
  assert!( block_opt.values.len() > 0 );
}