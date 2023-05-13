use std::collections::HashMap;

use crate::bbuff::streambuff::{ByteBuff, ByteReader, ByteWriter};

use super::{BlockErr, String16, String32};

/// Опции блока
///
/// Представляет из себя пары ключ/значение
#[derive(Clone, Debug)]
pub struct BlockOptions {
    // Значения - пары ключ/значение
    pub values: Box<HashMap<String16, String32>>,
}

impl BlockOptions {
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn set<K: TryInto<String16, Error = BlockErr>, V: TryInto<String32, Error = BlockErr>>(
        &mut self,
        key: K,
        value: V,
    ) -> Result<(), BlockErr> {
        let key: String16 = key.try_into()?;
        let value: String32 = value.try_into()?;
        self.values.insert(key, value);
        Ok(())
    }

    pub fn get<K: TryInto<String16, Error = BlockErr>>(&self, key: K) -> Option<String32> {
        let key_result = TryInto::<String16>::try_into(key);
        match key_result {
            Ok(key) => self.values.get(&key).map(|v| v.clone()),
            Err(_) => None,
        }
    }

    pub fn keys(&self) -> Vec<String16> {
        let res: Vec<String16> = self.values.keys().into_iter().map(|v| v.clone()).collect();
        res
    }

    pub fn entries(&self) -> Vec<(String16, String32)> {
        let res: Vec<(String16, String32)> = self.values.clone().into_iter().collect();
        res
    }

    pub fn delete<K: TryInto<String16, Error = BlockErr>>(
        &mut self,
        key: K,
    ) -> Option<(String16, String32)> {
        let key_result = TryInto::<String16>::try_into(key);
        match key_result {
            Ok(key) => self.values.remove(&key).map(|v| (key, v)),
            Err(_) => None,
        }
    }

    pub fn clear(&mut self) {
        self.values.clear()
    }
}

pub struct BlockOptionsIter {
    values: BlockOptions,
    keys: Vec<String16>,
    ptr: usize,
}

impl Iterator for BlockOptionsIter {
    type Item = (String16, String32);
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.keys.len() {
            return None;
        }
        let key = self.keys[self.ptr].clone();
        self.ptr += 1;
        match self.values.values.get(&key) {
            Some(value) => Some((key, value.clone())),
            None => None,
        }
    }
}

impl IntoIterator for BlockOptions {
    type Item = (String16, String32);
    type IntoIter = BlockOptionsIter;
    fn into_iter(self) -> Self::IntoIter {
        BlockOptionsIter {
            values: self.clone(),
            keys: self.keys(),
            ptr: 0,
        }
    }
}

impl Default for BlockOptions {
    fn default() -> Self {
        BlockOptions {
            values: Box::new(HashMap::<String16, String32>::new()),
        }
    }
}

impl ByteWriter<&BlockOptions> for ByteBuff {
    fn write(&mut self, options: &BlockOptions) {
        self.write(options.values.len());
        for (key, value) in options.values.clone().into_iter() {
            self.write(key);
            self.write(value);
        }
    }
}

/// Минимальное кол-во байт для блока опций
pub const BLOCK_OPTION_MIN_SIZE: u32 = 8;

impl ByteReader<BlockOptions> for ByteBuff {
    fn read(&mut self, target: &mut BlockOptions) -> Result<(), String> {
        let mut count: usize = 0;
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
    assert!(bbuf.buff.len() > 0);

    println!("{:x?}", bbuf.buff);

    let mut block_opt2 = BlockOptions::default();
    bbuf.position = 0;
    bbuf.read(&mut block_opt2).unwrap();

    assert!(block_opt2.values.len() == block_opt.values.len());
    assert!(block_opt.values.len() > 0);

    for (key, value) in block_opt2 {
        println!("{key} = {value}")
    }
}
