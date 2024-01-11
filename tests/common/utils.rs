use axum_web::application::config;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::{timeout_at, Instant};

pub async fn start_api() {
    std::env::set_var("ENV_TEST", "1");
    config::load();

    let (api_ready_tx, api_ready_rx) = oneshot::channel();

    // run the api server
    tokio::spawn(async move {
        axum_web::application::app::start_server(api_ready_tx).await;
    });

    let service_start_timeout = Instant::now() + Duration::from_secs(5);
    if (timeout_at(service_start_timeout, api_ready_rx).await).is_err() {
        println!("Could not start API Service in 5 seconds");
    }
}

pub fn build_url(version: &str, path: &str, url: &str) -> reqwest::Url {
    let url = format!(
        "{}/{}/{}/{}",
        config::get().service_http_addr(),
        version,
        path,
        url
    );
    reqwest::Url::parse(&url).unwrap()
}

pub fn build_path(version: &str, path: &str) -> reqwest::Url {
    let url = format!("{}/{}/{}", config::get().service_http_addr(), version, path);
    reqwest::Url::parse(&url).unwrap()
}
