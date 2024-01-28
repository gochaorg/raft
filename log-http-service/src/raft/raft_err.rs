use log_http_client::Error as ClientError;
use derive_more::Display;

#[derive(Debug,Display)]
pub enum RaftError {
    #[display(fmt="Can't create client {}", _0)]
    CantCreateClient(ClientError),

    #[display(fmt="Not found")]
    NotFound
}
