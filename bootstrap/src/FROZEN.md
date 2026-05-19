# 🧊 BOOTSTRAP COMPILER — FROZEN

> **Status:** Congelado post-`ba7f263` (2026-04-30).
> **Reason:** Plan v2.1 — Self-Hosted Migration. Toda feature/fix nueva
> debe hacerse en `livac/compiler/src/*.liva` (gen-2).
> **Plan:** Ver `livac/BACKLOG.md` § v2.1 y `livac/compiler/PARITY.md`.

## Qué significa "congelado"

- ❌ NO añadir features de lenguaje aquí.
- ❌ NO añadir nuevos `selfhost_apps/`.
- ✅ Sí: bug-fixes que rompen el ciclo de auto-compilación.
- ✅ Sí: cambios en `liva_rt` (runtime crate, no se elimina en v2.1).

## Cuándo se elimina

- Cuando `compiler/tests/selfhost_apps/run_gen2.sh` pase 21/21 con gen-2.
- Cuando todos los items Tier 1+2+3 de `compiler/PARITY.md` estén ✅.
- Plan: v2.1 release.

## Mientras tanto

El bootstrap sigue siendo:
- Punto de entrada de `cargo build --release` → produce `target/release/livac`.
- Compilador de gen-1 (que produce gen-2, que produce gen-3).
- Referencia técnica para portar fixes a gen-2.
