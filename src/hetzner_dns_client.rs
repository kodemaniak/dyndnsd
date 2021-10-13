use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use surf::{Body, StatusCode};

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

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
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
    client: surf::Client,
}

#[cfg_attr(test, automock)]
impl HetznerDnsClient {
    pub fn new(api_token: &str) -> Self {
        Self::new_with_url(api_token, API_URL)
    }

    pub fn new_with_url(api_token: &str, api_url: &str) -> Self {
        let client = surf::Client::new();
        Self {
            api_url: String::from(api_url),
            api_token: String::from(api_token),
            client,
        }
    }

    pub async fn find_zone(&self, domain: &str) -> Result<Option<Zone>, HetznerDnsClientError> {
        let request = self
            .client
            .get(format!("{}/zones?name={}", self.api_url, domain))
            .header("Auth-API-Token", &self.api_token)
            .build();
        let response = self.client.send(request).await;
        let mut response = response?;

        match response.status() {
            StatusCode::Ok => {
                let response: GetZonesResponse = response.body_json().await?;

                if response.zones.len() > 1 {
                    return Err(HetznerDnsClientError::AmbiguousZones);
                }

                let maybe_zone = response.zones.into_iter().next();

                return Ok(maybe_zone);
            }
            StatusCode::Unauthorized => return Err(HetznerDnsClientError::InvalidApiToken),
            _ => return Err(HetznerDnsClientError::UnknownError),
        }
    }

    pub async fn find_record(
        &self,
        zone_id: &str,
        subdomain: &str,
    ) -> Result<Option<Record>, HetznerDnsClientError> {
        let request = self
            .client
            .get(format!("{}/records?zone_id={}", self.api_url, zone_id))
            .header("Auth-API-Token", &self.api_token)
            .build();
        let mut response = self.client.send(request).await?;

        match response.status() {
            StatusCode::Ok => {
                let response: GetRecordsResponse = response.body_json().await?;
                if let Some(record) = response.records.into_iter().find(|r| r.name == subdomain) {
                    return Ok(Some(record));
                }
            }
            StatusCode::Unauthorized => return Err(HetznerDnsClientError::InvalidApiToken),
            _ => return Err(HetznerDnsClientError::UnknownError),
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
        let request = self
            .client
            .put(format!("{}/records/{}", self.api_url, record_id))
            .body(Body::from_json(&request_body)?)
            .header("Auth-API-Token", &self.api_token)
            .build();
        let response = self.client.send(request).await;
        let mut response = response?;

        match response.status() {
            StatusCode::Ok => {
                let response: UpdateRecordResponse = response.body_json().await?;

                return Ok(response.record);
            }
            StatusCode::Unauthorized => return Err(HetznerDnsClientError::InvalidApiToken),
            _ => return Err(HetznerDnsClientError::UnknownError),
        }
    }
}

impl From<surf::Error> for HetznerDnsClientError {
    fn from(_: surf::Error) -> Self {
        HetznerDnsClientError::UnknownError
    }
}

#[derive(Debug)]
pub enum HetznerDnsClientError {
    AmbiguousZones,
    InvalidApiToken,
    UnknownError,
    UnknownZone,
}
