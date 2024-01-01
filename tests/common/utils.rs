use axum_web::application::shared::config;

pub fn load_test_config() -> &'static config::Config {
    std::env::set_var("ENV_TEST", "1");
    config::load();
    config::get()
}

pub fn build_url(path: &str, url: &str) -> reqwest::Url {
    let url = format!("{}/{}/{}",config::get().service_http_addr(), path, url);
    reqwest::Url::parse(&url).unwrap()
}

pub fn build_path(path: &str) -> reqwest::Url {
    let url = format!("{}/{}",config::get().service_http_addr(), path);
    reqwest::Url::parse(&url).unwrap()
}
