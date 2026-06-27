use crate::domain::dto::client::NewClientPayload;
use crate::infra::{errors::AppError, storage::alloc_memory::Command};
use actix_web::{web, HttpResponse};
use tokio::sync::{mpsc, oneshot};

pub async fn new_client(
    tx: web::Data<mpsc::Sender<Command>>,
    payload: web::Json<NewClientPayload>,
) -> Result<HttpResponse, AppError> {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    let cmd = Command::CreateClient {
        client_name: payload.client_name.clone(),
        birth_date: payload.birth_date,
        document_number: payload.document_number.clone(),
        country: payload.country.clone(),
        balance: rust_decimal::Decimal::ZERO,
        respond_to: oneshot_tx,
    };

    tx.send(cmd)
        .await
        .map_err(|e| AppError::PersistenceError(e.to_string()))?;

    let business_answer = oneshot_rx
        .await
        .map_err(|e| AppError::PersistenceError(e.to_string()))?;

    match business_answer {
        Ok(client_id) => {
            log::info!("Nuevo cliente creado con ID exitosamente: id={}", client_id);
            Ok(HttpResponse::Created().json(serde_json::json!({ "client_id": client_id })))
        }
        Err(_) => Err(AppError::DuplicateDocument),
    }
}
