use std::env;
use tokio::task;

pub mod proxy;
pub mod upstream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let realloc = env::args().into_iter().skip(1).next().as_deref() == Some("realloc");
    println!("with reallocation: {realloc}");

    task::spawn(async move {
        if let Err(err) = upstream::run().await {
            eprintln!("upstream err: {err}");
        }
    });

    proxy::run(realloc).await
}
