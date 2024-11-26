use std::{net::SocketAddr, sync::Arc};

use axum::{
    body::{Body, Bytes},
    extract::{
        ws::{Message, WebSocket},
        Request, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::Response,
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tracing::warn;

pub struct EchoServer {
    router: Router,
    listener: TcpListener,
}

impl EchoServer {
    pub async fn new(print_request: bool) -> Self {
        let printer: Arc<dyn Print> = match print_request {
            true => Arc::new(VerbosePrint {}),
            false => Arc::new(NoPrint {}),
        };
        let router = Router::new()
            .route("/", get(Self::handle_http).post(Self::handle_http))
            .route("/ws", get(Self::handle_ws_upgrade))
            .with_state(printer);

        EchoServer {
            router,
            listener: TcpListener::bind("0.0.0.0:0").await.unwrap(),
        }
    }

    async fn handle_http(
        State(printer): State<Arc<dyn Print>>,
        r: Request,
    ) -> Result<Response, StatusCode> {
        printer.print_http_request(&r);
        let body = match axum::body::to_bytes(r.into_body(), 1024 * 1024).await {
            Ok(body) => body,
            Err(_) => {
                warn!("Got body bigger than 1 MB");
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }
        };
        printer.print_http_body(&body);

        Ok(Response::new(Body::from(body)))
    }

    async fn handle_ws_upgrade(
        State(printer): State<Arc<dyn Print>>,
        ws: WebSocketUpgrade,
    ) -> Response {
        ws.on_upgrade(|socket| Self::handle_ws(socket, printer))
    }

    async fn handle_ws(mut socket: WebSocket, printer: Arc<dyn Print>) {
        while let Some(message) = socket.recv().await {
            let message = if let Ok(message) = message {
                message
            } else {
                return;
            };
            printer.print_ws_message(&message);
            if socket.send(message).await.is_err() {
                return;
            }
        }
    }

    pub fn local_address(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }

    pub async fn run(self) {
        axum::serve(self.listener, self.router).await.unwrap();
    }
}

trait Print: Send + Sync {
    fn print_http_request(&self, r: &Request);
    fn print_http_body(&self, body: &Bytes);
    fn print_ws_message(&self, m: &Message);
}

struct NoPrint {}

impl Print for NoPrint {
    fn print_http_request(&self, _: &Request) {}
    fn print_http_body(&self, _: &Bytes) {}
    fn print_ws_message(&self, _: &Message) {}
}

struct VerbosePrint {}

impl Print for VerbosePrint {
    fn print_http_request(&self, r: &Request) {
        println!("Got request:");
        println!(
            "GET {} HTTP{:?}",
            r.uri().path_and_query().unwrap(),
            r.version()
        );
        r.headers().iter().for_each(|(n, v)| println!("{n}: {v:?}"));
        println!();
    }

    fn print_http_body(&self, body: &Bytes) {
        println!("{:?}", body);
    }

    fn print_ws_message(&self, m: &Message) {
        if let Ok(t) = m.to_text() {
            println!("{t}");
        }
    }
}
