use async_trait::async_trait;
use serde_json::Value;
use std::net::Ipv4Addr;
use std::str::FromStr;

use serde_json::json;
use surf::Client;
use surf::StatusCode;

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
        let client = surf::Client::new();
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
        let request = self.client.post(&self.ubus_url).body(request_body).build();
        let mut response = self.client.send(request).await?;

        match response.status() {
            StatusCode::Ok => {
                let response_body: Value = response.body_json().await?;
                let token = response_body["result"][1]["ubus_rpc_session"]
                    .as_str()
                    .ok_or(PublicIpServiceError::UnknownError)?;
                return Ok(Token(String::from(token)));
            }
            _ => return Err(PublicIpServiceError::UnknownError),
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
        let request = self.client.post(&self.ubus_url).body(request_body).build();
        let mut response = self.client.send(request).await?;

        match response.status() {
            StatusCode::Ok => {
                let response_body: Value = response.body_json().await?;
                let ip_str = response_body["result"][1]["ipv4-address"][0]["address"]
                    .as_str()
                    .ok_or(PublicIpServiceError::UnknownError)?;
                let ip =
                    Ipv4Addr::from_str(ip_str).map_err(|_| PublicIpServiceError::UnknownError)?;
                return Ok(ip);
            }
            _ => return Err(PublicIpServiceError::UnknownError),
        }
    }
}

impl From<surf::Error> for PublicIpServiceError {
    fn from(_: surf::Error) -> Self {
        PublicIpServiceError::UnknownError
    }
}
