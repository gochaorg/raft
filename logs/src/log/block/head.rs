use std::time::Instant;

use crate::{bbuff::{streambuff::*, absbuff::ReadBytesFrom}, perf::Tracker};

use super::{BlockId, DataId, BlockOptions, FileOffset, BlockErr, LIMIT_USIZE};

/// Размер буфера при чтении заголовка, в теории заголовок не должен быть больше этого значения
pub const PREVIEW_SIZE:usize = 1024 * 256;

/// Заголовок блока
#[derive(Clone,Debug, Default)]
pub struct BlockHead {
  /// Идентификатор блока
  pub block_id: BlockId, 

  /// Идентификатор типа данных 
  pub data_type_id: DataId,

  /// Ссылки на предшествующий блоки
  pub back_refs: BackRefs,

  /// Опции блока
  pub block_options: BlockOptions,
}

/// минимальный размер заголовка
pub const HEAD_MIN_SIZE : u32 = 22;

#[derive(Clone, Debug)]
#[cfg_attr(doc, aquamarine::aquamarine)]
/// Ссылка на предыдущий блок
///  ```mermaid
///  flowchart RL
///  a
///  b
///  c
///  d
///  e
///  f
///  g
///  h
///  h --> |расстояние 1| g --> |1| f --> |1| e --> |1| d --> |1| c --> |1| b --> |1| a
///  
///  h -.-> |расстояние 2| f 
///  f -.-> |2| d
///  g -.-> |2| e
///  e -.-> |2| c
///  c -.-> |2| a
///  d -.-> |2| b
///  
///  h -.-> |расстояние 4| d
///  g -.-> |4| c
///  f -.-> |4| b
///  d -.-> |4| a
///  
///  h -.-> |расстояние 8| a
///  ```  
pub struct BackRefs {
  pub refs: Box<Vec<(BlockId, FileOffset)>>
}
impl Default for BackRefs {
  fn default() -> Self {
    Self {
      refs: Box::new(vec![])
    }
  }
}

#[derive(Debug,Clone, Copy)]
pub struct BlockHeadSize(pub u32);

impl BlockHeadSize {
    pub fn value(self) -> u32 { self.0 }
}

#[derive(Debug,Clone, Copy)]
pub struct BlockDataSize(pub u32);

impl BlockDataSize {
  pub fn value(self) -> u32 { self.0 }
}

#[derive(Debug,Clone, Copy)]
pub struct BlockTailSize(pub u16);

impl BlockTailSize {
  pub fn value(self) -> u16 { self.0 }
}

/// Результат чтения заголовка
#[derive(Debug,Clone)]
pub struct BlockHeadRead {
  /// Смещение в файле
  pub position: FileOffset,
  // Заголовок
  pub head: BlockHead,
  /// Размер заголовка
  pub head_size: BlockHeadSize,
  /// Размер данных после заголовка
  pub data_size: BlockDataSize,
  /// Размер хвоста после данных
  pub tail_size: BlockTailSize,
}

impl BlockHeadRead {
  /// Возвращает размер всего блока
  pub fn block_size( &self ) -> u64 {
    (self.head_size.0 as u64) + (self.data_size.0 as u64) + (self.tail_size.0 as u64)
  }
}

/// Чтение заголовка
fn read_block_head( data: Box<Vec<u8>> ) -> Result<(BlockHead, BlockHeadSize, BlockDataSize, BlockTailSize), String> {  

  let mut head_size: u32 = 0;
  let mut data_size: u32 = 0;
  let mut tail_size: u16 = 0;

  let mut bh = BlockHead::default();
  let mut back_refs_count: u32 = 0;

  let mut bbuf = ByteBuff::from( data );

  bbuf.read(&mut head_size)?;
  if head_size < HEAD_MIN_SIZE {
    return Err("head to small".to_string());
  }

  bbuf.read(&mut data_size)?;
  bbuf.read(&mut tail_size)?;

  bbuf.read(&mut bh.block_id)?;
  bbuf.read(&mut bh.data_type_id)?;
  bbuf.read(&mut back_refs_count)?;

  for _ in 0..back_refs_count {
    let mut b_id = BlockId::default();
    let mut b_off = FileOffset::default();
    bbuf.read(&mut b_id)?;
    bbuf.read(&mut b_off)?;
    
    bh.back_refs.refs.push( (b_id, b_off) );
  }

  let head_opt_size = (head_size as i64) - (bbuf.position as i64);
  if head_opt_size > 0 {
    bbuf.read(&mut bh.block_options)?;
  }

  Ok((bh, BlockHeadSize(head_size), BlockDataSize(data_size), BlockTailSize(tail_size)))
}

impl BlockHead {
  pub fn write_block_head( &self, data_size:u32, tail_size:u16, tracker:&Tracker ) -> Box<Vec<u8>> {
    let mut bbuf = ByteBuff::new();
    bbuf.tracker = tracker.clone();
  
    let t0 = Instant::now();
    bbuf.position = 0;
    bbuf.write(0u32);
    bbuf.write(data_size);
    bbuf.write(tail_size);
    bbuf.write(self.block_id);
    bbuf.write(self.data_type_id);
    bbuf.write(self.back_refs.refs.len() as u32);
    for (b_id, b_off) in self.back_refs.refs.iter() {
      bbuf.write(*b_id);
      bbuf.write(*b_off);
    }
  
    bbuf.write(self.block_options);
  
    let size = bbuf.position;  
    bbuf.position = 0;
    bbuf.write(size as u32);
  
    let t99 = Instant::now();
    tracker.add("write_block_head", t99.duration_since(t0));
  
    bbuf.buff
  }
  
  /// Чтение заголовка
  #[allow(dead_code)]
  pub fn from_bytes( bytes: Box<Vec<u8>> ) -> Result<(BlockHead, BlockHeadSize, BlockDataSize, BlockTailSize), String> {
    read_block_head(bytes)
  }

  /// Чтение заголовка из указанной позиции
  pub fn read_form<S,P>( position:P, source:&S ) -> Result<BlockHeadRead, BlockErr> 
  where S: ReadBytesFrom, P: Into<FileOffset>
  {
    let mut buff: [u8; PREVIEW_SIZE] = [0; PREVIEW_SIZE];
    let position : FileOffset = position.into();
    let position : u64 = position.value();
    let reads = source.read_from(position, &mut buff)?;
    if reads < (HEAD_MIN_SIZE as u64) {
      return Err(BlockErr::no_data(reads,HEAD_MIN_SIZE as u64))
    }

    LIMIT_USIZE.check(reads, "read_form")?;

    let mut buff1 = Vec::<u8>::new();
    buff1.resize(reads as usize, 0);
    for i in 0..(reads as usize) { buff1[i] = buff[i] }
    
    let res = Self::from_bytes(Box::new(buff1))?;
    let (bh, head_size, data_size, tail_size) = res;
    Ok(BlockHeadRead { position: FileOffset::from(position), head: bh, head_size: head_size, data_size: data_size, tail_size: tail_size })
  }
}
