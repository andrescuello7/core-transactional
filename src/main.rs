mod domain;
mod infra;
use infra::http::server::HttpServer;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let (tx, rx) = mpsc::channel(10000);
    let memory = infra::storage::alloc_memory::Alloc::new();

    tokio::spawn(async move {
        memory.run(rx).await;
    });

    let server = HttpServer::new(tx);
    server.run().await
}
