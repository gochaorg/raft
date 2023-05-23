use std::{path::Path, sync::{RwLock, Arc}, io::Write};

use logs::{block::BlockId, bbuff::absbuff::FileBuff, LogFile, GetPointer, FlatBuff, LogPointer};

use crate::err::LogToolErr;

pub fn extract_to_stdout<P,R>( log_file_name: P, blocks: R ) -> Result<(), LogToolErr> 
where
    P: AsRef<Path>,
    R: IntoIterator<Item = BlockId>
{
    let buff = FileBuff::open_read_only(log_file_name)?;
    let log = LogFile::new(buff)?;
    let log = Arc::new(RwLock::new(log));

    let mut ptr = log.pointer_to_end()?;
    for block_id in blocks {
        ptr = ptr.jump(block_id)?;
        extract_block(&ptr)?;
    }
    Ok(())
}

fn extract_block<B: FlatBuff>( log_ptr: &LogPointer<B> ) -> Result<(), LogToolErr> {
    let data = log_ptr.current_data()?;
    std::io::stdout().write_all(&data)?;
    Ok(())
}