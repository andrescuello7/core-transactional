use crate::domain::dto;
use crate::domain::errors::PaymentError;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use tokio::sync::{mpsc, oneshot};

// Command enum represents the different operations that can be performed on the Alloc service.
pub enum Command {
    GetBalance {
        client_id: u64,
        respond_to: oneshot::Sender<Result<dto::client::Client, PaymentError>>,
    },
    CreateClient {
        document_number: String,
        client_name: String,
        birth_date: NaiveDate,
        country: String,
        respond_to: oneshot::Sender<Result<u64, PaymentError>>,
    },
    Credit {
        client_id: u64,
        amount: Decimal,
        respond_to: oneshot::Sender<Result<Decimal, PaymentError>>,
    },
    Debit {
        client_id: u64,
        amount: Decimal,
        respond_to: oneshot::Sender<Result<Decimal, PaymentError>>,
    },
    StoreBalances {
        respond_to: oneshot::Sender<Result<(), PaymentError>>,
    },
}

// Alloc struct represents the in-memory storage service that manages clients and their balances.
// clients: HashMap<u64, dto::client::Client> - A HashMap that stores clients with their unique IDs as keys.
// next_id: u64 - A counter to generate unique IDs for new clients.
// file_counter: u32 - A counter to generate unique filenames when storing balances to a file
pub struct Alloc {
    clients: HashMap<u64, dto::client::Client>,
    next_id: u64,
    file_counter: u32,
}

impl Default for Alloc {
    fn default() -> Self {
        Self {
            clients: HashMap::new(),
            next_id: 1,
            file_counter: 1,
        }
    }
}

impl Alloc {
    // Create directory for storing balance files IF DOESN'T EXIST and return a new instance of Alloc.
    pub fn new() -> Self {
        let _ = std::fs::create_dir_all("docs/data");
        Self::default()
    }

    pub async fn run(mut self, mut receiver: mpsc::Receiver<Command>) {
        log::info!("Core Transaccional iniciado exitosamente.");

        while let Some(cmd) = receiver.recv().await {
            match cmd {
                Command::GetBalance {
                    client_id,
                    respond_to,
                } => {
                    if let Some(client) = self.clients.get(&client_id) {
                        let _ = respond_to.send(Ok(client.clone()));
                    } else {
                        let _ = respond_to.send(Err(PaymentError::ClientNotFound));
                    }
                }
                Command::CreateClient {
                    document_number,
                    client_name,
                    birth_date,
                    country,
                    respond_to,
                } => {
                    let doc_exists = self
                        .clients
                        .values()
                        .any(|_| self.clients.values().any(|c| c.document_number == document_number));
                    if doc_exists {
                        let _ = respond_to.send(Err(PaymentError::DuplicateDocument));
                        continue;
                    }

                    let id = self.next_id;
                    self.next_id += 1;

                    let new_client = dto::client::Client {
                        client_id: id,
                        client_name,
                        birth_date,
                        document_number,
                        country,
                        balance: Decimal::ZERO,
                    };
                    self.clients.insert(id, new_client);

                    let _ = respond_to.send(Ok(id));
                }
                Command::Credit {
                    client_id,
                    amount,
                    respond_to,
                } => {
                    if amount <= Decimal::ZERO {
                        let _ = respond_to.send(Err(PaymentError::NegativeAmount));
                    } else if let Some(client) = self.clients.get_mut(&client_id) {
                        client.balance += amount;
                        let _ = respond_to.send(Ok(client.balance));
                    } else {
                        let _ = respond_to.send(Err(PaymentError::ClientNotFound));
                    }
                }

                Command::Debit {
                    client_id,
                    amount,
                    respond_to,
                } => {
                    if amount <= Decimal::ZERO {
                        let _ = respond_to.send(Err(PaymentError::NegativeAmount));
                    } else if let Some(client) = self.clients.get_mut(&client_id) {
                        if client.balance < amount {
                            let _ =
                                respond_to.send(Err(PaymentError::InsufficientFunds));
                        } else {
                            client.balance -= amount;
                            let _ = respond_to.send(Ok(client.balance));
                        }
                    } else {
                        let _ = respond_to.send(Err(PaymentError::ClientNotFound));
                    }
                }

                Command::StoreBalances { respond_to } => {
                    let filename = format!(
                        "docs/data/{}_{}.DAT",
                        Utc::now().format("%d%m%Y"),
                        self.file_counter
                    );

                    match File::create(&filename) {
                        Ok(mut file) => {
                            let mut success = true;

                            // store each client in the file with the format: "client_id balance"
                            for (id, client) in self.clients.iter() {
                                // Write the client ID and balance to the file, separated by a space.
                                if let Err(e) = writeln!(file, "{} {}", id, client.balance) {
                                    log::error!("Error escribiendo en archivo: {}", e);
                                    success = false;
                                    break;
                                }
                            }

                            if success {
                                for client in self.clients.values_mut() {
                                    // Reset the balance to zero after storing it in the file.
                                    client.balance = Decimal::ZERO;
                                }
                                self.file_counter += 1;
                                let _ = respond_to.send(Ok(()));
                            } else {
                                let _ =
                                    respond_to.send(Err(PaymentError::StorageError(
                                        "Error al escribir en archivo".into(),
                                    )));
                            }
                        }
                        Err(e) => {
                            let _ = respond_to.send(Err(PaymentError::StorageError(
                                {log::error!("...{}", e); e.to_string().into()},
                            )));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use tokio::sync::oneshot;

    fn birth_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(1990, 6, 15).unwrap()
    }

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    async fn spawn_actor() -> mpsc::Sender<Command> {
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(async move { Alloc::new().run(rx).await });
        tx
    }

    async fn create_client(tx: &mpsc::Sender<Command>, doc: &str) -> u64 {
        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(Command::CreateClient {
            document_number: doc.to_string(),
            client_name: "Test User".to_string(),
            birth_date: birth_date(),
            country: "AR".to_string(),
            respond_to: resp_tx,
        })
        .await
        .unwrap();
        resp_rx.await.unwrap().unwrap()
    }


    #[tokio::test]
    async fn create_client_returns_sequential_id() {
        let tx = spawn_actor().await;
        let id1 = create_client(&tx, "DOC-001").await;
        let id2 = create_client(&tx, "DOC-002").await;
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[tokio::test]
    async fn create_client_duplicate_document_returns_error() {
        let tx = spawn_actor().await;
        create_client(&tx, "DOC-DUP").await;

        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(Command::CreateClient {
            document_number: "DOC-DUP".to_string(),
            client_name: "Another".to_string(),
            birth_date: birth_date(),
            country: "AR".to_string(),
            respond_to: resp_tx,
        })
        .await
        .unwrap();

        assert!(matches!(resp_rx.await.unwrap(), Err(PaymentError::DuplicateDocument)));
    }

    #[tokio::test]
    async fn credit_increases_balance_and_returns_new_balance() {
        let tx = spawn_actor().await;
        let id = create_client(&tx, "DOC-C1").await;

        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(Command::Credit {
            client_id: id,
            amount: dec("150.50"),
            respond_to: resp_tx,
        })
        .await
        .unwrap();

        assert_eq!(resp_rx.await.unwrap().unwrap(), dec("150.50"));
    }

    #[tokio::test]
    async fn credit_accumulates_across_multiple_operations() {
        let tx = spawn_actor().await;
        let id = create_client(&tx, "DOC-C2").await;

        for _ in 0..3 {
            let (c_tx, c_rx) = oneshot::channel();
            tx.send(Command::Credit { client_id: id, amount: dec("100.00"), respond_to: c_tx })
                .await.unwrap();
            c_rx.await.unwrap().unwrap();
        }

        let (g_tx, g_rx) = oneshot::channel();
        tx.send(Command::GetBalance { client_id: id, respond_to: g_tx }).await.unwrap();
        assert_eq!(g_rx.await.unwrap().unwrap().balance, dec("300.00"));
    }

    #[tokio::test]
    async fn credit_on_unknown_client_returns_not_found() {
        let tx = spawn_actor().await;

        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(Command::Credit { client_id: 999, amount: dec("100.00"), respond_to: resp_tx })
            .await.unwrap();

        assert!(matches!(resp_rx.await.unwrap(), Err(PaymentError::ClientNotFound)));
    }

    #[tokio::test]
    async fn credit_with_zero_amount_returns_negative_amount_error() {
        let tx = spawn_actor().await;
        let id = create_client(&tx, "DOC-Z1").await;

        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(Command::Credit { client_id: id, amount: Decimal::ZERO, respond_to: resp_tx })
            .await.unwrap();

        assert!(matches!(resp_rx.await.unwrap(), Err(PaymentError::NegativeAmount)));
    }

    #[tokio::test]
    async fn credit_with_negative_amount_returns_negative_amount_error() {
        let tx = spawn_actor().await;
        let id = create_client(&tx, "DOC-N1").await;

        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(Command::Credit { client_id: id, amount: dec("-50.00"), respond_to: resp_tx })
            .await.unwrap();

        assert!(matches!(resp_rx.await.unwrap(), Err(PaymentError::NegativeAmount)));
    }

    #[tokio::test]
    async fn debit_decreases_balance_and_returns_new_balance() {
        let tx = spawn_actor().await;
        let id = create_client(&tx, "DOC-D1").await;

        let (c_tx, c_rx) = oneshot::channel();
        tx.send(Command::Credit { client_id: id, amount: dec("500.00"), respond_to: c_tx })
            .await.unwrap();
        c_rx.await.unwrap().unwrap();

        let (d_tx, d_rx) = oneshot::channel();
        tx.send(Command::Debit { client_id: id, amount: dec("200.00"), respond_to: d_tx })
            .await.unwrap();

        assert_eq!(d_rx.await.unwrap().unwrap(), dec("300.00"));
    }

    #[tokio::test]
    async fn debit_insufficient_funds_returns_error() {
        let tx = spawn_actor().await;
        let id = create_client(&tx, "DOC-D2").await;

        let (d_tx, d_rx) = oneshot::channel();
        tx.send(Command::Debit { client_id: id, amount: dec("1.00"), respond_to: d_tx })
            .await.unwrap();

        assert!(matches!(d_rx.await.unwrap(), Err(PaymentError::InsufficientFunds)));
    }

    #[tokio::test]
    async fn debit_on_unknown_client_returns_not_found() {
        let tx = spawn_actor().await;

        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(Command::Debit { client_id: 999, amount: dec("100.00"), respond_to: resp_tx })
            .await.unwrap();

        assert!(matches!(resp_rx.await.unwrap(), Err(PaymentError::ClientNotFound)));
    }

    #[tokio::test]
    async fn debit_with_zero_amount_returns_negative_amount_error() {
        let tx = spawn_actor().await;
        let id = create_client(&tx, "DOC-Z2").await;

        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(Command::Debit { client_id: id, amount: Decimal::ZERO, respond_to: resp_tx })
            .await.unwrap();

        assert!(matches!(resp_rx.await.unwrap(), Err(PaymentError::NegativeAmount)));
    }

    #[tokio::test]
    async fn get_balance_returns_client_with_correct_data() {
        let tx = spawn_actor().await;

        let (create_tx, create_rx) = oneshot::channel();
        tx.send(Command::CreateClient {
            document_number: "41982912".to_string(),
            client_name: "Juan Perez".to_string(),
            birth_date: birth_date(),
            country: "AR".to_string(),
            respond_to: create_tx,
        })
        .await.unwrap();
        let id = create_rx.await.unwrap().unwrap();

        let (g_tx, g_rx) = oneshot::channel();
        tx.send(Command::GetBalance { client_id: id, respond_to: g_tx }).await.unwrap();

        let client = g_rx.await.unwrap().unwrap();
        assert_eq!(client.client_id, id);
        assert_eq!(client.client_name, "Juan Perez");
        assert_eq!(client.document_number, "41982912");
        assert_eq!(client.balance, Decimal::ZERO);
    }

    #[tokio::test]
    async fn get_balance_on_unknown_client_returns_not_found() {
        let tx = spawn_actor().await;

        let (g_tx, g_rx) = oneshot::channel();
        tx.send(Command::GetBalance { client_id: 999, respond_to: g_tx }).await.unwrap();

        assert!(matches!(g_rx.await.unwrap(), Err(PaymentError::ClientNotFound)));
    }

    #[tokio::test]
    async fn store_balances_resets_all_balances_to_zero() {
        let tx = spawn_actor().await;
        let id = create_client(&tx, "41982912").await;

        let (c_tx, c_rx) = oneshot::channel();
        tx.send(Command::Credit { client_id: id, amount: dec("999.99"), respond_to: c_tx })
            .await.unwrap();
        c_rx.await.unwrap().unwrap();

        let (s_tx, s_rx) = oneshot::channel();
        tx.send(Command::StoreBalances { respond_to: s_tx }).await.unwrap();
        s_rx.await.unwrap().unwrap();

        let (g_tx, g_rx) = oneshot::channel();
        tx.send(Command::GetBalance { client_id: id, respond_to: g_tx }).await.unwrap();
        assert_eq!(g_rx.await.unwrap().unwrap().balance, Decimal::ZERO);
    }

    #[tokio::test]
    async fn multiple_clients_maintain_independent_balances() {
        let tx = spawn_actor().await;
        let id_a = create_client(&tx, "41982912").await;
        let id_b = create_client(&tx, "41982913").await;

        let (c_tx, c_rx) = oneshot::channel();
        tx.send(Command::Credit { client_id: id_a, amount: dec("100.00"), respond_to: c_tx })
            .await.unwrap();
        c_rx.await.unwrap().unwrap();

        let (c_tx, c_rx) = oneshot::channel();
        tx.send(Command::Credit { client_id: id_b, amount: dec("250.00"), respond_to: c_tx })
            .await.unwrap();
        c_rx.await.unwrap().unwrap();

        let (g_tx, g_rx) = oneshot::channel();
        tx.send(Command::GetBalance { client_id: id_a, respond_to: g_tx }).await.unwrap();
        assert_eq!(g_rx.await.unwrap().unwrap().balance, dec("100.00"));

        let (g_tx, g_rx) = oneshot::channel();
        tx.send(Command::GetBalance { client_id: id_b, respond_to: g_tx }).await.unwrap();
        assert_eq!(g_rx.await.unwrap().unwrap().balance, dec("250.00"));
    }

    #[tokio::test]
    async fn balance_cannot_go_below_zero_via_debit() {
        let tx = spawn_actor().await;
        let id = create_client(&tx, "41982912").await;

        let (c_tx, c_rx) = oneshot::channel();
        tx.send(Command::Credit { client_id: id, amount: dec("100.00"), respond_to: c_tx })
            .await.unwrap();
        c_rx.await.unwrap().unwrap();

        let (d_tx, d_rx) = oneshot::channel();
        tx.send(Command::Debit { client_id: id, amount: dec("100.01"), respond_to: d_tx })
            .await.unwrap();
        assert!(matches!(d_rx.await.unwrap(), Err(PaymentError::InsufficientFunds)));

        let (g_tx, g_rx) = oneshot::channel();
        tx.send(Command::GetBalance { client_id: id, respond_to: g_tx }).await.unwrap();
        assert_eq!(g_rx.await.unwrap().unwrap().balance, dec("100.00"));
    }
}
