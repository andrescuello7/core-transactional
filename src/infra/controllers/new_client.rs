use crate::domain::dto::client::NewClientPayload;
use crate::domain::errors::PaymentError;
use crate::infra::storage::alloc_memory::Command;
use actix_web::{web, HttpResponse};
use tokio::sync::{mpsc, oneshot};

pub async fn new_client(
    tx: web::Data<mpsc::Sender<Command>>,
    payload: web::Json<NewClientPayload>,
) -> Result<HttpResponse, PaymentError> {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    let cmd = Command::CreateClient {
        client_name: payload.client_name.clone(),
        birth_date: payload.birth_date,
        document_number: payload.document_number.clone(),
        country: payload.country.clone(),
        respond_to: oneshot_tx,
    };

    tx.send(cmd)
        .await
        .map_err(|e| PaymentError::StorageError(e.to_string()))?;

    let business_answer = oneshot_rx
        .await
        .map_err(|e| PaymentError::StorageError(e.to_string()))?;

    match business_answer {
        Ok(client_id) => {
            log::info!("Nuevo cliente creado con ID exitosamente: id={}", client_id);
            Ok(HttpResponse::Created().json(serde_json::json!({ "client_id": client_id })))
        }
        Err(e) => Err(e),
    }
}
