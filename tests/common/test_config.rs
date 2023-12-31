use axum_web::application::shared::config;
pub fn load_test_config() -> &'static config::Config {
    std::env::set_var("ENV_TEST", "1");
    config::load();
    config::get()
}
