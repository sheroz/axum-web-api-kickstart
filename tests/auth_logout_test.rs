use reqwest::StatusCode;

pub mod common;
use common::{auth, route, utils, *};

#[tokio::test]
async fn logout_test() {
    // run the api server
    utils::api_run().await;

    // load test configuration
    let config = utils::load_test_config();

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
