use uuid::Uuid;

pub mod common;
use common::{fetch, test_config};

#[tokio::test]
#[ignore]
async fn heartbeat_test() {
    // load test configuration
    let config = test_config::load_test_config();

    let heartbeat_id = Uuid::new_v4().to_string();
    let url = format!("{}/heartbeat/{}", config.service_http_addr(), heartbeat_id);

    // fetch using reqwest
    let response = reqwest::get(&url).await.unwrap();
    let body = response.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["service"], "axum-web");
    assert_eq!(json["version"], "1");
    assert_eq!(json["heartbeat-id"], heartbeat_id);

    // fetch using hyper
    let body = fetch::fetch_url_hyper(&url).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["service"], "axum-web");
    assert_eq!(json["version"], "1");
    assert_eq!(json["heartbeat-id"], heartbeat_id);
}
