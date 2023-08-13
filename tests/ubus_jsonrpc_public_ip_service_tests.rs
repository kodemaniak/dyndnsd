use dyndnsd::ubus_jsonrpc_public_ip_service::UbusJsonRpcClient;
use serde_json::json;
use std::net::Ipv4Addr;
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_token() {
    let mock_server = MockServer::start().await;

    let uri = &mock_server.uri();

    let expected = json!(
           {
                "params": [
                    "00000000000000000000000000000000",
                    "session",
                    "login",
                    {
                        "username": "user",
                        "password": "pass"
                    }
                ]
            }
    );

    Mock::given(method("POST"))
        .and(body_partial_json(expected))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "result": [
                0,
                {
                    "ubus_rpc_session": "session"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = UbusJsonRpcClient::new(uri, "user", "pass");
    assert_eq!(
        client.get_session().await.expect("get token failed").token,
        "session".to_string(),
    );
}

#[tokio::test]
async fn test_get_ip() {
    let mock_server = MockServer::start().await;

    let uri = &mock_server.uri();

    let expected_login = json!(
           {
                "params": [
                    "00000000000000000000000000000000",
                    "session",
                    "login",
                    {
                        "username": "user",
                        "password": "pass"
                    }
                ]
            }
    );

    Mock::given(method("POST"))
        .and(body_partial_json(expected_login))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "result": [
                0,
                {
                    "ubus_rpc_session": "session"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let expected = json!(
           {
                "params": [
                    "session",
                    "network.interface.wan",
                    "status",
                    {}
                ]
            }
    );

    Mock::given(method("POST"))
        .and(body_partial_json(expected))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(
            {
                "id": null,
                "jsonrpc": "2.0",
                "result": [
                    0,
                    {
                        "autostart": true,
                        "available": true,
                        "data": {},
                        "delegation": true,
                        "device": "br-wan",
                        "dns-search": [],
                        "dns-server": [
                            "10.0.0.2",
                            "10.0.0.3"
                        ],
                        "dns_metric": 0,
                        "dynamic": false,
                        "inactive": {
                            "dns-search": [],
                            "dns-server": [],
                            "ipv4-address": [],
                            "ipv6-address": [],
                            "neighbors": [],
                            "route": []
                        },
                        "ipv4-address": [
                            {
                                "address": "192.168.1.100",
                                "mask": 32,
                                "ptpaddress": "10.0.0.1"
                            }
                        ],
                        "ipv6-address": [],
                        "ipv6-prefix": [],
                        "ipv6-prefix-assignment": [],
                        "l3_device": "pppoe-wan",
                        "metric": 0,
                        "neighbors": [],
                        "pending": false,
                        "proto": "pppoe",
                        "route": [
                            {
                                "mask": 0,
                                "nexthop": "10.0.0.1",
                                "source": "0.0.0.0/0",
                                "target": "0.0.0.0"
                            }
                        ],
                        "up": true,
                        "updated": [
                            "addresses",
                            "routes"
                        ],
                        "uptime": 20403
                    }
                ]
            }
        )))
        .mount(&mock_server)
        .await;

    let client = UbusJsonRpcClient::new(uri, "user", "pass");
    assert_eq!(
        client.get_ip().await.expect("get ip failed").ipv4_address[0].address,
        "192.168.1.100".parse::<Ipv4Addr>().unwrap(),
    );
}
