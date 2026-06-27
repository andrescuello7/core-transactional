use crate::domain::errors::PaymentError;
use crate::infra::storage::alloc_memory::Command;
use actix_web::{web, HttpResponse};
use serde_json::json;
use tokio::sync::{mpsc, oneshot};

pub async fn store_balances(
    tx: web::Data<mpsc::Sender<Command>>,
) -> Result<HttpResponse, PaymentError> {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    tx.send(Command::StoreBalances {
        respond_to: oneshot_tx,
    })
    .await
    .map_err(|_| PaymentError::StorageError("channel closed".to_string()))?;

    let business_answer = oneshot_rx
        .await
        .map_err(|e| PaymentError::StorageError(e.to_string()))?;

    match business_answer {
        Ok(_) => {
            log::info!("Balances persistidos exitosamente en disco");
            Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Balances stored successfully"
            })))
        }
        Err(e) => Err(e),
    }
}
