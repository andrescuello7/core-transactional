use prex::infra::http::server::HttpServer;
use prex::infra::storage::alloc_memory::Alloc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let (tx, rx) = mpsc::channel(10000);
    tokio::spawn(async move { Alloc::new().run(rx).await });
    HttpServer::new(tx).run().await
}
