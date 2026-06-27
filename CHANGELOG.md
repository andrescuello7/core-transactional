# Changelog

## [0.0.4] 2026-06-27

### Changed
- feature: se implemento servicios de storage y se valido la creacion de los archivos dentro del `docs/data` - actualmente estamos dejando en 0 los balances de los clientes una ves generado el reporte.
    - Se implemento el manejo de mcps para los channels que manejan un buffer con las peticiones actuales, este mismo tiene un padding de 10000. La idea con esto es ponerle un tope al servicio de peticiones a espera y manejar asi la concurrencia un poco mas estable

## [0.0.3] 2026-06-26

### Changed
- refactor: Se implemento el storage para el mantenimiento de los datos y guardarlos en disco, por otro lado las rutas de `new_credit_transaction` - `new_debit_transaction` - `store_balances`, dato importante una ves solicitado el repote limpiamos los balance de los usuarios.

## [0.0.2] 2026-06-25

### Changed
- refactor: se removió el prefijo `/api` de las rutas HTTP para simplificar la estructura de endpoints. Ahora los endpoints están directamente en `/new_client` y `/client_balance/{user_id}`.

## [0.0.1] 2026-06-24

### Changed
- feat: primeros commits se hizo la implementacion se los servicios del servidor en el `main.rs` y se creo el servidor http en artrix web para el manejo de las peticiones http entrantes con la ruta GET "\" -> "hello world!".

