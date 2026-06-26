## Mini Payment Processor

Implementación de un procesador de pagos desarrollada en Rust.

- Requisitos
- Rust (última versión estable)
- Cargo

### Ejecutar en desarrollo
`cargo run`

La API quedará disponible en:

http://localhost:8080

### Ejecutar en producción

`
# Compilar el proyecto:
cargo build --release

# Ejecutar el binario generado:
./target/release/mini-payment-processor
`

### Ejecutar los tests
Correr los tests globalmente
`cargo test`

Las decisiones de arquitectura se encuentran documentadas en el directorio `docs/adr`.