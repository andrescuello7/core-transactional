use actix_web::{web, middleware};
use tokio::sync::mpsc;
use crate::domain::processor::Command;
use crate::infra::controllers; // Asumiendo que tus handlers viven aquí

pub struct HttpServer {
    port: u16,
    host: String,
    // Inyectamos el transmisor del canal como dependencia de infraestructura
    tx: mpsc::Sender<Command>, 
}

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8080;

impl HttpServer {
    pub fn new(port: u16, host: String, tx: mpsc::Sender<Command>) -> Self {
        let host = if host.is_empty() { DEFAULT_HOST.to_string() } else { host };
        let port = if port == 0 { DEFAULT_PORT } else { port };
        Self { port, host, tx }
    }

    pub async fn run(self) -> std::io::Result<()> {
        log::info!("HTTP server listening on {}:{}", self.host, self.port);
        
        // Movemos el tx adentro del closure clonándolo por cada hilo de ejecución de Actix
        let tx_data = self.tx.clone();

        let server = actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .wrap(middleware::Logger::default())
                // Inyección de dependencias nativa de Actix Web (App State)
                .app_data(web::Data::new(tx_data.clone()))
                .route("/", web::get().to(|| async { "Prex Core Transaction API Active" }))
                .configure(register_routes)
        })
        .bind((self.host, self.port))?
        .run();

        server.await
    }
}

fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/new_client", web::post().to(controllers::new_client::new_client));
    cfg.route("/client_balance/{user_id}", web::get().to(controllers::get_client_balance::get_client_balance));
}