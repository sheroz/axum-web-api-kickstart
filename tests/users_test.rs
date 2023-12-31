pub mod common;
use common::{*, auth, route, test_config};
use reqwest::StatusCode;

#[tokio::test]
#[ignore]
async fn list_users_test() {
    // load test configuration
    test_config::load_test_config();

    // try unauthorized access to the users handler
    let (status, _users) = users::list_users("").await.unwrap();
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    assert_eq!(
        route::fetch_root("").await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    let (access_token1, _refresh_token1) = auth::login(TEST_ADMIN_USERNAME, TEST_ADMIN_PASSWORD_HASH).await;

    // try authorized access to the users handler
    let (status, users) = users::list_users(&access_token1).await.unwrap();
    assert_eq!(status, reqwest::StatusCode::OK);
    assert!(!users.is_empty());
}
