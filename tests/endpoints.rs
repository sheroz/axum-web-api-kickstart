#[test]
fn root_path_test() {
    use hyper::{Body, Client, Method, Request};
    use tokio::runtime::Runtime;

    let login_url = "http://127.0.0.1:3000";
    let request = Request::builder()
        .method(Method::GET)
        .uri(login_url)
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .body(Body::empty());
    assert!(request.is_ok());

    let client = Client::new();

    let rt = Runtime::new().unwrap();
    let body = rt.block_on(async {
        let result = client.request(request.unwrap()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        let result = hyper::body::to_bytes(response.into_body()).await;
        assert!(result.is_ok());
        result.unwrap()
    });
    assert_eq!(body, "<h1>Axum-Web</h1>");
}
