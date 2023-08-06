use async_trait::async_trait;
use log::debug;
use mockall_double::double;
use std::net::Ipv4Addr;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[double]
use crate::hetzner_dns_client::HetznerDnsClient;

use crate::hetzner_dns_client::HetznerDnsClientError;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DnsService {
    async fn resolve_ip(
        &self,
        subdomain: &str,
        domain: &str,
    ) -> Result<Option<Ipv4Addr>, DnsServiceError>;
    async fn update_dns(
        &self,
        subdomain: &str,
        domain: &str,
        ip: Ipv4Addr,
    ) -> Result<(), DnsServiceError>;
}

#[derive(Debug, Error)]
pub enum DnsServiceError {
    #[error("unknown zone")]
    UnknownZone,
    #[error("unknown record")]
    UnknownRecord,
    #[error("client error")]
    ClientError,
    #[error("unknown error")]
    UnknownError,
}

impl From<HetznerDnsClientError> for DnsServiceError {
    fn from(_: HetznerDnsClientError) -> Self {
        DnsServiceError::ClientError
    }
}

pub struct HetznerDnsService {
    client: HetznerDnsClient,
}

impl HetznerDnsService {
    pub fn new(api_token: &str) -> Self {
        let client = HetznerDnsClient::new(api_token);
        Self::from_client(client)
    }

    pub fn from_client(client: HetznerDnsClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl DnsService for HetznerDnsService {
    async fn resolve_ip(
        &self,
        subdomain: &str,
        domain: &str,
    ) -> Result<Option<Ipv4Addr>, DnsServiceError> {
        debug!(
            "Resolve ip for domain {} and subdomain {}.",
            domain, subdomain
        );
        let zone = self.client.find_zone(domain).await?;

        if let Some(zone) = zone {
            let record = self.client.find_record(&zone.id, subdomain).await?;

            if let Some(record) = record {
                let ip_str = record.value;
                let ip: Ipv4Addr = ip_str.parse().map_err(|_| DnsServiceError::UnknownError)?;
                return Ok(Some(ip));
            }

            return Ok(None);
        }

        Err(DnsServiceError::UnknownZone)
    }

    async fn update_dns(
        &self,
        subdomain: &str,
        domain: &str,
        ip: Ipv4Addr,
    ) -> Result<(), DnsServiceError> {
        debug!(
            "Update dns for domain {} and subdomain {} with IP {}.",
            domain, subdomain, ip
        );
        let zone = self.client.find_zone(domain).await?;

        if let Some(zone) = zone {
            let record = self.client.find_record(&zone.id, subdomain).await?;

            if let Some(record) = record {
                println!(
                    "Updating record {} in zone {} tp ip {}",
                    record.id, zone.id, ip,
                );
                self.client
                    .update_ip(subdomain, &zone.id, &record.id, ip)
                    .await?;
                return Ok(());
            }

            return Err(DnsServiceError::UnknownRecord);
        }

        Err(DnsServiceError::UnknownZone)
    }
}

#[cfg(test)]
mod tests {
    use mockall::*;

    use super::*;
    use crate::hetzner_dns_client::{Record, Zone};

    const ZONE: &str = "example.com";
    const SUBDOMAIN: &str = "example.com";
    const ZONE_ID: &str = "zid123";
    const RECORD_ID: &str = "rid123";
    const OLD_IP: &str = "127.0.0.2";
    const NEW_IP: &str = "127.0.0.3";

    #[tokio::test]
    async fn resolve_ip() {
        let mut client = HetznerDnsClient::default();

        let zone = Zone {
            name: String::from(ZONE),
            id: String::from(ZONE_ID),
        };
        let record = Record {
            id: String::from(RECORD_ID),
            zone_id: String::from(ZONE_ID),
            name: String::from(SUBDOMAIN),
            r#type: String::from("A"),
            value: String::from(OLD_IP),
        };

        client
            .expect_find_zone()
            .with(predicate::eq(ZONE))
            .times(1)
            .returning(move |_| Ok(Some(zone.clone())));
        client
            .expect_find_record()
            .with(predicate::eq(ZONE_ID), predicate::eq(SUBDOMAIN))
            .times(1)
            .returning(move |_, _| Ok(Some(record.clone())));

        let svc = HetznerDnsService::from_client(client);
        let ip = svc
            .resolve_ip(SUBDOMAIN, ZONE)
            .await
            .expect("An error occured.")
            .expect("Did not receive the IP.");
        assert_eq!(ip.to_string(), OLD_IP, "Did not receive the DNS IP");
    }

    #[tokio::test]
    async fn resolve_ip_should_err_when_zone_not_exists() {
        let mut client = HetznerDnsClient::default();

        client
            .expect_find_zone()
            .with(predicate::eq(ZONE))
            .times(1)
            .returning(|_| Ok(None));
        client
            .expect_find_record()
            .with(predicate::always(), predicate::always())
            .never();

        let svc = HetznerDnsService::from_client(client);
        let ip = svc.resolve_ip(SUBDOMAIN, ZONE).await;
        assert!(matches!(ip.err().unwrap(), DnsServiceError::UnknownZone));
    }

    #[tokio::test]
    async fn resolve_ip_should_return_none_when_record_not_exists() {
        let mut client = HetznerDnsClient::default();

        let zone = Zone {
            name: String::from(ZONE),
            id: String::from(ZONE_ID),
        };
        client
            .expect_find_zone()
            .with(predicate::eq(ZONE))
            .times(1)
            .returning(move |_| Ok(Some(zone.clone())));
        client
            .expect_find_record()
            .with(predicate::always(), predicate::always())
            .times(1)
            .returning(|_, _| Ok(None));

        let svc = HetznerDnsService::from_client(client);
        let ip = svc.resolve_ip(SUBDOMAIN, ZONE).await.unwrap();
        assert!(ip.is_none());
    }

    #[tokio::test]
    async fn update_dns() {
        let mut client = HetznerDnsClient::default();

        let zone = Zone {
            name: String::from(ZONE),
            id: String::from(ZONE_ID),
        };
        let record = Record {
            id: String::from(RECORD_ID),
            zone_id: String::from(ZONE_ID),
            name: String::from(SUBDOMAIN),
            r#type: String::from("A"),
            value: String::from(OLD_IP),
        };
        let new_record = Record {
            id: String::from(RECORD_ID),
            zone_id: String::from(ZONE_ID),
            name: String::from(SUBDOMAIN),
            r#type: String::from("A"),
            value: String::from(NEW_IP),
        };

        client
            .expect_find_zone()
            .with(predicate::eq(ZONE))
            .times(1)
            .returning(move |_| Ok(Some(zone.clone())));
        client
            .expect_find_record()
            .with(predicate::eq(ZONE_ID), predicate::eq(SUBDOMAIN))
            .times(1)
            .returning(move |_, _| Ok(Some(record.clone())));
        let new_ip_addr: Ipv4Addr = NEW_IP.parse().unwrap();
        client
            .expect_update_ip()
            .with(
                predicate::eq(SUBDOMAIN),
                predicate::eq(ZONE_ID),
                predicate::eq(RECORD_ID),
                predicate::eq(new_ip_addr),
            )
            .times(1)
            .returning(move |_, _, _, _| Ok(new_record.clone()));

        let svc = HetznerDnsService::from_client(client);
        let result = svc
            .update_dns(SUBDOMAIN, ZONE, NEW_IP.parse().unwrap())
            .await;
        assert!(result.is_ok());
    }
}
