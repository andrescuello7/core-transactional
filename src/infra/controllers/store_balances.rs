use actix_web::{web, HttpResponse};
use tokio::sync::{mpsc, oneshot};
use serde_json::json;
use crate::domain::processor::Command;
use crate::infra::errors::AppError;

// POST "store_balances"
pub async fn store_balances(tx: web::Data<mpsc::Sender<Command>>) -> Result<HttpResponse, AppError> {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    
    // Despachamos el comando al motor
    tx.send(Command::StoreBalances { respond_to: oneshot_tx })
        .await
        .map_err(|_| AppError::PersistenceError("channel closed".to_string()))?;

    // El motor frena transacciones, escribe a disco, resetea a cero y nos responde
    let negocio_result = oneshot_rx.await
        .map_err(|e| AppError::PersistenceError(e.to_string()))?;

    match negocio_result {
        Ok(_) => {
            log::info!("Balances persistidos exitosamente en disco");
            Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Balances stored successfully"
            })))
        },
        Err(e) => Err(AppError::PersistenceError(e.to_string())),
    }
}