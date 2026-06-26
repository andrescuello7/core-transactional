## Arquitecurea Hexagonal

Para este challenge en particular, la Arquitectura Hexagonal no es solo una decisión estética; resuelve de manera brillante los requerimientos específicos de concurrencia, persistencia en archivos y testing.  

---
### Vamos directamente con las ventajas que tenemos con la arquitectura:

1. Aislamiento Total del Estado en Memoria (Thread-Safety Impecable)
El requerimiento exige llevar el saldo de los clientes en memoria viva y operar de forma concurrente.  
- Arquitectura Común: 
    - La lógica HTTP (Actix) estaría mezclada con el acceso a la memoria, forzándote a llenar los handlers de `Arc<RwLock<HashMap>>`, exponiendo tu estado a condiciones de carrera o bloqueos mutuos (deadlocks).

- Arquitectura Hexagonal: 
    - El mapa en memoria viva se convierte en un detalle de implementación dentro del Adaptador de Infraestructura o el de Dominio. Tu lógica de negocio (por ejemplo, validar si el saldo es suficiente o el documento duplicado) es puramente síncrona, limpia de locks y agnóstica a cómo se sincronizan los hilos en la red.  


2. Flexibilidad Total para la Persistencia Rara (store_balances)
El challenge pide guardar los balances en un archivo con formato específico `ID_CLIENTE<espacio>BALANCE` y luego limpiar los balances en memoria dejando a los clientes en cero.  

- Ventaja Hexagonal: Al estar la lógica de negocio desacoplada en el centro, el proceso de "guardar y resetear" es simplemente un caso de uso. 
    - Si mañana el equipo de Prex te dice: "Oye, ya no queremos guardar en un archivo plano .DAT, ahora queremos persistir en Redis y mantener el historial", el Core de tu aplicación (Dominio) no cambia en absoluto. Solo desconectas el adaptador de archivos y conectas un adaptador de Redis.  

3. Testabilidad sin Modificar el File System (Inyección de Dependencias)
El criterio de revisión evaluará fuertemente la calidad de código, prolijidad y tests.  
- Ventaja Hexagonal: 
    - Al definir un Puerto de Salida (pub trait BalanceRepository), puedes escribir tests unitarios ultra rápidos para tu lógica transaccional inyectando un MockRepository en memoria. 
    - No necesitas que tus tests unitarios escriban archivos reales en el disco rígido de la máquina que evalúa el código, evitando archivos basura (*.DAT) durante la fase de cargo test.

4. Mapeo Directo a los Criterios de Evaluación (SOLID, DRY, KISS)
El documento menciona explícitamente seguir las buenas prácticas de diseño: KISS, DRY y SOLID.  

- La Arquitectura Hexagonal: 
    - Inversión de Dependencia (la D de SOLID): los módulos de alto nivel (tu core de pagos) no dependen de los de bajo nivel (Actix Web o el sistema de archivos); ambos dependen de abstracciones (Puertos/Traits).

    - Responsabilidad Única (la S de SOLID): el handler solo procesa HTTP , el dominio solo procesa reglas de negocio, y el repositorio solo sabe escribir strings en un archivo plano.  
