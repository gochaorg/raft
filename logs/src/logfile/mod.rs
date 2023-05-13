//! Работа с лог файлом (ами)

/// Блок лог файла
pub mod block;

/// Лог файл - сумма блоков
mod logfile;
pub use logfile::*;
