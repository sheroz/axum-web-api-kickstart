use axum_web::application::shared::config;

use super::GenericResult;

pub async fn login(username: &str, password_hash: &str) -> (String, String) {
    let url = format!("{}/auth/login", config::get().service_http_addr());
    let params = serde_json::json!(
        {
            "username": username,
            "password_hash": password_hash
        }
    );

    let response = reqwest::Client::new()
        .post(&url)
        .header("Accept", "application/json")
        .json(&params)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    let access_token = json["access_token"].as_str().unwrap().to_string();
    let refresh_token = json["refresh_token"].as_str().unwrap().to_string();

    assert!(!access_token.is_empty());
    assert!(!refresh_token.is_empty());

    (access_token, refresh_token)
}

pub async fn refresh(refresh_token: &str) -> GenericResult<(String, String)> {
    let url = format!("{}/auth/refresh", config::get().service_http_addr());

    let authorization = format!("Bearer {}", refresh_token);
    let response = reqwest::Client::new()
        .post(&url)
        .header("Accept", "application/json")
        .header("Authorization", authorization)
        .send()
        .await?;

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    let access_token = json["access_token"].as_str().unwrap().to_string();
    let refresh_token = json["refresh_token"].as_str().unwrap().to_string();

    assert!(!access_token.is_empty());
    assert!(!refresh_token.is_empty());

    Ok((access_token, refresh_token))
}

pub async fn logout(refresh_token: &str) -> GenericResult<reqwest::StatusCode> {
    let url = format!("{}/auth/logout", config::get().service_http_addr());

    let authorization = format!("Bearer {}", refresh_token);
    let response = reqwest::Client::new()
        .post(&url)
        .header("Accept", "application/json")
        .header("Authorization", authorization)
        .send()
        .await?;

    Ok(response.status())
}

pub async fn revoke_all(access_token: &str) -> GenericResult<reqwest::StatusCode> {
    let url = format!("{}/auth/revoke_all", config::get().service_http_addr());

    let authorization = format!("Bearer {}", access_token);
    let response = reqwest::Client::new()
        .post(&url)
        .header("Accept", "application/json")
        .header("Authorization", authorization)
        .send()
        .await?;

    Ok(response.status())
}

pub async fn revoke_user(access_token: &str, user_id: &str) -> GenericResult<reqwest::StatusCode> {
    let url = format!("{}/auth/revoke_user", config::get().service_http_addr());
    let params = serde_json::json!(
        {
            "user_id": user_id
        }
    );
    let authorization = format!("Bearer {}", access_token);
    let response = reqwest::Client::new()
        .post(&url)
        .header("Accept", "application/json")
        .header("Authorization", authorization)
        .json(&params)
        .send()
        .await?;

    Ok(response.status())
}

pub async fn clean_up(access_token: &str) -> GenericResult<reqwest::StatusCode> {
    let url = format!("{}/auth/clean-up", config::get().service_http_addr());

    let authorization = format!("Bearer {}", access_token);
    let response = reqwest::Client::new()
        .post(&url)
        .header("Accept", "application/json")
        .header("Authorization", authorization)
        .send()
        .await?;

    Ok(response.status())
}
