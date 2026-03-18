# Audit Report: calculator

## Resumen
- Total errores: 7 (raíces únicas que causan 48 errores Rust)
- BUG: 7 | SKILL: 0 | GAP: 0 | AI: 0 | DESIGN: 0

## Observaciones iniciales

El código original es de **muy alta calidad**. La IA generó un proyecto de 3 archivos (274 líneas) con:
- Tokenizer correcto y bien estructurado
- Parser recursive descent con gramática documentada
- REPL interactivo con help y manejo de errores
- Todos los archivos pasan `livac check` sin errores

Los **48 errores** al compilar a Rust provienen de **7 bugs raíz** del compilador que se manifiestan en el código generado. La IA usó patrones que deberían funcionar según la spec del lenguaje.

## Errores encontrados

| # | Archivo | Línea(s) | Error | Categoría | Fix aplicado |
|---|---------|----------|-------|-----------|--------------|
| 1 | evaluator.liva | 19 | `=>` one-liner en método de clase genera `-> ()` en vez de inferir tipo de retorno `Token` | BUG | Eliminado: reestructurado como free functions + rust block |
| 2 | evaluator.liva | 40,53,60,etc | `let val, err = this.method()` genera `(self.method(), None)` sin destructurar el Result | BUG | Eliminado: parser en `rust { }` con `?` operator |
| 3 | evaluator.liva | 67,70 | `fail "msg"` al final de función genera `Error::chain` referenciando variable de error que está fuera de scope | BUG | Eliminado: parser en `rust { }` |
| 4 | evaluator.liva | 28-90 | Métodos que llaman `_advance()` (`&mut self`) son generados como `&self` — no se propaga mutabilidad | BUG | Eliminado: parser en `rust { }` con `&mut self` explícito |
| 5 | evaluator.liva | 19 | `self.tokens[idx]` intenta mover Token del Vec en vez de usar `.clone()` (a diferencia de arrays locales que usan `.get().cloned()`) | BUG | Eliminado: parser en `rust { }` con `&str` references |
| 6 | evaluator.liva | 40,53,etc | `or fail $"{err}"` no genera el operador `?` — es completamente ignorado para llamadas a métodos de clase e instancias | BUG | Eliminado: parser en `rust { }` con `?` |
| 7 | main.liva | — | `let val, err = fn()` para funciones importadas (multi-file) genera `(fn(), None)` en vez de `match fn() { Ok(v) => ..., Err(e) => ... }` — sólo funciona para funciones en el mismo archivo | BUG | Merged a single-file |

## Detalles de los bugs descubiertos

### BUG 1: Arrow method return type
```liva
_current() => this.tokens[this.pos]  // Genera: fn _current(&self) -> ()
```
El codegen genera `-> ()` para `=>` one-liners en métodos de clase que retornan valores de arrays de campos. Cascada: todos los `.kind` y `.value` se convierten en `get_field()` sobre `()`.

### BUG 2: Error binding para method calls (el más grave)
```liva
let val, err = this._parseExpr()  // Genera: (self._parse_expr(), None) — NO destructura el Result
```
El `match fn() { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) }` sólo se genera para **free function calls**. Para **method calls** (`this.method()` y `instance.method()`), el codegen produce `(method(), None)` sin destructurar, dejando `val` como `Result<T, Error>` en vez de `T`.

### BUG 7: Cross-file error binding
Igual que BUG 2, pero aplica a funciones importadas de otros archivos. En single-file, free functions generan `match` correcto. En multi-file, funciones importadas generan `(fn(), None)`.

## Análisis de dependencia de `rust { }`

| # | Función | Propósito del `rust { }` | Feature necesaria en Liva | Prioridad |
|---|---------|--------------------------|--------------------------|-----------|
| 1 | evaluate | Parser recursive descent completo (parse_factor/term/expr) | Fix BUG 2 (method call error binding) + Fix BUG 4 (mutability propagation) + Fix array ownership para parámetros | **Alta** |

**El `rust { }` block NO es necesario por un feature gap del lenguaje** — es exclusivamente un workaround para bugs del codegen. Si se corrigieran los bugs 1-7, el código original del AI compilaría sin cambios.

## Cambios realizados en fixed/

1. **Merged a single-file**: Los 3 archivos (token.liva, evaluator.liva, main.liva) fusionados en uno para evitar BUG #7 (cross-file error binding)
2. **Parser en `rust { }`**: El evaluador reimplementado como `rust { }` block dentro de `evaluate()` para evitar BUGs #1-6
3. **Tokenizer conservado en Liva puro**: El tokenizer funciona perfectamente sin cambios
4. **REPL conservado en Liva puro**: El main loop funciona perfectamente sin cambios

## Patrones problemáticos

- **`let val, err = instance.method()`**: Patrón más crítico — es el error binding estándar de Liva pero no funciona para method calls
- **Arrow methods que retornan valores de arrays de clase**: `=>` en métodos no infiere el tipo de retorno desde accesos a arrays de campos
- **Recursión con estado compartido**: Arrays pasados como parámetros a funciones recursivas pierden ownership (Vec pasado por valor, no por referencia)
- **`fail errVar`**: Genera `Error::chain(option_var, ...)` en vez de unwrap — el tipo `Option<Error>` no implementa `Into<String>`

## Verificación de ejecución

Programa ejecutado exitosamente con las siguientes pruebas:

| Input | Output | Estado |
|-------|--------|--------|
| `2 + 3` | `5` | ✅ |
| `(4 + 5) * 2` | `18` | ✅ |
| `-3.5 + 1.2` | `-2.3` | ✅ |
| `100 / (2 + 3)` | `20` | ✅ |
| `-7/3` | `-2.333...` | ✅ |
| `((2+3)*(4-1))` | `15` | ✅ |
| `1+2+3+4+5` | `15` | ✅ |
| `3.14*2` | `6.28` | ✅ |
| `-(-5)` | `5` | ✅ |
| `.5 + .5` | `1` | ✅ |
| `10 / 0` | Error: División por cero | ✅ |
| `2 * (3 + )` | Error: Token inesperado: ')' | ✅ |
| `abc` | Error: Carácter inesperado: 'a' | ✅ |
| (vacío) | (ignorado) | ✅ |
| `help` | Muestra ayuda | ✅ |
| `quit` | Sale del programa | ✅ |

## Conclusiones

1. **Calidad del código AI: Excelente**. El código original es correcto, bien estructurado y usa la sintaxis de Liva apropiadamente. NO hay errores de la IA.
2. **Todos los errores son bugs del compilador**. El código no compila por deficiencias en el codegen, no por uso incorrecto del lenguaje.
3. **BUG más crítico: error binding para method calls** (BUG #2). Este bug impide cualquier programa que use `let val, err = obj.method()` — un patrón fundamental en Liva.
4. **La versión fixed requiere `rust { }`** — no por feature gap sino como workaround para bugs del codegen.
5. **Sin la skill local**, la IA produjo un proyecto perfectamente diseñado — evidencia de que la sintaxis de Liva es intuitiva para los modelos.
