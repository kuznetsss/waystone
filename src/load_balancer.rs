mod upstream;

use std::{net::SocketAddr, sync::Arc};
use tracing::info;

use crate::config::Config;
use tokio::net::TcpListener;

use upstream::Upstream;

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::Response,
    routing::get,
    Router,
};

pub fn run(config: &Config) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(config.threads_number)
        .build()
        .unwrap()
        .block_on(async {
            let load_balancer = LoadBalancer::from_config(config).await;
            info!("Starting server on {}", load_balancer.local_address());
            load_balancer.run().await;
        })
}

struct LoadBalancer {
    router: Router,
    listener: TcpListener,
}

impl LoadBalancer {
    const MAX_BODY_SIZE : usize = 1024 * 1024;

    async fn new(ip_port: &str, upstream_servers: &[String]) -> Self {
        let upstream = Arc::new(Upstream::new(upstream_servers));
        let router = Router::new()
            .route("/", get(handle_http_request).post(handle_http_request))
            .with_state(upstream);
        let listener = TcpListener::bind(ip_port).await.unwrap();

        LoadBalancer { router, listener }
    }

    async fn from_config(config: &Config) -> Self {
        Self::new(&config.server.ip_port(), &config.upstream_servers).await
    }

    fn local_address(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }

    async fn run(self) {
        axum::serve(self.listener, self.router).await.unwrap();
    }
}

#[axum_macros::debug_handler]
async fn handle_http_request(
    State(upstream): State<Arc<Upstream>>,
    request: Request,
) -> Result<Response, StatusCode> {
    let request = convert(request).await?;

    let client = reqwest::Client::new();

    for server in upstream.start_from_random() {
        info!("Sending to {server}");
        let mut request_to_send = request.try_clone().unwrap();
        request_to_send
            .url_mut()
            .set_host(Some(&server.host))
            .unwrap();
        request_to_send
            .url_mut()
            .set_port(Some(server.port))
            .unwrap();

        dbg!(&request_to_send);

        match client.execute(request_to_send).await {
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

    Err(StatusCode::SERVICE_UNAVAILABLE)
}

async fn convert(request: Request) -> Result<reqwest::Request, StatusCode> {
    let (parts, body) = request.into_parts();
    let body = axum::body::to_bytes(body, LoadBalancer::MAX_BODY_SIZE)
        .await
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;

    let uri_string = format!(
        "http://host{}",
        parts.uri.path_and_query().unwrap().as_str()
    );

    let mut request = reqwest::Request::new(
        parts.method,
        reqwest::Url::parse(&uri_string).map_err(|_| StatusCode::BAD_REQUEST)?
    );
    request.url_mut().set_scheme("http").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    *request.version_mut() = parts.version;
    *request.headers_mut() = parts.headers;
    request.headers_mut().remove(axum::http::header::HOST);
    *request.body_mut() = Some(reqwest::Body::from(body));
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::echo_server::EchoServer;

    #[tokio::test]
    async fn get_request() {
        let upstream_server = EchoServer::new(true).await;
        let upstream_server_port = upstream_server.port();
        tokio::spawn(async move {
            upstream_server.run().await;
        });

        let load_balancer = LoadBalancer::new("127.0.0.1:0", &[format!("127.0.0.1:{upstream_server_port}")]).await;
        let load_balancer_address = format!("http://{}", load_balancer.local_address());
        tokio::spawn(async move {
            load_balancer.run().await;
        });

        let message = "hello world";
        let response = reqwest::Client::new().get(&load_balancer_address).body(message).send().await.unwrap();
        assert_eq!(response.text().await.unwrap(), message);
    }
}
