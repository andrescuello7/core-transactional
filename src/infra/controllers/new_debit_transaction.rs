use actix_web::{web, HttpResponse};
use tokio::sync::{mpsc, oneshot};
use serde_json::json;
use crate::domain::processor::Command;
use crate::domain::dto::client::DebitTransaction;
use crate::infra::errors::AppError;

// POST "new_debit_transaction"
pub async fn new_debit_transaction(
    tx: web::Data<mpsc::Sender<Command>>,
    payload: web::Json<DebitTransaction>,
) -> Result<HttpResponse, AppError> {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    tx.send(Command::Debit {
        client_id: payload.client_id,
        amount: payload.debit_amount,
        respond_to: oneshot_tx,
    })
    .await
    .map_err(|_| AppError::PersistenceError("channel closed".to_string()))?;

    let negocio_result = oneshot_rx.await
        .map_err(|e| AppError::PersistenceError(e.to_string()))?;

    match negocio_result {
        Ok(new_balance) => {
            log::info!("Débito aplicado: client_id={} amount={} balance={}", 
                payload.client_id, payload.debit_amount, new_balance);
            Ok(HttpResponse::Ok().json(json!({
                "client_id": payload.client_id,
                "amount": payload.debit_amount,
                "new_balance": new_balance,
                "type": "debit"
            })))
        },
        Err(e) => Err(AppError::PersistenceError(e.to_string())),
    }
}
