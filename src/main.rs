mod infra;

use infra::http::server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let server = server::HttpServer::new(8080, "0.0.0.0".to_string());
    server.run().await
}
