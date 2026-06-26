use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    ClientAlreadyExists { doc: String },
    ClientNotFound,
    InsufficientFunds,
    PersistenceError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientAlreadyExists { doc } => {
                write!(f, "El cliente con documento '{}' ya existe", doc)
            }
            Self::ClientNotFound => write!(f, "Cliente no encontrado"),
            Self::InsufficientFunds => {
                write!(f, "Fondos insuficientes para realizar la operación")
            }
            Self::PersistenceError(msg) => write!(f, "Error al persistir datos: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let body = json!({ "error": self.to_string() });
        match self {
            Self::ClientAlreadyExists { .. } => HttpResponse::Conflict().json(body),
            Self::ClientNotFound => HttpResponse::NotFound().json(body),
            Self::InsufficientFunds => HttpResponse::UnprocessableEntity().json(body),
            Self::PersistenceError(_) => HttpResponse::InternalServerError().json(body),
        }
    }
}
