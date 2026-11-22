# Formula do Homebrew para o Colmeia.
# Uso em desenvolvimento:
#   brew install --build-from-source ./colmeia.rb
class Colmeia < Formula
  desc "Gestor Git de frotas, TUI-first, determinístico em hardware mínimo"
  homepage "https://colmeia.local"
  license "MIT OR Apache-2.0"
  version "0.1.0"

  # Em releases reais, substituir pela URL do tarball publicado:
  # url "https://github.com/colmeia/colmeia/archive/refs/tags/v0.1.0.tar.gz"
  # sha256 "..."
  head "https://github.com/colmeia/colmeia.git", branch: "main"

  depends_on "rust" => :build
  depends_on "git"

  def install
    system "cargo", "install", *std_cargo_args
    bash_completion.install "completions/colmeia.bash" if File.exist?("completions/colmeia.bash")
  end

  test do
    assert_match "colmeia", shell_output("#{bin}/colmeia --version")
  end
end
