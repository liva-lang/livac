# Plan: AI Skills + Distribución — Estado

**Última actualización:** 2026-02-19

## Progreso

| Paso | Descripción | Estado |
|------|-------------|--------|
| 1 | Crear SKILL.md router | ✅ Completado |
| 2 | Refactorizar docs (splits + condenses) | ✅ Completado |
| 3 | Crear enums.md | ✅ Completado |
| 4 | Actualizar docs/README.md | ✅ Completado |
| 5 | Post-install scripts (.deb/.rpm) | ✅ Completado |
| 6 | Package manager manifests | ✅ Completado (templates) |
| 7 | Actualizar release.yml CI | ✅ Completado |

## Qué se hizo

### Paso 1: SKILL.md
- `skills/liva-lang/SKILL.md` — router con frontmatter YAML, sintaxis esencial, 30 referencias verificadas a docs

### Paso 2: Refactorización de docs
- **5 archivos partidos:**
  - `functions.md` (943) → `functions-basics.md` (556) + `functions-advanced.md` (408)
  - `classes.md` (1012) → `classes-basics.md` (364) + `classes-interfaces.md` (446) + `classes-data.md` (347)
  - `types.md` (694) → `types-primitives.md` (329) + `types-advanced.md` (406)
  - `generics.md` (869) → `generics-basics.md` (409) + `generics-advanced.md` (486)
  - `json.md` (921) → `json-basics.md` (555) + `json-advanced.md` (424)
- **4 archivos condensados:**
  - `pattern-matching.md`: 929 → 450
  - `error-handling.md`: 833 → 384
  - `http.md`: 780 → 396
  - `syntax-overview.md`: 753 → 319
- Archivos originales borrados

### Paso 3: enums.md
- `docs/language-reference/enums.md` (~190 líneas)

### Paso 4: docs/README.md
- Actualizado con nuevos nombres de archivo y sección de enums

### Paso 5: Post-install scripts
- `scripts/install-skills.sh` — script compartido que crea/elimina symlinks para 9 agentes
- `scripts/deb/postinst` + `scripts/deb/prerm` — hooks de Debian
- `scripts/rpm/post-install.sh` + `scripts/rpm/pre-uninstall.sh` — hooks de RPM
- `Cargo.toml` actualizado con assets y maintainer-scripts

### Paso 6: Package manager manifests (templates)
- `packaging/homebrew/livac.rb` — con post_install que crea symlinks para 9 agentes
- `packaging/scoop/livac.json` — con post_install message
- `packaging/winget/liva-lang.livac.yaml` — manifest singleton
- Todos usan placeholders (VERSION, SHA256_*) que CI reemplaza automáticamente

### Paso 7: CI (release.yml)
- Paso "Generate package manager manifests" que:
  - Extrae SHA256 de checksums.txt
  - Genera `dist/livac.rb`, `dist/livac.json`, `dist/liva-lang.livac.yaml` con hashes reales
  - Sube como artifact `package-manifests`
- Release body actualizado con instrucciones de Homebrew y Scoop

## Qué queda pendiente (manual, fuera del repo)

1. **Crear repo `liva-lang/homebrew-tap`** — copiar `dist/livac.rb` como `Formula/livac.rb`
2. **Crear repo `liva-lang/scoop-bucket`** — copiar `dist/livac.json`
3. **PR a `microsoft/winget-pkgs`** — copiar `dist/liva-lang.livac.yaml`
4. **(Opcional) AUR**: Crear cuenta en aur.archlinux.org y subir PKGBUILD
5. **(Opcional) APT/RPM repo**: Configurar GitHub Pages como repo de paquetes

## Estructura final

```
livac/
├── skills/liva-lang/SKILL.md              ← Router IA
├── scripts/
│   ├── install-skills.sh                  ← Script compartido
│   ├── deb/postinst, prerm                ← Hooks Debian
│   └── rpm/post-install.sh, pre-uninstall.sh ← Hooks RPM
├── packaging/
│   ├── homebrew/livac.rb                  ← Template Homebrew
│   ├── scoop/livac.json                   ← Template Scoop
│   └── winget/liva-lang.livac.yaml        ← Template Winget
├── docs/language-reference/               ← 28 archivos, todos <650 líneas
└── .github/workflows/release.yml          ← CI genera manifests con hashes
```

## 9 agentes soportados

| # | Agente | Ruta | Symlink a |
|---|--------|------|-----------|
| 1 | GitHub Copilot | `~/.copilot/skills/liva-lang` | `/usr/share/livac/skills/liva-lang` |
| 2 | Claude Code | `~/.claude/skills/liva-lang` | ↑ |
| 3 | Codex | `~/.codex/skills/liva-lang` | ↑ |
| 4 | Cursor | `~/.cursor/skills/liva-lang` | ↑ |
| 5 | Windsurf | `~/.codeium/windsurf/skills/liva-lang` | ↑ |
| 6 | Gemini CLI | `~/.gemini/skills/liva-lang` | ↑ |
| 7 | Antigravity | `~/.gemini/antigravity/skills/liva-lang` | ↑ |
| 8 | Continue | `~/.continue/skills/liva-lang` | ↑ |
| 9 | OpenClaw | `~/.openclaw/skills/liva-lang` | ↑ |
