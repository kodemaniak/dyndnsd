use async_trait::async_trait;
use std::net::Ipv4Addr;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait PublicIpService {
    async fn get_ip(&self) -> Result<Ipv4Addr, PublicIpServiceError>;
}

pub enum PublicIpServiceError {
    UnknownError,
}
