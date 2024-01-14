use axum_web::application::config;

use super::GenericResult;

pub async fn fetch_root(access_token: &str) -> GenericResult<reqwest::StatusCode> {
    let url = config::get().service_http_addr();

    let authorization = format!("Bearer {}", access_token);
    let response = reqwest::Client::new()
        .get(&url)
        .header("Authorization", authorization)
        .send()
        .await?;

    let response_status = response.status();
    if response_status == reqwest::StatusCode::OK {
        let found = response.text().await.unwrap();
        let expected = r#"{"message":"Hello from Axum-Web!"}"#;
        assert_eq!(found, expected);
    }
    Ok(response_status)
}
