//! Представляет из себя блок лог файла
//! ==========================================
//!
//! Размер блока может быть разным
//!
//! Структура блока
//! ---------------------------
//! 
//! - Заголовок:
//!   - head_size : u32 - Размер заголовка в байтах
//!   - data_size : u32 - Размер данных в байтах
//!   - tail_size : u16 - Размер хвоста в байтах
//!   - block_id  : u32 - Идентификатор текущего блока 
//!   - data_type_id : u32 - Тип данных
//!   - back_refs_count : u32 - Кол-во обратных ссылок
//!   - Обратные ссылки, повтор back_refs_count:
//!     - back_ref.b_id : u32 - Идентификатор на который сслыка
//!     - back_ref.b_off : u64 - Ссылка, смещение относительно начала
//!   - Опции блока, размер = остаток head_size - текущая позиция (back_refs_count*4*8 + 4+4+2+4+4+4)
//!     - key_value_count : u64 - кол-во пар ключ/значение
//!     - Пары ключ/значение, повтор key_value_count
//!       - Ключ : String16
//!         - key_string_size : u16 - Размер строки в байтах
//!         - key_string_data : [u8; key_string_size] - данные строки, размером key_string_size, кодировка utf-8
//!       - Значение : String32
//!         - value_string_size : u32 - Размер строки в байтах
//!         - value_string_data : [u8; value_string_size] - данные строки, размером key_string_size, кодировка utf-8
//! - Данные : [u8; data_size]. Начало - смещение относительно блока равное head_size
//! - Хвост : [u8; tail_size].  Начало - смещение относительно блока равное head_size+data_size
//!   - marker : [u8; 4] - строка содержащая "TAIL"
//!   - total_size : u32 - суммарный размер блока, включая хвост
//! 
mod fileoffset;
pub use fileoffset::*;

mod blockid;
pub use blockid::*;

mod dataid;
pub use dataid::*;

/// Опции блока
mod blockopts;
pub use blockopts::*;

/// Ошибки при работе с блоком
mod err;
pub use err::*;

/// Ограничение на размер данных для операций с блоком
mod limit;
pub use limit::*;

/// Чтение заголовка/блока относительно хвоста
mod tail;
pub use tail::*;

/// Чтение/запись заголовка блока
mod head;
pub use head::*;

/// Чтение/запись блока в массив байтов/поток байтовый
mod block;
pub use block::*;

/// Работа с String16, String32
mod string_n;
pub use string_n::*;
