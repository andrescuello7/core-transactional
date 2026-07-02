use core_transactional::infra::http::server::HttpServer;
use core_transactional::infra::storage::alloc_memory::Alloc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger, you can use other loggers if you want.
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // MPSC channel is used to send the request from the HTTP server to the Alloc service.
    // This is a simple implementation, you can use other methods to communicate between services.
    // We have 10k buffer size for the channel, you can adjust it according to your needs.

    // RX: Receiver for receiving requests from the HTTP server.
    // TX: Sender for sending requests to the Alloc service.
    let (tx, rx) = mpsc::channel(10000);
    // create a new Alloc service and run it in a separate task.
    tokio::spawn(async move { Alloc::new().run(rx).await });

    // Server run on port 8080, you can visit http://localhost:8080 to test it.
    HttpServer::new(tx).run().await
}
