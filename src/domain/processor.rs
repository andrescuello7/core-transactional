use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use chrono::{NaiveDate, Utc};
use tokio::sync::{mpsc, oneshot};
use rust_decimal::Decimal;
use crate::domain::dto; // Asumiendo tus structs base

/// 1. DEFINICIÓN DE COMANDOS (Mensajes del Canal MPSC)
/// Cada variante representa un caso de uso e incluye un canal `oneshot` 
/// para devolver la respuesta al handler de Actix Web de forma asíncrona.
pub enum Command {
    GetBalance{
        client_id: u64,
        respond_to: oneshot::Sender<Result<dto::client::Client, dto::errors::PaymentError>>,
    },
    CreateClient {
        document_number: String,
        client_name: String,
        birth_date: NaiveDate,
        country: String,
        balance: Decimal,
        respond_to: oneshot::Sender<Result<u64, dto::errors::PaymentError>>,
    },
    Credit {
        client_id: u64,
        amount: Decimal,
        respond_to: oneshot::Sender<Result<Decimal, dto::errors::PaymentError>>,
    },
    Debit {
        client_id: u64,
        amount: Decimal,
        respond_to: oneshot::Sender<Result<Decimal, dto::errors::PaymentError>>,
    },
    StoreBalances {
        respond_to: oneshot::Sender<Result<(), dto::errors::PaymentError>>,
    },
}

/// 2. EL ACTOR / MOTOR TRANSACTIONAL
pub struct TransactionProcessor {
    // El estado vive aquí en el Heap, CONFINADO a este struct.
    // Ningún otro hilo de la aplicación puede tocar este HashMap.
    clients: HashMap<u64, dto::client::Client>,
    next_id: u64,
    file_counter: u32,
}

impl TransactionProcessor {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            next_id: 1,
            file_counter: 1, // Contador histórico requerido para el nombre del archivo
        }
    }

    /// 3. EL BUCLE ASÍNCRONO CENTRAL (Tokio Task Loop)
    /// Este método se ejecuta en segundo plano. Escucha el canal MPSC de forma secuencial.
    pub async fn run(mut self, mut receiver: mpsc::Receiver<Command>) {
        log::info!("Motor Transaccional iniciado exitosamente.");

        // Mientras el canal esté abierto, procesa un mensaje a la vez
        while let Some(cmd) = receiver.recv().await {
            match cmd {
                Command::CreateClient { document_number, client_name, birth_date, country, balance, respond_to } => {
                    // Validación de documento único en la RAM (Rápido, sin Locks)
                    let doc_exists = self.clients.values().any(|c| c.document_number == document_number);
                    if doc_exists {
                        let _ = respond_to.send(Err(dto::errors::PaymentError::DuplicateDocument));
                        continue;
                    }

                    let id = self.next_id;
                    self.next_id += 1;

                    let new_client = dto::client::Client::new(id, client_name, birth_date, document_number, country, balance);
                    self.clients.insert(id, new_client);

                    let _ = respond_to.send(Ok(id));
                }

                Command::Credit { client_id, amount, respond_to } => {
                    if let Some(client) = self.clients.get_mut(&client_id) {
                        client.balance += amount; // Modificación directa y segura
                        let _ = respond_to.send(Ok(client.balance));
                    } else {
                        let _ = respond_to.send(Err(dto::errors::PaymentError::ClientNotFound));
                    }
                }

                Command::Debit { client_id, amount, respond_to } => {
                    if let Some(client) = self.clients.get_mut(&client_id) {
                        if client.balance < amount {
                            let _ = respond_to.send(Err(dto::errors::PaymentError::InsufficientFunds));
                        } else {
                            client.balance -= amount;
                            let _ = respond_to.send(Ok(client.balance));
                        }
                    } else {
                        let _ = respond_to.send(Err(dto::errors::PaymentError::ClientNotFound));
                    }
                }

                Command::StoreBalances { respond_to } => {
                    // --- COMIENZA FLUJO ATÓMICO I/O ---
                    // Al procesarse de forma secuencial, mientras estemos en este bloque, 
                    // ningún endpoint de Crédito/Débito puede alterar los saldos.
                    
                    let filename = format!("docs/data/{}_{}.DAT", Utc::now().format("%d%m%Y"), self.file_counter);
                    
                    match File::create(&filename) {
                        Ok(mut file) => {
                            let mut success = true;
                            
                            // 1. Volcado secuencial al archivo plano
                            for (id, client) in self.clients.iter() {
                                if let Err(e) = writeln!(file, "{} {}", id, client.balance) {
                                    log::error!("Error escribiendo en archivo: {}", e);
                                    success = false;
                                    break;
                                }
                            }

                            if success {
                                // 2. Reseteo estricto del balance en memoria pedido por el challenge
                                for client in self.clients.values_mut() {
                                    client.balance = Decimal::ZERO;
                                }
                                self.file_counter += 1; // Incrementamos contador para el próximo archivo
                                let _ = respond_to.send(Ok(()));
                            } else {
                                let _ = respond_to.send(Err(dto::errors::PaymentError::StorageError("Error al escribir en archivo".into())));
                            }
                        }
                        Err(_) => {
                            let _ = respond_to.send(Err(dto::errors::PaymentError::StorageError("Error al crear archivo".into())));
                        }
                    }
                    // --- TERMINA FLUJO ATÓMICO I/O ---
                }
                Command::GetBalance { client_id, respond_to } => {
                    if let Some(client) = self.clients.get(&client_id) {
                        let _ = respond_to.send(Ok(client.clone()));
                    } else {
                        let _ = respond_to.send(Err(dto::errors::PaymentError::ClientNotFound));
                    }
                }
            }
        }
    }
}