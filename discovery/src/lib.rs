use async_trait::async_trait;

/// Ошибки
pub enum DiscoveryError {
}

/// Служба обнаружения
#[async_trait]
pub trait DiscoveryService<A> {
    async fn start() -> Result<(),DiscoveryError>;
    async fn stop() -> Result<(),DiscoveryError>;
    async fn is_running() -> Result<bool,DiscoveryError>;
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
