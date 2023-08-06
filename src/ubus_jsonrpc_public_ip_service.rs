use async_trait::async_trait;
use reqwest::Client;
use reqwest::StatusCode;
use serde_json::Value;
use std::net::Ipv4Addr;
use std::str::FromStr;

use serde_json::json;

use crate::public_ip_service::PublicIpService;
use crate::public_ip_service::PublicIpServiceError;

#[derive(Debug)]
struct Token(String);

pub struct UbusJsonRpcPublicIpService {
    ubus_url: String,
    ubus_user: String,
    ubus_secret: String,
    client: Client,
}

impl UbusJsonRpcPublicIpService {
    pub fn new(ubus_url: &str, ubus_user: &str, ubus_secret: &str) -> Self {
        let client = Client::new();
        Self {
            ubus_url: String::from(ubus_url),
            ubus_user: String::from(ubus_user),
            ubus_secret: String::from(ubus_secret),
            client,
        }
    }

    async fn get_token(&self) -> Result<Token, PublicIpServiceError> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "call",
            "params": [
                "00000000000000000000000000000000",
                "session",
                "login",
                {
                    "username": self.ubus_user,
                    "password": self.ubus_secret
                }
            ]
        });
        let response = self
            .client
            .post(&self.ubus_url)
            .json(&request_body)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let response_body: Value = response.json().await?;
                let token = response_body["result"][1]["ubus_rpc_session"]
                    .as_str()
                    .ok_or(PublicIpServiceError::InvalidCredentials)?;
                Ok(Token(String::from(token)))
            }
            _ => Err(PublicIpServiceError::InvalidCredentials),
        }
    }
}

#[async_trait]
impl PublicIpService for UbusJsonRpcPublicIpService {
    async fn get_ip(&self) -> Result<Ipv4Addr, PublicIpServiceError> {
        let token = self.get_token().await?;
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "call",
            "params": [
                token.0,
                "network.interface.wan",
                "status",
                {}
            ]
        });
        let response = self
            .client
            .post(&self.ubus_url)
            .json(&request_body)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let response_body: Value = response.json().await?;
                let ip_str = response_body["result"][1]["ipv4-address"][0]["address"]
                    .as_str()
                    .ok_or(PublicIpServiceError::InvalidIpResponse)?;
                let ip = Ipv4Addr::from_str(ip_str)
                    .map_err(|_| PublicIpServiceError::InvalidIpResponse)?;
                return Ok(ip);
            }
            _ => return Err(PublicIpServiceError::InvalidIpResponse),
        }
    }
}
