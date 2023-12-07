use axum_web::config;
use hyper::Request;
use hyper_util::rt::TokioIo;
use tokio::runtime::Runtime;
use http_body_util::{BodyExt, Empty};
use bytes::{Bytes, Buf};
use tokio::net::TcpStream;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn root_path_test() {

    // parse configuration
    let config = config::from_dotenv();

    // build the root url
    let url = format!("http://{}:{}/", config.service_host, config.service_port);
    let uri = url.parse::<hyper::Uri>().unwrap();
    let rt = Runtime::new().unwrap();
    let body = rt.block_on(async {
        hyper_fetch_url(uri).await.unwrap()
    });
    assert_eq!(body, "<h1>Axum-Web</h1>");
}

async fn hyper_fetch_url(url: hyper::Uri) -> Result<String> {
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);

    let stream = TcpStream::connect(addr).await?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let authority = url.authority().unwrap().clone();

    // Fetch the url...
    let req = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    let res = sender.send_request(req).await?;

    // asynchronously aggregate the chunks of the body
    let body = res.collect().await?.aggregate();

    let mut buf = body.reader();
    let mut dst = vec![];
    std::io::copy(&mut buf, &mut dst).unwrap();
    let content = String::from_utf8(dst).unwrap();

    Ok(content)
}