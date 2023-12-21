use axum_web::shared::config;
use bytes::{Buf, Bytes};
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::test]
#[ignore]
async fn root_path_test() {
    // parse configuration
    let config = config::from_dotenv();

    let url = config.service_http_addr();
    let expected = "<h1>Axum-Web</h1>";

    // fetch using reqwest
    let body = fetch_url_reqwest(&url).await.unwrap();
    assert_eq!(body, expected);

    // fetch using hyper
    let body = fetch_url_hyper(&url).await.unwrap();
    assert_eq!(body, expected);
}

// fetch using `reqwest`
async fn fetch_url_reqwest(url: &str) -> Result<String> {
    let res = reqwest::get(url).await?;
    let body = res.text().await?;
    Ok(body)
}

// fetch using `hyper`
async fn fetch_url_hyper(url: &str) -> Result<String> {
    let uri = url.parse::<hyper::Uri>().unwrap();
    let host = uri.host().expect("uri has no host");
    let port = uri.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);

    let stream = TcpStream::connect(addr).await?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let authority = uri.authority().unwrap().clone();

    // Fetch the url
    let req = Request::builder()
        .uri(uri)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    let res = sender.send_request(req).await?;

    // asynchronously aggregate the chunks of the body
    let body = res.collect().await?.aggregate();
    let content = String::from_utf8(body.chunk().to_vec())?;
    Ok(content)
}
