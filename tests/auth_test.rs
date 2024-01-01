use axum_web::application::security::jwt_claims::{self, AccessClaims};
use reqwest::StatusCode;

pub mod common;
use common::{auth, route, utils, *};

#[tokio::test]
#[ignore]
async fn login_refresh_logout_tests() {
    // load test configuration
    let config = utils::load_test_config();

    // assert that revoked options are enabled
    assert!(config.jwt_enable_revoked_tokens);

    // try unauthorized access to the root handler
    assert_eq!(
        route::fetch_root("").await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    let (access_token, refresh_token) =
        auth::login(TEST_ADMIN_USERNAME, TEST_ADMIN_PASSWORD_HASH)
            .await
            .unwrap();

    // access to the root handler
    assert_eq!(
        route::fetch_root(&access_token).await.unwrap(),
        StatusCode::OK
    );

    // wait to expire access token
    tokio::time::sleep(tokio::time::Duration::from_secs(
        (config.jwt_expire_access_token_seconds + config.jwt_validation_leeway_seconds + 1) as u64,
    ))
    .await;

    // check the access to the root handler
    assert_eq!(
        route::fetch_root(&access_token).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    // refresh tokens
    let (access_token_new, refresh_token_new) = auth::refresh(&refresh_token).await.unwrap();

    // try access to the root handler with old token
    assert_eq!(
        route::fetch_root(&access_token).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    // try access to the root handler with new token
    assert_eq!(
        route::fetch_root(&access_token_new).await.unwrap(),
        StatusCode::OK
    );

    // try logout with old token
    assert_eq!(
        auth::logout(&refresh_token).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    // logout with new token
    assert_eq!(auth::logout(&refresh_token_new).await.unwrap(), StatusCode::OK);

    // try access to the root handler with new token after logout
    assert_eq!(
        route::fetch_root(&access_token_new).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
#[ignore]
async fn revoke_user_test() {
    // load test configuration
    let config = utils::load_test_config();

    // assert that revoked options are enabled
    assert!(config.jwt_enable_revoked_tokens);

    // try unauthorized access to the root handler
    assert_eq!(
        route::fetch_root("").await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    let (access_token, _) = auth::login(TEST_ADMIN_USERNAME, TEST_ADMIN_PASSWORD_HASH)
        .await
        .unwrap();

    // try authorized access to the root handler
    assert_eq!(
        route::fetch_root(&access_token).await.unwrap(),
        StatusCode::OK
    );

    let access_claims: AccessClaims = jwt_claims::decode_token(&access_token).unwrap();
    let user_id = access_claims.sub;

    assert_eq!(
        auth::revoke_user(&access_token, &user_id).await.unwrap(),
        StatusCode::OK
    );

    // try access to the root handler with the same token again
    assert_eq!(
        route::fetch_root(&access_token).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    // needs pause to pass authentication of next logins
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}

#[tokio::test]
#[ignore]
async fn revoke_all_test() {
    // load test configuration
    let config = utils::load_test_config();

    // assert that revoked options are enabled
    assert!(config.jwt_enable_revoked_tokens);

    // try unauthorized access to the root handler
    assert_eq!(
        route::fetch_root("").await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    let (access_token, _) = auth::login(TEST_ADMIN_USERNAME, TEST_ADMIN_PASSWORD_HASH)
        .await
        .unwrap();

    // try authorized access to the root handler
    assert_eq!(
        route::fetch_root(&access_token).await.unwrap(),
        StatusCode::OK
    );

    auth::revoke_all(&access_token).await.unwrap();

    // try access to the root handler with the same token again
    assert_eq!(
        route::fetch_root(&access_token).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    // needs pause to pass authentication of next logins
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}

#[tokio::test]
#[ignore]
async fn cleanup_test() {
    // load test configuration
    let config = utils::load_test_config();

    // assert that revoked options are enabled
    assert!(config.jwt_enable_revoked_tokens);

    // login
    let (access_token, refresh_token) = auth::login(TEST_ADMIN_USERNAME, TEST_ADMIN_PASSWORD_HASH)
        .await
        .unwrap();

    let _initial_cleanup = auth::cleanup(&access_token).await.unwrap();

    // refresh, expected 2 tokens to expire
    let (_, refresh_token) = auth::refresh(&refresh_token).await.unwrap();

    // logout, expected 2 tokens to expire
    assert_eq!(auth::logout(&refresh_token).await.unwrap(), StatusCode::OK);

    // wait to make sure that tokens expire
    tokio::time::sleep(tokio::time::Duration::from_secs(
        config.jwt_expire_access_token_seconds as u64,
    ))
    .await;
    tokio::time::sleep(tokio::time::Duration::from_secs(
        config.jwt_expire_refresh_token_seconds as u64,
    ))
    .await;

    let (access_token, _) = auth::login(TEST_ADMIN_USERNAME, TEST_ADMIN_PASSWORD_HASH)
        .await
        .unwrap();

    let deleted_tokens = auth::cleanup(&access_token).await.unwrap();
    assert!(deleted_tokens >= 4);
}
