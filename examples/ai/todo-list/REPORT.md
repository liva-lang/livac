# Audit Report: todo-list

> **Fecha:** 2026-03-18  
> **Archivos:** 3 (task.liva, manager.liva, main.liva — 251 líneas)  
> **Skill local:** No  
> **Resultado:** ✅ Compilado y funcionando tras reestructuración significativa (merged single-file)

---

## Resumen

- **Total errores: 12**
- BUG: 8 | SKILL: 2 | GAP: 0 | AI: 1 | DESIGN: 1

---

## Errores encontrados

| # | Archivo | Línea | Error | Categoría | Fix aplicado |
|---|---------|-------|-------|-----------|--------------|
| 1 | main.liva | 33-34 | `E2000: Expected identifier` — `let task = ...` falla porque `task` es keyword reservada | **SKILL** | Renombrar variable `task` → `item` |
| 2 | manager.liva | 14-61 | `E2000: Expected identifier` — Múltiples usos de `task` como variable/loop en todo el archivo | **SKILL** | Renombrar todas las variables `task` → `item`/`t` |
| 3 | main.liva | 116 | `E2000: Expected LBrace` — `main()` como llamada standalone se interpreta como declaración de función | **AI** | Eliminar `main()` — Liva auto-invoca `main()` como entry point |
| 4 | task.liva | 10-15 | `Priority.Alta` en RHS de switch expression genera `priority.get_field("Alta")` — `enum_names` no se puebla en `generate_module_code()` | **BUG** | Merge a single file (entry point popula `enum_names`) + usar string en vez de enum |
| 5 | main.liva | 7-9 | `use chrono::Local;` dentro de `rust { }` se stripea del bloque pero no se hoistea al top del archivo — `Local::now()` undefined | **BUG** | Usar path completo `chrono::Local::now()` sin `use` statement |
| 6 | task.liva | 55 | `priority_label(self.priority)` — move de campo String/enum desde `&self` ref. Codegen no añade `.clone()` para campos non-Copy en funciones que toman ownership | **BUG** | Inlinear la lógica con comparaciones `==` (que usan refs, no mueven) |
| 7 | manager.liva | 59-62 | `for item in this.tasks { item.completed = true }` — itera sobre clone, mutación no afecta original. Además `item` no es `mut` | **BUG** | Usar `findIndex` + acceso indexado `this.items[idx].completed = true` |
| 8 | manager.liva | 53 | `complete_task(&self)` — codegen no detecta `this.items[idx].field = value` como mutación, genera `&self` en vez de `&mut self` | **BUG** | Añadir asignación dummy `this.nextId = this.nextId` para forzar detección de `&mut self` |
| 9 | main.liva | 34 | `item.get_field("id")` en string interpolation — field access en variable que no está en `class_instance_vars` genera `get_field()` (heurística JSON) | **BUG** | Extraer campo a variable local antes de interpolar: `let tid = this.id` |
| 10 | main.liva | 58 | `manager.count()` → `manager.iter().filter().count()` — nombre de método `count()` colisiona con método de array. Codegen lo trata como array method | **BUG** | Renombrar a `totalCount()` para evitar colisión |
| 11 | main.liva | 69 | `console.input($"...#{id}...")` genera `print!(format!(...))` — `print!` macro no acepta resultado de `format!` como argumento | **BUG** | Usar strings sin interpolación en prompts de `console.input` |
| 12 | main.liva | 93-102 | `handle_add(manager.clone())` — class instances se clonan al pasar a free functions, mutaciones se pierden en el caller | **DESIGN** | Inlinear toda la lógica en `main()` en vez de free functions separadas |

---

## Patrones problemáticos

### 1. `task` como keyword reservada (SKILL)
La IA usó `task` como nombre de variable en 15+ lugares. Es totalmente natural (app de tareas → variable "task"), pero `task` es keyword en Liva (usado para `task async`). La skill sí lista las keywords pero la IA no las consultó. **Impacto: colapso total del parser.**

### 2. Codegen multi-módulo roto para enums (BUG — Crítico)
`generate_module_code()` no puebla `enum_names` ni `enum_variants`, a diferencia de `generate_entry_point()` y `generate_program()`. Cualquier enum variant usado como **expresión** (no como patrón de match) en un módulo genera `name.get_field("Variant")` en vez de `Name::Variant`. **Afecta a todo proyecto multi-archivo que use enums.**

**Root cause:** Solo `generate_program()` (L1181) y `generate_entry_point()` (L14904) hacen el pase de pre-población de `enum_names`. `generate_module_code()` (L14665) no lo hace.

### 3. Heurística `get_field` vs campo directo (BUG — Sistémico)
El codegen decide entre `obj.field` y `obj.get_field("field")` usando heurísticas negativas:
- Si `var_name` está en `class_instance_vars` → campo directo
- Si `var_name` es single-char → campo directo (hack)
- Si contiene "person" o "user" → campo directo (hardcoded)
- **Default: JSON `get_field()`**

Variables no trackeadas (parámetros de funciones, resultados de métodos de otros módulos, loop variables de longitud > 1) caen al fallback JSON. **La solución correcta es un type system real.**

### 4. Mutación de self: detección incompleta (BUG)
- `for item in this.list { item.field = value }` — no detectado como mutación
- `this.list[idx].field = value` — no detectado como mutación
- Solo `this.field = value` directo se detecta
- Resultado: método generado con `&self` en vez de `&mut self`

### 5. Paso de clases por valor (DESIGN)
Free functions que reciben class instances obtienen un `.clone()` automático. Cualquier mutación en la función se pierde al retornar. Liva necesita **semántica de referencia** para clases (como Python/TS), o al menos parámetros `&mut`.

### 6. Enum como campo → falla `Default` derive (BUG)
Todas las clases con constructor derivan `Default`. Si un campo es de tipo enum, y el enum no implementa `Default`, la compilación Rust falla. El codegen debería:
- Derivar `Default` en enums (pick first variant), o
- No derivar `Default` en structs con campos enum, o
- Generar impl Default manual

---

## Cambios realizados en fixed/

| Cambio | Motivo |
|--------|--------|
| Merge 3 archivos → 1 (main.liva) | Enum codegen funciona solo en entry point |
| `enum Priority` → `priority: string` + funciones helper | Enum como campo struct falla Default; enum variant expressions rotas en módulos |
| Eliminar free functions `handleAdd/Complete/Delete` → lógica inline en `main()` | Class instances se clonan al pasar a funciones; mutaciones perdidas |
| `priorityLabel(this.priority)` → `if this.priority == "alta"` inline | Move de campo non-Copy desde `&self` |
| `for t in this.items { t.completed = true }` → `this.items[idx].completed = true` | Loop iteration sobre clone no muta original |
| Añadir `this.nextId = this.nextId` en `completeItem` | Forzar detección de `&mut self` |
| `count()` → `totalCount()` | Colisión nombre con array method `count` |
| `console.input($"...#{id}...")` → `console.input("static prompt")` | `print!(format!(...))` inválido en Rust |
| `rust { use chrono::Local; Local::now()... }` → `rust { chrono::Local::now()... }` | Use hoisting no funciona |
| Extraer `this.id` → `let tid = this.id` antes de `$"...{tid}..."` | `get_field()` en string interpolation |
| Eliminar `main()` call al final | auto-invocado por runtime |
| `task` → `item`/`t` en todas las variables | keyword reservada |

---

## Conclusiones

### Acciones para el compilador (Bugs) — Ordenadas por impacto

1. **[BUG-CRITICAL]** Poblar `enum_names`/`enum_variants` en `generate_module_code()` igual que en `generate_entry_point()` y `generate_program()`. Sin esto, **ningún proyecto multi-archivo puede usar enum variants como expresiones**.

2. **[BUG-HIGH]** Reemplazar heurística `get_field`/campo directo por type tracking real. El approach actual (multiple HashSets + hardcoded names) es frágil y crea fallos en: string interpolation, function parameters, loop variables.

3. **[BUG-HIGH]** Detectar mutación transitiva para `&mut self`: `this.field[idx].prop = x`, `for item in this.field { item.prop = x }`, y llamadas a métodos que mutan.

4. **[BUG-MED]** Añadir `.clone()` automático cuando un campo `self.field` (non-Copy) se pasa a una función que toma ownership.

5. **[BUG-MED]** Fix `console.input` con string template: generar `print!("{}", format!(...))` en vez de `print!(format!(...))`.

6. **[BUG-MED]** Derivar `Default` en enums (primer variant), o no derivar `Default` en structs con campos enum.

7. **[BUG-LOW]** Fix `use` hoisting en `rust { }` blocks para el entry point (funciona en módulos pero no en main).

8. **[BUG-LOW]** Método `.count()` en clases colisiona con array `.count(fn)`. El dispatch debería priorizar métodos de clase sobre métodos de array.

### Acciones para la Skill

9. **[SKILL]** Destacar keywords reservadas más prominentemente. Agregar ejemplos de colisiones comunes (`task`, `type`, `move`) y alternativas (`item`, `taskItem`, `dataType`).

10. **[SKILL]** Documentar que `main()` es auto-invocada — no necesita llamada explícita al final del archivo.

### Acciones de Diseño

11. **[DESIGN]** Decidir semántica de paso de clases a funciones: ¿valor (clone) o referencia (&/&mut)? Actualmente las clases se clonan siempre, lo que hace imposible mutar un objeto desde una función externa. Python/TS usan referencia — Liva debería considerar lo mismo.

### Lo que la IA hizo bien
- Estructura multi-archivo limpia (task, manager, main) — correcta aunque triggers bugs
- Uso de `enum Priority { Alta, Media, Baja }` — correcto sintácticamente
- `switch lower { "alta" => ... }` — switch expression correcta
- `for task in this.tasks { print(task.display()) }` — iteración correcta
- `.filter(t => t.completed == false)` — lambdas con comparación correctas
- `.isEmpty()` en arrays — stdlib P0 correcta
- `this.tasks.push(item)` — mutación de array correcta
- `use rust "chrono" version "0.4"` + `rust { }` — interop syntax correcta
- `console.input("prompt")` — I/O syntax correcta
- `let id, err = parseInt(input)` + `if err { ... }` — error handling idiomático
- String templates `$"...{var}..."` — syntax correcta
- Data flow y lógica del programa — semánticamente correcta
