Es buena practica concidero usar los READMEs para explicar como levantar el proyecto pero no extenderla como un documento. En mi experiencia con los equipos implementamos los ARD (Architecture Decision Records) para contemplar los casos que queremos tener en cuenta para la toma de decisiones

Un ADR es un documento que responde principalmente a estas preguntas:

- ¿Qué decisión se tomó?
- ¿Por qué se tomó?
- ¿Qué alternativas se evaluaron?
- ¿Cuáles son las consecuencias de esa decisión?

---

**mini procesador de pagos**

***Puntos importantes***
- Conocimiento del lenguaje Rust.
- Prolijidad.
- Buenas prácticas de programación. (solid, clean architecture, etc)
- Capacidad de investigación. (arquitectura - funcionamiento)
- Pensamiento orientado a concurrencia y sistemas informáticos. (Threads - Procesos - Memory)

**POST /new_client**
`{
    "client_name": "Andres Cuello",
    "birth_date": "2001-02-03",
    "document_number": "43226461", //***UNIQUE!***
    "country": "Argentina"
}`

**POST /new_credit_transaction**
`{
    "client_id": 1,
    "credit_amount": 100.0
}`

**POST /new_debit_transaction**
`{
    "client_id": 1,
    "debit_amount": 100.0
}`

**POST /store_balances**
**GET /client_balance/{ID}**

---

**File of Clients**

***Sample***
01 503.00

02 1999.07

03 9.86
 
***format***
ID_CLIENTE<espacio>BALANCE<Salto de línea>
ID_CLIENTE<espacio>BALANCE<Salto de línea>
ID_CLIENTE<espacio>BALANCE<Salto de línea>

---

**Tipos de STATUS_HTTP**

- 200 OK
- 201 Created
- 400 Bad Request
- 404 Not Found Account or Client
- 500 Internal Server Error