use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub client_id: u64,
    pub document_number: String,
    pub client_name: String,
    pub birth_date: NaiveDate,
    pub country: String,
    pub balance: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct NewClientPayload {
    pub client_name: String,
    pub birth_date: NaiveDate,
    pub document_number: String,
    pub country: String,
}

#[derive(Debug, Deserialize)]
pub struct CreditTransaction {
    pub client_id: u64,
    pub credit_amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct DebitTransaction {
    pub client_id: u64,
    pub debit_amount: Decimal,
}

impl Client {
    pub fn new(
        client_id: u64,
        client_name: String,
        birth_date: NaiveDate,
        document_number: String,
        country: String,
        balance: Decimal,
    ) -> Self {
        Self {
            client_id,
            document_number,
            client_name,
            birth_date,
            country,
            balance,
        }
    }
}
