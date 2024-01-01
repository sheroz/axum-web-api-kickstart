use uuid::Uuid;

pub mod common;
use common::{fetch, utils};
const PATH_HEARTBEAT: &str = "heartbeat";
#[tokio::test]
#[ignore]
async fn heartbeat_test() {
    // load test configuration
    utils::load_test_config();

    let heartbeat_id = Uuid::new_v4().to_string();
    let url = utils::build_url(PATH_HEARTBEAT, &heartbeat_id);

    // fetch using reqwest
    let response = reqwest::get(url.as_str()).await.unwrap();
    let body = response.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["service"], "axum-web");
    assert_eq!(json["version"], "1");
    assert_eq!(json["heartbeat-id"], heartbeat_id);

    // fetch using hyper
    let body = fetch::fetch_url_hyper(url.as_str()).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["service"], "axum-web");
    assert_eq!(json["version"], "1");
    assert_eq!(json["heartbeat-id"], heartbeat_id);
}
