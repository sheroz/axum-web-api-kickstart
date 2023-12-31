use axum_web::application::security::jwt_claims::AccessClaims;
use reqwest::StatusCode;

pub mod common;
use common::{auth, route, test_config, *};

#[tokio::test]
#[ignore]
async fn login_refresh_logout_tests() {
    // load test configuration
    test_config::load_test_config();

    // try unauthorized access to the root handler
    assert_eq!(
        route::fetch_root("").await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    let (access_token1, refresh_token1) =
        auth::login(TEST_ADMIN_USERNAME, TEST_ADMIN_PASSWORD_HASH).await;

    // try authorized access to the root handler
    assert_eq!(
        route::fetch_root(&access_token1).await.unwrap(),
        StatusCode::OK
    );

    let (access_token2, refresh_token2) = auth::refresh(&refresh_token1).await.unwrap();

    // try access to the root handler with old token
    assert_eq!(
        route::fetch_root(&access_token1).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    // try access to the root handler with new token
    assert_eq!(
        route::fetch_root(&access_token2).await.unwrap(),
        StatusCode::OK
    );

    // try logout with old token
    assert_eq!(
        auth::logout(&refresh_token1).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    // try logout with new token
    assert_eq!(auth::logout(&refresh_token2).await.unwrap(), StatusCode::OK);

    // try access to the root handler with new token
    assert_eq!(
        route::fetch_root(&access_token2).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
#[ignore]
async fn revoke_user_test() {
    // need pause to ignore previous revoke results
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // load test configuration
    let config = test_config::load_test_config();

    // try unauthorized access to the root handler
    assert_eq!(
        route::fetch_root("").await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    let (access_token1, _refresh_token1) =
        auth::login(TEST_ADMIN_USERNAME, TEST_ADMIN_PASSWORD_HASH).await;

    let access_claims = jsonwebtoken::decode::<AccessClaims>(
        &access_token1,
        &config.jwt_keys.decoding,
        &jsonwebtoken::Validation::default(),
    )
    .unwrap()
    .claims;

    let user_id = access_claims.sub;
    auth::revoke_user(&access_token1, &user_id).await.unwrap();

    // try access to the root handler with the same token again
    assert_eq!(
        route::fetch_root(&access_token1).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    // need pause to pass next tests
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

}

#[tokio::test]
#[ignore]
async fn revoke_all_test() {
    // need pause to ignore previous revoke results
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // load test configuration
    test_config::load_test_config();

    // try unauthorized access to the root handler
    assert_eq!(
        route::fetch_root("").await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    let (access_token1, _refresh_token1) =
        auth::login(TEST_ADMIN_USERNAME, TEST_ADMIN_PASSWORD_HASH).await;

    auth::revoke_all(&access_token1).await.unwrap();

    // try access to the root handler with the same token again
    assert_eq!(
        route::fetch_root(&access_token1).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    // need pause to pass next tests
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
