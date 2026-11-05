# Gramática de rabi

## Índice

1. [EBNF de la gramática sintáctica](#1-ebnf-de-la-gramática-sintáctica)
2. [Estructura léxica](#estructura-léxica)
3. [Sintaxis](#sintaxis)

## EBNF de la gramática sintáctica

```ebnf

program        = { NEWLINE | statement } , EOF ;

statement      = simple_stmt , terminator | compound_stmt ;

terminator     = NEWLINE | DEDENT | EOF | "else" ;

simple_stmt    = print_stmt
               | let_decl
               | return_stmt
               | "break"
               | "continue"
               | expression ;

compound_stmt  = func_decl
               | if_stmt
               | while_stmt
               | for_stmt ;

print_stmt     = "print" , expression ;

let_decl       = "let" , IDENTIFIER , [ "=" , expression ] ;

return_stmt    = "return" , [ expression ] ;

func_decl      = "func" , IDENTIFIER , "(" , parameters , ")" , block ;
parameters     = { IDENTIFIER , [ "," ] } ;

if_stmt        = "if" , expression , block , [ "else" , ( if_stmt | block ) ] ;

while_stmt     = "while" , expression , block ;

for_stmt       = "for" , IDENTIFIER , "in" , expression , block ;


block          = ":" , ( indented_block | statement ) ;

indented_block = NEWLINE , { NEWLINE } , INDENT ,
                 { NEWLINE | statement } ,
                 ( DEDENT | EOF ) ;


expression     = assignment ;

assignment     = logic_or , [ "=" , expression ] ;

logic_or       = logic_and , { "or" , logic_and } ;

logic_and      = equality , { "and" , equality } ;

equality       = comparison , { ( "==" | "!=" ) , comparison } ;

comparison     = term , { ( "<" | "<=" | ">" | ">=" ) , term } ;

term           = factor , { ( "+" | "-" ) , factor } ;

factor         = unary , { ( "*" | "/" | "%" ) , unary } ;

unary          = ( "!" | "-" | "len" | "input" ) , unary
               | "input"
               | call_or_index ;

call_or_index  = primary , { "(" , [ expr_list ] , ")"
                           | "[" , expression , "]" } ;

expr_list      = expression , { "," , expression } , [ "," ] ;

primary        = INTEGER | DECIMAL | STRING
               | "true" | "false" | "null"
               | IDENTIFIER
               | "[" , [ expr_list ] , "]"
               | "(" , expression , ")" ;
```

## Estructura léxica

### Caracteres y líneas

El código fuente se procesa línea por línea. Cada línea termina con un token `NEWLINE`, por lo que una sentencia no puede continuar en la línea siguiente.

```python
print (false or
true)            # -> error: Expected expression
```

### Indentación

Un nivel de indentación es una tabulación o cuatro espacios. Una línea no puede mezclar tabulaciones y espacios, y si usa espacios, la cantidad debe ser múltiplo de 4.

El scanner compara el nivel de cada línea con la cantidad de bloques abiertos, y genera un token `INDENT` por cada nivel que aumenta y un `DEDENT` por cada nivel que disminuye. Al final del archivo se cierran todos los bloques abiertos con `DEDENT` y se agrega `EOF`.

Un bloque nuevo puede aumentar **solo un nivel** de indentación.

```python
if true:
        print 1    # -> error: Unexpected indentation at start of statement
```

Las líneas vacías o que contienen solo un comentario no afectan la indentación.

### Espacios y comentarios

Los espacios entre tokens se descartan: `1+2` y `1 + 2` son equivalentes.

Un comentario comienza con `#` y se extiende hasta el final de la línea, salvo dentro de un string. 

### Identificadores y palabras reservadas

Un identificador comienza con una letra o `_`, y sigue con letras, dígitos o `_`. Las letras pueden ser Unicode.

Las palabras reservadas son: `and`, `or`, `true`, `false`, `null`, `if`, `else`, `while`, `for`, `in`, `break`, `continue`, `func`, `return` y `let`.

`print`, `len` e `input` no son palabras reservadas: se escanean como identificadores, y el parser les da un significado especial.

### Literales

- `INTEGER`: una secuencia de dígitos (`0`, `42`, `007`). Debe entrar en un entero de 64 bits con signo. No hay literales negativos: `-5` es el operador `-` aplicado a `5`.
- `DECIMAL`: dígitos con un punto (`3.14`, `1.`). No pueden comenzar con `.` y no hay notación científica.
- `STRING`: texto entre comillas dobles, en una sola línea (`"hola"`). No hay secuencias de escape.
- `true`, `false` y `null`.

## Sintaxis

### Programas y sentencias

Un programa es una secuencia de sentencias. Las sentencias simples (`print`, `let`, `return`, `break`, `continue` y expresiones) terminan al final de la línea, al cerrarse un bloque o antes de un `else`. Las sentencias compuestas (`func`, `if`, `while` y `for`) terminan junto con su bloque.

### Bloques

Un bloque comienza con `:` y puede escribirse de dos formas: indentado, en las líneas siguientes y con un nivel más de indentación, o en la misma línea, en cuyo caso contiene una sola sentencia.

```python
while x > 0:
    print x
    x = x - 1

while x > 0: x = x - 1

if false: print 1 else: print 2
# -> 2
```

### Precedencia y asociatividad de expresiones

De mayor a menor precedencia:

| Nivel | Operadores | Asociatividad | Ejemplo |
|-------|------------|---------------|---------|
| 1 | llamada `f(...)`, índice `a[i]` | izquierda | `f(1)[0]` → `(f(1))[0]` |
| 2 | `!` `-` `len` `input` (unarios) | derecha | `-a * b` → `(-a) * b` |
| 3 | `*` `/` `%` | izquierda | `a / b * c` → `(a / b) * c` |
| 4 | `+` `-` | izquierda | `a - b - c` → `(a - b) - c` |
| 5 | `<` `<=` `>` `>=` | izquierda | `a + 1 < b` → `(a + 1) < b` |
| 6 | `==` `!=` | izquierda | `a < b == c` → `(a < b) == c` |
| 7 | `and` | izquierda | `a == b and c` → `(a == b) and c` |
| 8 | `or` | izquierda | `a and b or c` → `(a and b) or c` |
| 9 | `=` (asignación) | derecha | `a = b = c` → `a = (b = c)` |

