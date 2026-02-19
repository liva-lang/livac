# Plan: AI Skills + Documentación compartida

**TL;DR**: Refactorizar los docs para que sean AI-friendly (<300 líneas/archivo), crear SKILL.md como router en `livac/skills/liva-lang/`, que apunte a los docs refactorizados. Los symlinks del instalador apuntan ahí. Una sola fuente de verdad para humanos y IAs.

## Estructura objetivo

```
livac/skills/liva-lang/
├── SKILL.md                    ← Router para agentes IA
└── docs/                       ← Symlink a livac/docs/ (o refs relativas)

livac/docs/                     ← Docs existentes, refactorizados
├── README.md                   (índice para humanos)
├── QUICK_REFERENCE.md          (cheat sheet unificado)
├── language-reference/         (archivos <300 líneas c/u)
│   ├── variables.md
│   ├── functions-basics.md     ← split de functions.md (943→~250+250)
│   ├── functions-advanced.md
│   ├── classes-basics.md       ← split de classes.md (1012→~300+300+300)
│   ├── classes-interfaces.md
│   ├── classes-data.md
│   ├── enums.md                ← NUEVO
│   ├── types-primitives.md     ← split de types.md (694→~250+250)
│   ├── types-advanced.md
│   ├── error-handling.md       ← condensar de 834 a <400
│   ├── concurrency.md          (544 OK, quizá split)
│   ├── pattern-matching.md     ← condensar de 930 a <400
│   ├── ...etc
│   └── stdlib/                 (estos ya están bien de tamaño)
└── ...
```

## Paso 1: Crear la estructura del skill

1. Crear `livac/skills/liva-lang/SKILL.md` con frontmatter YAML (`name: liva-lang`, `description: ...`)
2. SKILL.md incluye: descripción del lenguaje (~20 líneas), sintaxis esencial inline (~50 líneas), y la guía de navegación apuntando a archivos en `../../docs/` con rutas relativas
3. Cada referencia en SKILL.md dice **cuándo** usar cada archivo: "Para consultas sobre async/await y paralelismo → `../../docs/language-reference/concurrency.md`"

## Paso 2: Refactorizar docs grandes (>400 líneas)

Archivos a partir:

| Archivo actual | Líneas | Acción |
|---|---|---|
| `functions.md` | 943 | → `functions-basics.md` + `functions-advanced.md` |
| `classes.md` | 1012 | → `classes-basics.md` + `classes-interfaces.md` + `classes-data.md` |
| `pattern-matching.md` | 930 | → `pattern-matching.md` (~400, condensar) |
| `json.md` | 921 | → `json-basics.md` + `json-typed-parsing.md` |
| `generics.md` | 869 | → `generics-basics.md` + `generics-advanced.md` |
| `error-handling.md` | 834 | → `error-handling.md` (~400, condensar) |
| `http.md` | 781 | → `http.md` (~400, condensar) |
| `syntax-overview.md` | 754 | → `syntax-overview.md` (~400, es redundante con QUICK_REFERENCE) |
| `types.md` | 694 | → `types-primitives.md` + `types-advanced.md` |
| Archivos <500 líneas | — | Dejar como están |

## Paso 3: Crear `enums.md` en language-reference

El enum que acabamos de implementar no tiene doc en `language-reference/`. Crearla.

## Paso 4: Actualizar README.md de docs

Actualizar el índice para reflejar los nuevos archivos partidos.

## Paso 5: Post-install script para .deb/.rpm

Actualizar `release.yml` para que el paquete:
- Instale `livac/skills/liva-lang/` en `/usr/share/livac/skills/liva-lang/`
- Instale `livac/docs/` en `/usr/share/livac/docs/`
- Post-install: cree symlinks en los 9 agentes para cada usuario

### 9 agentes target

| # | Agente | Ruta global |
|---|---|---|
| 1 | GitHub Copilot | `~/.copilot/skills/` |
| 2 | Claude Code | `~/.claude/skills/` |
| 3 | Codex | `~/.codex/skills/` |
| 4 | Cursor | `~/.cursor/skills/` |
| 5 | Windsurf | `~/.codeium/windsurf/skills/` |
| 6 | Gemini CLI | `~/.gemini/skills/` |
| 7 | Antigravity | `~/.gemini/antigravity/skills/` |
| 8 | Continue | `~/.continue/skills/` |
| 9 | OpenClaw | `~/.openclaw/skills/` |

## Paso 6: Distribución en gestores de paquetes

- **Homebrew tap**: Crear repo `liva-lang/homebrew-tap` con fórmula → Linux + macOS
- **Winget**: Crear manifest y PR a `microsoft/winget-pkgs` → Windows
- **Scoop**: Crear bucket en `liva-lang/scoop-bucket` → Windows
- **APT**: Repo en GitHub Pages → Debian/Ubuntu
- **RPM**: Repo en GitHub Pages → Fedora/RHEL
- **AUR**: Crear cuenta en aur.archlinux.org + PKGBUILD → Arch

### Dependencias del usuario
- Rust/Cargo **requerido** para compilar programas `.liva`
- En APT/RPM/AUR/Homebrew → se instala como dependencia automática
- En Winget/Scoop → error claro si no está instalado: `"Run: winget install Rustlang.Rustup"`

## Paso 7: Automatizar releases en CI

Actualizar el workflow de release para publicar a cada gestor de paquetes automáticamente al crear un tag `v*`.

## Verificación

- `npx skills add liva-lang/livac --list` muestra la skill "liva-lang"
- Instalar el `.deb` y verificar que `~/.copilot/skills/liva-lang/SKILL.md` existe y apunta a `/usr/share/livac/skills/liva-lang/SKILL.md`
- Abrir Copilot/Claude/Cursor y verificar que reconocen la skill
- Docs siguen funcionando para lectura humana (links no rotos)

## Decisiones tomadas

- Skill en `livac/skills/liva-lang/` (no en docs/)
- Refactorizar docs existentes en vez de duplicar
- Archivos >400 líneas se parten o condensan
- 9 agentes en el post-install automático
- Sin `--install-skills` por ahora (pospuesto)
- Symlinks apuntan a `/usr/share/livac/skills/liva-lang/` — actualizar Liva actualiza skills para todos los agentes automáticamente
- Windows: junctions en vez de symlinks
