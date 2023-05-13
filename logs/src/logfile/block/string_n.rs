use std::fmt::Display;

use crate::bbuff::streambuff::{ByteArrayRead, ByteBuff, ByteReader, ByteWriter};

use super::BlockErr;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
/// Строка не больше 4gb данных
pub struct String32(String);

impl String32 {
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl Default for String32 {
    fn default() -> Self {
        Self(String::new())
    }
}

impl TryFrom<&String32> for String32 {
    type Error = BlockErr;
    fn try_from(value: &String32) -> Result<Self, Self::Error> {
        Ok(value.clone())
    }
}

impl TryFrom<String> for String32 {
    type Error = BlockErr;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let bytes_size = value.as_bytes().len();
        if bytes_size > u32::MAX as usize {
            Err(BlockErr::Limit {
                message: format!("can't store string more 4gb"),
                limit: u32::MAX as u64,
                target: bytes_size as u64,
            })
        } else {
            Ok(String32(value.clone()))
        }
    }
}

impl TryFrom<&str> for String32 {
    type Error = BlockErr;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes_size = value.as_bytes().len();
        if bytes_size > u32::MAX as usize {
            Err(BlockErr::Limit {
                message: format!("can't store string more 4gb"),
                limit: u32::MAX as u64,
                target: bytes_size as u64,
            })
        } else {
            Ok(String32(value.to_string()))
        }
    }
}

impl Display for String32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ByteWriter<String32> for ByteBuff {
    fn write(&mut self, v: String32) {
        self.write(v.value().as_bytes().len() as u32);
        self.write(v.value().as_bytes());
    }
}

impl ByteReader<String32> for ByteBuff {
    fn read(&mut self, target: &mut String32) -> Result<(), String> {
        let mut size: u32 = 0;
        self.read(&mut size)?;

        let mut bytes = ByteArrayRead {
            data: Box::new(Vec::<u8>::new()),
            expect_size: size as u32,
        };
        self.read(&mut bytes)?;

        let string = String::from_utf8(*bytes.data).unwrap();
        target.0.clear();
        target.0.push_str(&string);

        Ok(())
    }
}

////////////////////////

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
/// Строка не больше 64kb данных
pub struct String16(String);

impl String16 {
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl Default for String16 {
    fn default() -> Self {
        Self(String::new())
    }
}

impl TryFrom<&String16> for String16 {
    type Error = BlockErr;
    fn try_from(value: &String16) -> Result<Self, Self::Error> {
        Ok(value.clone())
    }
}

impl TryFrom<String> for String16 {
    type Error = BlockErr;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let bytes_size = value.as_bytes().len();
        if bytes_size > u16::MAX as usize {
            Err(BlockErr::Limit {
                message: format!("can't store string more 64kb"),
                limit: u16::MAX as u64,
                target: bytes_size as u64,
            })
        } else {
            Ok(String16(value.clone()))
        }
    }
}

impl TryFrom<&str> for String16 {
    type Error = BlockErr;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes_size = value.as_bytes().len();
        if bytes_size > u16::MAX as usize {
            Err(BlockErr::Limit {
                message: format!("can't store string more 64kb"),
                limit: u16::MAX as u64,
                target: bytes_size as u64,
            })
        } else {
            Ok(String16(value.to_string()))
        }
    }
}

impl Display for String16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ByteWriter<String16> for ByteBuff {
    fn write(&mut self, v: String16) {
        self.write(v.value().as_bytes().len() as u16);
        self.write(v.value().as_bytes());
    }
}

impl ByteReader<String16> for ByteBuff {
    fn read(&mut self, target: &mut String16) -> Result<(), String> {
        let mut size: u16 = 0;
        self.read(&mut size)?;

        let mut bytes = ByteArrayRead {
            data: Box::new(Vec::<u8>::new()),
            expect_size: size as u32,
        };
        self.read(&mut bytes)?;

        let string = String::from_utf8(*bytes.data).unwrap();
        target.0.clear();
        target.0.push_str(&string);

        Ok(())
    }
}

///////////////

impl ByteWriter<String> for ByteBuff {
    fn write(&mut self, v: String) {
        self.write(v.as_bytes().len());
        self.write(v.as_bytes());
    }
}

impl ByteReader<String> for ByteBuff {
    fn read(&mut self, target: &mut String) -> Result<(), String> {
        let mut size: usize = 0;
        self.read(&mut size)?;

        if size > u32::MAX as usize {
            return Err(format!("can't read string more than u32::MAX"));
        }

        let mut bytes = ByteArrayRead {
            data: Box::new(Vec::<u8>::new()),
            expect_size: size as u32,
        };
        self.read(&mut bytes)?;

        let string = String::from_utf8(*bytes.data).unwrap();
        target.clear();
        target.push_str(&string);

        Ok(())
    }
}

#[test]
fn test_store_restore() {
    let mut bbuf = ByteBuff::new();

    let str1 = String16::try_from("value").unwrap();
    bbuf.write(str1.clone());

    let mut str2: String16 = String16::default();
    bbuf.position = 0;
    bbuf.read(&mut str2).unwrap();

    assert!(str1 == str2);

    let mut bbuf = ByteBuff::new();

    let str1 = String32::try_from("value").unwrap();
    bbuf.write(str1.clone());

    let mut str2: String32 = String32::default();
    bbuf.position = 0;
    bbuf.read(&mut str2).unwrap();

    assert!(str1 == str2)
}
