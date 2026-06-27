use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde_json::json;
use std::fmt;

/// Errores de negocio centralizados en domain
#[derive(Debug)]
pub enum PaymentError {
    ClientNotFound,
    DuplicateDocument,
    InsufficientFunds,
    NegativeAmount,
    StorageError(String),
}

impl fmt::Display for PaymentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentError::ClientNotFound => write!(f, "Cliente no encontrado en el sistema."),
            PaymentError::DuplicateDocument => {
                write!(f, "El número de documento ya se encuentra registrado.")
            }
            PaymentError::InsufficientFunds => {
                write!(f, "Fondos insuficientes para completar la transacción.")
            }
            PaymentError::StorageError(err) => {
                write!(f, "Error crítico de persistencia en disco: {}", err)
            }
            PaymentError::NegativeAmount => {
                write!(f, "El monto de la transacción no puede ser negativo.")
            }
        }
    }
}

impl ResponseError for PaymentError {
    fn status_code(&self) -> StatusCode {
        match self {
            PaymentError::ClientNotFound => StatusCode::NOT_FOUND,
            PaymentError::DuplicateDocument => StatusCode::BAD_REQUEST,
            PaymentError::InsufficientFunds => StatusCode::BAD_REQUEST,
            PaymentError::NegativeAmount => StatusCode::BAD_REQUEST,
            PaymentError::StorageError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        HttpResponse::build(status).json(json!({
            "status": status.as_u16(),
            "error": match self {
                PaymentError::ClientNotFound => "CLIENT_NOT_FOUND",
                PaymentError::DuplicateDocument => "DUPLICATED_DOCUMENT",
                PaymentError::InsufficientFunds => "INSUFFICIENT_FUNDS",
                PaymentError::NegativeAmount => "NEGATIVE_AMOUNT",
                PaymentError::StorageError(_) => "INTERNAL_PERSISTENCE_ERROR",
            },
            "message": self.to_string()
        }))
    }
}
