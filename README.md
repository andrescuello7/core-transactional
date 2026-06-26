# Prex

Prex es una API HTTP en Rust para gestionar clientes y operaciones simples de saldo.

## Como levantar el proyecto

### Opcion 1: Local con Rust

Requisitos:
- Rust
- Cargo

Pasos:
1. Ejecutar: cargo run
2. La API queda disponible en: http://localhost:8080

### Opcion 2: Docker

Pasos:
1. Construir imagen: docker build -t prex:latest .
2. Levantar contenedor: docker run --rm -p 8080:8080 prex:latest

La API queda disponible en: http://localhost:8080

# Compilar el proyecto:
`cargo build --release`

# Ejecutar el binario generado:
`./target/release/mini-payment-processor`

### Ejecutar los tests
Correr los tests globalmente
`cargo test`

Las decisiones de arquitectura se encuentran documentadas en el directorio `docs/adr`.