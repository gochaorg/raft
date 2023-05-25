//! Работа с лог файлом

/// Блок лог файла
pub mod block;

/// Лог файл - сумма блоков
mod logfile;
pub use logfile::*;
