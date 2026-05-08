use http_body_util::Empty;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::task;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:7000").await?;

    loop {
        let (stream, _) = listener.accept().await?;

        task::spawn(async move {
            // Memory leak appears only when the upstream is http1
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    TokioIo::new(stream),
                    service_fn(move |req: Request<Incoming>| handle_request(req)),
                )
                .await
            {
                eprintln!("Client Conn: {err:?}");
            }
        });
    }
}

async fn handle_request(_req: Request<Incoming>) -> Result<Response<Empty<Bytes>>, String> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("x-header-1", "some header data 1")
        .header("x-header-2", "some header data 2")
        .header("x-header-3", "some header data 3")
        .header("x-header-4", "some header data 4")
        .header("x-header-5", "some header data 5")
        .header("x-header-6", "some header data 6")
        .header("x-header-7", "some header data 7")
        .header("x-header-8", "some header data 8")
        .body(Empty::new())
        .unwrap())
}
