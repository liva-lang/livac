# Homebrew formula for Liva compiler
# This file lives in the main repo — users tap directly:
#   brew tap liva-lang/livac https://github.com/liva-lang/livac
#   brew install livac
#
# VERSION and SHA256_* placeholders are replaced by CI (release.yml)
# after each release, then committed back to main.

class Livac < Formula
  desc "Liva programming language compiler — compiles to Rust"
  homepage "https://github.com/liva-lang/livac"
  version "VERSION"  # Updated by CI
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/liva-lang/livac/releases/download/vVERSION/livac-darwin-arm64.tar.gz"
      sha256 "SHA256_DARWIN_ARM64"
    else
      url "https://github.com/liva-lang/livac/releases/download/vVERSION/livac-darwin-x64.tar.gz"
      sha256 "SHA256_DARWIN_X64"
    end
  end

  on_linux do
    url "https://github.com/liva-lang/livac/releases/download/vVERSION/livac-linux-x64.tar.gz"
    sha256 "SHA256_LINUX_X64"
  end

  depends_on "rust"

  def install
    bin.install "livac"

    # Install AI skills and documentation (included in release archive)
    (share/"livac/skills/liva-lang").install Dir["skills/liva-lang/*"] if Dir.exist? "skills/liva-lang"

    if Dir.exist? "docs"
      (share/"livac/docs").install Dir["docs/README.md", "docs/QUICK_REFERENCE.md", "docs/ERROR_CODES.md"]
      (share/"livac/docs/language-reference").install Dir["docs/language-reference/*.md"] if Dir.exist? "docs/language-reference"
      (share/"livac/docs/language-reference/stdlib").install Dir["docs/language-reference/stdlib/*.md"] if Dir.exist? "docs/language-reference/stdlib"
      (share/"livac/docs/guides").install Dir["docs/guides/*.md"] if Dir.exist? "docs/guides"
    end
  end

  def post_install
    # Create AI skill symlinks for coding agents
    agents = {
      ".copilot/skills"                => "GitHub Copilot",
      ".claude/skills"                 => "Claude Code",
      ".codex/skills"                  => "Codex",
      ".cursor/skills"                 => "Cursor",
      ".codeium/windsurf/skills"       => "Windsurf",
      ".gemini/skills"                 => "Gemini CLI",
      ".gemini/antigravity/skills"     => "Antigravity",
      ".continue/skills"               => "Continue",
      ".openclaw/skills"               => "OpenClaw",
    }

    skill_source = share/"livac/skills/liva-lang"
    return unless skill_source.exist?

    agents.each do |rel_path, name|
      target_dir = Pathname.new(Dir.home)/rel_path
      link_path = target_dir/"liva-lang"

      target_dir.mkpath
      link_path.unlink if link_path.symlink?
      next if link_path.directory? # Don't overwrite real directories

      link_path.make_symlink(skill_source)
      ohai "Linked AI skill for #{name}: #{link_path}"
    end
  end

  def caveats
    <<~EOS
      AI skills for 9 coding agents (Copilot, Claude, Cursor...) have been
      installed automatically. If you use a new agent later, run:
        livac --install-skills
      or re-run:
        brew postinstall livac
    EOS
  end

  test do
    assert_match "livac", shell_output("#{bin}/livac --version")
  end
end
