use axum_web::application::app_const::*;
use serial_test::serial;
use uuid::Uuid;

pub mod common;
use common::{fetch, utils};

const PATH_HEARTBEAT: &str = "heartbeat";
const API_V1: &str = "v1";

#[tokio::test]
#[serial]
async fn heartbeat_test() {
    // load the test configuration and start the api server
    utils::start_api().await;

    let heartbeat_id = Uuid::new_v4().to_string();
    let url = utils::build_url(API_V1, PATH_HEARTBEAT, &heartbeat_id);

    // fetch using reqwest
    let response = reqwest::get(url.as_str()).await.unwrap();
    let body = response.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["service"], SERVICE_NAME);
    assert_eq!(json["version"], SERVICE_VERSION);
    assert_eq!(json["heartbeat-id"], heartbeat_id);

    // fetch using hyper
    let body = fetch::fetch_url_hyper(url.as_str()).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["service"], SERVICE_NAME);
    assert_eq!(json["version"], SERVICE_VERSION);
    assert_eq!(json["heartbeat-id"], heartbeat_id);
}
