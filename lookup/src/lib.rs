use std::sync::Arc;

pub mod dns_lookup;
pub mod udp_lookup;

/// Поиск серверов
pub trait ServersLookup<Addr> {
    /// Поиск серверов
    ///
    /// # Результат:
    /// Список серверов (адресов)
    fn lookup(&self) -> Arc<Vec<Addr>>;
}
