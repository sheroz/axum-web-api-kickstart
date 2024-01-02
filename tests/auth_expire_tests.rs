use reqwest::StatusCode;

pub mod common;
use common::{auth, route, utils, *};

#[tokio::test]
#[ignore]
async fn access_token_expire_test() {
    // load test configuration
    let config = utils::load_test_config();

    // assert that revoked options are enabled
    assert!(config.jwt_enable_revoked_tokens);

    let (status, result) = auth::login(TEST_ADMIN_USERNAME, TEST_ADMIN_PASSWORD_HASH)
        .await
        .unwrap();
    assert_eq!(status, StatusCode::OK);
    let (access_token, refresh_token) = result.unwrap();

    // wait to expire access token
    tokio::time::sleep(tokio::time::Duration::from_secs(
        (config.jwt_expire_access_token_seconds + config.jwt_validation_leeway_seconds + 1) as u64,
    ))
    .await;

    // check the access to the root handler with expired token
    assert_eq!(
        route::fetch_root(&access_token).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    // refresh tokens
    let (status, result) = auth::refresh(&refresh_token).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    let (access_token_new, _) = result.unwrap();

    // try access to the root handler with new token
    assert_eq!(
        route::fetch_root(&access_token_new).await.unwrap(),
        StatusCode::OK
    );
}

#[tokio::test]
#[ignore]
async fn refresh_token_expire_test() {
    // load test configuration
    let config = utils::load_test_config();

    // assert that revoked options are enabled
    assert!(config.jwt_enable_revoked_tokens);

    // login
    let (status, result) = auth::login(TEST_ADMIN_USERNAME, TEST_ADMIN_PASSWORD_HASH)
        .await
        .unwrap();
    assert_eq!(status, StatusCode::OK);
    let (_, refresh_token) = result.unwrap();

    // wait to expire refresh token
    tokio::time::sleep(tokio::time::Duration::from_secs(
        (config.jwt_expire_refresh_token_seconds + config.jwt_validation_leeway_seconds + 1) as u64,
    ))
    .await;

    // try to refresh with expired token
    let (status, _) = auth::refresh(&refresh_token).await.unwrap();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
