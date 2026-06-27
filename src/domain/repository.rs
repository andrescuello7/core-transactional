use std::collections::HashMap;
use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::domain::dto;
use crate::infra::errors::AppError;

pub struct AppState {
    clients: HashMap<u64, dto::client::Client>,
    next_id: u64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_client(
        &mut self,
        document_number: String,
        client_name: String,
        birth_date: NaiveDate,
        country: String,
    ) -> Result<dto::client::Client, AppError> {
        if self.clients.values().any(|c| c.document_number == document_number) {
            return Err(AppError::ClientAlreadyExists { doc: document_number });
        }
        let client_id = self.next_id;
        self.next_id += 1;
        let client = dto::client::Client {
            client_id,
            document_number,
            client_name,
            birth_date,
            country,
            balance: Decimal::ZERO,
        };
        self.clients.insert(client_id, client.clone());
        Ok(client)
    }

    pub fn get_balance(&self, client_id: u64) -> Result<Decimal, AppError> {
        self.clients
            .get(&client_id)
            .map(|c| c.balance)
            .ok_or(AppError::ClientNotFound)
    }

    pub fn credit(&mut self, client_id: u64, amount: Decimal) -> Result<Decimal, AppError> {
        let client = self
            .clients
            .get_mut(&client_id)
            .ok_or(AppError::ClientNotFound)?;
        client.balance += amount;
        Ok(client.balance)
    }

    pub fn debit(&mut self, client_id: u64, amount: Decimal) -> Result<Decimal, AppError> {
        let client = self
            .clients
            .get_mut(&client_id)
            .ok_or(AppError::ClientNotFound)?;
        if client.balance < amount {
            return Err(AppError::InsufficientFunds);
        }
        client.balance -= amount;
        Ok(client.balance)
    }

    /// Returns (id, balance) pairs for all clients, ordered by id.
    pub fn balances_snapshot(&self) -> Vec<(u64, Decimal)> {
        let mut pairs: Vec<_> = self.clients.values().map(|c| (c.client_id, c.balance)).collect();
        pairs.sort_by_key(|(id, _)| *id);
        pairs
    }

    pub fn reset_balances(&mut self) {
        for client in self.clients.values_mut() {
            client.balance = Decimal::ZERO;
        }
    }
}
