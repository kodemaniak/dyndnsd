use dyndnsd::hetzner_dns_client::{
    GetRecordsResponse, GetZonesResponse, HetznerDnsClient, Record, UpdateRecordResponse, Zone,
};
use wiremock::matchers::{headers, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const API_TOKEN: &str = "XXX";
const ZONE: &str = "example.com";
const SUBDOMAIN: &str = "example.com";
const EXPECTED_ZONE_ID: &str = "zid123";
const EXPECTED_RECORD_ID: &str = "rid123";
const OLD_IP: &str = "127.0.0.2";
const NEW_IP: &str = "127.0.0.3";

#[tokio::test]
async fn test_get_zone_id() {
    let mock_server = MockServer::start().await;

    let uri = &mock_server.uri();

    let zone = Zone {
        name: String::from(ZONE),
        id: String::from(EXPECTED_ZONE_ID),
    };

    let zones_response = GetZonesResponse {
        zones: vec![zone.clone()],
    };
    Mock::given(method("GET"))
        .and(path("/zones"))
        .and(query_param("name", ZONE))
        .and(headers("Auth-API-Token", vec![API_TOKEN]))
        .respond_with(ResponseTemplate::new(200).set_body_json(zones_response))
        .mount(&mock_server)
        .await;

    let client = HetznerDnsClient::new_with_url(API_TOKEN, uri);
    let returned_zone = client
        .find_zone(ZONE)
        .await
        .expect("find zone id failed")
        .expect("no zone id returned");
    assert_eq!(returned_zone, zone);
}

#[tokio::test]
async fn test_get_record() {
    let mock_server = MockServer::start().await;

    let uri = &mock_server.uri();

    let record = Record {
        id: String::from(EXPECTED_RECORD_ID),
        zone_id: String::from(EXPECTED_ZONE_ID),
        name: String::from(SUBDOMAIN),
        r#type: String::from("A"),
        value: String::from(OLD_IP),
    };

    let records_response = GetRecordsResponse {
        records: vec![record],
    };
    Mock::given(method("GET"))
        .and(path("/records"))
        .and(query_param("zone_id", EXPECTED_ZONE_ID))
        .and(headers("Auth-API-Token", vec![API_TOKEN]))
        .respond_with(ResponseTemplate::new(200).set_body_json(records_response))
        .mount(&mock_server)
        .await;

    let client = HetznerDnsClient::new_with_url(API_TOKEN, uri);
    let record = client
        .find_record(EXPECTED_ZONE_ID, SUBDOMAIN)
        .await
        .expect("find record id failed")
        .expect("no record id returned");
    assert_eq!(record.id, EXPECTED_RECORD_ID);
}

#[tokio::test]
async fn test_update_record() {
    let mock_server = MockServer::start().await;

    let uri = &mock_server.uri();

    let record = Record {
        id: String::from(EXPECTED_RECORD_ID),
        zone_id: String::from(EXPECTED_ZONE_ID),
        name: String::from(SUBDOMAIN),
        r#type: String::from("A"),
        value: String::from(OLD_IP),
    };

    let update_response = UpdateRecordResponse { record };
    Mock::given(method("PUT"))
        .and(path(format!("/records/{}", EXPECTED_RECORD_ID)))
        .and(headers("Auth-API-Token", vec![API_TOKEN]))
        .respond_with(ResponseTemplate::new(200).set_body_json(update_response))
        .mount(&mock_server)
        .await;

    let client = HetznerDnsClient::new_with_url(API_TOKEN, uri);
    let record = client
        .update_ip(
            SUBDOMAIN,
            EXPECTED_ZONE_ID,
            EXPECTED_RECORD_ID,
            NEW_IP.parse().unwrap(),
        )
        .await
        .expect("find record id failed");
    assert_eq!(record.id, EXPECTED_RECORD_ID);
}
