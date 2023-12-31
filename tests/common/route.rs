use axum_web::application::shared::config;

use super::GenericResult;

pub async fn fetch_root(access_token: &str) -> GenericResult<reqwest::StatusCode> {
    let config = config::get();
    let url = format!("{}/", config.service_http_addr());

    let authorization = format!("Bearer {}", access_token);
    let response = reqwest::Client::new()
        .get(&url)
        .header("Authorization", authorization)
        .send()
        .await?;

    let response_status = response.status();
    if response_status == reqwest::StatusCode::OK {
        let body = response.text().await.unwrap();
        let result: serde_json::Value = serde_json::from_str(&body).unwrap();
        let expected = serde_json::json!({"message": "Hello from Axum-Web!"});
        assert_eq!(result, expected);
    }
    Ok(response_status)
}
