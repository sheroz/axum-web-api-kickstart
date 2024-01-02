use reqwest::StatusCode;

pub mod common;
use common::{auth, route, utils, *};

#[tokio::test]
#[ignore]
async fn login_test() {
    // load test configuration
    utils::load_test_config();

    // try unauthorized access to the root handler
    assert_eq!(
        route::fetch_root("").await.unwrap(),
        StatusCode::UNAUTHORIZED
    );

    let username_wrong = format!("{}1", TEST_ADMIN_USERNAME);
    let (status, _) = auth::login(&username_wrong, TEST_ADMIN_PASSWORD_HASH)
        .await
        .unwrap();
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let password_wrong = format!("{}1", TEST_ADMIN_PASSWORD_HASH);
    let (status, _) = auth::login(TEST_ADMIN_USERNAME, &password_wrong)
        .await
        .unwrap();
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = auth::login(&username_wrong, &password_wrong).await.unwrap();
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, result) = auth::login(TEST_ADMIN_USERNAME, TEST_ADMIN_PASSWORD_HASH)
        .await
        .unwrap();
    assert_eq!(status, StatusCode::OK);
    let (access_token, _) = result.unwrap();

    // access to the root handler
    assert_eq!(
        route::fetch_root(&access_token).await.unwrap(),
        StatusCode::OK
    );
}
