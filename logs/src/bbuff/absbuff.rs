use std::{
  sync::{Arc, RwLock, PoisonError, RwLockWriteGuard, RwLockReadGuard}, 
  fs::File,
  fs::OpenOptions,
  io::{
    SeekFrom,
    prelude::*
  }, fmt, path::Path,
};

use crate::perf::Tracker;

/// Ошибка чтения/записи
#[derive(Debug,Clone)]
pub enum ABuffError{
  Generic(String),
  IO {
    message: String,
    os_error: Option<i32>
  },
  Limit {
    message: String,
    limit: u64,
    target: u64,
  },
  PartialWrited {
    message: String,
    actual: u64,
    expect: u64,
  }
}

#[derive(Debug,Clone, Copy)]
pub struct Limit(u64);
impl Limit {
  fn check<V:Into<u64>>(self, value:V, operation:&str) -> Result<(), ABuffError> 
  {
    let v64:u64 = value.into();
    if v64 > self.0 {
      Err(ABuffError::limit(operation, self.clone(), v64))
    }else{
      Ok(())
    }
  }
}
const LIMIT_USIZE : Limit = Limit(usize::MAX as u64);

impl fmt::Display for Limit {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl ABuffError {
  pub fn generic<A: Into<String>>( message:A ) -> ABuffError {
    ABuffError::Generic(
      message.into()
    )
  }
  pub fn limit( operation_name: &str, limit:Limit, target:u64 ) -> ABuffError {
    ABuffError::Limit { 
      message: format!(
        "can't execute {operation_name} by limit size, current limit {limit}, target size {target}"
      ), 
      limit: limit.0, 
      target: target 
    }
  }
  pub fn partial_writed( expect:u64, actual:u64 ) -> ABuffError {
    ABuffError::PartialWrited { 
      message: format!("partial writed bytes, expect {expect}, actual {actual}"), 
      actual: actual, 
      expect: expect
    }
  }
}

impl<A> From<PoisonError<RwLockWriteGuard<'_, A>>> for ABuffError {
  fn from(value: PoisonError<RwLockWriteGuard<'_, A>>) -> Self {
    Self::generic(
      format!("can't write lock field at {value}")
    )
  }
}

impl From<std::io::Error> for ABuffError {
  fn from(value: std::io::Error) -> Self {
    Self::IO {
      message: value.to_string(),
      os_error: value.raw_os_error()
    }
  }
}

impl<A> From<PoisonError<RwLockReadGuard<'_, A>>> for ABuffError {
  fn from(value: PoisonError<RwLockReadGuard<'_, A>>) -> Self {
    Self::generic(
      format!("can't read lock field at {value}")
    )
  }
}

/// Чтение байтов по абсолютной позиции
pub trait ReadBytesFrom {
  /// Чтение массива байтов
  /// 
  /// # Возвращает
  /// кол-во прочитанных байтов
  /// 
  /// может быть возврещено меньше чем запрагиваемое значение
  /// 
  /// 0 - если нет данных, например выход за пределы массива или файла
  fn read_from( &self, pos:u64, data_consumer:&mut [u8] ) -> Result<u64,ABuffError>;
}

/// Запись байтов в указанную позицию
pub trait WriteBytesTo {
  /// Запись байтов в определенную позицию
  fn write_to( &mut self, pos:u64, data_provider: &[u8] ) -> Result<(),ABuffError>;
}

/// Возвращает кол-во байтов
pub trait BytesCount {
  /// Возвращает кол-во байтов
  fn bytes_count( &self ) -> Result<u64,ABuffError>;
}

/// Изменяет размер массива
pub trait ResizeBytes {
  /// Изменяет размер массива  
  fn resize_bytes( &mut self, new_size:u64 ) -> Result<(),ABuffError>;
}

/// Байтовый массив в памяти
#[derive(Debug,Clone)]
pub struct ByteBuff{ 
  pub data: Arc<RwLock<Vec<u8>>>, 
  pub resizeable: bool,
  pub max_size: Option<usize>
}

impl ByteBuff {
  #[allow(dead_code)]
  pub fn new_empty_unlimited() -> Self {
    Self { data: Arc::new( RwLock::new(Vec::<u8>::new()) ), resizeable: true, max_size: None }
  }
}

impl ReadBytesFrom for ByteBuff {
  fn read_from( &self, pos:u64, data_consumer:&mut [u8] ) -> Result<u64, ABuffError> {
    LIMIT_USIZE.check(pos, "read_from pos argument")?;
    LIMIT_USIZE.check(pos + data_consumer.len() as u64, "read_from last byte consumer")?;

    let data = self.data.read()?;

    if pos > data.len() as u64 {
      Ok( 0 )
    } else {
      let available = data.len() as u64 - pos;
      let read_size = (data_consumer.len() as u64).min( available );
      LIMIT_USIZE.check( read_size-1+pos, "read_from last byte of source" )?;
      for i in 0..read_size {
        data_consumer[i as usize] = data[(i + pos) as usize]
      }
      Ok(read_size)
    }
  }
}

impl WriteBytesTo for ByteBuff {
  fn write_to( &mut self, pos:u64, data_provider: &[u8] ) -> Result<(),ABuffError> {
    let mut data = self.data.write()?;    
    let min_size = pos + data_provider.len() as u64;

    LIMIT_USIZE.check(min_size, "write_to min target size")?;
    
    if (data.len() as u64) < min_size {
      if self.resizeable {
        match self.max_size {
          Some(max_size) => {
            if min_size > (max_size as u64) {
              return Err(
                ABuffError::limit(
                  "write_to", 
                  Limit(max_size as u64), 
                  min_size)
              )
            } else {
              data.resize(min_size as usize, 0);
            }
          },
          None => {
            data.resize(min_size as usize, 0);
          }
        }
      }
    }

    for i in 0..data_provider.len() {
      data[(pos + i as u64) as usize] = data_provider[i];
    }

    Ok(())
  }
}

impl BytesCount for ByteBuff {
  fn bytes_count( &self ) -> Result<u64,ABuffError> {
    let data = self.data.read()?;
    Ok(data.len() as u64)
  }
}

impl ResizeBytes for ByteBuff {
  fn resize_bytes( &mut self, new_size:u64 ) -> Result<(),ABuffError> {
    let mut data = self.data.write()?;
    LIMIT_USIZE.check(new_size, "resize_bytes")?;
    data.resize(new_size as usize, 0);
    Ok(())
  }
}

/// Файловый буффер
#[derive(Debug,Clone)]
pub struct FileBuff {
  pub file: Arc<RwLock<File>>,
  pub tracker: Arc<Tracker>,
}

impl FileBuff {
  pub fn change_tracker( &mut self, new_tracker: Arc<Tracker> ) {
    self.tracker = new_tracker;
  }

  pub fn open_read_write<P: AsRef<Path>>(path: P) -> Result<Self,ABuffError> {
    let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
    Ok(Self { 
      file: Arc::new(RwLock::new(file)),
      tracker: Arc::new(Tracker::new()),
    })
  }
  
  pub fn open_read_only<P: AsRef<Path>>(path: P) -> Result<Self,ABuffError> {
    Ok(Self {
      file: Arc::new(RwLock::new(OpenOptions::new().read(true).write(false).create(false).open(path)?)),
      tracker: Arc::new(Tracker::new()),
    })
  }
}

impl WriteBytesTo for FileBuff {
  fn write_to( &mut self, pos:u64, data_provider: &[u8] ) -> Result<(),ABuffError> {
    let mut file = self.file.write()?;
    let file_len = file.metadata()?.len();

    let min_size = pos + data_provider.len() as u64;

    if file_len < min_size {
      self.tracker.track("file.set_len", || file.set_len( min_size as u64 ))?;
      //file.set_len( min_size as u64 )?
    }

    self.tracker.track( "file.seek", || file.seek(SeekFrom::Start(pos as u64)) )?;

    let writed = self.tracker.track("file.write", || file.write(data_provider) )?;
    if writed < data_provider.len() {
      let expect = data_provider.len();      
      return Err(
        ABuffError::partial_writed(expect as u64, writed as u64)
      );
    }

    self.tracker.track("file.flush", || file.flush())?;
    
    Ok(())
  }
}

impl ReadBytesFrom for FileBuff {
  fn read_from( &self, pos:u64, data_consumer:&mut [u8] ) -> Result<u64,ABuffError> {
    let mut file = self.file.write()?;

    let file_len = file.metadata()?.len();
    if pos > file_len { return Ok(0); }

    file.seek(SeekFrom::Start(pos as u64))?;

    let reads = file.read(data_consumer)?;
    Ok(reads as u64)
  }
}

impl BytesCount for FileBuff {
  fn bytes_count( &self ) -> Result<u64, ABuffError> {
    let file = self.file.read()?;
    let file_len = file.metadata()?.len();
    Ok(file_len)
  }
}

impl ResizeBytes for FileBuff {
  fn resize_bytes( &mut self, new_size:u64 ) -> Result<(),ABuffError> {
    let mut file = self.file.write()?;
    file.set_len(new_size as u64)?;
    file.flush()?;
    Ok(())
  }
}