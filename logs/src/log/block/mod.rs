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

use crate::{bbuff::absbuff::{ ReadBytesFrom, WriteBytesTo }, perf::Tracker};

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

mod head;
pub use head::*;

mod block;
pub use block::*;

