use std::fmt::Debug;
use serde::{Serialize, Deserialize};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum DiscoveryRequest<A> 
where
    A: Clone+Debug
{
    Hello {
        pub_address: A
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum DiscoveryResponse<A> 
where
    A: Clone+Debug
{
    Wellcome {
        pub_address: A
    },
    Error {
        error_message: A
    }
}

