use async_trait::async_trait;
use std::net::Ipv4Addr;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait PublicIpService {
    async fn get_ip(&self) -> Result<Ipv4Addr, PublicIpServiceError>;
}

#[derive(Debug, Error)]
pub enum PublicIpServiceError {
    #[error("internal error")]
    InternalError,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("invalid response")]
    InvalidIpResponse,
    #[error("client request error")]
    ClientError {
        #[from]
        source: reqwest::Error,
    },
}
