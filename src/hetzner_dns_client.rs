use log::error;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

const API_URL: &str = "https://dns.hetzner.com/api/v1";

#[derive(Deserialize, Serialize, Debug)]
pub struct GetZonesResponse {
    pub zones: Vec<Zone>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetRecordsResponse {
    pub records: Vec<Record>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateRecordResponse {
    pub record: Record,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateRecordRequest {
    pub name: String,
    pub ttl: u16,
    pub r#type: String,
    pub value: String,
    pub zone_id: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct Zone {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Record {
    pub id: String,
    pub zone_id: String,
    pub name: String,
    pub r#type: String,
    pub value: String,
}

pub struct HetznerDnsClient {
    api_url: String,
    api_token: String,
    client: Client,
}

#[cfg_attr(test, automock)]
impl HetznerDnsClient {
    pub fn new(api_token: &str) -> Self {
        Self::new_with_url(api_token, API_URL)
    }

    pub fn new_with_url(api_token: &str, api_url: &str) -> Self {
        let client = Client::new();
        Self {
            api_url: String::from(api_url),
            api_token: String::from(api_token),
            client,
        }
    }

    pub async fn find_zone(&self, zone: &str) -> Result<Option<Zone>, HetznerDnsClientError> {
        let response = self
            .client
            .get(format!("{}/zones?name={}", self.api_url, zone))
            .header("Auth-API-Token", &self.api_token)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let response: GetZonesResponse = response.json().await?;

                // THere can only be one zone with a given name, therefore we just take the first
                // result if present
                let maybe_zone = response.zones.into_iter().next();

                Ok(maybe_zone)
            }
            StatusCode::UNAUTHORIZED => Err(HetznerDnsClientError::InvalidApiToken),
            _ => {
                error!("Failed to resolve zone for {}.", zone);
                Err(HetznerDnsClientError::FailedToResolveZone {
                    zone: zone.to_string(),
                })
            }
        }
    }

    pub async fn find_record(
        &self,
        zone_id: &str,
        subdomain: &str,
    ) -> Result<Option<Record>, HetznerDnsClientError> {
        let response = self
            .client
            .get(format!("{}/records?zone_id={}", self.api_url, zone_id))
            .header("Auth-API-Token", &self.api_token)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let response: GetRecordsResponse = response.json().await?;
                if let Some(record) = response.records.into_iter().find(|r| r.name == subdomain) {
                    return Ok(Some(record));
                }
            }
            StatusCode::UNAUTHORIZED => return Err(HetznerDnsClientError::InvalidApiToken),
            _ => {
                error!("Failed to resolve records for zone id {}.", zone_id);
                return Err(HetznerDnsClientError::FailedToResolveRecord {
                    record: zone_id.to_string(),
                });
            }
        }

        Ok(None)
    }

    pub async fn update_ip(
        &self,
        name: &str,
        zone_id: &str,
        record_id: &str,
        ip: Ipv4Addr,
    ) -> Result<Record, HetznerDnsClientError> {
        let request_body = UpdateRecordRequest {
            name: name.into(),
            ttl: 60,
            r#type: "A".into(),
            zone_id: zone_id.into(),
            value: ip.to_string(),
        };
        let response = self
            .client
            .put(format!("{}/records/{}", self.api_url, record_id))
            .json(&request_body)
            .header("Auth-API-Token", &self.api_token)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let response: UpdateRecordResponse = response.json().await?;

                Ok(response.record)
            }
            StatusCode::UNAUTHORIZED => Err(HetznerDnsClientError::InvalidApiToken),
            _ => {
                error!("Failed to update IP for record id {}.", record_id);
                Err(HetznerDnsClientError::FaliedToUpdateIp {
                    record: record_id.to_string(),
                })
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum HetznerDnsClientError {
    #[error("We could not authenticate against the API.")]
    InvalidApiToken,
    #[error("Failed to retrieve zone: {zone}")]
    FailedToResolveZone { zone: String },
    #[error("Failed to update ip: {record}")]
    FaliedToUpdateIp { record: String },
    #[error("Failed to retrieve record: {record}")]
    FailedToResolveRecord { record: String },
    #[error("request failed")]
    RequestFailed {
        #[from]
        source: reqwest::Error,
    },
    #[error("internal error")]
    InternalError,
}
