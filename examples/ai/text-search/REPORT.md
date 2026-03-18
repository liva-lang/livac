# Audit Report: text-search

## Resumen
- Total errores: 9
- BUG: 5 | SKILL: 3 | GAP: 0 | AI: 0 | DESIGN: 1

## Errores encontrados

| # | Archivo | Línea(s) | Error | Categoría | Fix |
|---|---------|----------|-------|-----------|-----|
| 1 | search.liva | 134, 137 | E1000: Invalid token `─` en string template `$"  {"─".repeat(46)}"` | BUG | Extraer separador a variable con carácter ASCII: `let separator = "-".repeat(46)` |
| 2 | search.liva | 134, 137 | String literal anidada dentro de expresión de template `$"...{"─".repeat(46)}..."` — el lexer no soporta strings dentro de `{expr}` | DESIGN | Extraer la expresión a una variable antes de interpolar |
| 3 | search.liva | 222-253 | E2000: Parse error — `let` en top-level después de funciones. El parser espera `main()` para el punto de entrada cuando hay funciones definidas | SKILL | Envolver código principal en `main() { ... }` |
| 4 | search.liva | 241 | `console.prompt("search> ")` no existe — el compilador solo soporta `console.input()` | SKILL | Cambiar a `console.input("search> ")` |
| 5 | search.liva | 79 | `index.set(word, file)` en rama `else` mueve `file` (ownership). Luego `file` se usa en `print($"  Indexed: {file}")` — E0382: borrow of moved value | BUG | Usar `$"{file}"` para crear nuevo string sin mover el original |
| 6 | search.liva | 113 | `uniqueA.filter(f => b.includes(f))` genera `.copied()` en Rust, pero `String` no implementa `Copy` — E0277/E0599 | BUG | Reemplazar con bucle manual `for item in uniqueA { if b.includes(item) { ... } }` |
| 7 | search.liva | 153 | `let displayCount = parseInt(countStr) or 0` genera tupla `(i32, String)` en vez de solo `i32` — E0277: Display not implemented for tuple | BUG | Usar error binding explícito: `let displayCount, parseErr = parseInt(countStr)` |
| 8 | search.liva | 90-96, 162-216 | `searchWord(index, word)` pasa Map por valor, moviendo `index`. Llamadas múltiples causan E0382: use of moved value | BUG | Reestructurar con clase `SearchEngine` — métodos acceden `this.index` por referencia (&self) |
| 9 | search.liva | 222-225 | `Sys.args()` incluye nombre del programa en args[0]. El código usa `args[0]` como primer argumento de usuario | SKILL | Cambiar a `args[1]` y `args.length > 1` |

## Patrones problemáticos

### 1. Pasar Map a funciones (BUG — Ownership)
El codegen genera parámetros `HashMap<String, String>` por valor. Cada llamada a función que recibe un Map lo mueve (ownership transfer). Si el Map se usa después, Rust genera E0382. **Workaround:** usar clases donde `this.field` genera `&self.field` (borrow).

### 2. `filter()` + `includes()` en arrays de strings (BUG — `.copied()` vs `.cloned()`)
La combinación `.filter(f => b.includes(f))` genera `.copied()` que requiere `Copy`, pero `String` no lo implementa. Curiosamente, `.filter(f => f == val)` genera `.cloned()` que sí funciona. Inconsistencia en el codegen.

### 3. `parseInt(x) or default` genera tupla (BUG)
`parseInt(countStr) or 0` debería generar `match str.parse::<i32>() { Ok(v) => v, Err(_) => 0 }` pero genera la tupla completa con el error string. El `or` no simplifica el resultado de funciones fallibles.

### 4. Strings dentro de template expressions (DESIGN)
`$"text {expr}"` no soporta que `expr` contenga strings literales como `$"text {"value".method()}"`. El lexer/parser se confunde con las comillas anidadas. Requiere extraer a variable.

### 5. No documentar `main()` como punto de entrada (SKILL)
La skill no menciona que los archivos con funciones necesitan un `main()` para el código de entrada. La IA (y cualquier usuario) asume que el código top-level es válido.

### 6. `Sys.args()` incluye programa (SKILL)
`Sys.args()` devuelve todos los args incluyendo el ejecutable en `args[0]`, como Rust. Pero la mayoría de lenguajes de alto nivel (Python, JS, Go) filtran el programa. La skill no lo documenta.

## Reestructuración aplicada

El código original usaba funciones standalone que recibían `Map<string, string>` como parámetro. Esto causa ownership moves en Rust. La versión corregida usa una clase `SearchEngine` que encapsula el índice como campo `this.index`, permitiendo que los métodos accedan al Map por referencia (`&self`).

Standalone functions que no usan Map se mantienen: `cleanWord`, `computeFrequency`, `intersectFiles`, `displayResults`.

## Verificación

- **Compila:** ✅ `livac check` y `livac build` exitosos
- **Ejecuta:** ✅ Probado con 5 archivos .txt (448 palabras indexadas)
- **Single word search:** ✅ "science" → 5 resultados ordenados por frecuencia
- **AND search:** ✅ "nature AND world" → intersección correcta
- **OR search:** ✅ "science OR technology" → unión correcta
- **Edge cases:** ✅ Palabra inexistente → "No results". Input vacío → se ignora. "quit"/"exit" → cierra.

## Conclusiones

1. **Ownership de Map es el problema más grave** — pasar Map a funciones lo mueve en Rust. La skill debería recomendar clases cuando se comparten datos complejos entre funciones.
2. **5 de 9 errores son BUGS del compilador** — codegen genera Rust incorrecto para patrones que parecen válidos en Liva.
3. **3 errores de SKILL** — `main()`, `console.prompt` vs `console.input`, y `Sys.args()` con programa incluido. Todos fáciles de documentar.
4. **El proyecto es un buen caso de test** — ejercita Map, File I/O, Dir, loops, filter, includes, split, join, y clases. Ideal para tests de regresión.
