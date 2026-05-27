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
| 6 | Package manager manifests | ✅ Completado |
| 7 | Actualizar release.yml CI | ✅ Completado |
| 8 | Script PowerShell para Windows | ✅ Completado |
| 9 | Archives incluyen skills+docs | ✅ Completado |
| 10 | RPM incluye todos los docs | ✅ Completado |
| 11 | Homebrew instala docs completos | ✅ Completado |
| 12 | Scoop ejecuta install-skills.ps1 | ✅ Completado |
| 13 | CI auto-commit manifests a main | ✅ Completado |
| 14 | Release body con instrucciones manuales | ✅ Completado |

## Distribución — Cómo Funciona

### Estrategia: Todo en el Mismo Repo

**No se necesitan repos separados.** El repo principal `liva-lang/livac` funciona como:
- **Homebrew tap** → `Formula/livac.rb` (formulario con placeholders; CI actualiza con hashes reales)
- **Scoop bucket** → `bucket/livac.json` (manifest con placeholders; CI actualiza con hashes reales)
- **Winget** → `packaging/winget/liva-lang.livac.yaml` (template; requiere PR a `microsoft/winget-pkgs`)

### Flujo de Release

```
1. git tag v1.3.0 && git push --tags
2. CI build job:
   - Compila binarios en 4 targets (linux-x64, darwin-x64, darwin-arm64, windows-x64)
   - Empaqueta .tar.gz/.zip CON skills/ + docs/ + install-skills.sh/.ps1
   - Genera .deb y .rpm (incluyen skills + todos los docs + post-install hooks)
3. CI release job:
   - Genera checksums SHA256
   - Actualiza Formula/livac.rb y bucket/livac.json con versión + hashes reales
   - Commit automático de manifests a main
   - Crea GitHub Release con todos los artifacts
4. Usuarios instalan:
   - brew tap liva-lang/livac https://github.com/liva-lang/livac && brew install livac
   - scoop bucket add liva-lang https://github.com/liva-lang/livac && scoop install livac
   - sudo dpkg -i livac_amd64.deb
   - sudo rpm -i livac.x86_64.rpm
   - Descarga directa + bash install-skills.sh / .\install-skills.ps1
```

### Qué Instala Cada Formato

| Formato | Binario | Skills | Docs | Auto-link agentes |
|---------|---------|--------|------|-------------------|
| **.deb** | ✅ | ✅ | ✅ (42 archivos) | ✅ (postinst) |
| **.rpm** | ✅ | ✅ | ✅ (42 archivos) | ✅ (post-install.sh) |
| **Homebrew** | ✅ | ✅ | ✅ | ✅ (post_install) |
| **Scoop** | ✅ | ✅ | ✅ | ✅ (install-skills.ps1) |
| **.tar.gz** | ✅ | ✅ | ✅ | 🔧 Manual (install-skills.sh) |
| **.zip** | ✅ | ✅ | ✅ | 🔧 Manual (install-skills.ps1) |
| **Winget** | ✅ | ✅ | ✅ | 🔧 Manual |

### 9 Agentes Soportados

| # | Agente | Linux/macOS | Windows |
|---|--------|-------------|---------|
| 1 | GitHub Copilot | `~/.copilot/skills/liva-lang` → symlink | `%USERPROFILE%\.copilot\skills\liva-lang` → junction |
| 2 | Claude Code | `~/.claude/skills/liva-lang` | `%USERPROFILE%\.claude\skills\liva-lang` |
| 3 | Codex | `~/.codex/skills/liva-lang` | `%USERPROFILE%\.codex\skills\liva-lang` |
| 4 | Cursor | `~/.cursor/skills/liva-lang` | `%USERPROFILE%\.cursor\skills\liva-lang` |
| 5 | Windsurf | `~/.codeium/windsurf/skills/liva-lang` | `%USERPROFILE%\.codeium\windsurf\skills\liva-lang` |
| 6 | Gemini CLI | `~/.gemini/skills/liva-lang` | `%USERPROFILE%\.gemini\skills\liva-lang` |
| 7 | Antigravity | `~/.gemini/antigravity/skills/liva-lang` | `%USERPROFILE%\.gemini\antigravity\skills\liva-lang` |
| 8 | Continue | `~/.continue/skills/liva-lang` | `%USERPROFILE%\.continue\skills\liva-lang` |
| 9 | OpenClaw | `~/.openclaw/skills/liva-lang` | `%USERPROFILE%\.openclaw\skills\liva-lang` |

## Estructura de Archivos

```
livac/
├── Formula/livac.rb                       ← Homebrew tap (CI actualiza hashes)
├── bucket/livac.json                      ← Scoop bucket (CI actualiza hashes)
├── packaging/winget/liva-lang.livac.yaml  ← Template Winget (para PR externo)
├── skills/liva-lang/SKILL.md              ← Router IA con 30 refs a docs
├── scripts/
│   ├── install-skills.sh                  ← Linux/macOS: symlinks para 9 agentes
│   ├── install-skills.ps1                 ← Windows: junctions para 9 agentes
│   ├── deb/postinst, prerm                ← Hooks Debian
│   └── rpm/post-install.sh, pre-uninstall.sh ← Hooks RPM
├── docs/                                  ← 42 archivos MD referenciados por SKILL.md
│   ├── language-reference/                ← 30 archivos
│   ├── language-reference/stdlib/         ← 5 archivos
│   └── guides/                            ← 5 archivos
└── .github/workflows/release.yml          ← CI: build + package + auto-commit manifests
```

## Lo Único que Queda (Winget)

Winget no permite "taps" o "buckets" propios. Para publicar en Winget se necesita:
1. Hacer un PR a `microsoft/winget-pkgs` con el manifest generado (`dist/liva-lang.livac.yaml`)
2. El artifact `winget-manifest` se genera automáticamente en cada release
3. Esto es un proceso manual por release (o puede automatizarse con `wingetcreate`)
