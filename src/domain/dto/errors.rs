use serde_json::json;
use std::fmt;
use actix_web::{HttpResponse, ResponseError, http::StatusCode};

#[derive(Debug)]
pub enum PaymentError {
    ClientNotFound,
    DuplicateDocument,
    InsufficientFunds,
    StorageError(String),
}

impl fmt::Display for PaymentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentError::ClientNotFound => write!(f, "Cliente no encontrado en el sistema."),
            PaymentError::DuplicateDocument => write!(f, "El número de documento ya se encuentra registrado."),
            PaymentError::InsufficientFunds => write!(f, "Fondos insuficientes para completar la transacción."),
            PaymentError::StorageError(err) => write!(f, "Error crítico de persistencia en disco: {}", err),
        }
    }
}

impl ResponseError for PaymentError {
    fn status_code(&self) -> StatusCode {
        match self {
            PaymentError::ClientNotFound => StatusCode::NOT_FOUND,          // 404
            PaymentError::DuplicateDocument => StatusCode::BAD_REQUEST,     // 400
            PaymentError::InsufficientFunds => StatusCode::BAD_REQUEST,     // 400
            PaymentError::StorageError(_) => StatusCode::INTERNAL_SERVER_ERROR, // 500
        }
    }

    // Estructura el JSON de respuesta que verá el cliente final o la colección de Postman
    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        
        HttpResponse::build(status).json(json!({
            "status": status.as_u16(),
            "error": match self {
                PaymentError::ClientNotFound => "CLIENT_NOT_FOUND",
                PaymentError::DuplicateDocument => "DUPLICATED_DOCUMENT",
                PaymentError::InsufficientFunds => "INSUFFICIENT_FUNDS",
                PaymentError::StorageError(_) => "INTERNAL_PERSISTENCE_ERROR",
            },
            "message": self.to_string()
        }))
    }
}