#[test]
fn root_path_test() {
    use axum_web::config;
    use hyper::{Body, Client, Method, Request};
    use tokio::runtime::Runtime;

    // parse configuration
    let config = config::from_dotenv();

    // build the root url
    let root_url = format!("http://{}:{}/", config.service_host, config.service_port);

    // build the request
    let request = Request::builder()
        .method(Method::GET)
        .uri(root_url)
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .body(Body::empty());
    assert!(request.is_ok());

    // execute the request
    let rt = Runtime::new().unwrap();
    let body = rt.block_on(async {
        let client = Client::new();
        let result = client.request(request.unwrap()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        let result = hyper::body::to_bytes(response.into_body()).await;
        assert!(result.is_ok());
        result.unwrap()
    });
    assert_eq!(body, "<h1>Axum-Web</h1>");
}
