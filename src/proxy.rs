use http_body_util::Empty;
use hyper::body::{Bytes, Incoming};
use hyper::header::HeaderValue;
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper::{Request, Response, Version};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::{TokioExecutor, TokioIo};
use std::sync::LazyLock;
use tokio::net::TcpListener;
use tokio::task;

static POOL: LazyLock<hyper_util::client::legacy::Client<HttpConnector, Empty<Bytes>>> =
    LazyLock::new(|| {
        let mut conn = HttpConnector::new();
        conn.enforce_http(true);

        hyper_util::client::legacy::Client::builder(TokioExecutor::new())
            .pool_max_idle_per_host(100)
            .http2_only(false)
            .build(conn)
    });

pub async fn run(realloc: bool) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    loop {
        let (stream, _) = listener.accept().await?;

        task::spawn(async move {
            // keep it as minimal as possible
            // will not work from a browser, but with http2 prior knowledge
            if let Err(err) = http2::Builder::new(TokioExecutor::new())
                .serve_connection(
                    TokioIo::new(stream),
                    service_fn(move |req: Request<Incoming>| handle_request(req, realloc)),
                )
                .await
            {
                eprintln!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(
    _req: Request<Incoming>,
    realloc: bool,
) -> Result<Response<Incoming>, String> {
    let req = Request::builder()
        .uri("http://127.0.0.1:7000/")
        .version(Version::HTTP_11)
        .method("GET")
        .body(Empty::new())
        .unwrap();

    match POOL.request(req).await {
        Ok(resp) => {
            let (mut parts, body) = resp.into_parts();

            // If realloc is set, we build a new Response here, and force a re-allocation
            if realloc {
                // the status probably does not produce any leak because it's Copy
                let mut resp = Response::builder().status(parts.status).body(body).unwrap();

                for (name, value) in parts.headers.drain() {
                    let Some(name) = name else { continue };
                    // If we don't force a new allocation here, we will leak memory in this
                    // specific situation.
                    let owned = HeaderValue::from_bytes(value.as_bytes()).unwrap();
                    resp.headers_mut().insert(name, owned);
                }

                Ok(resp)
            } else {
                Ok(Response::from_parts(parts, body))
            }
        }
        Err(err) => Err(format!("upstream conn err: {}", err)),
    }
}
