use core::fmt::Debug;
use std::sync::{Arc, RwLock};

use crate::logfile::FlatBuff;
use crate::logfile::block::FileOffset;
use super::super::logfile::LogFile;
use super::*;

// pub struct LoggingQueue<LogId,FILE,BUFF> 
// where
//     LogId: Clone + Debug,
//     FILE: Clone + Debug,
//     BUFF: FlatBuff
// {
//     pub queue: Box<dyn LogQueue<RecID<LogId>, LogId, FILE, LogFile<BUFF>>>
// }
