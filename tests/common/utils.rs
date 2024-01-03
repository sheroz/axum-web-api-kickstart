use axum_web::application::shared::config;

pub fn load_test_config() -> &'static config::Config {
    std::env::set_var("ENV_TEST", "1");
    config::load();
    config::get()
}

pub fn build_url(version: &str, path: &str, url: &str) -> reqwest::Url {
    let url = format!("{}/{}/{}/{}",config::get().service_http_addr(), version, path, url);
    reqwest::Url::parse(&url).unwrap()
}

pub fn build_path(version: &str, path: &str) -> reqwest::Url {
    let url = format!("{}/{}/{}",config::get().service_http_addr(), version, path);
    reqwest::Url::parse(&url).unwrap()
}
