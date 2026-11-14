# Ejemplos

Dos programas escritos en rabi.

## Intérprete de Brainfuck

[`brainfuck.rabi`](brainfuck.rabi) es un intérprete de [Brainfuck](https://es.wikipedia.org/wiki/Brainfuck) escrito enteramente en rabi. Implementa las ocho instrucciones del lenguaje (`>`, `<`, `+`, `-`, `.`, `,`, `[`, `]`) sobre una cinta de 30000 celdas de 8 bits.

### Uso

Desde la raíz del repositorio:

```bash
make run FILE=example/brainfuck.rabi
make run FILE=example/brainfuck.rabi ARGS="--backend vm"
```

o, si previamente instalaste rabi con `make install`:

```bash
rabi example/brainfuck.rabi
rabi example/brainfuck.rabi --backend vm
```

Trae cuatro programas de ejemplo y los corre uno atrás del otro:

```
Intérprete de Brainfuck en rabi

--- hola mundo (106 instrucciones) ---
Hello World!

--- eco de la entrada (17 instrucciones) ---
rabi
```

### Cómo correr otro programa

Un programa de Brainfuck se escribe como un array de strings de un carácter:

```python
# ++++++++[>+++++++++<-]>.
let PROGRAMA = ["+"] * 8 + ["["] + [">"] + ["+"] * 9 + ["<"] + ["-"] + ["]"] + [">"] + ["."]

correr("mi programa", PROGRAMA, [])
```

El tercer argumento de `correr` es lo que consume la entrada al programa (consumida por la instrucción `,`), tiene que ser un array de caracteres. Cuando se agota, `,` devuelve 0:

```python
correr("eco", [","] + ["["] + ["."] + [","] + ["]"], ["h", "o", "l", "a"])
```

La salida se acumula y se imprime cuando el programa emite un salto de línea (código 10) o cuando termina.

Dos limitaciones conocidas a tener en cuenta:
- El intérprete no puede leer un programa desde la entrada estándar, porque `input` devuelve un string y después no hay forma de descomponerlo.
- No puede emitir el carácter `"` (código 34). rabi no tiene secuencias de escape y las comillas dobles son el único delimitador de strings, así que ese carácter no se puede escribir en un literal.

## Ta-te-ti

[`tateti.rabi`](tateti.rabi) es un ta-te-ti por consola para dos jugadores, que usa la expresión `input` para pedir las jugadas.

### Uso

```bash
make run FILE=example/tateti.rabi
make run FILE=example/tateti.rabi ARGS="--backend vm"
```

o con rabi instalado:

```bash
rabi example/tateti.rabi
rabi example/tateti.rabi --backend vm
```

### Cómo jugar

Cada jugador elige una casilla escribiendo su número (1 a 9):

```
 1 | 2 | 3
---+---+---
 4 | 5 | 6
---+---+---
 7 | 8 | 9
```

El primero en completar una fila de 3 (fila, columna o diagonal) gana.
