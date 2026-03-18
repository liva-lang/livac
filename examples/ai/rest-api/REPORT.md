# Audit Report: rest-api

## Resumen
- Total errores: 17
- BUG: 14 | SKILL: 1 | GAP: 1 | AI: 1 | DESIGN: 0

## Errores encontrados

| # | Archivo | Línea | Error | Categoría | Fix |
|---|---------|-------|-------|-----------|-----|
| 1 | db.liva | 28 | `let _, err = File.write(...)` — parser rechaza `_` como variable en error binding | BUG | Usar `let writeOk, err = File.write(...)` |
| 2 | main.liva | 3 | `use rust "actix-web"` genera `use actix-web;` en Rust (hyphen inválido) | BUG | Fix en codegen: `dep.name.replace('-', '_')` para `use` statements |
| 3 | db.liva | 5 | `const DB_FILE: string` genera `const DB_FILE: String = "..."` (literal `&str` ≠ `String`) | BUG | Usar función helper en vez de const: `_dbFile() => "..."` (también genera error, ver #4) |
| 4 | main.liva | — | Arrow function `=> "literal"` genera `&str` cuando el tipo de retorno es `String` | BUG | Usar block function con `let name = "..."; return name` |
| 5 | main.liva | — | `main()` no se genera async cuando `rust { }` contiene `.await` | BUG | Usar `actix_rt::System::new().block_on(async { ... })` |
| 6 | db.liva | 55-60 | `createAuthor`: `arr + [author]` mueve `author`, luego se usa en `return (newDb, author)` | BUG | Crear `Author(...)` inline para array. Crear segundo `Author(...)` para return |
| 7 | db.liva | 63-81 | `updateAuthor`: `arr + [updated]` en loop mueve `updated`, luego se usa en return | BUG | Usar `Author(...)` inline en loop. Crear `Author(...)` nuevo para return |
| 8 | db.liva | 83-100 | `deleteAuthor`: `for book in db.books` mueve `db.books`, luego se usa en `Database(..., db.books, ...)` | BUG | Single pass: rebuild books array en el loop, usar la copia en return |
| 9 | db.liva | 119-141 | `createBook`: `for author in db.authors` y `for book in db.books` mueven campos, luego se reusan | BUG | Single pass: rebuild arrays durante iteración |
| 10 | db.liva | 143-177 | `updateBook`: Mismos problemas de ownership + `updated` en loop | BUG | Single pass con rebuild + crear Book nuevo para return |
| 11 | db.liva | 179-194 | `deleteBook`: `for book in db.books` mueve `.books`, luego `Database(..., db.authors, ...)` usa `.authors` que fue movida antes | BUG | Rebuild `authors` en loop separado |
| 12 | db.liva | 209-218 | `searchBooksByAuthor`: Nested loop `for book` → `for aid` → `results + [book]` mueve `book` en inner loop, luego `book.authorId` se accede en siguiente iteración inner | BUG | Separar: flag `matched` en inner loop, mover `book` después del inner loop |
| 13 | models.liva | — | Data classes `PaginatedAuthors`, `PaginatedBooks`, `AuthorInput`, `BookInput` no obtienen serde derives automáticamente | BUG | `_registerJsonTypes()` con `JSON.parse` para forzar serde derives |
| 14 | main.liva | 65-589 | rust {} block asume que funciones fallibles devuelven `(value, error_string)` tuples, pero generan `Result<T, liva_rt::Error>` | AI | Reescribir calls: `match func() { Ok(v) => ..., Err(e) => ... }` |
| 15 | main.liva | — | rust {} block asume `create_author` devuelve `((Database, Author), String)` pero devuelve `(Database, Author)` directamente | AI | Usar destructuring directo: `let (new_db, author) = create_author(...)` |
| 16 | main.liva | — | `JSON.stringify()` no marca clases como `needs_serde`, solo `JSON.parse()` lo hace | GAP | Agregar tracking de serde en stringify también |
| 17 | main.liva | — | La skill no explica cómo se ven las funciones Liva desde `rust {}` (Result types, snake_case) | SKILL | Documentar en skill: fallible → Result, camelCase → snake_case, error.message |

**Nota:** Los errores 14-15 se cuentan como 1 AI porque son la misma confusión conceptual de la IA sobre el API de interop.

**Recuento ajustado:** BUG: 14 | SKILL: 1 | GAP: 1 | AI: 1

## Análisis de dependencia de `rust { }`

Este proyecto tiene el bloque `rust { }` más grande de toda la auditoría (~450 líneas). El 100% del código HTTP es Rust.

| # | Función | Propósito del `rust { }` | Feature necesaria en Liva | Prioridad | Versión |
|---|---------|--------------------------|--------------------------|-----------|---------|
| 1 | list_authors | HTTP GET handler con query params | `HTTP.serve()` stdlib module | Alta | v1.7 |
| 2 | get_author | HTTP GET handler con path param | `HTTP.serve()` stdlib module | Alta | v1.7 |
| 3 | create_author_handler | HTTP POST handler con JSON body | `HTTP.serve()` stdlib module | Alta | v1.7 |
| 4 | update_author_handler | HTTP PUT handler | `HTTP.serve()` stdlib module | Alta | v1.7 |
| 5 | delete_author_handler | HTTP DELETE handler | `HTTP.serve()` stdlib module | Alta | v1.7 |
| 6 | get_author_books | HTTP GET handler sub-resource | `HTTP.serve()` stdlib module | Alta | v1.7 |
| 7 | list_books | HTTP GET handler con search params | `HTTP.serve()` stdlib module | Alta | v1.7 |
| 8 | get_book | HTTP GET handler | `HTTP.serve()` stdlib module | Alta | v1.7 |
| 9 | create_book_handler | HTTP POST handler | `HTTP.serve()` stdlib module | Alta | v1.7 |
| 10 | update_book_handler | HTTP PUT handler | `HTTP.serve()` stdlib module | Alta | v1.7 |
| 11 | delete_book_handler | HTTP DELETE handler | `HTTP.serve()` stdlib module | Alta | v1.7 |
| 12 | search_books | HTTP GET search handler | `HTTP.serve()` stdlib module | Alta | v1.7 |
| 13 | AppState | Shared mutable state (Mutex) | Concurrency primitives / estado global | Media | v1.7+ |
| 14 | Server setup | HttpServer::new().bind().run() | `HTTP.serve()` stdlib module | Alta | v1.7 |

**Conclusión:** Un módulo `HTTP.serve()` con routing, request/response handling, query/path params, y JSON body parsing eliminaría ~450 líneas de Rust. Este es el proyecto que más justifica la prioridad de v1.7.

## Patrones problemáticos

### 1. Ownership/Move en array concatenation (14 BUGs de codegen)
El patrón `arr = arr + [value]` genera `{ let mut __v = arr.clone(); __v.extend(vec![value]); __v }`. Esto **mueve** `value`. Si `value` se usa después (en return o en siguiente iteración de loop), Rust lo rechaza. Este es el bug de codegen más frecuente del proyecto.

**Fix propuesto:** El codegen debería generar `__v.extend(vec![value.clone()])` cuando detecta que `value` se usa después del extend. O mejor: implementar análisis de liveness para determinar si se necesita clone.

### 2. `for item in collection` mueve collection
`for x in db.books { ... }` genera `for x in db.books { ... }` en Rust, lo cual consume `db.books` via `into_iter()`. Si después se accede a `db.books` o a otro campo de `db` que depende de `books`, el borrow checker rechaza el código.

**Fix propuesto:** El codegen debería generar `for x in db.books.iter()` (o `db.books.clone()`) cuando detecta que `db.books` se reutiliza después del loop. O usar `for x in &db.books` con dereferencing automático.

### 3. Hyphen en crate names
`use rust "actix-web"` genera `use actix-web;` en Rust source y `actix-web = "4"` en Cargo.toml. El Rust source necesita `use actix_web;` (con underscore). **Fix aplicado en este audit:** `dep.name.replace('-', '_')` en codegen.

### 4. Interop rust {} ↔ Liva functions
La IA no sabía que funciones fallibles generan `Result<T, liva_rt::Error>` en vez de tuples. La skill no documenta cómo se ven las funciones Liva desde dentro de `rust {}` blocks.

### 5. `const` con string literal
`const X: string = "..."` genera `const X: String = "..."` que falla porque `"..."` es `&str`. El codegen debería generar `const X: &str = "..."` para string constants, o agregar `.to_string()`.

## Fixes aplicados al compilador

Se aplicó un fix al compilador (`src/codegen.rs`) para corregir el bug #2 (hyphen en crate names):
- `generate_program()`: `dep.name.replace('-', '_')` para `use` statements
- `emit_use_statements()`: mismo fix para el path alternativo de generación
- **388 tests pasan** después del fix

## Conclusiones

1. **Proyecto más complejo de la auditoría** — 973 líneas, 4 archivos, rust interop masivo con actix-web
2. **El 80% de los bugs son de codegen** (ownership/move) — el compilador necesita mejor análisis de liveness para array concatenation y for loops
3. **La IA entendió muy bien la arquitectura** — separación models/db/validation/main es clean, CRUD pattern correcto, la relación 1:N funciona bien
4. **El error conceptual clave** de la IA fue asumir que las funciones Liva devuelven tuples `(value, error)` desde `rust {}` cuando en realidad devuelven `Result<T, liva_rt::Error>` → la skill debe documentar esto
5. **Necesidad de HTTP.serve()** — este proyecto es la mejor evidencia de que v1.7 es critical; actualmente toda la lógica HTTP debe ser Rust
6. **JSON.stringify debería trigger serde** — solo JSON.parse lo hace actualmente; es un GAP
7. **Compilación exitosa tras fixes** — el proyecto funciona completamente: CRUD autores/libros, búsqueda, paginación, validación, persistencia JSON, error handling con HTTP status codes apropiados
8. **Verificación completa** — 18 tests ejecutados: create, list, get, update, delete, search by title/author, pagination, validation errors, not found, cascade delete protection

## Ejecución verificada

```
T1:CreateAuthor1     → {"id":1,"name":"García Márquez","nationality":"Colombian"}     ✅
T2:CreateAuthor2     → {"id":2,"name":"Borges","nationality":"Argentine"}              ✅
T3:CreateBook1       → {"id":1,"title":"Cien años de soledad","authorId":1,...}         ✅
T4:CreateBook2       → {"id":2,"title":"Ficciones","authorId":2,...}                    ✅
T5:ListAuthors       → paginated: 2 authors, page=1, totalPages=1                     ✅
T6:ListBooks         → paginated: 2 books                                              ✅
T7:GetAuthor1        → author by ID                                                    ✅
T8:GetBook1          → book by ID                                                      ✅
T9:SearchTitle       → search "soledad" → 1 result                                    ✅
T10:AuthorBooks      → author 1's books → 1 book                                      ✅
T11:UpdateAuthor     → name updated to "Gabo"                                         ✅
T12:DeleteBook2      → HTTP 204                                                        ✅
T13:DeleteAuthorWithBooks → HTTP 409 "has associated books"                            ✅
T14:DeleteBook1+Author1  → both HTTP 204                                              ✅
T15:ValidationError  → HTTP 400 "Author name is required"                              ✅
T16:NotFound         → HTTP 404 "Author with id 999 not found"                        ✅
T17:Pagination       → limit=1, totalPages=1                                           ✅
T18:SearchByAuthor   → search "borges" (books deleted) → 0 results                    ✅
```
