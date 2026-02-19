# Homebrew formula for Liva compiler
# To be placed in liva-lang/homebrew-tap repo as Formula/livac.rb
#
# Users install with:
#   brew tap liva-lang/tap
#   brew install livac

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

  depends_on "rust" => :recommended

  def install
    bin.install "livac"

    # Install AI skills
    (share/"livac/skills/liva-lang").install buildpath/"skills/liva-lang/SKILL.md" if File.exist? "skills/liva-lang/SKILL.md"
  end

  def post_install
    # Install AI skill symlinks for coding agents
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

  test do
    assert_match "livac", shell_output("#{bin}/livac --version")
  end
end
