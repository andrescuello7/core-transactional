use crate::domain::errors::PaymentError;
use crate::infra::storage::alloc_memory::Command;
use actix_web::{web, HttpResponse};
use tokio::sync::{mpsc, oneshot};

pub async fn get_client_balance(
    tx: web::Data<mpsc::Sender<Command>>,
    path: web::Path<u64>,
) -> Result<HttpResponse, PaymentError> {
    let client_id = path.into_inner();
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    tx.send(Command::GetBalance {
        client_id,
        respond_to: oneshot_tx,
    })
    .await
    .map_err(|_| PaymentError::StorageError("channel closed".to_string()))?;

    let business_answer = oneshot_rx
        .await
        .map_err(|e| PaymentError::StorageError(e.to_string()))?;

    match business_answer {
        Ok(client_info) => Ok(HttpResponse::Ok().json(client_info)),
        Err(payment_error) => Err(payment_error),
    }
}
