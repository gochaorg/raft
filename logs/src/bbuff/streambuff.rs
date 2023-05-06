/// Потоковая работа с байтовым буфером

/// Запись байтового представления в текщую позиции и перемещение позиции
pub trait ByteWriter<V:Sized> {
  fn write( &mut self, v:V );
}

/// Буфер записи
pub struct ByteBuff {
  /// Буфер
  pub buff: Box<Vec<u8>>,

  /// Позиция записи
  pub position: usize
}

#[allow(dead_code)]
impl ByteBuff {
  /// Создание пустого буфера
  pub fn new() -> Self {
    ByteBuff { buff: Box::new(Vec::<u8>::new()), position: 0 }
  }

  /// Создание буфера из вектора
  pub fn from( buff: Box<Vec<u8>> ) -> Self {
    ByteBuff { buff: Box::clone(&buff), position: 0 }
  }

  /// Запись массива байт в текущую позицию и смещение позиции
  pub fn write_byte_arr( &mut self, data: &[u8] ) {
    let available = self.buff.len() - self.position;
    // extends if need
    if data.len() > available {
      let add = data.len() - available;
      self.buff.resize(self.buff.len() + add, 0);
    }
    for i in 0..data.len() {
      self.buff[i + self.position] = data[i];
    }
    self.position += data.len();
  }
}

// Запись массива
impl ByteWriter<&[u8]> for ByteBuff {
  fn write( &mut self, v:&[u8] ) {
    self.write_byte_arr(v);
  }
}

// Запись байта
impl ByteWriter<u8> for ByteBuff {
  fn write( &mut self, v:u8 ) {
    let data = v.to_le_bytes();
    self.write_byte_arr(&data);
  }
}

// Запись 2 байтов
impl ByteWriter<u16> for ByteBuff {
  fn write( &mut self, v:u16 ) {
    let data = v.to_le_bytes();
    self.write_byte_arr(&data);
  }
}

// Запись 4 байтов
impl ByteWriter<u32> for ByteBuff {
  fn write( &mut self, v:u32 ) {
    let data = v.to_le_bytes();
    self.write_byte_arr(&data);
  }
}

// Запись 8 байтов
impl ByteWriter<u64> for ByteBuff {
  fn write( &mut self, v:u64 ) {
    let data = v.to_le_bytes();
    self.write_byte_arr(&data);
  }
}

/// Чтение данных из байтового массива
pub trait ByteReader<V> {
  fn read( &mut self, target:&mut V ) -> Result<(),String>;
}

impl ByteReader<u8> for ByteBuff {
  fn read( &mut self, target:&mut u8 ) -> Result<(),String> {
    let available = self.buff.len() - self.position;
    if available < 1 {
      Err("no data".to_string())
    } else {
      *target = self.buff[self.position];
      self.position += 1;
      Ok(())
    }
  }
}

impl ByteReader<u16> for ByteBuff {
  fn read( &mut self, target:&mut u16 ) -> Result<(),String> {
    let available = self.buff.len() - self.position;
    if available < 2 {
      Err("no data".to_string())
    } else {
      let bb:[u8;2] = [
        self.buff[self.position],
        self.buff[self.position+1]
      ];
      (*target) = u16::from_le_bytes(bb);
      self.position += 2;
      Ok(())
    }
  }
}

impl ByteReader<u32> for ByteBuff {
  fn read( &mut self, target:&mut u32 ) -> Result<(),String> {
    let available = self.buff.len() - self.position;
    if available < 4 {
      Err("no data".to_string())
    } else {
      let bb:[u8;4] = [
        self.buff[self.position],
        self.buff[self.position+1],
        self.buff[self.position+2],
        self.buff[self.position+3],
      ];
      (*target) = u32::from_le_bytes(bb);
      self.position += 4;
      Ok(())
    }
  }
}

impl ByteReader<u64> for ByteBuff {
  fn read( &mut self, target:&mut u64 ) -> Result<(),String> {
    let available = self.buff.len() - self.position;
    if available < 8 {
      Err("no data".to_string())
    } else {
      let bb:[u8;8] = [
        self.buff[self.position],
        self.buff[self.position+1],
        self.buff[self.position+2],
        self.buff[self.position+3],
        self.buff[self.position+4],
        self.buff[self.position+5],
        self.buff[self.position+6],
        self.buff[self.position+7],
      ];
      (*target) = u64::from_le_bytes(bb);
      self.position += 8;
      Ok(())
    }
  }
}

/// Чтение массива из текущей позиции и смещение позиции
pub struct ByteArrayRead{ data:Box<Vec<u8>>, expect_size:u32 }

impl ByteReader<ByteArrayRead> for ByteBuff {
  fn read( &mut self, target:&mut ByteArrayRead ) -> Result<(),String> {
    let available = (self.buff.len() as i64) - (self.position as i64);
    if available < (target.expect_size as i64) {
      Err("no data".to_string())
    } else {
      let mut bw = ByteBuff::from(Box::clone(&target.data));
      for _ in 0..target.expect_size {
        let mut b:u8 = 0;
        self.read(&mut b)?;
        bw.write(b);
      }
      self.position += target.expect_size as usize;
      Ok(())
    }
  }
}

#[test]
fn read_write_test() -> Result<(),String> {
  let mut bb = ByteBuff::new();
  bb.write(1u8);
  bb.write(2u16);
  bb.write(3u32);
  bb.write(4u64);

  let mut a:u8 = 0;
  let mut b:u16 = 0;
  let mut c:u32 = 0;
  let mut d:u64 = 0;

  bb.position = 0;

  bb.read(&mut a)?;
  assert!( a==1u8 );

  bb.read(&mut b)?;
  assert!( b==2 );

  bb.read(&mut c)?;
  assert!( c==3 );

  bb.read(&mut d)?;
  assert!( d==4 );

  Ok(())
}
