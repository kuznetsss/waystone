use waystone::echo_server;
use tracing::info;

fn main() {
    let tracer = tracing_subscriber::fmt().with_ansi(false).finish();
    tracing::subscriber::set_global_default(tracer).unwrap();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(1)
        .build()
        .unwrap()
        .block_on(async {
            let server = echo_server::EchoServer::new(true).await;
            info!("Running echo server on 127.0.0.1:{}", server.port());
            server.run().await;
        });
}
