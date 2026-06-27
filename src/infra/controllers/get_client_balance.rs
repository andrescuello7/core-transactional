use crate::infra::{errors::AppError, storage::alloc_memory::Command};
use actix_web::{web, HttpResponse};
use tokio::sync::{mpsc, oneshot};

pub async fn get_client_balance(
    tx: web::Data<mpsc::Sender<Command>>,
    path: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    let client_id = path.into_inner();
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    tx.send(Command::GetBalance {
        client_id,
        respond_to: oneshot_tx,
    })
    .await
    .map_err(|_| AppError::PersistenceError("channel closed".to_string()))?;

    let business_answer = oneshot_rx
        .await
        .map_err(|e| AppError::PersistenceError(e.to_string()))?;

    match business_answer {
        Ok(client_info) => Ok(HttpResponse::Ok().json(client_info)),
        Err(payment_error) => Err(AppError::PersistenceError(payment_error.to_string())),
    }
}
