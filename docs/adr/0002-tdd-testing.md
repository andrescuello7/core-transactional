## TDD - Test Driven Desing

El debate sobre si usar TDD (Test-Driven Development / Design) es bueno o malo suele reducirse a eso: el nivel de incertidumbre del proyecto. Cuando estás explorando un producto sin requerimientos claros, hacer TDD te puede hacer sentir atrapado rediseñando tests a cada minuto.

### Pero en este desafio del CORE, la situación es la ideal para TDD: 
- En este caso tengo reglas de negocio sumamente claras, un alcance cerrado y la necesidad imperiosa de demostrar cualidades como prolijidad, buenas prácticas y pensamiento orientado a sistemas informáticos.  

- Ventajas de aplicar TDD:
    Diseño de Puertos Limpio (La "D" de TDD): 
    - Al escribir el test antes, te pones en los zapatos del "consumidor" de tu código. Estote obliga a definir contratos (Traits/Puertos) sencillos y lógicos para tus entidadesantes de enredarte con la implementación concurrente.Confianza en la Lógica Financiera: 
    
    - En sistemas transaccionales, errar un signo o permitir que un saldo quede negativo poruna condición de carrera es crítico. 
    - Con TDD aseguramos que los flujos de "saldo insuficiente" o "documento duplicado"queden blindados desde el minuto uno.  
    - Refactorización Segura hacia Canales: Como decidimos usar una arquitectura de mensajescon canales (MPSC), podemos desarrollar primero la lógica pura del dominio con TDD yluego meter la infraestructura de hilos de Tokio con la certeza absoluta de que elcomportamiento matemático del saldo no se va a romper.
        
- Cómo aplicar TDD:
    - Arquitectura Hexagonal:
    El TDD se aplica de manera quirúrgica en el Hexágono Central (Dominio) y en los Puertos.Miremos el ciclo de cómo estructuraríamos el primer flujo aplicando TDD