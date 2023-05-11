use std::time::Instant;

use crate::{bbuff::{absbuff::{ReadBytesFrom, WriteBytesTo}, streambuff::{ByteBuff}}, perf::Tracker};

use super::*;

/// Блок лога
pub struct Block {
  /// Заголовок
  pub head: BlockHead,

  /// Данные
  pub data: Box<Vec<u8>>
}

/// Размер буфера при чтении файла
const READ_BUFF_SIZE: usize = 1024*8;

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
  fn to_bytes( &self, bbuf:&mut ByteBuff, tracker:&Tracker ) -> (BlockHeadSize, BlockDataSize, BlockTailSize) {
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

    // let mut bbuf = ByteBuff::new();

    // write head
    tracker.track("to_bytes/write_block_head", || {
      self.head.write_block_head(bbuf, self.data.len() as u32, tail.len() as u16, tracker)
    });

    let block_head_size = tracker.track("to_bytes/block_head_size",||BlockHeadSize(bbuf.buff.len() as u32));
    let block_data_size = tracker.track("to_bytes/block_data_size",||BlockDataSize(self.data.len() as u32));
    let block_tail_size = tracker.track("to_bytes/block_tail_size",||BlockTailSize(tail.len() as u16));

    // copy data
    tracker.track("to_bytes/copy data", || {
      bbuf.write_byte_arr(&self.data)
    });

    // update tail data
    tracker.track("to_bytes/tail update", ||{
      let total_size = bbuf.buff.len() as u32 + tail.len() as u32;
      let total_size = total_size.to_le_bytes();

      tail[4] = total_size[0];
      tail[5] = total_size[1];
      tail[6] = total_size[2];
      tail[7] = total_size[3];

      bbuf.write_byte_arr(&tail);
    });

    let t4 = Instant::now();
    tracker.add("to_bytes", t4.duration_since(t0));

    (block_head_size, block_data_size, block_tail_size)
  }

  /// Запись блока в массив байтов
  pub fn write_to<Destination>( &self, position:u64, dest:&mut Destination, block_buff:&mut ByteBuff, tracker: &Tracker ) -> Result<BlockHeadRead,BlockErr> 
  where Destination: WriteBytesTo
  {
    let sub_track = tracker.sub_tracker("block.to_bytes/");
    let mut bbuf = block_buff;//ByteBuff::new();
    bbuf.reset();

    let t0 = Instant::now();
    let (head_size,data_size,tail_size) = self.to_bytes(&mut bbuf, &sub_track);

    let t1 = Instant::now();
    let bytes = &bbuf.buff;
    dest.write_to( position, &bytes )?;

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
  use super::super::super::bbuff::absbuff::ByteBuff;
  use crate::bbuff::absbuff::BytesCount;
  use crate::block::{BlockId, DataId, BackRefs, BlockOptions};
  use crate::{bbuff::{streambuff::{self}}, perf::Tracker};

  let mut bb = ByteBuff::new_empty_unlimited();

  let mut data = Box::new(Vec::<u8>::new());
  for i in 0..134 { data.push(i as u8) }

  let mut opts = BlockOptions::default();
  opts.set("key", "value").unwrap();

  let block = Block {
    head: BlockHead { 
      block_id: BlockId::new(0), 
      data_type_id: DataId::new(1), 
      back_refs: BackRefs::default(), 
      block_options: opts 
    },
    data: data
  };

  let tracker = Tracker::new();
  let mut block_buff = streambuff::ByteBuff::new();

  block.write_to(0, &mut bb, &mut block_buff, &tracker).unwrap();
  println!("{block_size}", block_size=bb.bytes_count().unwrap() );

  let (rblock,_) = Block::read_from(0, &bb).unwrap();
  assert!( rblock.head.block_id == block.head.block_id );
  assert!( rblock.head.data_type_id == block.head.data_type_id );
  assert!( rblock.data == block.data );
  assert!( rblock.head.back_refs.refs == block.head.back_refs.refs );

  println!("{:?}", rblock.head);
}

// /// Построение блока
// pub struct BlockBuilder<'a> {
//   /// Ссылка на данные
//   pub data: &'a[u8],

//   pub tail: &'a[u8],

//   pub block_id:  &'a BlockId,
//   pub data_id:   &'a DataId,
//   pub back_refs: &'a Vec<(BlockId, FileOffset)>,
//   pub options:   &'a BlockOptions
// }