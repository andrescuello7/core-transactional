use actix_web::{web, HttpResponse};
use tokio::sync::{mpsc, oneshot};
use serde_json::json;
use crate::{domain::processor::Command, infra::errors::AppError};


// POST "store_balances"
pub async fn store_balances(tx: web::Data<mpsc::Sender<Command>>) -> Result<HttpResponse, AppError> {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    
    // Despachamos el comando al motor
    tx.send(Command::StoreBalances { respond_to: oneshot_tx })
    .await
    .map_err(|_| AppError::PersistenceError);

    // El motor frena transacciones, escribe a disco por file_store, resetea a cero y nos responde
    match oneshot_rx.await.map_err(|_| AppError::PersistenceError) {
        Ok(filename) => {
            log::info!("Balances persistidos exitosamente en '{}'", filename.unwrap_err());
            Ok(HttpResponse::Ok().json(json!({ "file": filename.unwrap_err()})))
        },
        Err(e) => Err(AppError::from(e)),
    }
}