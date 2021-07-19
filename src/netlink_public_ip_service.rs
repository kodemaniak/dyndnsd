use pnet::{datalink::interfaces, ipnetwork::IpNetwork};
use std::net::Ipv4Addr;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait PublicIpService {
    fn get_ip(&self) -> Result<Ipv4Addr, PublicIpServiceError>;
}

pub enum PublicIpServiceError {
    UnknownError,
}

pub struct NetlinkPublicIpService {
    interface: String,
}

impl NetlinkPublicIpService {
    pub fn new(interface: &str) -> Self {
        Self {
            interface: interface.to_owned(),
        }
    }
}

impl PublicIpService for NetlinkPublicIpService {
    fn get_ip(&self) -> Result<Ipv4Addr, PublicIpServiceError> {
        match interfaces().iter().find(|i| i.name == self.interface) {
            Some(interface) => {
                let maybe_ip = interface.ips.iter().find(|n| match n {
                    IpNetwork::V4(_ipn) => true,
                    _ => false,
                });
                match maybe_ip {
                    Some(IpNetwork::V4(ip)) => {
                        println!("local ip: {}", ip);
                        Ok(ip.ip())
                    }
                    _ => {
                        println!("Did not find ip on interface {}", self.interface);
                        Err(PublicIpServiceError::UnknownError)
                    }
                }
            }
            None => {
                println!("Did not find interface {}", self.interface);
                Err(PublicIpServiceError::UnknownError)
            }
        }
    }
}
