use crate::domain::dto::client::DebitTransaction;
use crate::domain::errors::PaymentError;
use crate::infra::storage::alloc_memory::Command;
use actix_web::{web, HttpResponse};
use serde_json::json;
use tokio::sync::{mpsc, oneshot};

pub async fn new_debit_transaction(
    tx: web::Data<mpsc::Sender<Command>>,
    payload: web::Json<DebitTransaction>,
) -> Result<HttpResponse, PaymentError> {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    tx.send(Command::Debit {
        client_id: payload.client_id,
        amount: payload.debit_amount,
        respond_to: oneshot_tx,
    })
    .await
    .map_err(|_| PaymentError::StorageError("channel closed".to_string()))?;

    let business_answer = oneshot_rx
        .await
        .map_err(|e| PaymentError::StorageError(e.to_string()))?;

    match business_answer {
        Ok(new_balance) => {
            log::info!(
                "Débito aplicado: client_id={} amount={} balance={}",
                payload.client_id,
                payload.debit_amount,
                new_balance
            );
            Ok(HttpResponse::Ok().json(json!({
                "client_id": payload.client_id,
                "amount": payload.debit_amount,
                "new_balance": new_balance,
                "type": "debit"
            })))
        }
        Err(e) => Err(e),
    }
}
