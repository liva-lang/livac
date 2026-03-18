# Audit Report: mini-interpreter

## Resumen
- Total errores: 15 (raíces únicas que causan 164+ errores Rust)
- BUG: 11 | SKILL: 0 | GAP: 0 | AI: 3 | DESIGN: 1
- **Proyecto más complejo de la auditoría**: 6 archivos, 1494 líneas
- **Mayor volumen de bugs de codegen descubiertos**: 11 bugs, incluyendo 2 nuevos no vistos antes

## Observaciones iniciales

El código original es de **alta calidad técnica**. La IA generó un proyecto completo de 6 archivos con:
- Modelo de datos usando clases (Token, ASTNode, Value, Environment, EvalResult)
- 18 funciones factory para nodos AST
- Lexer con soporte para strings, números, operadores multi-carácter
- Parser recursive descent completo
- Evaluador recursivo con closures y scoping léxico
- Demo script de 140 líneas con: variables, aritmética, strings, comparaciones, condicionales, while loops, funciones recursivas (factorial, fibonacci, fizzbuzz, max)

Sin embargo, es el proyecto con **más bugs de codegen** encontrados, principalmente porque combina intensamente patrones que exponen debilidades del compilador: clases, `rust {}` interop, recursión, closures, y manipulación de estructuras de datos complejas.

## Errores encontrados

| # | Archivo(s) | Línea(s) | Error | Categoría | Fix aplicado |
|---|------------|----------|-------|-----------|--------------|
| 1 | tokens.liva | 8,15 | `type` como nombre de campo: es keyword reservada de Rust. `tok.type` genera Rust inválido | BUG | Renombrado a `tokType` en Token class y todos los usos |
| 2 | ast.liva | 85,92,etc | `get_field()` generado para acceso a campos de variables locales (structs) en vez de `.field` directo | BUG | Reestructurado con `rust {}` para acceso directo |
| 3 | evaluator.liva | 40,53,etc | `ident_err` variable de error leak: scope de error binding se filtra a ramas posteriores de if/else | BUG | Extraído a variables locales antes de cada uso |
| 4 | evaluator.liva | 28+ | Métodos que modifican solo campos (`self.x = y`) no reciben `&mut self` automáticamente | BUG | Reestructurado como funciones libres con `rust {}` |
| 5 | evaluator.liva | via methods | Transitive `&mut self` no se propaga: método que llama a otro `&mut self` se genera como `&self` | BUG | Eliminado: funciones libres |
| 6 | evaluator.liva | — | Array element assignment (`arr[i] = val`) genera LHS inválido en Rust | BUG | Eliminado: gestión de arrays en `rust {}` |
| 7 | lexer.liva | 45,52 | `String >= &str` comparación: no existe `PartialOrd` entre `String` y `&str` en Rust | BUG | Reescrito `_isDigit`/`_isAlpha` con `rust {}` char comparison |
| 8 | lexer.liva | 100+ | `pos + 1 as usize` se parsea como `pos + (1 as usize)` = `i32 + usize` — no compila | BUG | Reescrito como `rust { chars[(pos + 1) as usize].clone() }` |
| 9 | evaluator.liva | todo | Move/ownership: pasar struct como argumento consume el valor, sin auto-clone. Afecta loops, if/else, function calls | BUG | `.clone()` explícito via `rust {}` en cada uso |
| 10 | (fixed) | comments | `find_rust_blocks()` en lexer.rs matchea `rust` keyword dentro de `//` comments, creando bloques fantasma | BUG | Renombrado "rust" → "interop" en todos los comments |
| 11 | (fixed) | rust block | `find_balanced_brace()` confunde `'` de lifetimes Rust (`'a`) y apostrophes en strings con char literals, consumiendo braces intermedios | BUG | Eliminados lifetimes y apostrophes del `rust {}` block |
| 12 | ast.liva, evaluator.liva | varios | `.type` acceso a campo inexistente (en código con `.tokType`), ternarios con ambas ramas idénticas, `.message` sobre string | AI | Corregido: `.tokType`, eliminados ternarios no-op, eliminados `.message` |
| 13 | tokens.liva | 12 | `numValue: number` — el tipo correcto en Liva es `float`, no `number` | AI | Cambiado a `float` |
| 14 | evaluator.liva | varios | `err.message` sobre variable que ya es `string` — genera `get_field("message")` inválido | AI | Eliminados: el error ya es el string |
| 15 | — | — | El proyecto multi-archivo requiere merge a single-file para evitar bugs de imports cross-file | DESIGN | Merged 6 archivos → 1 |

## Bugs nuevos descubiertos (no vistos en proyectos anteriores)

### BUG 10: `find_rust_blocks()` matchea comments
```liva
// This uses rust {} interop for struct construction
```
El lexer busca `\brust\b` seguido de `{` para identificar bloques `rust {}`, pero **no salta `//` comments**. Si un comentario contiene la palabra "rust" seguida de `{`, el lexer crea un bloque `rust {}` fantasma que consume código posterior.

**Ubicación en compilador:** `src/lexer.rs`, función `find_rust_blocks()` (L296-340).

### BUG 11: `find_balanced_brace()` confunde lifetimes/apostrophes con char literals
```rust
fn peek<'a>(tokens: &'a [Token], pos: usize) -> &'a Token  // 'a consumed as char literal start
panic!("Expected {} but got {} ('{}') ...");                // ' consumed as char literal start
```
La función `find_balanced_brace()` en `src/lexer.rs` (L346-413) trata `'` como inicio de char literal y consume hasta el siguiente `'`, saltándose braces intermedios. Esto causa que lifetimes Rust (`'a`), apostrophes en strings de panic, y comments con `'keyword'` rompan el conteo de braces.

**Impacto:** El `rust {}` block del parser (L415) se extendía hasta L1399 en vez de L869, absorbiendo las 530 líneas de la función `evaluate()` entera. Este fue el error más difícil de diagnosticar de toda la auditoría.

## Análisis de dependencia de `rust { }`

### Métricas generales

| Métrica | Valor |
|---------|------:|
| Total líneas del archivo fixed | 1,590 |
| Bloques `rust {}` | 92 |
| Bloques single-line (inline) | 55 |
| Bloques multi-line | 37 |
| Líneas dentro de `rust {}` | ~777 |
| Líneas Liva puro | ~813 |
| **Ratio rust/total** | **48.9%** |

### Desglose por función y motivo

| # | Función/Sección | Bloques | Propósito del `rust {}` | Feature necesaria en Liva | Prioridad |
|---|-----------------|---------|------------------------|--------------------------|-----------|
| 1 | `_tokType/Value/Line` | 3 | Acceso a campos de struct | Fix BUG #2 (get_field en locals) | **Alta** |
| 2 | `_isDigit`, `_isAlpha` | 2 | Comparación de chars | `String >= String` o stdlib `Char.isDigit()` | Media |
| 3 | `_charAtOffset` | 1 | Indexación `chars[(pos+1) as usize]` | Fix cast priority (BUG #8) + array indexing con expresiones | **Alta** |
| 4 | `tokenize()` | 6 | Array indexing con cast a usize | Auto-cast number→usize para índices | Media |
| 5 | 18× `make*` AST factories | 18 | Construcción de struct literals | Liva struct literal syntax `StructName { field: val }` | **Alta** |
| 6 | `parseTokens()` | 1 (456 líneas) | Parser recursive descent completo | Fix BUGs #2-6 (method calls, mutability, ownership) | **Alta** |
| 7 | 4× `make*Value` factories | 4 | Construcción de Value structs | Struct literal syntax | **Alta** |
| 8 | `valueToString()` | 2 | Cast int/float para string | `Math.floor()` → `int` cast | Baja |
| 9 | `makeChildEnv/envSet/envDefine` | 3 | Construcción/mutación de Environment | Struct literals + HashMap API en stdlib | **Alta** |
| 10 | `envGet` | 3 | HashMap lookup con `Option` | Stdlib `Map.get()` con manejo de `Option` | **Alta** |
| 11 | `envFlatten/envFlattenVals` | 2 | Flatten del parent chain | Iteración recursiva sin ownership issues | Media |
| 12 | `makeReturnResult` | 1 | Construcción de EvalResult | Struct literal syntax | **Alta** |
| 13 | `evaluate()` | 46 | `.clone()` para ownership + field access | Auto-clone en paso de argumentos + fix get_field | **Alta** |

### Resumen de features necesarias

| Feature | Bloques que eliminaría | Prioridad estimada |
|---------|----------------------:|-------------------|
| Struct literal syntax | 23 | v1.8+ |
| Fix get_field para locals (BUG #2) | 49 | v1.6 |
| Auto-clone / ownership inteligente | 46 | v2.0+ |
| HashMap/Option en stdlib | 5 | v1.7 |
| Fix cast priority `as` (BUG #8) | 6 | v1.6 |
| Fix find_rust_blocks comments (BUG #10) | preventivo | v1.6 |
| Fix find_balanced_brace apostrophes (BUG #11) | preventivo | v1.6 |

**Conclusión:** Si Liva tuviera struct literals y corrigiera los bugs de ownership/get_field, el 90%+ de los bloques `rust {}` serían innecesarios. El parser podría escribirse en Liva puro.

## Proceso de corrección

Este proyecto requirió la **iteración más larga de toda la auditoría**:

```
Original:  164 errores Rust (primera compilación)
Fase 1:     Parse error — apostrophes en rust block (BUG #11)
Fase 2:     46 errores — tras fix del parse error
Fase 3:     34 errores — camelCase→snake_case en rust {} blocks
Fase 4:      8 errores — types (String vs &str, i32 vs usize)
Fase 5:      0 errores — ownership/move (.clone() en todos los usos)
```

## Cambios realizados en fixed/

1. **Merged a single-file**: 6 archivos → 1 (main.liva, 1590 líneas) para evitar bugs de imports cross-file
2. **Campo `type` → `tokType`**: Reservada en Rust, renombrado en todas las ocurrencias
3. **`numValue: number` → `float`**: Tipo correcto de Liva
4. **Todas las comments con "rust {}" → "interop"**: Previene BUG #10
5. **parseTokens sin lifetimes ni apostrophes**: Previene BUG #11
6. **camelCase → snake_case en `rust {}` blocks**: El codegen transforma nombres pero no el contenido de `rust {}`
7. **`_isDigit`/`_isAlpha` con `rust {}` char comparison**: Evita `String >= &str`
8. **Array indexing via `rust { chars[(pos+1) as usize].clone() }`**: Evita cast priority bug
9. **Pre-clone de todos los campos de struct antes de uso**: Evita move-after-use
10. **Eliminados `.message` sobre strings**: El error ya es un string

## Ejecución y verificación

```
$ livac run fixed/main.liva -- examples/demo.mini
```

✅ **Todas las features del demo script verificadas:**

| Feature | Test | Resultado |
|---------|------|-----------|
| Variables | `let x = 10; print(x)` → `10` | ✅ |
| Aritmética | `x + y`, `x - y`, `x * y`, `x / y`, `x % y` | ✅ |
| Strings | `"Hello, " + "World!"` → `"Hello, World!"` | ✅ |
| Comparaciones | `<`, `>`, `==`, `!=` | ✅ |
| Condicionales | `if/else`, anidados | ✅ |
| While loops | Conteo 1-5, suma, producto | ✅ |
| Funciones | Declaración, llamada, parámetros | ✅ |
| Recursión | `factorial(10)` = 3628800 | ✅ |
| Fibonacci | `fib(10)` = 55, secuencia completa 0-10 | ✅ |
| FizzBuzz | 1-20, correcto (Fizz/Buzz/FizzBuzz) | ✅ |
| Max function | `max(42, 17)` = 42, `max(23, 99)` = 99 | ✅ |
| Nested expressions | Operaciones con `-` unario y combinadas | ✅ |

## Patrones problemáticos

1. **Ownership es el enemigo #1**: 46 de 92 bloques `rust {}` existen solo para hacer `.clone()`. Cada vez que un struct se pasa como argumento, se consume. En un evaluador recursivo que pasa `Environment` y `ASTNode` repetidamente, esto es devastador.

2. **Struct literals no existen en Liva**: 23 bloques `rust {}` son solo para construir structs. Sin `Token { tokType: "NUM", ... }` en Liva, cada tipo necesita una función factory wrapper.

3. **`rust {}` content no se name-transforma**: A diferencia del resto del código, las variables dentro de `rust {}` NO se convierten de camelCase a snake_case. Esto es confuso y poco documentado.

4. **El lexer es frágil con `rust {}` blocks**: Dos bugs independientes (#10, #11) hacen que el contenido de `rust {}` blocks sea peligroso si contiene: la palabra "rust" en comments, lifetimes con `'`, o apostrophes en strings.

## Conclusiones

1. **Proyecto más difícil de la auditoría**: 1494 líneas, 164 errores iniciales, iterar 5 fases para llegar a 0 errores. Requirió investigar el código fuente del compilador (`src/lexer.rs`) para diagnosticar el bug de apostrophes.

2. **11 bugs de codegen**: El mayor número de bugs descubiertos en un solo proyecto. Los bugs #10 y #11 son nuevos y afectan a cualquier proyecto que use `rust {}` con patrones comunes de Rust (lifetimes, apostrophes en strings).

3. **48.9% del código final es `rust {}`**: La mitad del programa terminó escrito en Rust embebido, principalmente por falta de struct literals y ownership automático.

4. **Calidad del código AI**: A pesar del volumen de errores, el diseño del intérprete es correcto y completo. Los errores son 100% de codegen y diseño del compilador, no de lógica de la IA.

5. **Priorización de fixes**: Los bugs más impactantes para eliminar `rust {}` son:
   - Struct literal syntax (elimina 23 bloques)
   - Auto-clone / smart ownership (elimina 46 bloques)  
   - Fix `get_field` para locals (elimina 3 bloques + previene errores)
   - Fix `find_balanced_brace` y `find_rust_blocks` (previene errores difíciles de diagnosticar)
