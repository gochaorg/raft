use log_http_client::Error as ClientError;
use derive_more::Display;

use super::rest_api::sync::LogShippingError;

#[derive(Debug,Display)]
pub enum RaftError {
    #[display(fmt="Can't create client {}", _0)]
    CantCreateClient(ClientError),

    #[display(fmt="Not found")]
    NotFound,

    #[display(fmt="LogShipping: {}", _0)]
    LogShipping(LogShippingError)
}
