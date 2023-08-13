use async_trait::async_trait;
use reqwest::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;
use std::net::Ipv4Addr;

use uuid::Uuid;

use crate::public_ip_service::PublicIpService;
use crate::public_ip_service::PublicIpServiceError;

const JSONRPC2: &str = "2.0";
const METHOD_CALL: &str = "call";
const NULL_SESSION: &str = "00000000000000000000000000000000";

#[derive(Debug)]
pub struct SessionResponse {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct Ipv4AddressInfo {
    pub address: Ipv4Addr,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NetworkInterfaceStatusResponse {
    pub ipv4_address: Vec<Ipv4AddressInfo>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UbusJsonResponseContainer {
    pub jsonrpc: String,
    pub id: Value,
    pub result: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UbusJsonRequestContainer {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    pub params: Vec<Value>,
}

impl UbusJsonRequestContainer {
    fn new(session: Value, ubus_object: Value, ubus_method: Value, params: Vec<Value>) -> Self {
        let mut call_params = vec![session, ubus_object, ubus_method];
        call_params.extend(params);
        Self {
            params: call_params,
            ..Default::default()
        }
    }

    fn login(username: &str, password: &str) -> Self {
        let login_params = LoginParams {
            username: username.to_string(),
            password: password.to_string(),
        };
        let login_params = serde_json::to_value(login_params).unwrap();

        UbusJsonRequestContainer::new(
            Value::String(NULL_SESSION.to_string()),
            Value::String("session".to_string()),
            Value::String("login".to_string()),
            vec![login_params],
        )
    }

    fn network_interface_status(token: String) -> Self {
        UbusJsonRequestContainer::new(
            Value::String(token),
            Value::String("network.interface.wan".to_string()),
            Value::String("status".to_string()),
            vec![Value::Object(Map::new())],
        )
    }

    #[cfg(test)]
    fn get_command_params(&self) -> Vec<Value> {
        self.params.to_owned()
    }
}

impl Default for UbusJsonRequestContainer {
    fn default() -> Self {
        Self {
            jsonrpc: JSONRPC2.to_string(),
            id: serde_json::to_value(Uuid::new_v4()).unwrap(),
            method: METHOD_CALL.to_string(),
            params: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct LoginParams {
    username: String,
    password: String,
}

pub struct UbusJsonRpcClient {
    ubus_url: String,
    ubus_user: String,
    ubus_secret: String,
    client: Client,
}

impl UbusJsonRpcClient {
    pub fn new(ubus_url: &str, ubus_user: &str, ubus_secret: &str) -> Self {
        let client = Client::new();
        Self {
            ubus_url: String::from(ubus_url),
            ubus_user: String::from(ubus_user),
            ubus_secret: String::from(ubus_secret),
            client,
        }
    }

    pub async fn get_session(&self) -> Result<SessionResponse, PublicIpServiceError> {
        let login_request = UbusJsonRequestContainer::login(&self.ubus_user, &self.ubus_secret);
        let response = self
            .client
            .post(&self.ubus_url)
            .json(&login_request)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let json: Value = serde_json::from_value(response.json().await?).unwrap();
                let response_body: UbusJsonResponseContainer = serde_json::from_value(json)
                    .map_err(|_| PublicIpServiceError::InternalError)?;
                let session = response_body.result[1]["ubus_rpc_session"]
                    .as_str()
                    .ok_or(PublicIpServiceError::InvalidCredentials)?;
                Ok(SessionResponse {
                    token: String::from(session),
                })
            }
            _ => Err(PublicIpServiceError::InvalidCredentials),
        }
    }

    pub async fn get_ip(&self) -> Result<NetworkInterfaceStatusResponse, PublicIpServiceError> {
        let session = self.get_session().await?;
        let status_request = UbusJsonRequestContainer::network_interface_status(session.token);
        let response = self
            .client
            .post(&self.ubus_url)
            .json(&status_request)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let json: Value = serde_json::from_value(response.json().await?).unwrap();
                let response_body: UbusJsonResponseContainer = serde_json::from_value(json)
                    .map_err(|_| PublicIpServiceError::InternalError)?;
                let status = serde_json::from_value(response_body.result[1].clone())
                    .map_err(|_| PublicIpServiceError::InvalidIpResponse)?;
                Ok(status)
            }
            _ => Err(PublicIpServiceError::InvalidIpResponse),
        }
    }
}

#[async_trait]
impl PublicIpService for UbusJsonRpcClient {
    async fn get_ip(&self) -> Result<Ipv4Addr, PublicIpServiceError> {
        Ok(self.get_ip().await?.ipv4_address[0].address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ubus_container_new() {
        let login_params = LoginParams {
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        let login_params = serde_json::to_value(login_params).unwrap();

        let login_call = UbusJsonRequestContainer::new(
            Value::String(NULL_SESSION.to_string()),
            Value::String("session".to_string()),
            Value::String("login".to_string()),
            vec![login_params.clone()],
        );

        let session: Value = Value::String(NULL_SESSION.to_string());
        let ubus_object = Value::String("session".to_string());
        let ubus_method = Value::String("login".to_string());

        assert_eq!(
            login_call,
            UbusJsonRequestContainer {
                id: login_call.id.to_owned(),
                params: vec![session, ubus_object, ubus_method, login_params],
                ..Default::default()
            }
        )
    }

    #[tokio::test]
    async fn ubus_container_login() {
        let login_params = LoginParams {
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        let login_params = serde_json::to_value(login_params).unwrap();

        let login_call = UbusJsonRequestContainer::login("user", "pass");

        let params = login_call.get_command_params();

        assert_eq!(
            params[0].as_str().unwrap(),
            "00000000000000000000000000000000"
        );
        assert_eq!(params[1].as_str().unwrap(), "session");
        assert_eq!(params[2].as_str().unwrap(), "login");
        assert_eq!(params[3], serde_json::to_value(login_params).unwrap())
    }

    #[tokio::test]
    async fn ubus_container_network_interface_status() {
        let login_call = UbusJsonRequestContainer::network_interface_status("session".to_string());

        let params = login_call.get_command_params();

        assert_eq!(params[0].as_str().unwrap(), "session");
        assert_eq!(params[1].as_str().unwrap(), "network.interface.wan");
        assert_eq!(params[2].as_str().unwrap(), "status");
        assert_eq!(params[3], serde_json::to_value(Map::new()).unwrap());
    }
}
