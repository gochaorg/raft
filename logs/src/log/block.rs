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
use std::fmt;

use crate::bbuff::{streambuff::{ByteBuff, ByteReader, ByteWriter}, absbuff::{ABuffError}};
use crate::bbuff::absbuff::{ ReadBytesFrom, WriteBytesTo };

/// Блок лога
#[allow(dead_code)]
pub struct Block {
  /// Заголовок
  pub head: BlockHead,

  /// Данные
  pub data: Box<Vec<u8>>
}

// Макрос для NewType - генерация ByteWriter, ByteReader
macro_rules! bytes_rw_new_type {
  ( $( #[$meta:meta] )*
    $vis:vis struct $struct_name:ident 
    (
      $field_type:ty
    )
  ) => {
    $(#[$meta])*
    pub struct $struct_name ( $field_type );

    impl ByteWriter<$struct_name> for ByteBuff {
      fn write( &mut self, v:$struct_name ) {
        self.write( v.0 )
      }
    }

    impl ByteReader<$struct_name> for ByteBuff {
      fn read( &mut self, target:&mut $struct_name ) -> Result<(),String> {    
        self.read(&mut target.0)
      }
    }
  };
}

bytes_rw_new_type!(
  #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
  pub struct BlockId(u32)
);

impl fmt::Display for BlockId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f,"BlockId({})",self.0)
  }
}

#[allow(dead_code)]
impl BlockId {
  pub fn new( value: u32 ) -> Self {
    Self(value)
  }

  pub fn value( self ) -> u32 {
    self.0
  }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct FileOffset(u64);

impl fmt::Display for FileOffset {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f,"FileOffset({})",self.0)
  }
}

#[allow(dead_code)]
impl FileOffset {
  pub fn new( value: u64 ) -> Self {
    Self(value)
  }

  pub fn value( self ) -> u64 {
    self.0
  }
}

impl From<u64> for FileOffset {
  fn from(value: u64) -> Self {
    Self::new(value)
  }
}

impl From<u32> for FileOffset {
  fn from(value: u32) -> Self {
    Self::new(value as u64)
  }
}

impl From<u16> for FileOffset {
  fn from(value: u16) -> Self {
    Self::new(value as u64)
  }
}

impl From<usize> for FileOffset {
  fn from(value: usize) -> Self {
    Self::new(value as u64)
  }
}

impl ByteReader<FileOffset> for ByteBuff {
  fn read( &mut self, target:&mut FileOffset ) -> Result<(),String> {
    let mut off:u64 = 0;
    self.read(&mut off)?;

    *target = FileOffset::from(off);
    Ok(())
  }
}

impl ByteWriter<FileOffset> for ByteBuff {
  fn write( &mut self, v:FileOffset ) {
    self.write(v.0 as u64)
  }
}

bytes_rw_new_type!(
  #[derive(Copy, Clone, Debug, Default, PartialEq)]
  pub struct DataId(u32)
);

#[allow(dead_code)]
impl DataId {
  pub fn new( value: u32 ) -> Self {
    Self(value)
  }

  pub fn value( self ) -> u32 {
    self.0
  }

  pub fn user_data() -> Self {
    Self::new(1024)
  }
}


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

#[allow(dead_code)]
const TAIL_SIZE : u16 = 8;

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
#[allow(dead_code)]
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

#[allow(dead_code)]
fn write_block_head( head:BlockHead, data_size:u32, tail_size:u16 ) -> Box<Vec<u8>> {
  let mut bbuf = ByteBuff::new();

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

  bbuf.buff
}

#[test]
fn test_block() {
  let bh = BlockHead {
    block_id: BlockId(10),
    data_type_id: DataId(2),
    back_refs: BackRefs { refs: Box::new(vec![
      (BlockId(9), FileOffset(7)),
      (BlockId(8), FileOffset(20)),
    ])},
    block_options: BlockOptions::default()
  };

  let block_data = write_block_head(bh, 134, TAIL_SIZE);
  println!("{:?}",read_block_head(block_data));
}

#[allow(dead_code)]
const TAIL_MARKER : &str = "TAIL";

/// Хвост блока
#[allow(dead_code)]
pub struct Tail;

#[allow(dead_code)]
impl Tail {
  /// Чтение блока по значению хвоста
  /// 
  /// # Параметры
  /// - `position` - указатель на конец хвоста, первый байт после хвоста
  /// - `source` - источник данных
  pub fn try_read_block_at<Source>( position: u64, source: &Source ) -> Result<(Block,u64), BlockErr> 
  where Source : ReadBytesFrom
  {
    if position<(TAIL_SIZE as u64) { 
      return Err(
        BlockErr::tail_position_to_small(TAIL_SIZE, position)
      ) 
    }

    let mut tail_data :[u8;TAIL_SIZE as usize] = [0; (TAIL_SIZE as usize)];
    let reads = source.read_from((position)-(TAIL_SIZE as u64), &mut tail_data)?;
    if reads < tail_data.len() as u64 { return Err(BlockErr::no_data(reads as u64, TAIL_SIZE as u64)) }

    let marker_matched = (0usize .. 4usize).fold( true, |res,idx| res && TAIL_MARKER.as_bytes()[idx] == tail_data[idx] );
    if ! marker_matched { return Err(BlockErr::TailMarkerMismatched {tail_data: tail_data}) }

    let total_size: [u8; 4] = [ tail_data[4],tail_data[5],tail_data[6],tail_data[7] ];
    let total_size = u32::from_le_bytes(total_size);

    let next_pos = (position as i128) - (total_size as i128);
    if next_pos < 0 { return Err(BlockErr::TailPointerOuside { pointer: next_pos }) }

    let next_pos = next_pos as u64;
    Block::read_from(next_pos, source)
  }

  /// Чтение заголовка
  pub fn try_read_head_at<Source>( position: u64, source: &Source ) -> Result<BlockHeadRead, BlockErr> 
  where Source : ReadBytesFrom
  {
    if position<(TAIL_SIZE as u64) { 
      return Err(
        BlockErr::tail_position_to_small(TAIL_SIZE, position)
      ) 
    }

    let mut tail_data :[u8;TAIL_SIZE as usize] = [0; (TAIL_SIZE as usize)];
    let reads = source.read_from((position)-(TAIL_SIZE as u64), &mut tail_data)?;
    if reads < tail_data.len() as u64 { return Err(BlockErr::no_data(reads, TAIL_SIZE as u64)) }

    let marker_matched = (0usize .. 4usize).fold( true, |res,idx| res && TAIL_MARKER.as_bytes()[idx] == tail_data[idx] );
    if ! marker_matched { return Err(BlockErr::TailMarkerMismatched {tail_data: tail_data}) }

    let total_size: [u8; 4] = [ tail_data[4],tail_data[5],tail_data[6],tail_data[7] ];
    let total_size = u32::from_le_bytes(total_size);

    let next_pos = (position as i128) - (total_size as i128);
    if next_pos < 0 { return Err(BlockErr::TailPointerOuside { pointer: next_pos }) }

    let next_pos = next_pos as u64;
    BlockHead::read_form(next_pos as usize, source)
  }
}

impl Block {
  /// Формирование массива байтов представлющих блок
  #[allow(dead_code)]
  pub fn to_bytes( &self ) -> (Box<Vec<u8>>, BlockHeadSize, BlockDataSize, BlockTailSize) {
    // write tail marker
    let mut tail = Box::new(Vec::<u8>::new());
    for i in 0..TAIL_MARKER.len() {
      tail.push(TAIL_MARKER.as_bytes()[i]);
    }
    tail.push(0);tail.push(0);tail.push(0);tail.push(0);

    // write head
    let mut bytes = write_block_head(self.head.clone(), self.data.len() as u32, tail.len() as u16);
    let block_head_size = BlockHeadSize(bytes.len() as u32);
    let block_data_size = BlockDataSize(self.data.len() as u32);
    let block_tail_size = BlockTailSize(tail.len() as u16);

    let off = bytes.len();
    if self.data.len()>0 {      
      bytes.resize(bytes.len() + self.data.len() + tail.len(), 0)
    }

    // copy data
    for i in 0..(self.data.len()) {
      bytes[off + i] = self.data[i]
    }

    // update tail data
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

    (bytes, block_head_size, block_data_size, block_tail_size)
  }
}

impl BlockHead {
  /// Чтение заголовка
  #[allow(dead_code)]
  fn from_bytes( bytes: Box<Vec<u8>> ) -> Result<(BlockHead, BlockHeadSize, BlockDataSize, BlockTailSize), String> {
    read_block_head(bytes)
  }

  /// Формирование байтового представления
  #[allow(dead_code)]
  pub fn to_bytes( &self, data_size:u32 ) -> Box<Vec<u8>> {
    write_block_head( self.clone(), data_size, 8)
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

/// Ошибка при операциях с блоком лога
#[derive(Debug,Clone)]
pub enum BlockErr { 
  Generic(String),
  IO {
    message: String,
    os_error: Option<i32>
  },
  AbsBuff( ABuffError ),
  PositionToSmall { 
    min_position: FileOffset,
    actual: FileOffset,
  },
  NoData {
    reads: u64,
    expect: u64,
  },
  TailMarkerMismatched {
    tail_data: [u8; TAIL_SIZE as usize]
  },
  TailPointerOuside {
    pointer: i128
  },
  Limit {
    message: String,
    limit: u64,
    target: u64
  }
}

#[derive(Debug,Clone, Copy)]
struct Limit(u64);
impl Limit {
  fn check<V:Into<u64>>(self, value:V, operation:&str) -> Result<(), BlockErr> 
  {
    let v64:u64 = value.into();
    if v64 > self.0 {
      Err(BlockErr::limit(operation, self.clone(), v64))
    }else{
      Ok(())
    }
  }
}
impl fmt::Display for Limit {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}
const LIMIT_USIZE : Limit = Limit(usize::MAX as u64);

impl BlockErr {
  fn generic<A: Into<String>>( message:A ) -> Self {
    Self::Generic(message.into())
  }
  fn tail_position_to_small<
    A:Into<FileOffset>,
    B:Into<FileOffset>,
  >( min_pos:A, actual_pos:B ) -> Self {
    Self::PositionToSmall { 
      min_position: min_pos.into(), 
      actual: actual_pos.into() 
    }
  }
  fn no_data( reads: u64, expect: u64 ) -> Self {
    Self::NoData { reads: reads, expect: expect }
  }
  fn limit( operation_name: &str, limit:Limit, target:u64 ) -> Self {
    Self::Limit { 
      message: format!(
        "can't execute {operation_name} by limit size, current limit {limit}, target size {target}"
      ), 
      limit: limit.0, 
      target: target 
    }
  }
}

impl From<std::io::Error> for BlockErr {
  fn from(value: std::io::Error) -> Self {
    Self::IO {
      message: value.to_string(),
      os_error: value.raw_os_error()
    }
  }
}
impl From<String> for BlockErr {
  fn from(value: String) -> Self {
    Self::generic(value)
  }
}
impl From<&str> for BlockErr {
  fn from(value: &str) -> Self {
    Self::generic(value.to_string())
  }
}
impl From<ABuffError> for BlockErr {
  fn from(value: ABuffError) -> Self {
    Self::AbsBuff(value.clone())
  }
}

/// Размер буфера при чтении файла
#[allow(dead_code)]
const READ_BUFF_SIZE: usize = 1024*8;

/// Размер буфера при чтении заголовка, в теории заголовок не должен быть больше этого значения
const PREVIEW_SIZE:usize = 1024 * 64;

impl Block {
  /// Чтение блока из массива байт
  #[allow(dead_code)]
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

  /// Запись блока в массив байтов
  #[allow(dead_code)]
  pub fn write_to<Destination>( &self, position:u64, dest:&mut Destination ) -> Result<BlockHeadRead,BlockErr> 
  where Destination: WriteBytesTo
  {
    let (bytes,head_size,data_size,tail_size) = self.to_bytes();
    dest.write_to( position, &bytes[0 .. bytes.len()])?;
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
    head: BlockHead { block_id: BlockId(0), data_type_id: DataId(1), back_refs: BackRefs::default(), block_options: BlockOptions::default() },
    data: Box::new( vec![1,2,3] )
  };

  block.write_to(0, &mut bb).unwrap();
  println!("{block_size}", block_size=bb.bytes_count().unwrap() );

  let (rblock,_) = Block::read_from(0, &bb).unwrap();
  assert!( rblock.head.block_id == block.head.block_id );
  assert!( rblock.head.data_type_id == block.head.data_type_id );
}
