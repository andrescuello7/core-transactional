use log;

pub struct HttpServer {
    port: u16,
    host: String,
    server: Option<actix_web::dev::Server>,
}

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8080;

impl HttpServer {
    pub fn new(port: u16, host: String) -> Self {
        let host = if host.is_empty() { DEFAULT_HOST.to_string() } else { host };
        let port = if port == 0 { DEFAULT_PORT } else { port };
        Self { port, host, server: None }
    }

    pub async fn run(mut self) -> std::io::Result<()> {
        log::info!("HTTP server listening on {}:{}", self.host, self.port);
        self.server = Some(actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .wrap(actix_web::middleware::Logger::default())
                // .configure(controllers::register_routes)
                .route("/", actix_web::web::get().to(|| async { "Hello, World!" }))
        }).bind((self.host.clone(), self.port))?.run());
        Ok(self.server.unwrap().await?)
    }
}