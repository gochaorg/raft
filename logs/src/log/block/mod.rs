//! Представляет из себя блок лог файла
//! 
//! Размер блока может быть разным
//! 
//! Структура блока
//! 
//! | Поле | Тип | Описание |
//! |------|-----|----------|
//! | head | Head | Заголовок блока |
//! | data | Data | Данные блока |
//! | tail | Tail | Хвост блока - хранит маркер конца блока, который указывает на заголовок |
//! 
//! Структура заголовка
//! 
//! | Поле                 | Тип/размер   | Описание |
//! |----------------------|--------------|-----------|
//! | head                 | head         | Заголовок |
//! | head.head_size       | u32          | Размер заголовка |
//! | head.data_size       | u32          | Размер данных |
//! | head.tail_size       | u16          | Размер хвоста |
//! | head.block_id        | BlockId(u32) | Идентификатор блока |
//! | head.data_type_id    | DataId       | Тип данных |
//! | head.back_refs_count | u32          | Кол-во обратных ссылок (head.back_ref) |
//! | head.back_ref.b_id   | u32          | Идентификатор блока |
//! | head.back_ref.b_off  | u64          | Смещение блока |
//! | head.block_options   | BlockOptions | Опции блока    |
use std::{time::Instant};

use crate::{bbuff::{streambuff::{ByteBuff, ByteReader, ByteWriter}}, perf::Tracker};
use crate::bbuff::absbuff::{ ReadBytesFrom, WriteBytesTo };

mod fileoffset;
pub use fileoffset::*;

mod blockid;
pub use blockid::*;

mod dataid;
pub use dataid::*;

mod blockopts;
pub use blockopts::*;

mod err;
pub use err::*;

mod limit;
pub use limit::*;

mod tail;
pub use tail::*;

/// Блок лога
#[allow(dead_code)]
pub struct Block {
  /// Заголовок
  pub head: BlockHead,

  /// Данные
  pub data: Box<Vec<u8>>
}

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

/// минимальный размер заголовка
#[allow(dead_code)]
pub const HEAD_MIN_SIZE : u32 = 22;

#[derive(Debug,Clone, Copy)]
pub struct BlockHeadSize(u32);

impl BlockHeadSize {
    pub fn value(self) -> u32 { self.0 }
}

#[derive(Debug,Clone, Copy)]
pub struct BlockDataSize(u32);

impl BlockDataSize {
  pub fn value(self) -> u32 { self.0 }
}

#[derive(Debug,Clone, Copy)]
pub struct BlockTailSize(u16);

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
  #[allow(dead_code)]
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

fn write_block_head( head:&BlockHead, data_size:u32, tail_size:u16, tracker:&Tracker ) -> Box<Vec<u8>> {
  let mut bbuf = ByteBuff::new();
  bbuf.tracker = tracker.clone();

  let t0 = Instant::now();
  bbuf.position = 0;
  bbuf.write(0u32);
  bbuf.write(data_size);
  bbuf.write(tail_size);
  bbuf.write(head.block_id);
  bbuf.write(head.data_type_id);
  bbuf.write(head.back_refs.refs.len() as u32);
  for (b_id, b_off) in head.back_refs.refs.iter() {
    bbuf.write(*b_id);
    bbuf.write(*b_off);
  }

  bbuf.write(head.block_options);

  let size = bbuf.position;  
  bbuf.position = 0;
  bbuf.write(size as u32);

  let t99 = Instant::now();
  tracker.add("write_block_head", t99.duration_since(t0));

  bbuf.buff
}

#[test]
fn test_block() {
  let bh = BlockHead {
    block_id: BlockId::new(10),
    data_type_id: DataId::new(2),
    back_refs: BackRefs { refs: Box::new(vec![
      (BlockId::new(9), FileOffset::new(7)),
      (BlockId::new(8), FileOffset::new(20)),
    ])},
    block_options: BlockOptions::default()
  };

  let tracker = Tracker::new();
  let block_data = write_block_head(&bh, 134, TAIL_SIZE, &tracker);
  println!("{:?}",read_block_head(block_data));
}

impl BlockHead {
  /// Чтение заголовка
  #[allow(dead_code)]
  fn from_bytes( bytes: Box<Vec<u8>> ) -> Result<(BlockHead, BlockHeadSize, BlockDataSize, BlockTailSize), String> {
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

/// Размер буфера при чтении файла
const READ_BUFF_SIZE: usize = 1024*8;

/// Размер буфера при чтении заголовка, в теории заголовок не должен быть больше этого значения
const PREVIEW_SIZE:usize = 1024 * 64;

impl Block {
  /// Чтение блока из массива байт
  pub fn read_from<Source>( position: u64, file: &Source ) -> Result<(Self,u64), BlockErr> 
  where Source : ReadBytesFrom
  {
    let mut head_preview: [u8;PREVIEW_SIZE] = [0; PREVIEW_SIZE];

    let reads = file.read_from(position, &mut head_preview)?;
    if reads < (HEAD_MIN_SIZE as u64) { return Err(BlockErr::from("readed to small header")) }

    let (bh, head_size, data_size, tail_size) = BlockHead::from_bytes(Box::new(head_preview.to_vec()))?;
    let mut buff: [u8;READ_BUFF_SIZE] = [0;READ_BUFF_SIZE];
    let mut left_bytes = data_size.0 as u64;
    LIMIT_USIZE.check(left_bytes, "read_from")?;

    let mut block_data = Vec::<u8>::new();
    block_data.resize(left_bytes as usize, 0);

    let mut block_data_ptr = 0usize;
    let mut file_pos = (position) + (head_size.0 as u64);

    while left_bytes>0 {
      let readed = file.read_from( file_pos,&mut buff)?;
      if readed==0 { return Err(BlockErr::from("data block truncated")) }

      for i in 0..(readed.min(left_bytes as u64)) {
        block_data[block_data_ptr] = buff[i as usize];
        block_data_ptr += 1;
        file_pos += 1;
        left_bytes -= 1;
      }
    }

    Ok((Self{head: bh, data:Box::new(block_data)}, position + (head_size.0 as u64) + (data_size.0 as u64) + (tail_size.0 as u64)))
  }

  /// Формирование массива байтов представлющих блок
  pub fn to_bytes( &self, tracker:&Tracker ) -> (Box<Vec<u8>>, BlockHeadSize, BlockDataSize, BlockTailSize) {
    // write tail marker
    let t0 = Instant::now();
    
    let mut tail = tracker.track("to_bytes/tail marker", || { 
      let mut tail = Box::new(Vec::<u8>::new());
      for i in 0..TAIL_MARKER.len() {
        tail.push(TAIL_MARKER.as_bytes()[i]);
      }
      tail.push(0);tail.push(0);tail.push(0);tail.push(0);
      tail
    });

    // write head
    let mut bytes = 
      tracker.track("to_bytes/write_block_head", || {
        write_block_head(&self.head, self.data.len() as u32, tail.len() as u16, tracker)
      });

    let block_head_size = tracker.track("to_bytes/block_head_size",||BlockHeadSize(bytes.len() as u32));
    let block_data_size = tracker.track("to_bytes/block_data_size",||BlockDataSize(self.data.len() as u32));
    let block_tail_size = tracker.track("to_bytes/block_tail_size",||BlockTailSize(tail.len() as u16));

    let off = bytes.len();
    
    if self.data.len()>0 {      
      tracker.track("to_bytes/bytes.resize", || {
        bytes.resize(bytes.len() + self.data.len() + tail.len(), 0)
      })
    }

    // copy data
    tracker.track("to_bytes/copy data", || {
      let data_part = &mut bytes[off .. (off + self.data.len())];
      data_part.copy_from_slice(&self.data);
    });

    // update tail data
    tracker.track("to_bytes/tail update", ||{
      let total_size = bytes.len() as u32;
      let total_size = total_size.to_le_bytes();
      tail[4] = total_size[0];
      tail[5] = total_size[1];
      tail[6] = total_size[2];
      tail[7] = total_size[3];

      let blen = bytes.len();
      for i in 0..tail.len() {
        bytes[ blen - tail.len() + i ] = tail[i];
      }
    });

    let t4 = Instant::now();
    tracker.add("to_bytes", t4.duration_since(t0));

    (bytes, block_head_size, block_data_size, block_tail_size)
  }

  /// Запись блока в массив байтов
  pub fn write_to<Destination>( &self, position:u64, dest:&mut Destination, tracker: &Tracker ) -> Result<BlockHeadRead,BlockErr> 
  where Destination: WriteBytesTo
  {
    let sub_track = tracker.sub_tracker("block.to_bytes/");

    let t0 = Instant::now();
    let (bytes,head_size,data_size,tail_size) = self.to_bytes(&sub_track);

    let t1 = Instant::now();    
    dest.write_to( position, &bytes[0 .. bytes.len()])?;

    let t2 = Instant::now();
    tracker.add("write_to", t2.duration_since(t0));
    tracker.add("write_to/self.to_bytes", t1.duration_since(t0));
    tracker.add("write_to/dest.write_to", t2.duration_since(t1));

    Ok(
      BlockHeadRead { 
        position: FileOffset::from(position), 
        head: self.head.clone(), 
        head_size: head_size, 
        data_size: data_size, 
        tail_size: tail_size, 
      }
    )
  }
}

#[test]
fn test_block_rw(){
  use super::super::bbuff::absbuff::ByteBuff;
  use crate::bbuff::absbuff::BytesCount;

  let mut bb = ByteBuff::new_empty_unlimited();

  let block = Block {
    head: BlockHead { block_id: BlockId::new(0), data_type_id: DataId::new(1), back_refs: BackRefs::default(), block_options: BlockOptions::default() },
    data: Box::new( vec![1,2,3] )
  };

  let tracker = Tracker::new();

  block.write_to(0, &mut bb, &tracker).unwrap();
  println!("{block_size}", block_size=bb.bytes_count().unwrap() );

  let (rblock,_) = Block::read_from(0, &bb).unwrap();
  assert!( rblock.head.block_id == block.head.block_id );
  assert!( rblock.head.data_type_id == block.head.data_type_id );
}
