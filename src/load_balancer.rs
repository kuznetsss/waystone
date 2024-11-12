mod upstream;

use reqwest::{Body as ReqwestBody, StatusCode};
use std::sync::Arc;
use tracing::info;

use crate::config::Config;
use sync_wrapper::SyncStream;
use tokio::net::TcpListener;

use upstream::Upstream;

use axum::{
    body::{Body, Bytes},
    extract::{Request, State},
    http::{request::Parts, HeaderMap, Uri},
    response::Response,
    routing::get,
    Router,
};

pub fn run(config: &Config) {
    let state = Arc::new(Upstream::from_config(config));

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(config.threads_number)
        .build()
        .unwrap()
        .block_on(async {
            let app = Router::new()
                .route("/", get(handle_get_request))
                .with_state(state);
            let listener = TcpListener::bind(config.server.ip_port()).await.unwrap();
            info!("Starting server on {}", listener.local_addr().unwrap());
            axum::serve(listener, app).await.unwrap();
        })
}

#[axum_macros::debug_handler]
async fn handle_get_request(
    State(upstream): State<Arc<Upstream>>,
    headers: HeaderMap,
    uri: Uri,
    body: Bytes,
) -> Result<Response, StatusCode> {
    for server in upstream.start_from_random() {
        info!("Sending to {server}");
        let request = reqwest::Client::new()
            .request(
                reqwest::Method::GET,
                reqwest::Url::parse(
                    format!("http://{}:{}{}", server.host, server.port, uri).as_str(),
                )
                .unwrap(),
            )
            .body(body.clone())
            .headers(headers.clone());

        match request.send().await {
            Ok(r) => {
                let response = Response::from(r);
                let response = response.map(Body::new);
                return Ok(response);
            }
            Err(e) => {
                info!("{e}");
            }
        }
    }

    Err(StatusCode::from_u16(501).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{self, Config};

    #[tokio::test]
    async fn get_request() {
        const YAML_CONFIG: &str = "
server:
  port: 12344
upstream_servers:
  - 127.0.0.1:12345
threads_number: 2
";
        let config = Config::new(YAML_CONFIG).unwrap();

        tokio::spawn(async move {
            run(&config);
        });
    }
}
