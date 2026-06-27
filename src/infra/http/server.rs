use crate::infra::{controllers::{get_client_balance, new_client, new_credit_transaction,new_debit_transaction, store_balances}, storage::alloc_memory::Command};
use actix_web::{middleware, web};
use tokio::sync::mpsc;

pub struct HttpServer {
    port: u16,
    host: String,
    tx: mpsc::Sender<Command>,
}

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8080;

impl HttpServer {
    pub fn new(tx: mpsc::Sender<Command>) -> Self {

        let port: u16 = std::env::var("PORT")
            .unwrap_or_else(|_| DEFAULT_PORT.to_string())
            .parse()
            .expect("PORT must be a valid u16");
        let host = std::env::var("HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
        Self { port, host, tx }
    }

    pub async fn run(self) -> std::io::Result<()> {
        log::info!("HTTP server listening on {}:{}", self.host, self.port);

        // Movemos el tx adentro del closure clonándolo por cada hilo de ejecución de Actix
        let tx_data = self.tx.clone();

        let server = actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .wrap(middleware::Logger::default())
                .app_data(web::Data::new(tx_data.clone()))
                .configure(register_routes)
        })
        .bind((self.host, self.port))?
        .run();

        server.await
    }
}

fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/",
        web::get().to(|| async { "Prex Core Transaction API Active" }),
    );
    cfg.route(
        "/client_balance/{user_id}",
        web::get().to(get_client_balance::get_client_balance),
    );
    cfg.route(
        "/new_client",
        web::post().to(new_client::new_client),
    );
    cfg.route(
        "/new_credit_transaction",
        web::post().to(new_credit_transaction::new_credit_transaction),
    );
    cfg.route(
        "/new_debit_transaction",
        web::post().to(new_debit_transaction::new_debit_transaction),
    );
    cfg.route(
        "/store_balances",
        web::post().to(store_balances::store_balances),
    );
}
