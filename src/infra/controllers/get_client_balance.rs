use actix_web::{web, HttpResponse};
use tokio::sync::{mpsc, oneshot};
use crate::domain::processor::Command;
use crate::infra::errors::AppError;

// GET "client_balance"
pub async fn get_client_balance(
    tx: web::Data<mpsc::Sender<Command>>,
    path: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    let client_id = path.into_inner();
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    let _ = tx.send(Command::GetBalance { client_id, respond_to: oneshot_tx })
        .await
        .map_err(|_| AppError::PersistenceError); 

    let negocio_result = oneshot_rx.await
        .map_err(|e| AppError::PersistenceError(e.to_string()))?;

    match negocio_result {
        Ok(client_info) => Ok(HttpResponse::Ok().json(client_info)),
        Err(payment_error) => Err(AppError::PersistenceError(payment_error.to_string())), 
    }
}