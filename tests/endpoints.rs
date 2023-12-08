use axum_web::config;
use hyper::Request;
use hyper_util::rt::TokioIo;
use tokio::runtime::Runtime;
use http_body_util::{BodyExt, Empty};
use bytes::{Bytes, Buf};
use tokio::net::TcpStream;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[test]
#[ignore]
fn root_path_test() {
    // parse configuration
    let config = config::from_dotenv();

    // build the url
    let url = format!("http://{}:{}/", config.service_host, config.service_port);

    // fetch url
    let rt = Runtime::new().unwrap();
    let body = rt.block_on(async {
        let body1 = fetch_url_reqwest(&url).await.unwrap();
        
        let uri = url.parse::<hyper::Uri>().unwrap();
        let body2 = fetch_url_hyper(uri).await.unwrap();

        assert_eq!(body1, body2);
        body2
    });

    let body_expected = "<h1>Axum-Web</h1>";
    assert_eq!(body, body_expected);
}

// fetch using `reqwest`
async fn fetch_url_reqwest(url: &str) -> Result<String> {
    let res = reqwest::get(url).await?;
    let body = res.text().await?;
    Ok(body)
}

// fetch using `hyper`
async fn fetch_url_hyper(uri: hyper::Uri) -> Result<String> {
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