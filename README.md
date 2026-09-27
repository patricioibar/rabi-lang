# rabi-lang

Un lenguaje de programación básico implementado en Rust.

Este proyecto se desarrolló como Trabajo Práctico para la materia Lenguajes y Compiladores de la Facultad de Ingeniería de la Universidad de Buenos Aires.

## Instalación

Para instalar el proyecto se debe tener instalado [Rust y Cargo](https://www.rust-lang.org/tools/install).

Luego de clonar el repositorio, se puede instalar el proyecto ejecutando el siguiente comando en la raíz del proyecto:

```bash
cargo install --path .
```

## Uso

```bash
rabi <archivo>  [ --scanning | --parsing ]
```

Para ejecutar un compilador en modo interactivo, simplemente ejecutar el programa sin argumentos

Si se provee un archivo, el programa lo ejecutará y mostrará el resultado en la salida estándar.

Los parámetros de `scanning` y `parsing` son mutuamente excluyentes. Si se provee alguno de ellos, el programa detendrá la ejecución en la etapa correspondiente y mostrará el resultado en la salida estándar.

## Entrega parcial - Intérprete

## Tests de integración
Se encuentran tests de integración en el directorio `tests/`. Para ejecutarlos, se puede usar el siguiente comando:

```bash
cargo test --test interpreter
```

Los tests corren los programas encontrados en `tests/test-programs/`, los cuales son muy similares a los provistos por la cátedra, pero con algunas diferencias en la sintaxis y semántica.

## Benchmark
Se encuentran adjuntos tests de benchmark. Se implementó un mismo programa en rabi, Python y Rust. Para correrlos y ver los tiempos de ejecución, se puede usar el siguiente comando:

```bash
tests/run_benchmark.sh
```

Es requisito previo tener instalado Python 3 y Rust. El script compila el programa en Rust y luego ejecuta los tres programas, mostrando los tiempos de ejecución.
