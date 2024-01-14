use axum_web::application::config;
use reqwest::StatusCode;
use serial_test::serial;

pub mod common;
use common::{auth, route, utils, *};

#[tokio::test]
#[serial]
async fn logout_test() {
    // load the test configuration and start the api server
    utils::start_api().await;
    let config = config::get();

    // assert that revoked options are enabled
    assert!(config.jwt_enable_revoked_tokens);

    // try unauthorized access to the root handler
    assert_eq!(
        route::fetch_root("").await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    let (status, result) = auth::login(TEST_ADMIN_USERNAME, TEST_ADMIN_PASSWORD_HASH)
        .await
        .unwrap();
    assert_eq!(status, StatusCode::OK);
    let (access_token, refresh_token) = result.unwrap();

    // access to the root handler
    assert_eq!(
        route::fetch_root(&access_token).await.unwrap(),
        StatusCode::OK
    );

    // logout
    assert_eq!(auth::logout(&refresh_token).await.unwrap(), StatusCode::OK);

    // try access to the root handler after logout
    assert_eq!(
        route::fetch_root(&access_token).await.unwrap(),
        StatusCode::UNAUTHORIZED
    );
}
