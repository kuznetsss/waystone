use axum::{body::Body, extract::Request, http::StatusCode, response::Response, routing::get, Router};
use tokio::net::TcpListener;
use tracing::warn;

pub struct EchoServer {
    router: Router,
    listener: TcpListener,
}

impl EchoServer {
    pub async fn new(print_request: bool) -> Self {
        let router = Router::new().route(
            "/",
            get(move |r: Request| async move {
                Self::get_handler(r, print_request).await
            }),
        );
        let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
        EchoServer { router, listener }
    }

    async fn get_handler(r: Request, print_request: bool) -> Result<Response, StatusCode> {
        if print_request {
            println!("Got request:");
            println!(
                "GET {} HTTP{:?}",
                r.uri().path_and_query().unwrap(),
                r.version()
            );
            r.headers().iter().for_each(|(n, v)| println!("{n}: {v:?}"));
            println!();
        }

        let body = match axum::body::to_bytes(r.into_body(), 1024 * 1024).await {
            Ok(body) => body,
            Err(_) => {
                warn!("Got body bigger than 1 MB");
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }
        };

        println!("{:?}", body);
        Ok(Response::new(Body::from(body)))
    }

    pub fn port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }

    pub async fn run(self) {
        axum::serve(self.listener, self.router).await.unwrap();
    }
}
