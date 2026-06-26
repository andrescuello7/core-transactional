## GUARDAR EN MEMORIA

El requerimiento es un híbrido muy interesante: tenemos que llevar el saldo en memoria viva para la operación en tiempo real y persistirlos en un archivo en disco de manera diferida/atómica cuando se invoque el endpoint **"/store_balances"**.  
Para ponernos bien técnicos a bajo nivel sobre cómo estructurar la Escritura en Memoria en el Motor Transaccional de Rust antes de impactar el archivo I/O, evaluemos las 3 alternativas de bajo nivel y elijamos la mejor

### Alternativas Técnicas para el Almacenamiento en Memoria

1. HashMap tradicional protegido por Locks globales (Arc<RwLock<HashMap<u64, Client>>>)
    - Cómo funciona: Se aloja un mapa estándar en el Heap de la RAM. Para acceder a él desde los hilos de Actix Web, se envuelve en un puntero atómico compartido (Arc) y un Lock de lectura/escritura (RwLock).
    - RAM: 
        - Cada consulta de saldo (GET) adquiere un Read Lock (hilos en paralelo). 
        - Cada transacción de crédito/débito (POST) adquiere un Write Lock exclusivo.  
    - Problema a bajo nivel: Contención de hilos (Thread Contention). Si entran miles de transaccionesconcurrentes, los hilos del procesador se quedan en un estado de spinning o son suspendidos por el sistemaoperativo esperando que se libere el Lock global del mapa, generando latencia (tail latency). `link: https://doc.rust-lang.org/std/sync/struct.RwLock.html`


2. Concurrencia segmentada por Sharding (DashMap<u64, Client>)
    - Cómo funciona: DashMap es una crate de la comunidad de Rust ultra optimizada que implementa un HashMap concurrente. 
    - RAM: En lugar de bloquear toda la memoria con un único Lock, divide el mapa en $N$ fragmentos (shards), usualmente emparejados con la cantidad de cores de la CPU. 
        - Si una transacción modifica al Cliente A (Shard 1), el Cliente B (Shard 2) puede ser modificado en paralelo sin tocar el mismo espacio de memoria mutuable.
    - Problema a bajo nivel: Resuelve la contención en la RAM de forma brillante para operaciones aisladas, pero complica la atomicidad del requerimiento store_balances. `link: https://github.com/xacrimon/dashmap`

---
***Ganadora***

3. Estado Confinado a un Único Hilo vía Canales (std::collections::HashMap dentro de un Actor Loop)
    - Cómo funciona: El mapa de la memoria RAM de los clientes se declara como una variable local no compartida dentro de la tarea asíncrona de Tokio (nuestro Motor Transaccional).  
    - RAM: No existen Arc, ni RwLock, ni primitivas de sincronización concurrentes sobre el mapa. 
    - La memoria es propiedad exclusiva de un único bucle secuencial que procesa mensajes de un canal MPSC de alta velocidad.

    - ***Eficiencia de bajo nivel***: Las lecturas y escrituras en la RAM son rapidas (cero sobrecosto de Locks / Kernel bypass a nivel de sincronización). Rust garantiza en tiempo de compilación que ningún otro hilo puede viciar o corromper esa porción de la memoria. `link: https://tokio.rs/tokio/tutorial/channels`

---

**Los Dos Tipos de Locks más Comunes**
En el ecosistema de Rust y en la informática en general, nos encontramos principalmente con dos variantes:

1. Mutex (Mutual Exclusion)
- Es el cerrojo más estricto. 
    - Solo un hilo a la vez puede acceder al dato, sin importar si solo quiere leerlo o modificarlo.
- Si el Hilo A está leyendo el saldo de un cliente, el Hilo B (que también solo quiere leer) tiene que esperar a que el Hilo A termine.

2. RwLock (Read-write Lock)
Es un cerrojo más inteligente y eficiente para ciertos escenarios. Separa los permisos en dos categorías:
- Lectores Concurrentes: Múltiples hilos pueden adquirir el lock de lectura al mismo tiempo. Si 50 personas quieren consultar su saldo (GET), entran todas a la vez sin esperarse.
- Escritor Exclusivo: Si un hilo quiere modificar el dato (un POST de crédito o débito), necesita acceso exclusivo. El lock bloquea a todos los lectores y a otros escritores hasta que la modificación termine.