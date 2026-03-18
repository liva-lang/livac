# Audit Report: json-parser

## Resumen
- Total errores: 16
- BUG: 13 | SKILL: 0 | GAP: 0 | AI: 1 | DESIGN: 2

## Errores encontrados

| # | Archivo | Línea | Error | Categoría | Fix |
|---|---------|-------|-------|-----------|-----|
| 1 | parser.liva | 25+ | `let val, _ = parseValue(...)` — `_` no aceptado como identificador en error binding. El lexer emite `Token::Underscore` pero el parser no lo reconoce. | BUG | Reemplazar `_` con variable nombrada (`err`, `err2`, etc.) |
| 2 | value.liva | 100+ | `$"\{{compactBody}\}"` — brace escaping `\}` no funciona como terminador de interpolación. Solo `}}` funciona para literal `}` después de interpolación. | DESIGN | Usar `$"\{{compactBody}}}"` (backslash-open + double-close) |
| 3 | main.liva / parser.liva | varios | `$"...{expr.join(",")}..."` — comillas `"` dentro de expresiones de interpolación cierran prematuramente el template string (regex del lexer: `\$"([^"\\]|\\.)*"`). | BUG | Extraer la expresión con comillas a variable local antes del template |
| 4 | parser.liva | varios | `return parseValue(pos) or fail` — `or fail` no funciona con `return`, solo con `let` assignment. Parse error. | DESIGN | Eliminar `or fail` y manejar errores manualmente |
| 5 | lexer.liva | varios | `charAt(n)` genera `fn_call().map(\|c\| c.to_string())` que retorna `String` en vez de `char`. Comparaciones como `ch == '"'` fallan (String vs char). | BUG | Reemplazar `charAt()` con `substring(pos, pos + 1)` y comparar con strings `"x"` |
| 6 | lexer.liva | varios | Char escape sequences `'\n'`, `'\r'`, `'\t'` todas generan `'\\'` en Rust (codegen trunca el escape). | BUG | Usar string literals `"\n"`, `"\r"`, `"\t"` en vez de char literals |
| 7 | token.liva | — | Enum destructuring `Token.TString(v)` genera `Token::TString { v }` en vez de `Token::TString { value: v }` — el nombre del campo del enum no se mapea correctamente. | BUG | No usar enum destructuring; usar token string encoding en su lugar |
| 8 | token.liva | — | Arrow function `tokenToString(t: Token): string => ...` retornando string literal no genera `.to_string()` en codegen. | BUG | No usar arrow functions para este caso |
| 9 | parser.liva | varios | `or fail` codegen completamente roto — el operador `?` / `.map_err()` nunca se genera en Rust. El resultado fallible se ignora silenciosamente. | BUG | Eliminar `or fail`, manejar errores con `if err` pattern |
| 10 | parser.liva | varios | Error binding de funciones fallibles del usuario genera `(func_call(), None)` en vez de match con `Ok`/`Err` destructuring. | BUG | Usar single-variable binding y `if err` pattern |
| 11 | parser.liva | varios | Array element pasado como argumento a función: `tokens[i]` no genera `.clone()`. Solo `let x = tokens[i]` lo hace. | BUG | Siempre asignar array access a `let` antes de pasar como argumento |
| 12 | parser.liva | varios | Variable reassignment desde array index: `x = tokens[i]` no genera `.clone()`. Solo `let` bindings obtienen clone. | BUG | Usar `while true { let x = tokens[i]; if !cond { break } }` |
| 13 | lexer.liva/parser.liva | varios | `Vec<String>` parameter movido en primera llamada a función, no disponible para usos posteriores. E0382. | BUG | Restructurar como clase `JsonParser` con `this.tokens` (field access genera clone) |
| 14 | lexer.liva | varios | `result = result + "x"` (string concatenation con `+`) genera `.extend("x")` que falla con `&str is not an iterator`. | BUG | Usar `_strCat(result, "x")` helper function en vez de operador `+` |
| 15 | lexer.liva/value.liva | varios | Template string interpolation de variables mutables reasignadas usa `{:?}` (Debug format) en vez de `{}` (Display). `{:?}` agrega comillas y re-escapa caracteres, causando corrupción de datos. | BUG | Usar helper function `_strCat(a, b)` en vez de `$"{mutableVar}..."` |
| 16 | value.liva | varios | Encoding de arrays/objects con delimitadores `<<ITEM>>` y `<<ENTRY>>` se corrompe con nesting (delimitadores anidados son indistinguibles de los del nivel superior). | AI | Reescribir con encoding length-prefixed: `"5:hello4:test"` |

## Análisis de dependencia de `rust { }`

**Este proyecto NO contiene bloques `rust { }`.** Es 100% Liva puro. Esto es positivo y demuestra que la IA (con la skill) puede generar proyectos complejos sin necesidad de interop con Rust.

## Patrones problemáticos

### 1. Ownership / Move semantics (errores 11, 12, 13)
El patrón más costoso del proyecto. Las funciones en Liva compilan a funciones Rust que toman ownership de sus argumentos `String`. Cuando el mismo valor necesita usarse más de una vez — sea como argumento a dos funciones, o en un loop — el compilador Rust lo rechaza.

**Workarounds aplicados:**
- Reestructurar el parser como clase `JsonParser` con campo `tokens` (field access genera clone)
- Usar `while true { let x = arr[i]; if !cond { break } }` en vez de `while cond { ... x = arr[i] }`
- Almacenar resultados en arrays temporales `[result]` para poder acceder vía `arr[0]` (que genera `.clone()`)

**Impacto**: 50%+ del esfuerzo de corrección fue dedicado a ownership issues.

### 2. Template string interpolation con `{:?}` (error 15)
Variables mutables que se reasignan se interpolan con `format!("{:?}", var)` en vez de `format!("{}", var)`. El formato Debug agrega comillas y escapa caracteres especiales, causando corrupción silenciosa de datos (backslash explosion, comillas espurias).

**Impacto**: Este bug es particularmente insidioso porque el programa compila y ejecuta pero produce datos incorrectos. Solo se detecta en runtime.

### 3. String concatenation con `+` (error 14)
`result = result + "x"` genera `result.extend("x")` que es un método de iteradores, no de strings. La concatenación de strings debería generar `push_str()` o `format!()`.

### 4. `or fail` completamente no funcional (errores 4, 9, 10)
El mecanismo de propagación de errores `or fail` no genera código Rust funcional. Ni el operador `?`, ni `.map_err()`, ni ningún patrón de propagación se emite. El error se ignora silenciosamente.

### 5. `_` en error binding (error 1)
Patrón recurrente en múltiples auditorías. El compilador sugiere `let val, _ = func()` en sus hints pero no acepta `_` como identificador.

### 6. Delimiter-based encoding (error 16)
La IA diseñó un sistema de encoding basado en delimitadores (`<<KV>>`, `<<ENTRY>>`, `<<ITEM>>`) que funciona para estructuras planas pero falla con nesting. Este es un error de diseño algorítmico de la IA, no del compilador.

## Verificación de ejecución

**Programa ejecutado y verificado.** Todos los demos probados:
- ✅ Demo 1: Building JSON programmatically — compact y pretty print (2 y 4 spaces) con nested objects/arrays
- ✅ Demo 2: Parsing a JSON string — nested objects, arrays con strings, booleans, null
- ✅ Demo 3: Round-trip test — parse → stringify → parse → stringify = stable (valores idénticos)
- ✅ Demo 4: All JSON types — string, integer, float, negative, bool, null, array, nested object. Type inspection correcta (9 keys, todos los tipos detectados)

**Output verificado:**
```
Compact: {"name":"Alice","age":30,"active":true,"score":95.5,"nickname":null,...}
Round-trip: ✓ Round-trip is stable!
Type checks: isJObject:true, 9 keys, all types correctly identified
```

## Notas sobre la versión corregida

La versión corregida (`fixed/main.liva`) difiere significativamente del original:

1. **5 archivos → 1**: Merged en single file para evitar bugs de multi-file error binding codegen
2. **Encoding reescrito**: De delimiter-based (`<<ENTRY>>`) a length-prefixed (`"5:hello"`) para soportar nesting
3. **Parser como clase**: `JsonParser` class con campo `tokens` (workaround para Vec ownership)
4. **Token encoding como strings**: `T_LBRACE`, `T_STRING::value` en vez del enum `Token`
5. **Helper `_strCat(a, b)`**: Para evitar `{:?}` en templates y `.extend()` en concatenación
6. **Array-based cloning**: `let arr = [value]; _fn(arr[0]); _fn2(arr[0])` para clonar sin mover

## Conclusiones

- **Calidad del código original**: Alta. Arquitectura bien diseñada (lexer → parser → serializer) con buena separación de responsabilidades. 5 archivos, 882 líneas.
- **13 de 16 errores son bugs del compilador** — la gran mayoría son codegen issues. La IA generó código Liva válido que el compilador tradujo incorrectamente a Rust.
- **Este proyecto expone la mayor cantidad de bugs de codegen de toda la auditoría**: charAt, char escapes, enum destructuring, or fail, error binding, string +, template {:?}, array clone, ownership/move.
- **El bug de `{:?}` (error 15) es el más peligroso**: compila, ejecuta, pero produce datos corruptos silenciosamente. Prioridad alta para fix.
- **Ownership/move (errores 11-13) es el patrón más caro**: requiere reestructuración masiva del código, patrones antiidiomáticos, y elimina la elegancia del diseño original.
- **Solo 1 error de la IA** (encoding design), **0 de la skill**, demostrando que la skill guía bien la generación de código Liva complejo.
- **0 bloques `rust { }`** — prueba de que proyectos no-triviales son posibles en Liva puro.
