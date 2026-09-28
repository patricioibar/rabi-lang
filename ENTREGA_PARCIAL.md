# Entrega Parcial TP Opción II - Lenguajes y Compiladores

<center>

| Patricio Ibar | pibar@fi.uba.ar |  109569 |
|---------------|:---------------:|--------:|

</center>


En esta entrega parcial se implementa un intérprete para la primera versión del lenguaje de programación `rabi`. 

Para detalles de cómo ejecutar el intérprete, ver [README.md](README.md).

## Índice

1. [Introducción](#introducción)
1. [Sintaxis, semántica y diferencias contra Lox](#sintaxis-semánticas-y-diferencias-contra-lox)
1. [Detalles de implementación](#detalles-de-implementación)
1. [Resultados de benchmarking](#resultados-de-benchmarking)
1. [Trabajo Futuro](#trabajo-futuro)
 
## Introducción

El presente proyecto consiste en la implementación de un *tree-walk interpreter* para un lenguaje de programación de autoría propia: `rabi`. El nombre del lenguaje proviene de mi apellido (*Ibar*) leido de atrás hacia adelante.

Decidí optar por la opción II para el trabajo práctico para no estar ligado a la sintaxis, semántica y funcionalidades de Lox, y poder experimentar con un lenguaje propio.

Este documento describe en qué consiste la entrega parcial, y cuáles son los objetivos que se pretenden alcanzar en la entrega final.

## Sintaxis, semántica y diferencias contra Lox

Tanto la sintaxis como la semántica de `rabi` están fuertemente influenciadas por lenguajes modernos como Python, con una estructura simple y legible.

A continuación un listado con las diferencias entre `rabi` y la implementación de Lox provista por la cátedra:
- **Bloques de código**: en `rabi` los bloques de código se definen por indentación, mientras que en Lox se definen por llaves `{}`. Esto supuso problemas adicionales principalmente en la fase de *scanning*. También pueden definirse bloques de código de una sola línea.
- **Tipos numéricos**: en `rabi` se implementan tanto enteros como flotantes, mientras que Lox solo implementa flotantes.
- **Control de flujo en bucles**: `rabi` implementa las sentencias `break` y `continue`.
- **Funciones y closures**: en `rabi` las funciones siempre se resuelven contra el ámbito global, es decir, no se implementan closures. 
- **Arrays**: en `rabi` se implementan arrays. Un array puede contener elementos de diferentes tipos, y se puede acceder  sus elementos mediante índices enteros. También se puede consultar el largo mediante la función nativa `len <array>`.
- **Bucles for**: los bucles `for` en `rabi` son similares a los de Python, su principal función es iterar sobre arrays.
- **Veracidad**: en `rabi` los valores `0`, `0.0`, `""`, `[]`, `null` y `false` son considerados *falsy*, mientras que todos los demás valores son considerados *truthy*.

También hay algunas diferencias menores en la sintaxis, como el uso de `:` para indicar el inicio de un bloque de código, el uso de `#` para comentarios, `let` para declarar variables, `func` para declarar funciones, y la ausencia del punto y coma `;` al final de las líneas de código.

Programa de ejemplo escrito en `rabi`:

```python
func sum_if_positive(arr):
    let total = 0
    for i in arr:
        if i > 0: total = total + i
    return total

print sum_if_positive([1, 2, 3, -4, 5]) # prints 11
```

### Otras características del lenguaje

Además de las diferencias listadas arriba, `rabi` implementa:

- **Operadores lógicos**: `and` y `or` evalúan con corto circuito. `or` devuelve el operando que definió el resultado (puede devolver valores no booleanos), mientras que `and` devuelve únicamente su veracidad como booleano.
- **Aritmética mixta**: las operaciones entre enteros y flotantes devuelven el resultado como flotante.
- **Verificación de overflow**: el overflow de enteros se verifica y se reporta como error en tiempo de ejecución.
- **Operaciones sobre arrays**: los arrays se pueden concatenar con `+` (`[1] + [2]` devuelve `[1, 2]`) y sus elementos se pueden asignar por índice (`arr[0] = 99`).

## Detalles de implementación

El lenguaje está implementado en Rust, y se compone de las fases propuestas en clase: *scanning*, *parsing* y *interpreting*. Cada una se encuentra implementada en un módulo separado (carpetas `src/scanner`, `src/parser` y `src/interpreter`), mientras que en la raíz `src/` se encuentran, junto con main, los módulos `token`, `expression` y `statement`, que contienen las estructuras de datos que representan los tokens, expresiones y sentencias del lenguaje, y que son compartidas por todas las fases.

En `src/main.rs` puede identificarse el flujo scanner -> parser -> interpreter (principalmente en la función `file_mode`). Se puede correr el programa con los flags `--scanning` o `--parsing` para ver el resultado de cada fase.

La estrategia de implementación del lenguaje es muy similar a la implementación de Lox provista por la cátedra.

### Scanner - Análisis léxico

Se divide en un `tokenizer` que se encarga de leer el código fuente y generar los tokens, y un `scanner` que coordina el tokenizer y se encarga de manejar los niveles de indentación y tokens delimitadores de línea y archivo.

El `tokenizer` es una abstracción sin estado. Simplemente se encarga de tomar un cursor, leer el código fuente y devolver el siguiente token. 

El `scanner`, en cambio, mantiene un estado que le permite manejar los niveles de indentación. Antes de tokenizar una línea de código, verifica el nivel de indentación actual, compara contra la cantidad de bloques abiertos y genera tokens `Indent` / `Dedent` para apertura y cierre de bloques según corresponda. Al finalizar una línea agrega tokens `Newline` y al finalizar el archivo agrega tokens `Dedent` hasta cerrar todos los bloques abiertos, y finalmente un token `Eof`.

### Parser - Análisis sintáctico

El parser es un *recursive descent parser* que implementa la gramática del lenguaje. Toma la lista de tokens generados por el *scanner*, reconoce las estructuras sintácticas del lenguaje y genera un *abstract syntax tree* (AST) compuesto por `expressions` y `statements`. El resultado de la etapa de parsing es un vector de `statements` que representan el programa completo.

Los tipos de expresiones y sentencias están definidos en los módulos `src/expression.rs` y `src/statement.rs`, y son enums sin lógica. El parser los implementa sus "constructores" y el intérprete los "ejecuta".

### Interpreter - Interpretación y ejecución

El módulo `src/interpreter` expone una interfaz mínima para su utilización externa, el struct `Interpreter`. La lógica de interpretación y ejecución del lenguaje se encuentra en el módulo `src/interpreter/runtime`, que contiene los submódulos `expressions` y `statements`, que implementan la ejecución de expresiones y sentencias respectivamente.

A continuación algunos detalles interesantes de la implementación del intérprete:
- **Scopes**: los bindings y definiciones en un scope se implementa con un `HashMap` que mapea nombres de variables a valores. Cada scope tiene un puntero a su scope padre implementado como un `Rc<RefCell<Scope>>` para permitir múltiples referencias y mutabilidad de los scopes padres.
- **Valores**: los valores se implementan mediante un enum `Value` que puede contener diferentes tipos de datos, incluyendo enteros, flotantes, booleanos, strings, arrays y funciones. Sobre el mismo enum se encuentran implementados los operadores binarios aritméticos y de comparación, por lo que se puede leer la lógica de evaluación de expresiones directamente en un mismo submódulo.
- **Control de flujo**: en la implementación de la cátedra, se implementa `return` mediante el lanzado de excepciones. En este proyecto decidí implementarlo mediante un enum `ControlFlow` que puede contener un valor de retorno, o una señal de `break` o `continue`.

## Resultados de benchmarking

Para medir el desempeño del intérprete se implementó un mismo programa en rabi, Python y Rust (puede encontrarse en `/tests/benchmark`). El programa consiste en un bucle de un millón de iteraciones que realiza operaciones con valores enteros, flotantes y strings.

El test de benchmark puede correrse usando el comando `make bench` desde la raíz del proyecto. El mismo compila el programa de Rust optimizado, y luego corre los tres programas, mostrando los tiempos de ejecución.

Las mediciones se tomaron en una notebook con procesador Intel Core i3-1215U y 8 GB de RAM, corriendo Ubuntu 26.04 LTS (kernel 7.0), con `rustc` 1.98.1. Los tiempos son de proceso completo, por lo que incluyen el arranque de cada intérprete.

Los resultados obtenidos fueron los siguientes:
| Lenguaje | Corrida 1 | Corrida 2 | Corrida 3 | Promedio |
|----------|----------:|----------:|----------:|---------:|
| Rust (compilado, `-O`) | 12 ms | 11 ms | 9 ms | **10 ms** |
| Python 3.14.7 (CPython) | 220 ms | 211 ms | 212 ms | **214 ms** |
| `rabi` (release) | 567  ms | 564 ms | 566 ms | **565 ms** |

Como era de esperar, el más rápido fue el programa compilado en Rust, seguido por el intérprete de Python, y finalmente el intérprete de `rabi`.

Podemos observar que el intérprete de `rabi` es aproximadamente 2.6 veces más lento que el intérprete de Python. Aún considerando que el intérprete de Python es un proyecto mucho más grande y maduro, personalmente considero que la performance medida no está nada mal, teniendo en cuenta que no se agregaron optimizaciones de ningún tipo. 

La intención de este benchmark es meramente ilustrativo y no pretende analizar o criticar en profundida el desempeño de los lenguajes. Servirá para contrastar contra la versión compilada del lenguaje que se implementará en la entrega final, y ver si el desempeño es comparable al de un lenguaje compilado.

Para mejorar la performance del intérprete se podrían implementar optimizaciones como un *resolver* de variables que evite la búsqueda de variables en scopes padres.

## Trabajo Futuro

Para la entrega final se proponen los siguientes objetivos:

- Mejorar el manejo de errores, para que sean más descriptivos y fáciles de localizar.
- Implementar un *resolver* de variables para mejorar la performance del intérprete.
- Implementar un compilador para el lenguaje `rabi`.
- Opcionalmente, agregar más funcionalidades al lenguaje. Si logro implementar los dos objetivos anteriores con tiempo de sobra, me gustaría implementar algo parecido a las goroutines y channels de Go, para poder realizar programación concurrente.