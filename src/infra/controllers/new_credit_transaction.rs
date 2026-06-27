use crate::domain::dto::client::CreditTransaction;
use crate::infra::{errors::AppError, storage::alloc_memory::Command};
use actix_web::{web, HttpResponse};
use serde_json::json;
use tokio::sync::{mpsc, oneshot};

pub async fn new_credit_transaction(
    tx: web::Data<mpsc::Sender<Command>>,
    payload: web::Json<CreditTransaction>,
) -> Result<HttpResponse, AppError> {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    tx.send(Command::Credit {
        client_id: payload.client_id,
        amount: payload.credit_amount,
        respond_to: oneshot_tx,
    })
    .await
    .map_err(|_| AppError::PersistenceError("channel closed".to_string()))?;

    let business_answer = oneshot_rx
        .await
        .map_err(|e| AppError::PersistenceError(e.to_string()))?;

    match business_answer {
        Ok(new_balance) => {
            log::info!(
                "Crédito aplicado: client_id={} amount={} balance={}",
                payload.client_id,
                payload.credit_amount,
                new_balance
            );
            Ok(HttpResponse::Ok().json(json!({
                "client_id": payload.client_id,
                "amount": payload.credit_amount,
                "new_balance": new_balance,
                "type": "credit"
            })))
        }
        Err(e) => Err(AppError::PersistenceError(e.to_string())),
    }
}
