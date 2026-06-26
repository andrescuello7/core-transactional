use actix_web::{web, HttpResponse};
use tokio::sync::{mpsc, oneshot};
use serde_json::json;
use crate::domain::processor::Command;
use crate::domain::models::{DebitTransaction, NewClientPayload, AppError};


// POST "new_debit_transaction"
pub async fn new_debit_transaction(
    tx: web::Data<mpsc::Sender<Command>>,
    payload: web::Json<DebitTransaction>, [cite: 46]
) -> Result<HttpResponse, AppError> {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    tx.send(Command::Debit {
        client_id: payload.client_id, [cite: 48]
        amount: payload.debit_amount, [cite: 49]
        respond_to: oneshot_tx,
    }).await.map_err(|_| AppError::InternalError)?;

    match oneshot_rx.await.map_err(|_| AppError::InternalError)? {
        Ok(new_balance) => {
            log::info!("Débito aplicado: client_id={} amount={} balance={}", payload.client_id, payload.debit_amount, new_balance); [cite: 53]
            Ok(HttpResponse::Ok().json(json!({
                "client_id": payload.client_id,
                "amount": payload.debit_amount,
                "new_balance": new_balance,
                "type": "debit"
            }))) [cite: 54]
        },
        Err(e) => Err(AppError::from(e)),
    }
}
