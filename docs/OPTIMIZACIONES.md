# Optimizaciones de la VM de rabi

Cuando la VM empezó a funcionar, era 1,44 veces **más lenta** que el intérprete (1363 ms contra 945 ms).

Antes de ponerme a optimizar, medí dónde había más cuello de botella usando `perf` sobre el benchmark, un bucle de un millón de vueltas que usa solo variables globales. También usé una versión del mismo programa con variables locales. Los scripts de perfilado están en [perfilado/](../perfilado/).

Encontré los siguientes resultados:

- La mayor parte del tiempo de ejecución estaba en acceder a las variables globales: cada acceso copiaba el nombre de la variable a un `String` nuevo y lo buscaba en un `HashMap` usando un hash por defecto de Rust. Aun sin esas copias, la búsqueda en el `HashMap` seguía siendo casi la mitad del tiempo.
- Otra parte importante se iba en copiar valores a la pila (`GetConstant` ~15 %, `GetLocal` y `SetLocal` ~30 % con locales).
- La aritmética (~8 %) y el despacho de operaciones (~7 %) en relación pesaban poco.

Los porcentajes son del perfil tomado después de quitar las copias de los nombres (sección 1).

Por lo tanto, apliqué las siguientes optimizaciones en orden de importancia:

### 1. Globales sin copiar el nombre

Cada `GetGlobal` y `SetGlobal` creaba un `String` con el nombre de la variable solo para buscarlo, lo cual provocaba alrededor de 12 millones de reservas de memoria en el benchmark. Esto inicialmente me sorprendió porque había implementado los nombres de las variables con punteros inteligentes (`Rc<String>`) para evitar eso. La solución fue muy sencilla, simplemente faltaba agregar `as_str()` en donde era necesario y arreglar algunos tipos en parámetros y retornos de funciones. 

El tiempo total de la VM en el benchmark pasó de 1363 a 678 ms (−50 %).

### 2. Hasher rápido para las globales

El `HashMap` de Rust usa SipHash, que resiste ataques de colisiones pero es lento para claves cortas. Reemplazando el hasher por defecto por un hasher no criptográfico de 15 líneas se obtuvo un −21,6 % en el tiempo de ejecución del benchmark con globales.

### 3. Copiar los valores una sola vez

Para apilar un valor (una constante o una variable), la VM primero lo copiaba a una variable temporal y después copiaba esa temporal a la pila. Además, al buscar una constante se devolvía envuelta en un `Result`, lo cual suponía una copia adicional más.

La solución fue copiar cada valor directamente a su lugar final en la pila (con `extend_from_slice` y `extend_from_within`) y devolver las constantes por referencia. En total se redujo el tiempo de ejecución alrededor de 15 % con globales y 33 % con locales.

Con las asignaciones de variables pasaba lo mismo, `SetLocal` y `SetGlobal` clonaban el valor a un temporal antes de asignarlo. La solución fue usar `clone_from` que se escribe directo sobre la variable. Con esta adición se redujo el tiempo un 10,5 % con locales.

### 4. LTO

LTO (*link-time optimization*) es una opción de compilación que se activa en el perfil de release de `Cargo.toml`:

```toml
[profile.release]
lto = true
codegen-units = 1
```

Con estas opciones, el compilador optimiza todo el crate junto. Se obtuvo −12,1 % en tiempo de ejecución con locales.

### 5. Atajo para la aritmética

Probé resolver la aritmética entre números directo en la VM, sin pasar por `Value`. Esta optimización reducía alrededor de un 10 % el tiempo de ejecución con variables locales, pero duplicaba lógica que ya estaba en `Value`, lo cual no solo hacía el código menos limpio, sino que también menos mantenible. 

Como este proyecto es académico y no de producción, decidí quitar la optimización para mantener la fuente de la verdad de la aritmética en un único lugar para la VM y el intérprete.

### Resultado de las optimizaciones a la VM

Después de aplicar todas las optimizaciones, la VM terminó siendo alrededor de 2 veces más rápida que el intérprete (296 ms contra 602 ms en `make bench`).

# Optimizaciones sobre el código generado

Finalmente, decidí implementar la optimización **Constant Folding** sobre el código generado. Decidí implementarla para probar cómo se realizaría la optimización más que por el rendimiento en sí.

Implementé **Constant Folding** agregando una etapa adicional entre el parser y el compilador, a la cual decidí llamar [optimizer](../src/optimizer/mod.rs). `optimizer` recorre el AST y reemplaza por un literal cada operación entre literales (por ejemplo, `60 * 60 * 24` se compila como `86400`). El resultado se calcula con los mismos métodos de `Value` que usa la VM. Si una operación fallaría, como `1 / 0`, se deja sin plegar para que el error ocurra al ejecutar.

El benchmark no tiene expresiones constantes, así que su tiempo no cambió (277 ms contra 280 ms, dentro del ruido). En un bucle con expresiones constantes ([perfilado/benchmark-constantes.rabi](../perfilado/benchmark-constantes.rabi)), el tiempo bajó de 130 a 87 ms (−33 %).
