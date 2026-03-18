# Audit Report: chat-server

## Resumen
- Total errores: 1 (+ 4 cascading)
- BUG: 1 | SKILL: 0 | GAP: 0 | AI: 0 | DESIGN: 0

## Errores encontrados

| # | Archivo | Línea | Error | Categoría | Fix |
|---|---------|-------|-------|-----------|-----|
| 1 | main.liva | 6-195 | `main()` no se genera como `async fn main()` con `#[tokio::main]` porque `is_async_inferred` no inspecciona `rust { }` blocks. El código dentro del `rust { }` usa `.await` directamente en el scope de `main()`, pero el compilador solo detecta async en código Liva nativo. Error Rust: E0728 "await is only allowed inside async functions and blocks" | BUG | Envolver contenido del `rust { }` en `tokio::runtime::Runtime::new().unwrap().block_on(async move { ... })` para crear runtime propio sin depender de `#[tokio::main]`. También se añadieron features `"rt-multi-thread"` y `"macros"` al `use rust`. |
| 2-5 | main.liva | — | 4 errores E0282 "type annotations needed" en `socket.into_split()`, `writer.write_all()` — **cascading** del error #1. Sin async main, `.await` falla, y el tipo de retorno no se puede inferir. | — (cascading) | Se resuelven automáticamente al fix #1. |

## Análisis de dependencia de `rust { }`

Este proyecto es un **caso extremo**: el 100% de la lógica (190 de 195 líneas) está dentro de un solo `rust { }` block. Solo 5 líneas son Liva nativo (`use rust`, `main()`, dos `print()`).

| # | Función | Propósito del `rust { }` | Feature necesaria en Liva | Prioridad |
|---|---------|--------------------------|--------------------------|-----------|
| 1 | TCP Server | `TcpListener::bind().await` + accept loop | Módulo `Net` o `TCP` con `Net.listen("addr")` | Alta |
| 2 | Async Spawn | `tokio::spawn(async move { ... })` | `task async` ya existe pero requiere Net/IO | Media |
| 3 | Channels | `mpsc::unbounded_channel::<String>()` | `Channel<T>` tipo nativo o wrapper | Media |
| 4 | Shared State | `Arc<Mutex<HashMap<...>>>` | Colecciones concurrency-safe o wrapper `Shared<T>` | Media |
| 5 | Async IO | `BufReader`, `read_line().await`, `write_all().await` | Trait IO async: `stream.readLine()`, `stream.write()` | Alta |
| 6 | Socket Split | `socket.into_split()` → reader/writer independientes | API de streams bidireccionales | Media |

**Conclusión:** No tiene sentido escribir este proyecto sin `rust { }` hasta que Liva tenga un módulo de networking (candidato: v1.7 Stdlib P2 + HTTP Server).

## Patrones problemáticos
- **`rust { }` como escape hatch total**: El 97% del código es Rust puro. Esto es válido para interop con crates sin equivalente Liva, pero indica que networking/IO async es un GAP importante.
- **Async inference no detecta `rust { }` blocks**: Si un `rust { }` block contiene `.await` y hay `use rust "tokio"`, el compilador debería generar `#[tokio::main] async fn main()`. Actualmente, `expr_contains_async()` devuelve `false` para `Expr::RustBlock { .. }`.

## Detalles técnicos del bug

En `semantic.rs`, la función `expr_contains_async()` tiene un catch-all `_ => false` que incluye `Expr::RustBlock`. Para detectar si un `rust { }` contiene `.await`, se podría:
1. **Heurística textual**: Escanear `code` por `.await` → si lo contiene, marcar como async.
2. **Atributo manual**: Permitir `async rust { }` como sintaxis explícita.

La opción 1 es simple pero imprecisa (podría haber `.await` en strings). La opción 2 es más limpia y explícita.

## Ejecución y verificación
- ✅ Servidor compilado y ejecutado correctamente
- ✅ Conexión TCP funcional (single client: nickname, /help, /list, /quit)
- ✅ Multi-cliente: broadcast y DM verificados (Alice + Bob)
- ✅ Desconexión limpia con notificación a otros usuarios
- ✅ Conteo de usuarios actualizado en join/leave

## Conclusiones
- **Código de alta calidad**: El Rust generado por la IA es correcto, idiomático, y maneja edge cases (nickname vacío, duplicado, auto-DM, desconexión).
- **1 solo bug real**: Todos los errores de compilación se deben a la falta de `#[tokio::main]` en la función main generada.
- **Proyecto orientado a networking**: Demuestra la necesidad de un módulo TCP/Net en Liva para que los usuarios no tengan que escribir 190 líneas de Rust puro.
- **Calidad de la skill**: La IA usó `rust { }` correctamente para lo que no tiene equivalente Liva. El único error fue no manejar que `main()` con `rust { }` async necesita ser async.
