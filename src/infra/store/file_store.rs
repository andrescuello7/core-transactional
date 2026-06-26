use std::fs::File;
use std::io::Write;
use chrono::Local;
use rust_decimal::Decimal;
use crate::domain::dto::errors::PaymentError; // Tu enum de errores de dominio

pub struct FileBalanceRepository {
    file_counter: u32,
}

impl FileBalanceRepository {
    pub fn new() -> Self {
        Self { file_counter: 1 }
    }

    pub fn save_balances(&mut self, balances: &[(u64, Decimal)]) -> Result<String, PaymentError> {
        let date_str = Local::now().format("%d%m%Y").to_string();
        let filename = format!("{}_{}.DAT", date_str, self.file_counter);

        // CORRECCIÓN 1: Usamos la closure |e| para instanciar la variante pasando el string,
        // Y agregamos el '?' al final para extraer el 'File' de adentro del 'Result'.
        let mut file = File::create(&filename)
            .map_err(|e| PaymentError::StorageError(e.to_string()))?;

        // Volcado secuencial
        for (id, balance) in balances {
            writeln!(file, "{} {}", id, balance)
                .map_err(|e| PaymentError::StorageError(e.to_string()))?;
        }

        // CORRECCIÓN 2: Ahora que 'file' es un 'File' real y no un 'Result', 
        // el método .flush() existe y compila a la perfección.
        file.flush()
            .map_err(|e| PaymentError::StorageError(e.to_string()))?;

        self.file_counter += 1;

        Ok(filename)
    }
}