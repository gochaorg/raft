use crate::bbuff::absbuff::ABuffError;

use super::{FileOffset, Limit, TAIL_SIZE};

/// Ошибка при операциях с блоком лога
#[derive(Debug, Clone)]
pub enum BlockErr {
    IO {
        message: String,
        os_error: Option<i32>,
    },
    AbsBuff(ABuffError),
    BlockHeaderToSmall {
        actual: u64,
        min_size: u64,
    },
    BlockDataTruncated {
        expect_data_size: u64,
        reads_data_size: u64,
    },
    BlockHeadReadFail {
        head_data: Vec<u8>,
        error: String,
    },
    PositionToSmall {
        min_position: FileOffset,
        actual: FileOffset,
    },
    NoData {
        reads: u64,
        expect: u64,
    },
    TailMarkerMismatched {
        tail_data: [u8; TAIL_SIZE as usize],
    },
    TailPointerOuside {
        pointer: i128,
    },
    Limit {
        message: String,
        limit: u64,
        target: u64,
    },
}

impl From<std::io::Error> for BlockErr {
    fn from(value: std::io::Error) -> Self {
        Self::IO {
            message: value.to_string(),
            os_error: value.raw_os_error(),
        }
    }
}
impl From<ABuffError> for BlockErr {
    fn from(value: ABuffError) -> Self {
        Self::AbsBuff(value.clone())
    }
}

impl BlockErr {
    pub fn tail_position_to_small<A: Into<FileOffset>, B: Into<FileOffset>>(
        min_pos: A,
        actual_pos: B,
    ) -> Self {
        Self::PositionToSmall {
            min_position: min_pos.into(),
            actual: actual_pos.into(),
        }
    }
    pub fn no_data(reads: u64, expect: u64) -> Self {
        Self::NoData {
            reads: reads,
            expect: expect,
        }
    }

    pub fn limit(operation_name: &str, limit: Limit, target: u64) -> Self {
        Self::Limit {
            message: format!(
        "can't execute {operation_name} by limit size, current limit {limit}, target size {target}"
      ),
            limit: limit.0,
            target: target,
        }
    }
}
