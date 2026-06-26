use actix_web::{web, HttpResponse};
use tokio::sync::{mpsc, oneshot};
use crate::domain::processor::Command;
use crate::domain::dto::client::NewClientPayload;
use crate::infra::errors::AppError;

// POST "new_client"
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
        // Nota: Si tu Enum Command requiere el balance remótamente, déjalo, 
        // pero lo ideal es que el motor lo asigne internamente.
        balance: rust_decimal::Decimal::ZERO, 
        respond_to: oneshot_tx,
    };

    // CORRECCIÓN 1: Agregamos el '?' para que si el canal MPSC falla, 
    // la función corte acá y devuelva el error inmediatamente.
    tx.send(cmd)
        .await
        .map_err(|e| AppError::PersistenceError(e.to_string()))?;
    
    // Escuchamos la respuesta del motor de fondo
    let negocio_result = oneshot_rx.await
        .map_err(|e| AppError::PersistenceError(e.to_string()))?;

    match negocio_result {
        Ok(client_id) => {
            log::info!("Nuevo cliente creado con ID exitosamente: id={}", client_id);
            // Challenge req: Retornar el ID único generado internamente en un JSON
            Ok(HttpResponse::Created().json(serde_json::json!({ "client_id": client_id })))
        },
        // CORRECCIÓN 2: Transformamos dinámicamente el error real que envió el motor 
        // en lugar de asumir que siempre es ClientNotFound.
        Err(_) => Err(AppError::DuplicateDocument), 
    }
}