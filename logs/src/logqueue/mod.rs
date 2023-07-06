//! Очередь логов, в данном случае это очередь состоящая из цепочки файлов.
//! 
//! В очереди выделяются следующие роли файлов
//! - Начальный файл
//! - Промежуточный файл
//! - Последний файл
//! 
//! С данными роля возможны следующие операции
//! 
//! | Операция | Начальный файл | Промежуточный файл | Последний файл |
//! |----------|----------------|--------------------|----------------|
//! | Чтение   | да             | да                 | да             |
//! | Запись   |                |                    | да             |
//! 
//! Возможны следующие комбинации
//! 
//! - 0 файлов - нет логов
//! - 1 файл - этот файл совмещает все роли
//! - 2 файла
//!   - первый файл - только readonly
//!   - второй файл - read/write
//! - 3 и более файла
//!   - первый файл - только readonly
//!   - второй файл и последующие, кроме последнего - только readonly
//!   - последний - read/write
//! 
//! Между файлами должна быть связь такая, что бы выстроить последовательность:
//! 
//! - Первый файл - содержит уникальный индентификатор, генерируемый первоначально
//! - Последующие файлы, в качестве первой записи содержат ссылку на предыдущий файл
//! 
//! В нормальной очереди логов должны соблюдаться следующие условия
//! 
//! - Есть первый файл
//!     - Ни на кого не ссылается
//!     - Содержит уникальный номер (в пределах очереди)
//!     - В очереди один
//! - Второй и последующие файлы
//!     - Содержит уникальный номер (в пределах очереди)
//!     - Содержит ссылку на предшедствующий файл
/// Общий api лог очереди
mod log_api;
pub use log_api::*;

/// Поиск лог файлов
pub mod find_logs;

/// Шаблон пути
mod path_tmpl;

/// Генерация нового файла
mod new_file;

/// Открытие лог файлов
mod logs_open;

/// Идентификатор лога
mod log_id;

// Валидация логов
mod log_seq_verifier;

/// Переключение лог файла
mod log_switch;

/// Очередь лог файлов
mod log_queue;