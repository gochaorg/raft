use std::{path::Path, sync::{Arc, RwLock}};

use logs::{bbuff::absbuff::FileBuff, LogFile, logfile::*};
use sha2::{Digest, Sha256};

use crate::err::LogToolErr;

/// Просмотр заголовков
pub fn view_logfile<P: AsRef<Path>>(log_file: P, sha256: bool) -> Result<(), LogToolErr> {
    let buff = FileBuff::open_read_only(log_file)?;
    let log = LogFile::new(buff)?;
    let log = Arc::new(RwLock::new(log));

    let mut ptr = log.pointer_to_end()?;
    loop {
        let h = ptr.current_head();
        print!(
            "{b_id:0>6} {d_size:0>10}",
            b_id = h.head.block_id.value(),
            d_size = h.data_size.value()
        );

        if sha256 {
            match ptr.current_data() {
                Ok(data) => {
                    let mut hasher = Sha256::new();
                    hasher.update(*data);
                    let hash = hasher.finalize();
                    let hash = hex::encode(hash);
                    print!(" {hash}");
                }
                Err(err) => {
                    print!("can't read data {err:?}")
                }
            }
        }

        //print!("{:?}", h);

        if !h.head.block_options.is_empty() {
            for (key, value) in h.head.block_options.clone() {
                print!(" {key}={value}")
            }
        }

        println!();

        match ptr.previous() {
            Ok(next_ptr) => ptr = next_ptr,
            Err(_) => {
                break;
            }
        }
    }

    Ok(())
}
