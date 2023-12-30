use axum_web::application::shared::config;
use reqwest::StatusCode;
type GenericResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::test]
#[ignore]
async fn login_refresh_logout_test() {
    // load configuration
    config::load_from_dotenv();

    // try unauthorized access to the root handler
    assert_eq!(fetch_root("").await.unwrap(), StatusCode::UNAUTHORIZED);

    let username = "admin";
    let password_hash = "7c44575b741f02d49c3e988ba7aa95a8fb6d90c0ef63a97236fa54bfcfbd9d51";

    let (access_token1, refresh_token1) = login(username, password_hash).await;

    // try authorized access to the root handler
    assert_eq!(fetch_root(&access_token1).await.unwrap(), StatusCode::OK);

    let (access_token2, refresh_token2) = refresh(&refresh_token1).await.unwrap();

    // try access to the root handler with old token
    assert_eq!(
        fetch_root(&access_token1).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    // try access to the root handler with new token
    assert_eq!(fetch_root(&access_token2).await.unwrap(), StatusCode::OK);

    // try logout with old token
    assert_eq!(
        logout(&refresh_token1).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    // try logout with new token
    assert_eq!(logout(&refresh_token2).await.unwrap(), StatusCode::OK);

    // try access to the root handler with new token
    assert_eq!(
        fetch_root(&access_token2).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );
}

async fn fetch_root(access_token: &str) -> GenericResult<reqwest::StatusCode> {
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

async fn login(username: &str, password_hash: &str) -> (String, String) {
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

async fn refresh(refersh_token: &str) -> GenericResult<(String, String)> {
    let url = format!("{}/auth/refresh", config::get().service_http_addr());

    let response = reqwest::Client::new()
        .post(&url)
        .header("Accept", "application/json")
        .body(refersh_token.to_string())
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

async fn logout(refresh_token: &str) -> GenericResult<reqwest::StatusCode> {
    let url = format!("{}/auth/logout", config::get().service_http_addr());

    let response = reqwest::Client::new()
        .post(&url)
        .header("Accept", "application/json")
        .body(refresh_token.to_string())
        .send()
        .await?;

    Ok(response.status())
}
