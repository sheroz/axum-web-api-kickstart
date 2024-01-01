use crate::common::utils;

use super::GenericResult;

const PATH_AUTH: &str = "auth";

pub async fn login(username: &str, password_hash: &str) -> GenericResult<(String, String)> {
    let url = utils::build_url(PATH_AUTH, "login");

    let params = format!(
        "{{\"username\":\"{}\", \"password_hash\":\"{}\"}}",
        username, password_hash
    );

    let response = reqwest::Client::new()
        .post(url.as_str())
        .header("Accept", "application/json")
        .header("Content-type", "application/json; charset=utf8")
        .body(params)
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

pub async fn refresh(refresh_token: &str) -> GenericResult<(String, String)> {
    let url = utils::build_url(PATH_AUTH, "refresh");

    let authorization = format!("Bearer {}", refresh_token);
    let response = reqwest::Client::new()
        .post(url.as_str())
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
    let url = utils::build_url(PATH_AUTH, "logout");

    let authorization = format!("Bearer {}", refresh_token);
    let response = reqwest::Client::new()
        .post(url.as_str())
        .header("Accept", "application/json")
        .header("Authorization", authorization)
        .send()
        .await?;

    Ok(response.status())
}

pub async fn revoke_all(access_token: &str) -> GenericResult<reqwest::StatusCode> {
    let url = utils::build_url(PATH_AUTH, "revoke-all");
    let authorization = format!("Bearer {}", access_token);
    let response = reqwest::Client::new()
        .post(url.as_str())
        .header("Accept", "application/json")
        .header("Authorization", authorization)
        .send()
        .await?;
    Ok(response.status())
}

pub async fn revoke_user(access_token: &str, user_id: &str) -> GenericResult<reqwest::StatusCode> {
    let url = utils::build_url(PATH_AUTH, "revoke-user");
    let params = format!("{{\"user_id\":\"{}\"}}", user_id);
    let authorization = format!("Bearer {}", access_token);
    let response = reqwest::Client::new()
        .post(url.as_str())
        .header("Accept", "application/json")
        .header("Content-type", "application/json; charset=utf8")
        .header("Authorization", authorization)
        .body(params)
        .send()
        .await?;
    Ok(response.status())
}

pub async fn cleanup(access_token: &str) -> GenericResult<u64> {
    let url = utils::build_url(PATH_AUTH, "cleanup");
    let authorization = format!("Bearer {}", access_token);
    let response = reqwest::Client::new()
        .post(url.as_str())
        .header("Accept", "application/json")
        .header("Authorization", authorization)
        .send()
        .await?;

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    let deleted_tokens = json["deleted_tokens"].as_u64().unwrap();

    Ok(deleted_tokens)
}
