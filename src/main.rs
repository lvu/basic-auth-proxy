use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::{TokioIo, TokioTimer};
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpListener;

mod app;
mod basic;
mod err;
mod oidc;

#[derive(Deserialize, Debug)]
struct Config {
    #[serde(default = "default_listen_addr")]
    listen_addr: String,
    oidc_issuer: String,
    oidc_client_id: String,
    oidc_client_secret: String,
    groups_claim: Option<String>,
    #[serde(default = "Vec::new")]
    additional_scopes: Vec<String>,
}

fn default_listen_addr() -> String {
    "0.0.0.0:8080".to_string()
}

#[tokio::main]
async fn main() {
    let config = envy::from_env::<Config>().unwrap();
    let app = Arc::new(app::App::new(&config).await);
    let listener = TcpListener::bind(&config.listen_addr).await.unwrap();
    println!("Listening on {}", &config.listen_addr);

    loop {
        let (tcp, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(tcp);
        let app = app.clone();
        tokio::task::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .timer(TokioTimer::new())
                .title_case_headers(true)
                .serve_connection(io, service_fn(async |req| app.handle_auth(req).await))
                .await
            {
                println!("Error serving connection: {}", e);
            }
        });
    }
}
