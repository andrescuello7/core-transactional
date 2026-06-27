use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Decimal::from_str_exact(&s).map_err(D::Error::custom),
        Value::Number(n) => {
            Decimal::from_str_exact(&n.to_string()).map_err(D::Error::custom)
        }
        _ => Err(D::Error::custom("expected string or number for Decimal")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub client_id: u64,
    pub document_number: String,
    pub client_name: String,
    pub birth_date: NaiveDate,
    pub country: String,
    #[serde(deserialize_with = "deserialize_decimal")]
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
    #[serde(deserialize_with = "deserialize_decimal")]
    pub credit_amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct DebitTransaction {
    pub client_id: u64,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub debit_amount: Decimal,
}