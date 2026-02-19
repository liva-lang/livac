# install-skills.ps1 — Install Liva AI skills for all supported coding agents (Windows)
# Creates directory junctions from each agent's skills directory to the Liva skills folder.
#
# Usage:
#   install-skills.ps1 [-Uninstall] [-SkillSource <path>]
#
# When called by Scoop, $dir points to the app directory.
# When called manually, -SkillSource should point to the directory containing skills/liva-lang/.

param(
    [switch]$Uninstall,
    [string]$SkillSource = ""
)

$ErrorActionPreference = "Stop"

# Determine skill source directory
if ($SkillSource -eq "") {
    # Try common locations
    $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
    $candidates = @(
        (Join-Path $scriptDir ".." "skills" "liva-lang"),        # scripts/../skills/liva-lang (repo layout)
        (Join-Path $scriptDir ".." ".." "skills" "liva-lang"),   # Scoop: scripts/../../skills/liva-lang
        (Join-Path $env:ProgramFiles "livac" "skills" "liva-lang"),
        (Join-Path $env:LocalAppData "livac" "skills" "liva-lang")
    )
    foreach ($c in $candidates) {
        if (Test-Path $c) {
            $SkillSource = (Resolve-Path $c).Path
            break
        }
    }
    if ($SkillSource -eq "") {
        Write-Warning "Could not find Liva skills directory. Use -SkillSource to specify."
        exit 0
    }
}

$SkillName = "liva-lang"

# Agent directories (relative to $env:USERPROFILE)
$AgentDirs = @(
    @{ Path = ".copilot\skills";                  Name = "GitHub Copilot" },
    @{ Path = ".claude\skills";                   Name = "Claude Code" },
    @{ Path = ".codex\skills";                    Name = "Codex" },
    @{ Path = ".cursor\skills";                   Name = "Cursor" },
    @{ Path = ".codeium\windsurf\skills";         Name = "Windsurf" },
    @{ Path = ".gemini\skills";                   Name = "Gemini CLI" },
    @{ Path = ".gemini\antigravity\skills";       Name = "Antigravity" },
    @{ Path = ".continue\skills";                 Name = "Continue" },
    @{ Path = ".openclaw\skills";                 Name = "OpenClaw" }
)

$action = if ($Uninstall) { "Removing" } else { "Installing" }
Write-Host "$action Liva AI skills for coding agents..."

foreach ($agent in $AgentDirs) {
    $targetDir = Join-Path $env:USERPROFILE $agent.Path
    $linkPath = Join-Path $targetDir $SkillName

    if ($Uninstall) {
        if (Test-Path $linkPath) {
            # Remove junction or directory
            cmd /c rmdir "$linkPath" 2>$null
            if (Test-Path $linkPath) {
                Remove-Item $linkPath -Force -Recurse
            }
            Write-Host "  Removed: $linkPath"
        }
    } else {
        # Create parent directory
        if (-not (Test-Path $targetDir)) {
            New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
        }

        # Remove existing junction/symlink
        if (Test-Path $linkPath) {
            $item = Get-Item $linkPath -Force
            if ($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) {
                cmd /c rmdir "$linkPath" 2>$null
            } else {
                # Real directory — skip (user may have custom content)
                Write-Host "  Skip (real dir): $linkPath ($($agent.Name))"
                continue
            }
        }

        # Create junction (doesn't require admin privileges)
        cmd /c mklink /J "$linkPath" "$SkillSource" >$null 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  Linked: $linkPath -> $SkillSource ($($agent.Name))"
        } else {
            # Fallback: copy directory if junction fails
            Copy-Item -Path $SkillSource -Destination $linkPath -Recurse -Force
            Write-Host "  Copied: $linkPath ($($agent.Name)) [junction failed, used copy]"
        }
    }
}

Write-Host "Done."
