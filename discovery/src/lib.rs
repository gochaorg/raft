mod udp;

mod msg;
pub use msg::*;

mod ip_range;
pub use ip_range::*;

mod sockaddr_range;
pub use sockaddr_range::*;

use async_trait::async_trait;

/// Ошибки
pub enum DiscoveryError {
}

/// Служба обнаружения
#[async_trait]
pub trait DiscoveryService {
    async fn start( &mut self ) -> Result<(),DiscoveryError>;
    async fn stop( &mut self ) -> Result<(),DiscoveryError>;
    async fn is_running( &self ) -> Result<bool,DiscoveryError>;
}

/// Клиент службы обнаружения
#[async_trait]
pub trait DiscoverClient<A> {
    /// Обнаружить сервисы в сети
    async fn discovery() -> Result<Vec<A>,DiscoveryError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
