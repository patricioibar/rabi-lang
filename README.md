# rabi-lang

Un lenguaje de programación básico implementado en Rust.

Este proyecto se desarrolló como Trabajo Práctico para la materia Lenguajes y Compiladores de la Facultad de Ingeniería de la Universidad de Buenos Aires.

## Instalación

Para instalar el proyecto se debe tener instalado [Rust y Cargo](https://www.rust-lang.org/tools/install).

Luego de clonar el repositorio, se puede instalar el proyecto ejecutando el siguiente comando en la raíz del proyecto:

```bash
make install
```

Todos los comandos del proyecto están en el `Makefile`. Para ver la lista completa:

```bash
make
```

## Uso

```bash
rabi [ --backend interpreter | --backend vm ] [ --scanning | --parsing ] [ <archivo> ]
```

Si se provee un archivo, el programa lo ejecutará y mostrará el resultado en la salida estándar. Sin archivo, abre el modo interactivo (REPL) que conserva las variables y funciones entre entradas.

Sin instalar el binario, se puede correr un programa desde la raíz del proyecto con:

```bash
make run FILE=<archivo> [ ARGS="--backend vm" ]
```

### Intérprete o compilador

El parámetro `--backend` elige cómo se ejecuta el programa:

- `--backend interpreter` (valor por defecto): ejecuta con el intérprete (tree-walk interpreter).
- `--backend vm`: compila el programa a bytecode y lo ejecuta en una máquina virtual.

### Etapas intermedias

Los parámetros de `scanning` y `parsing` son mutuamente excluyentes. Si se provee alguno de ellos, el programa detendrá la ejecución en la etapa correspondiente y mostrará el resultado en la salida estándar. En ese caso `--backend` no tiene efecto.

## Entrega parcial - Intérprete

Leer [ENTREGA_PARCIAL.md](ENTREGA_PARCIAL.md) para más información sobre la entrega parcial del proyecto.

## Tests de integración
Se encuentran tests de integración en el directorio `tests/`. Para ejecutarlos, se puede usar el siguiente comando:

```bash
make test
```

Los tests corren los programas encontrados en `tests/test-programs/`, los cuales son equivalentes a los provistos por la cátedra, adaptados para el lenguaje `rabi`. Cada programa se ejecuta con los dos backends: `tests/interpreter.rs` usa el intérprete y `tests/vm.rs` usa el compilador y la VM.También se verifica que la salida sea la misma en las dos ejecuciones.

## Benchmark
Se encuentra adjunto un benchmark. Se implementó un mismo programa en rabi, Python y Rust. Para correrlo y ver los tiempos de ejecución, se puede usar el siguiente comando:

```bash
make bench
```

Es requisito previo tener instalado Python 3 y Rust. El benchmark compila el programa en Rust y luego ejecuta los tres programas, mostrando los tiempos de ejecución. No verifica nada ni forma parte de la suite de tests: ni `make test` ni `cargo test` lo corren.
