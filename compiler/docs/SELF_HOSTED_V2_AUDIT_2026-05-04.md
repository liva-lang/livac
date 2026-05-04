# Informe de revisión self-hosted v2.0 — Liva

**Fecha:** 2026-05-04  
**Ámbito principal:** `livac/compiler` — compilador self-hosted escrito en Liva.  
**Ámbito secundario:** `livac/src` solo como bootstrap congelado y referencia de release.  
**Objetivo:** detectar riesgos, fallos de diseño, huecos de tests/benchmarks y mejoras para llegar a una v2.0 sólida.

## Resumen ejecutivo

El estado técnico del self-hosted es muy bueno: los gates rápidos de `livac/compiler` pasan con gen-2, incluyendo `bootstrap_apps` 21/21, `multifile_apps` 3/3, regression 5/5, `complex_apps` 4/4, `e2e_selfhost` 5/5 y `cargo test --release`. Además, los programas complejos comparados contra bootstrap generan stdout idéntico y, en varios casos, mucho menos Rust que el bootstrap.

No veo un bloqueo funcional grande para v2.0 si el criterio es "gen-2 funciona y se puede validar". Sí veo varios puntos que conviene cerrar antes de etiquetar una release pública: versión publicada desalineada, CI normal incompleto frente a los gates reales de self-hosting, diagnostics/editor desalineados con gen-2, documentación histórica obsoleta dentro de `compiler/docs`, y un riesgo arquitectónico claro en `compiler/src/codegen.liva`, que ya está en 9.086 líneas.

Mi recomendación: **v2.0 puede salir como release self-hosted si se corrigen primero los puntos P0/P1 de este informe**. El trabajo más importante no es añadir features; es hacer que la validación, la versión, la documentación y el tooling cuenten la misma historia.

## Validación ejecutada

### Gates self-hosted rápidos

Comando ejecutado:

```bash
NO_COLOR=1 CLICOLOR=0 bash compiler/tests/run_all.sh --quick
```

Resultado:

| Gate | Resultado |
| --- | --- |
| `bootstrap_apps (gen-2)` | 21 pass / 0 fail |
| `multifile_apps (gen-2)` | 3 pass / 0 fail |
| `regression` | 5 pass / 0 fail |
| `complex_apps` | 4 pass / 0 fail |
| `e2e_selfhost` | 5 pass / 0 fail |
| `cargo test --release` | pass |

Nota: se usó `--quick`, por lo que no se reejecutó `rebuild_selfhost (gen-2 == gen-3)`. El script completo sí existe en `compiler/tests/run_all.sh` y ejecuta ese gate cuando no se pasa `--quick`.

### `cargo test` normal

Comando ejecutado:

```bash
cargo test
```

Resultado: **falló** con snapshots de parser porque los diagnostics salieron con códigos ANSI de color.

Después se ejecutó:

```bash
NO_COLOR=1 CLICOLOR=0 cargo test --test parser_tests
```

Resultado: **27 passed, 0 failed, 1 ignored**.

Conclusión: no parece un bug funcional del parser; es un problema de hermeticidad del test harness. `cargo test` plano debería pasar en cualquier terminal/CI sin depender de variables de color.

## Hallazgos prioritarios

### P0 — La versión pública sigue en `1.5.0`

`Cargo.toml` declara:

```toml
version = "1.5.0"
```

Mientras la documentación principal habla de `v2.0.0-dev` y release-ready. Como `main.rs` usa `env!("CARGO_PKG_VERSION")`, cualquier binario local o paquete generado desde el manifiesto actual puede informar `1.5.0`.

Impacto:

- Riesgo de publicar artefactos con versión incorrecta.
- Confusión para `livac --version`, package managers y changelog.
- La release workflow cambia `Cargo.toml` desde el tag, pero el repo en desarrollo sigue contando una historia distinta.

Acción recomendada:

- Antes de taggear, decidir si será `2.0.0` final o `2.0.0-rc1`.
- Sincronizar `Cargo.toml`, `Cargo.lock`, `README.md`, `ROADMAP.md`, `.github/copilot-instructions.md` y `docs/README.md`.

### P0 — CI normal no ejecuta los gates reales del self-hosted

`.github/workflows/ci.yml` ejecuta `cargo build`, `cargo test`, `cargo clippy` y `cargo fmt --check`, pero no ejecuta `compiler/tests/run_all.sh` ni los gates gen-2.

Impacto:

- Un PR puede estar verde en CI y romper `bootstrap_apps`, `e2e_selfhost` o la idempotencia gen-2/gen-3.
- La validación que realmente define v2.0 vive en scripts locales/documentación, no como protección automática de la rama.

Acción recomendada:

- Añadir un job Linux de self-hosting con `NO_COLOR=1 CLICOLOR=0 bash compiler/tests/run_all.sh --quick` para PRs.
- Añadir un job manual/nightly o release-gate con `bash compiler/tests/run_all.sh` completo, incluyendo `rebuild_selfhost`.
- Mantener `cargo test --release` dentro del gate, como ya hace `run_all.sh`.

### P0 — `cargo test` no es hermético por color ANSI

`cargo test` falló en snapshots de `parser_tests` por diferencias de color: los snapshots esperaban texto plano y la ejecución generó ANSI.

Impacto:

- Reproducibilidad local frágil.
- CI puede cambiar de comportamiento si el entorno se detecta como TTY/color-capable.
- Los snapshots de diagnostics son especialmente sensibles; justo son la garantía de calidad de errores.

Acción recomendada:

- Forzar `NO_COLOR=1` o equivalente dentro del test harness de diagnostics.
- Alternativamente, normalizar/strip ANSI antes de comparar snapshots.
- Evitar que el resultado dependa de configuración global del terminal.

### P1 — VS Code Problems reporta errores en `bootstrap_apps` que gen-2 compila

El panel de problemas actual reporta diagnostics en varios archivos de `compiler/tests/bootstrap_apps`:

- `app17_pipeline.liva`: llamadas fallibles con `or 0` marcadas como no manejadas.
- `app8_orders.liva` y `app16_fsm.liva`: `or fail` marcado como error handling no manejado.
- `app28_closures.liva`: function type `(number) => number` marcado como parse error.

Pero `compiler/tests/run_all.sh --quick` confirma que `bootstrap_apps (gen-2)` pasa 21/21.

Impacto:

- El compilador self-hosted acepta patrones que el tooling/editor aún puede marcar como rotos.
- Mala experiencia para dogfooding: el usuario ve rojo en código que el gate oficial compila.
- Riesgo de que LSP/check path use semántica antigua o no esté alineado con gen-2.

Acción recomendada:

- Auditar si esos diagnostics vienen del bootstrap Rust, del LSP, de la extensión o de caché.
- Añadir tests LSP/diagnostics para `or <default>`, `or fail` y function types.
- Definir explícitamente qué motor valida el editor en v2.0: bootstrap, gen-2, o ambos.

### P1 — `compiler/docs` está parcialmente obsoleto

`compiler/docs/PLAN.md` dice que `codegen.liva` tiene 4.941 líneas y que la Fase 10 está en curso. La medición actual muestra:

| Archivo | Líneas actuales |
| --- | ---: |
| `compiler/src/codegen.liva` | 9.086 |
| `compiler/src/parser.liva` | 2.373 |
| `compiler/src/semantic.liva` | 1.735 |
| `compiler/src/main.liva` | 766 |
| `compiler/src/liveness.liva` | 661 |

Impacto:

- La documentación de arquitectura del self-hosted ya no refleja el sistema actual.
- Puede inducir a decisiones equivocadas sobre complejidad, cobertura y prioridades.
- Algunos issues figuran como abiertos en `compiler/docs/ISSUES.md` aunque `compiler/PARITY.md` y los gates indican que varios casos ya pasan en práctica.

Acción recomendada:

- Convertir `compiler/PARITY.md` en fuente viva de paridad y cerrar/confirmar los items resueltos en práctica.
- Actualizar `compiler/docs/PLAN.md` con métricas actuales y estado de Phase 10/11.
- Marcar `compiler/docs/ISSUES.md` como histórico o reconciliarlo con los tests actuales.

### P1 — `compiler/src/codegen.liva` necesita modularización antes de crecer más

El self-hosted evita muchos defectos del bootstrap, pero `codegen.liva` ya tiene 9.086 líneas. El propio `compiler/PARITY.md` propone modularizarlo en:

- `codegen/expr.liva`
- `codegen/stmt.liva`
- `codegen/types.liva`
- `codegen/class.liva`
- `codegen/method.liva`
- `codegen/runtime.liva`
- `codegen/error.liva`

Impacto:

- Alto coste de revisión por cambio.
- Mayor probabilidad de divergencia entre dispatch de métodos tipados y fallback genérico.
- Más difícil aislar tests unitarios por área.

Acción recomendada:

- No bloquear v2.0 por esto si los gates están verdes.
- Sí abrirlo como primer bloque post-v2.0 o v2.1, antes de añadir más stdlib o features.
- Mantener la modularización mecánica: mover sin rediseñar primero, test completo, y solo después limpiar dispatch.

### P1 — Paridad documentada como verde, pero inventario formal aún tiene items pendientes

`compiler/PARITY.md` indica que `bootstrap_apps` pasa 21/21 con gen-2, pero conserva muchos items `⏳` en Tier 1/2/3. El propio documento explica que algunos pueden estar resueltos en práctica y pendientes de auditoría 1-a-1.

Impacto:

- Para v2.0 es aceptable si el criterio es funcionalidad validada por apps.
- Para eliminar bootstrap o vender gen-2 como único compilador, no basta: hay que cerrar el inventario formal.

Acción recomendada:

- Crear una tabla de cierre por ID: test que lo cubre, commit/fix, estado real.
- Separar "pasa por cobertura indirecta" de "cubierto por test específico".
- Mantener Tier 2 error handling y Tier 3 Map/self-mutation como scope v2.1 si no son necesarios para v2.0.

### P1 — LSP y gen-2 necesitan una decisión de producto

La extensión VS Code y el LSP son parte crítica del lenguaje, pero v2.0 self-hosted plantea una pregunta: ¿el editor usa el compilador self-hosted, el bootstrap Rust, o una mezcla?

Impacto:

- Si el usuario instala v2.0 pero el editor diagnostica con reglas antiguas, el lenguaje parece inconsistente.
- Si el LSP depende del bootstrap, la migración self-hosted queda incompleta desde el punto de vista de experiencia de desarrollo.

Acción recomendada:

- Definir una política para v2.0: "LSP sigue en bootstrap" o "LSP usa gen-2".
- Si sigue en bootstrap, documentarlo y asegurar paridad semántica visible.
- Añadir un smoke test de extensión/LSP contra ejemplos de `compiler/tests/bootstrap_apps`.

## Tests y cobertura

Lo positivo:

- `compiler/tests` tiene una estructura madura: bootstrap apps, multifile apps, regression, complex apps, e2e self-host y suite Liva.
- El gate rápido ejecutado pasó completo.
- `compiler/tests/run_all.sh` ya modela bastante bien lo que debe ser un release gate.

Huecos a cerrar:

- `run_all.sh --quick` no comprueba idempotencia gen-2/gen-3; está bien para desarrollo, pero no para release.
- La cobertura `cargo-llvm-cov` está documentada como baseline, pero no se ve como gate en CI normal.
- Hay tests ignored en Rust (`parser_tests::test_imports`, `semantics_tests` sobre `.length` en tipos inválidos). No parecen bloquear self-hosted, pero conviene decidir si son deuda real o tests obsoletos.
- El panel de Problems muestra errores sobre casos que pasan por gen-2; eso pide tests de diagnostics, no solo de compilación.

Recomendación de matriz mínima para v2.0:

| Contexto | Comando |
| --- | --- |
| PR rápido | `NO_COLOR=1 CLICOLOR=0 cargo test` |
| PR self-host Linux | `NO_COLOR=1 CLICOLOR=0 bash compiler/tests/run_all.sh --quick` |
| Release local/CI manual | `NO_COLOR=1 CLICOLOR=0 bash compiler/tests/run_all.sh` |
| Bench release | `benchmarks/run_official.sh` |
| Cobertura | `cargo llvm-cov` o job informativo con baseline |

## Benchmarks

`benchmarks/RESULTS.md` muestra los objetivos principales bajo el gate documentado:

| Benchmark oficial documentado | Ratio |
| --- | ---: |
| Line processing | 1,03x |
| CSV building | 0,99x |
| Word counting | 0,98x |
| Map build+lookup | 1,07x |

Hay dos métricas secundarias llamativas:

| Métrica | Ratio |
| --- | ---: |
| Filter+Map | 1,50x |
| Sort | 2,50x |

No las trataría como bloqueantes si el gate oficial de v2.0 no las incluye, pero sí las dejaría como objetivos post-release. También hay métricas con `0ms`, especialmente en `classes`, que no son fiables para comparar rendimiento fino.

Acción recomendada:

- Documentar explícitamente qué métricas forman parte del gate v2.0.
- Aumentar tamaño/iteraciones de benches con `0ms` para evitar ratios sin señal.
- Mantener Sort y Filter+Map como work items v2.1.

## Arquitectura

### Lo que está bien

- La separación `compiler/src` como compilador Liva puro es correcta y potente.
- El diseño type-directed del self-hosted es mucho más sano que los HashSets de tracking del bootstrap.
- `liveness.liva` es una pieza clave: preservar ownership/clone decisions ahí evita parches locales en codegen.
- Los gates comparan stdout bootstrap vs gen-2, lo cual es una validación de comportamiento muy valiosa.

### Riesgos

- `codegen.liva` vuelve a concentrar demasiada responsabilidad.
- El dispatch de stdlib y métodos puede divergir entre rutas tipadas y fallback genérico.
- El error reporting del codegen todavía tiende a producir Rust inválido/panic en algunos casos en vez de diagnostics Liva de primera clase.
- La documentación y el inventario de paridad no están sincronizados con el resultado real de los gates.

### Dirección recomendada

1. Congelar features nuevas hasta cerrar release hygiene.
2. Modularizar codegen en v2.1 sin cambiar comportamiento.
3. Introducir `Diagnostic`/errores acumulables en codegen self-hosted.
4. Convertir cada item de `compiler/PARITY.md` en test específico o cerrarlo como cubierto.
5. Unificar la historia editor/LSP/gen-2.

## Release y packaging

Lo positivo:

- `release.yml` extrae versión desde tag.
- Genera checksums `sha256sum * > checksums.txt` y los adjunta al release.
- Incluye skills y docs en artefactos.

Riesgos:

- El repo actual sigue con `Cargo.toml` en `1.5.0`.
- README mantiene badge de tests `337 passing`, obsoleto frente a 518/528+.
- La extensión VS Code está en `0.14.0`; no es necesariamente malo, pero falta política clara de versionado frente al compilador v2.0.
- CI no protege los gates self-hosted antes de release.

Acción recomendada:

- Añadir checklist de release v2.0 con: versión, `run_all.sh` completo, benches, checksums, extension compatibility y docs.
- Publicar en README qué significa "self-hosted" en v2.0: si el binario final es gen-2, si bootstrap sigue en repo, y qué queda para v2.1.

## Lista de acciones recomendada

### Antes de v2.0

1. Sincronizar versión de `Cargo.toml`/docs/tag a `2.0.0` o `2.0.0-rc1`.
2. Hacer que `cargo test` sea hermético sin depender de color del terminal.
3. Añadir `compiler/tests/run_all.sh --quick` a CI en Linux.
4. Ejecutar y registrar `compiler/tests/run_all.sh` completo antes del tag.
5. Resolver o documentar la desalineación de diagnostics VS Code/LSP con `bootstrap_apps`.
6. Actualizar `compiler/docs/PLAN.md` y `compiler/docs/ISSUES.md` para reflejar el estado real.
7. Actualizar README: badge de tests, estado v2.0, y explicación self-hosted.

### Justo después de v2.0 / v2.1

1. Modularizar `compiler/src/codegen.liva`.
2. Cerrar formalmente `compiler/PARITY.md` item por item.
3. Implementar error propagation/diagnostics propios en codegen.
4. Mejorar benchmarks de Sort, Filter+Map y casos `0ms`.
5. Alinear LSP/extensión con gen-2 o documentar claramente la transición.
6. Convertir cobertura en job informativo o gate de regresión.

## Veredicto

El compilador self-hosted está en una posición fuerte para v2.0. Lo que falta no parece ser "más compilador", sino disciplina de release: automatizar el gate real, limpiar la versión, evitar tests dependientes del terminal, reconciliar documentación histórica y asegurar que el editor no contradice a gen-2.

Si se corrigen los P0 y se decide una política clara para diagnostics/LSP, la v2.0 puede salir con bastante confianza. La siguiente gran inversión técnica debería ser modularizar `codegen.liva` antes de que el self-hosted herede el mismo problema de concentración que se quiso dejar atrás en el bootstrap.