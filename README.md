# Prex - Core Transaccional

Motor transaccional de alto rendimiento construido en Rust para procesar operaciones de crédito/débito en tiempo real, manteniendo balances en memoria y persistiéndolos en archivos de forma atómica.
La idea central de este microservicio es gestionar transacciones de los usuarios en tiempo real. Su arquitectura mantiene los balances en memoria RAM para operaciones rápidas y los persiste en disco bajo demanda. El principal desafío fue diseñar un sistema capaz de manejar alta concurrencia sin degradar el rendimiento.

Para resolver esto, se evaluaron varias alternativas de sincronización —incluyendo Mutex y primitivos propios de Tokio— pero se optó por un patrón Actor Model con MPSC (Multi-Producer, Single-Consumer). Este enfoque crea un canal de comunicación donde todas las peticiones HTTP entrantes son encoladas y procesadas secuencialmente por un único actor asíncrono, eliminando la contención de locks y garantizando atomicidad operacional.

- **Gestión de clientes**: Crear y consultar información de clientes
- **Transacciones en tiempo real**: Operaciones de crédito/débito con atomicidad garantizada
- **Persistencia híbrida**: Balances en RAM para operaciones rápidas + persistencia en disco bajo demanda
- **Concurrencia segura**: Manejo de múltiples clientes simultáneamente sin locks contención
---
<img width="925" height="546" alt="Captura de pantalla 2026-06-27 a la(s) 12 47 53 p  m" src="https://github.com/user-attachments/assets/ffce64c8-cdc8-4604-bdf4-83647aa235cf" />


### Algunas Características Clave
**Hexagonal Architecture**: Dominio desacoplado de infraestructura  
**Type-Safe**: Rust garantiza memory safety y thread safety en compilación  
**High-Performance**: MPSC channels para IPC de bajo overhead  


### Decisiones de Arquitectura (ADR)
Las decisiones críticas del proyecto están documentadas en **Architecture Decision Records** (ADR).
Un ADR es un documento que captura una decisión arquitectónica importante, sus contexto, opciones evaluadas y consecuencias. Esto permite:
- Entender **por qué** se eligió una solución (no solo el qué)
- Documentar tradeoffs evaluados
- Facilitar onboarding de nuevos desarrolladores

**ADRs del Proyecto**

#### - [0001-use-hexagonal-architecture.md](docs/adr/0001-use-hexagonal-architecture.md)
#### - [0002-tdd-testing.md](docs/adr/0002-tdd-testing.md)
#### - [0003-memory-alloc.md](docs/adr/0003-memory-alloc.md)

---

## Instalación y Ejecución

### Requisitos Previos

- **Rust** 1.71+ ([Install](https://rustup.rs/))
- **Cargo** (viene con Rust)
- **Docker** (opcional, para producción)

### Correr en Local

#### Opción 1: Desde el código fuente

```bash
# Clonar/navegar al proyecto
git clone https://github.com/andrescuello7/core-transactional
cd /core-transactional

# Ejecutar en modo debug
cargo run --bin core-transactional

# Output esperado
# INFO [main] HTTP server listening on 127.0.0.1:8080
# INFO [alloc_memory] Motor Transaccional iniciado exitosamente
```

La API estará disponible en `http://127.0.0.1:8080`
#### Opción 2: Build + Run

```bash
# Compilar en modo debug (más rápido para desarrollo)
cargo build

# Ejecutar el binario
./target/debug/core-transactional

# O compilar en modo release (optimizado)
cargo build --release
./target/release/core-transactional
```

### Correr en Producción

#### Opción 1: Docker

```bash
# Construir imagen
docker build -t prex:latest .

# Ejecutar contenedor
docker run \
  --rm \
  -p 8080:8080 \
  -e RUST_LOG=info \
  prex:latest

# Con persistencia de volumen
docker run \
  --rm \
  -p 8080:8080 \
  -v $(pwd)/docs/data:/app/docs/data \
  -e HOST=0.0.0.0 \
  -e PORT=8080 \
  prex:latest
```

## Ejecutar los Tests

### Suite Completa de Tests

```bash
# Ejecutar todos los tests
cargo test

# Ejecutar con output detallado
cargo test -- --nocapture

# Ejecutar tests de un módulo
cargo test domain::

# Ejecutar el test concurrente
cargo run --bin core-transactional
cargo run --bin stress_test
```
---

### Links de Referencia
- https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.channel.html
- https://doc.rust-lang.org/nomicon/vec/vec-alloc.html?search=
- https://tokio.rs/tokio/tutorial/channels
- https://doc.rust-lang.org/std/sync/struct.RwLock.html
- https://github.com/xacrimon/dashmap

---
**Author:** Andres Cuello
