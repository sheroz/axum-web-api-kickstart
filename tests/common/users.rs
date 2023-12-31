use axum_web::{application::shared::config, domain::models::user::User};

use super::GenericResult;

pub async fn list_users(access_token: &str) -> GenericResult<(reqwest::StatusCode, Vec<User>)> {
    let url = format!("{}/users", config::get().service_http_addr());
    let authorization = format!("Bearer {}", access_token);
    let response = reqwest::Client::new()
        .get(&url)
        .header("Accept", "application/json")
        .header("Authorization", authorization)
        .send()
        .await?;

    let status = response.status();
    if status == reqwest::StatusCode::OK {
        let body = response.text().await.unwrap();
        let users: Vec<User> = serde_json::from_str(&body).unwrap();
        return Ok((status, users));
    }
    Ok((status, vec![]))
}
