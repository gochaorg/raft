use crate::bbuff::absbuff::ReadBytesFrom;

use super::{Block, BlockErr, BlockHeadRead, BlockHead};

pub const TAIL_SIZE : u16 = 8;

pub const TAIL_MARKER : &str = "TAIL";

/// Хвост блока
pub struct Tail;

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
