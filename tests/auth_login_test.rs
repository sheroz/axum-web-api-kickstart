use reqwest::StatusCode;
use serial_test::serial;

pub mod common;
use common::{auth, route, utils, *};

#[tokio::test]
#[serial]
async fn login_test() {
    // load the test configuration and start the api server
    utils::start_api().await;

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
