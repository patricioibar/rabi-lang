# Ta-te-ti

Ta-te-ti por consola para dos jugadores, implementado en rabi.

## Uso

Desde la raíz del repositorio:

```bash
make run FILE=example/tateti.rabi
make run FILE=example/tateti.rabi ARGS="--backend vm"
```

o, si previamente instalaste rabi con make install:

```bash
rabi example/tateti.rabi
rabi example/tateti.rabi --backend vm
```

## Cómo jugar

Cada jugador elige una casilla escribiendo su número (1 a 9):

```
 1 | 2 | 3
---+---+---
 4 | 5 | 6
---+---+---
 7 | 8 | 9
```

El primero en completar una fila de 3 (fila, columna o diagonal) gana.