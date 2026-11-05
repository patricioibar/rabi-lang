# Semántica de rabi

## Índice

1. [Introducción](#1-introducción)
2. [Estructura de un programa](#2-estructura-de-un-programa)
3. [Valores y expresiones](#3-valores-y-expresiones)
4. [Estado](#4-estado)
5. [Arrays](#5-arrays)
6. [Control de flujo](#6-control-de-flujo)
7. [Funciones](#7-funciones)
8. [Scopes y closures](#8-scopes-y-closures)
9. [Errores](#9-errores)

## 1. Introducción

En este documento se describe la semántica de rabi como lenguaje de programación. 
Cualquier implementación del lenguaje (incluyendo las que se encuentran en este repositorio) debe cumplir con esta especificación.

`rabi` es un lenguaje de programación imperativo y de tipado dinámico.
Tanto la sintaxis como la semántica de `rabi` están fuertemente influenciadas por lenguajes modernos como Python, con una estructura simple y legible.


### Programa de ejemplo

A continuación se muestra un pequeño ejemplo de programa en `rabi`, donde se muestra la definición de una función, uso de condicionales y bucle for.

```python
func factorial(n):
    if n <= 1: return 1
    return n * factorial(n - 1)

for x in [1, 2, 3, 4, 5]:
    print factorial(x)
# -> 1
# -> 2
# -> 6
# -> 24
# -> 120
```

## 2. Estructura de un programa

En esta sección se describe brevemente la estructura de un programa. La gramática completa se encuentra en [GRAMATICA.md](GRAMATICA.md).

### Sentencias y líneas

Los programas de `rabi` consisten en una secuencia de sentencias. Cada sentencia puede ser:
- Una expresión que se evalúa.
- Una declaración de variable.
- Una declaración de función.
- Una sentencia `print`.
- Una instrucción de control de flujo (condicionales, bucles, `break`, `continue` o `return`).

Las sentencias se delimitan con saltos de línea. 

Se permite el uso de comentarios, que comienzan con `#` y se extienden hasta el final de la línea.

### Bloques e indentación

Los bloques de código utilizados en condicionales, bucles y funciones se delimitan mediante indentación. Un nivel de indentación puede realizarse utilizando una tabulación (`\t`) o cuatro espacios. **No** se pueden mezclar tabulaciones y espacios en una misma línea. Aunque está permitido mezclar entre diferentes líneas, se recomienda mantener la consistencia en el uso de indentación a lo largo del programa. 

```python
let x = 5
if x > 0:
    print "positivo"
else if x < 0:
    # esto es un comentario
    print "negativo"
else: print "cero"
```

## 3. Valores y expresiones

### Tipos y valores

`rabi` tiene los siguientes tipos de valores:
- `Boolean`: valores lógicos `true` y `false`, se copian por valor.
- `Integer`: números enteros, se copian por valor.
- `Decimal`: números de punto flotante, se copian por valor.
- `Null`: valor nulo, se copia por valor.
- `String`: cadenas de texto, inmutables. Se copian por referencia.
- `Array`: listas de valores, se copian por referencia.
- `Function`: funciones, se copian por referencia.

El lenguaje es de tipado dinámico, por lo que el tipo de una variable se determina en tiempo de ejecución y puede cambiar durante la ejecución del programa.

```python
let entero = 42
let decimal = 3.14
let texto = "hola"
let booleano = true
let nada = null
let lista = [1, "dos", 3.0]
func f(): return 1     # las funciones también son valores
let funcion = f        # se puede asignar una función a una variable
```

### Verdadero y falso

Cualquier valor puede ser evaluado como un booleano en un condicional.

Los valores que se evalúan como `false` son:
- `false`
- `0` (Integer)
- `0.0` (Decimal)
- `""` (String vacío)
- `[]` (Array vacío)
- `null`

Todos los demás valores se evalúan como `true` (incluyendo las funciones).

```python
if 0: print "verdadero" else: print "falso"        # -> falso
if "": print "verdadero" else: print "falso"       # -> falso
if null: print "verdadero" else: print "falso"     # -> falso
if [0]: print "verdadero" else: print "falso"      # -> verdadero
func f(): return 1
if f: print "verdadero" else: print "falso"         # -> verdadero
```

### Orden de evaluación

Las expresiones se evalúan de izquierda a derecha, y cada subexpresión se evalúa una sola vez.

```python
func f(x):
    print x
    return x

print f(1) + f(2)
# -> 1
# -> 2
# -> 3
```

### Aritmética

Las operaciones entre números del mismo tipo producen un valor del mismo tipo. Las operaciones entre `Integer` y `Decimal` producen un `Decimal`. La división entre enteros redondea hacia cero. La división o el módulo por cero y el overflow de enteros producen un error en tiempo de ejecución.

Además, se definen las operaciones de suma y de multiplicación para `String` y `Array`. La suma concatena dos valores del mismo tipo, y la multiplicación por un `Integer` repite el valor. Tanto la suma como la multiplicación crean un nuevo valor (no por referencia) y no modifican los operandos.

```python
print 7 / 2          # -> 3
print 7 / 2.0        # -> 3.5
print -7 / 2         # -> -3
print -7 % 2         # -> -1
print 0.1 + 0.2      # -> 0.30000000000000004
print "ab" * 3       # -> ababab
print 1 / 0          # -> error: Division by zero
```

### Comparaciones

Las comparaciones están definidas para `Integer`, `Decimal` y `String`. Se permite la comparación entre `Integer` y `Decimal`, pero no entre otros tipos. Comparar valores de tipos distintos produce un error en tiempo de ejecución.

```python
print 1 < 2.5          # -> true
print "abc" < "abd"    # -> true
print 1 < 2 < 3        # -> error: compara un Boolean con un Integer (true < 3)
```

### Igualdad

Las operaciones de igualdad y desigualdad están definidas para todos los tipos de valores. Comparar valores de tipos distintos produce `false`, excepto para `Integer` y `Decimal`, que se comparan por valor.

```python
print 1 == 1.0              # -> true
print 1 == "1"              # -> false
print null == false         # -> false
print [1, [2]] == [1, [2]]  # -> true
```

### Operadores unarios

El operador `-` se aplica a `Integer` y `Decimal` y produce su negación. El operador `!` se aplica a cualquier valor y produce su negación lógica.

```python
print -5         # -> -5
print !0         # -> true
print -"a"       # -> error: Unsupported operand type for unary minus: String
```

## 4. Estado

### Expresiones y sentencias

`rabi` distingue entre expresiones y sentencias (statements). Las expresiones producen un valor, y algunas también producen un efecto sobre el estado del programa. Las sentencias se ejecutan y no producen un valor.

Algunos ejemplos de expresiones pueden ser operaciones aritméticas, comparaciones, asignaciones o llamadas a funciones. En cambio, algunas sentencias podrían ser la declaración de variables, imprimir texto por pantalla, entre otras.

### `print`

`print` es una sentencia que imprime el valor de su expresión en la salida estándar. Uso: `print <expresión>`.

```python
print [1, "a", 2.5, null]    # -> [1, a, 2.5, null]
print 1.0                    # -> 1
func f(): return 1
print f                      # -> <function f>
```

### Declaración con `let`

La palabra reservada `let` declara una variable en el scope actual. La variable puede declararse inicializándola con un valor utilizando la sintaxis `let nombre = valor`, o sin inicializar, en cuyo caso su valor inicial será `null`. 

```python
let x
print x        # -> null
```

### Asignación

La asignación de un valor a una variable se realiza con el operador `=`. La variable debe haber sido declarada previamente. La asignación devuelve el valor asignado, lo que permite encadenar asignaciones.

```python
let a
let b
a = b = 3
print a          # -> 3
print (a = 5)    # -> 5
z = 1            # -> error: Variable 'z' not defined
```

### Bloques y scopes

Los bloques pueden crearse únicamente dentro de condicionales, bucles y funciones. Cada bloque crea un nuevo scope, que es un espacio donde se pueden declarar variables y funciones. Las variables y funciones declaradas en un scope son accesibles únicamente dentro de ese scope.

```python
if true:
    let x = 1
    print x        # -> 1
print x            # -> error: Variable 'x' not defined
```

```python
let total = 0
for x in [1, 2, 3]:
    total = total + x
print total    # -> 6
```

Si se intenta utilizar una variable o función que no está declarada en el scope actual, se buscará en los scopes externos hasta llegar al scope global. Si no se encuentra, se produce un error en tiempo de ejecución. Las funciones son una excepción: ver [Scopes y closures](#8-scopes-y-closures).

```python
let x = 1
if true:
    let y = 2
    print x        # -> 1
    print y        # -> 2
    print z        # -> error: Variable 'z' not defined
```

Si se define una variable o función con un nombre que ya existe en un scope externo, la nueva definición "oculta" a la anterior dentro del scope interno (shadowing). Al salir del scope interno, la definición externa vuelve a ser visible.

```python
let y = 1
if true:
    let y = 2
    print y    # -> 2
print y        # -> 1
```


## 5. Arrays

### Creación

Los arrays son valores que contienen una secuencia de otros valores y se crean con la sintaxis `[<valor1>, <valor2>, ...]`. Los arrays pueden contener valores de cualquier tipo, incluyendo otros arrays.

```python
let a = [1, 2, 3, 4]
let vacio = []
let matriz = [[1, 2], [3, 4]]
```

Además los arrays son heterogéneos, es decir, pueden contener valores de distintos tipos.

```python
let mixto = [1, "dos", 3.0, null, [5, 6]]
```

### Indexación

Para acceder a un elemento de un array se utiliza la sintaxis `<array>[<índice>]`, donde `<índice>` es un valor entero que representa la posición del elemento en el array, comenzando desde 0. Si se intenta acceder a un índice fuera del rango del array, se produce un error en tiempo de ejecución.

Los elementos de un array pueden ser modificados utilizando la sintaxis de indexación.

```python
let a = [1, "dos", 3.0]
print a[1]     # -> dos
a[1] = 2
print a[1]     # -> 2
print a[5]     # -> error: Array index out of bounds: 5 (length 3)
```

### Semántica de referencia

Los arrays se copian por referencia, por lo que si se asigna un array a otra variable ambas accederán a la misma estructura subyacente. Modificar un elemento a través de cualquiera de las dos variables afectará también a la otra.

```python
let a = [1, 2, 3]
let b = a
b[0] = 100
print a        # -> [100, 2, 3]
```

### `len`

El operador `len` devuelve la cantidad de elementos de un array. Uso: `len <array>`. Si se intenta usar `len` con un valor que no sea un array, se produce un error en tiempo de ejecución.

```python
print len [1, 2, 3]    # -> 3
```

### Concatenación y repetición

Como se mencionó en la [sección de aritmética](#aritmética), los arrays pueden concatenarse y repetirse utilizando los operadores `+` y `*`, respectivamente. Ambas operaciones no se manejan por referencia, sino que crean un nuevo array.

```python
print [1, 2] + [3]     # -> [1, 2, 3]
print [0] * 3          # -> [0, 0, 0]

let a = [1, 2]
let b = a + []
b[0] = 9
print a                # -> [1, 2]
```

## 6. Control de flujo

### `if` / `else`

Se pueden utilizar sentencias condicionales para ejecutar bloques de código dependiendo del valor de una expresión. El código a ejecutar puede ser un bloque de sentencias o una sola sentencia. La sintaxis es la siguiente:

```python
let n = 15
if n % 15 == 0: print "FizzBuzz"
else if n % 3 == 0: print "Fizz"
else if n % 5 == 0: print "Buzz"
else: print n
# -> FizzBuzz

if false:
    print "false"
else:
    print "true"
# -> true
```

### Operadores lógicos y cortocircuito

Los operadores lógicos `and` y `or` se evalúan de izquierda a derecha y utilizan cortocircuito, es decir, no evalúan el segundo operando si el resultado ya puede determinarse con el primero. 

Estos operadores pueden utilizarse con cualquier tipo de valor, y siguen las reglas de veracidad descritas en la sección de [tipos de datos](#verdadero-y-falso). El operador `and` devuelve un valor booleano, mientras que el operador `or` devuelve el primer operando que sea "truthy" o `false` operando si todos son "falsy".

```python
print null or "por defecto"   # -> por defecto
print 5 or no_existe          # -> 5      (no evalúa el derecho)
print 0 and no_existe         # -> false
print 1 and 5                 # -> true   (and siempre devuelve un Boolean)
```

### `while`

La sentencia `while` ejecuta un bloque de código mientras su condición se evalúe como `true`.

```python
let i = 0
while i < 3:
    print i
    i = i + 1
# -> 0
# -> 1
# -> 2
```

### `for`

La sentencia `for` recorre los elementos de un array. Uso: `for <variable> in <array>: <bloque>`. En cada iteración, la variable toma el valor del siguiente elemento y se declara en un nuevo scope. Si la expresión no es un array, se produce un error en tiempo de ejecución.

El array se evalúa una sola vez, pero sus elementos se leen en cada iteración: si el bloque modifica un elemento que todavía no se recorrió, el bucle ve el valor nuevo.

```python
let arr = [1, 2, 3]
for x in arr:
    arr[2] = 30
    print x
# -> 1
# -> 2
# -> 30
```

### `break` y `continue`

`break` termina el bucle más interno, y `continue` pasa a su siguiente iteración. Utilizarlos fuera de un bucle produce un error.

```python
for x in [1, 2, 3, 4, 5]:
    if x == 2: continue
    if x == 4: break
    print x
# -> 1
# -> 3
```

## 7. Funciones

### Declaración

Las funciones se declaran con la palabra reservada `func`. Uso: `func <nombre>(<parámetros>): <bloque>`. La declaración crea una variable con el nombre de la función en el scope actual.

```python
func suma(a, b):
    return a + b

func uno(): return 1
```

### Funciones como valores

Las funciones son valores: pueden asignarse a variables, pasarse como argumento, devolverse desde otras funciones, compararse e imprimirse.

```python
func aplicar(f, x): return f(x)
func doble(n): return n * 2

print aplicar(doble, 5)   # -> 10
let g = doble
print g(4)                # -> 8
print g == doble          # -> true
```

### Llamadas

Para llamar a una función se utiliza la sintaxis `<función>(<argumentos>)`. Primero se evalúa la función y luego los argumentos, de izquierda a derecha. La cantidad de argumentos debe coincidir con la de parámetros. Llamar a un valor que no es una función produce un error en tiempo de ejecución.

Los argumentos se copian igual que en una asignación, por lo que una función puede modificar un array que recibe como argumento, pero no puede modificar una variable primitiva, como un número, un string o un booleano.

```python
func suma(a, b): return a + b
print suma(1)       # -> error: Expected 2 arguments but got 1
```

### Retorno

La sentencia `return` termina la función y devuelve el valor de su expresión. Uso: `return <expresión>`. Si se omite la expresión, o si la función termina sin `return`, devuelve `null`. Utilizar `return` fuera de una función produce un error.

```python
func saludar(): print "hola"
print saludar()
# -> hola
# -> null
```

### Recursión

Las funciones declaradas en el scope global pueden llamarse a sí mismas y entre ellas (recursión mutua). Una función anidada no puede llamarse a sí misma (ver [rabi no tiene closures](#rabi-no-tiene-closures)).

```python
func es_par(n):
    if n == 0: return true
    return es_impar(n - 1)

func es_impar(n):
    if n == 0: return false
    return es_par(n - 1)

print es_par(10)    # -> true
```

### Funciones anidadas

Una función puede declararse dentro de otra. Su nombre es local a la función que la contiene: puede llamarse desde adentro, pero no desde afuera.

```python
func externa():
    func interna(): return 42
    return interna()

print externa()     # -> 42
print interna()     # -> error: Variable 'interna' not defined
```

### Scopes y closures

Dentro de una función solo son visibles sus parámetros, las variables declaradas dentro de ella y las variables globales.

Una función tampoco ve las variables de la función que la llama. Lo que ve una función no depende de desde dónde se la llama.

```python
func f():
    let local = 1
    return g()

func g(): return local

print f()     # -> error: Variable 'local' not defined
```

#### rabi no tiene closures

Una función no captura el entorno donde se declaró. Por eso, una función anidada no ve las variables de la función que la contiene, ni su propio nombre.

```python
func crear_contador():
    let n = 0
    func contador():
        n = n + 1
        return n
    return contador

let c = crear_contador()
print c()     # -> error: Variable 'n' not defined
```

No tener closures es una decisión de diseño: lo que ve una función depende solamente de sus parámetros, sus variables locales y las globales, lo que hace al lenguaje más simple de entender e implementar. A cambio, no se pueden crear funciones con estado propio, como el contador del ejemplo anterior.

```python
func externa():
    func cuenta(n):
        if n == 0: return 0
        return cuenta(n - 1)
    return cuenta(3)

print externa()   # -> error: Variable 'cuenta' not defined
```


En un lenguaje con closures, la función `showA` del siguiente ejemplo podría imprimir primero `global` y luego `block`, o incluso imprimir ambas veces `block`. Esto se debe a que en el entorno que captura al declararse se declara una nueva variable `a`, y la función puede ver esa variable en lugar de la global.

`rabi` no tiene este bug. Como `showA` no captura el bloque donde se declaró, la `a` que ve es siempre la global.

```python
let a = "global"
if true:
    func showA(): print a
    showA()
    let a = "block"
    showA()
    func showA_again(): print a
    showA_again()
# -> global
# -> global
# -> global
```

Las variables globales se buscan en el momento en que se usan. Si una variable global cambia de valor luego de definir una función, la función ve el valor actualizado. 

```python
let a = "global"
func showA(): print a
showA()       # -> global
let a = "redeclarada"
showA()       # -> redeclarada
```

Esto también implica que la variable global puede ser declarada después de la función que la usa.

```python
func mostrar(): print mensaje
let mensaje = "declarada después"
mostrar()     # -> declarada después
```

## 9. Errores

### Errores de sintaxis

Los errores de sintaxis (una expresión incompleta, un string sin cerrar, etc.) se detectan antes de ejecutar el programa. Si un programa tiene un error de sintaxis, no se ejecuta ninguna sentencia.

```python
print "esto no se imprime"
print (1 +
# -> error: Expected expression
```

### Errores en tiempo de ejecución

Un error en tiempo de ejecución termina el programa. Lo que se imprimió antes del error se mantiene. `rabi` no tiene manejo de excepciones.

| Categoría | Mensajes |
|-----------|----------|
| Variables | `Variable '<name>' not defined` |
| Aritmética | `Division by zero`, `Integer overflow in <op>`, `Unsupported operand types for <op>: <type> and <type>`, `Cannot multiply string by negative integer`, `Cannot multiply array by negative integer` |
| Unarios | `Unsupported operand type for unary minus: <type>` |
| Comparaciones | `Unsupported operand types for comparison: <type> and <type>` |
| Llamadas | `Attempted to call a non-function value`, `Expected <n> arguments but got <m>` |
| Arrays | `Unsupported type for indexing: <type>`, `Array index must be an Integer, found <type>`, `Array index out of bounds: <índice> (length <largo>)`, `Unsupported type for len: <type>`, `Unsupported type for iteration: <type>` |

```python
print "antes"
print 1 / 0
print "después"
# -> antes
# -> error: Division by zero
```

### Errores en el REPL

En el REPL, un error no termina la sesión: se informa y se puede seguir escribiendo. Se conservan las variables y funciones declaradas hasta ese momento.

```python
> let x = 1
> print 1 / 0      # -> error: Division by zero
> print x          # -> 1
```
