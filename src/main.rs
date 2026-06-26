mod domain;
mod infra;
use infra::http::server;
use tokio::sync::mpsc;
// use crate::infra::store::file_store::FileBalanceRepository;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // 1. Definimos la capacidad máxima del búfer del canal (Control de ráfagas)
    let (tx, rx) = mpsc::channel(10000);

    // 2. Instanciamos el adaptador de salida de persistencia I/O
    // let repo = FileBalanceRepository::new();

    // 3. Inicializamos el Motor Transaccional del Dominio inyectándole el repositorio
    let processor = domain::processor::TransactionProcessor::new();

    // 4. Lanzamos el loop infinito del motor en segundo plano (Tokio Worker Thread)
    // Este hilo será el ÚNICO dueño de los datos en la RAM.
    tokio::spawn(async move {
        processor.run(rx).await;
    });
    
    let server = server::HttpServer::new(8080, "0.0.0.0".to_string(), tx);

    // Running the server
    server.run().await
}
