// La definición de poder poner cada funcion en un archivo es a terminos practicos del challenge
// En produccion se pondrian todas las funciones en un solo archivo.
// Si tenemos una API con un archivo por router no escalaría bien, pero para este challenge es suficiente.

pub mod get_client_balance;
pub mod new_client;
pub mod new_credit_transaction;
pub mod new_debit_transaction;
pub mod store_balances;
