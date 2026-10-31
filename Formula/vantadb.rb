class Vantadb < Formula
  desc "VantaDB: an embedded persistent memory and vector retrieval engine for local-first AI applications"
  homepage "https://vantadb.dev"
  license "Apache-2.0"
  # SHA256s verified 2026-10-25 (FIND-342): fetched from the v0.9.0
  # release tarball sidecars (*.tar.gz.sha256) uploaded by
  # .github/workflows/release-binaries.yml.
  # On each new release, refresh the version above and the SHA256s from:
  #   for plat in x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu; do
  #     curl -sL "https://github.com/ness-e/Vantadb/releases/download/v$VERSION/vantadb-$plat.tar.gz.sha256"
  #   done
  version "0.9.0"

  livecheck do
    url :stable
    strategy :github_latest
  end

  on_macos do
    on_intel do
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-x86_64-apple-darwin.tar.gz"
      sha256 "bc496a12ff2acda3bd4f50a63a8b3f5e75a9a2e20ac9f24fcbeb62e9d9bdc4dd"
    end
    on_arm do
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-aarch64-apple-darwin.tar.gz"
      sha256 "aec5976a7eec7f86d63544d1e4d7bbf0fb4e1cc6bccd4db655412687349fb900"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "00a782d9281e9f9a2e0bbde69373ea30b1073ad2588c1489e66564a9fc99bec4"
    end
    on_arm do
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "7c1e5aee9d854e1d9bebe00be77a0aed27cbcb651f738163cb8a18ea41fae020"
    end
  end

  def install
    # Release tarballs ship exactly vanta-cli + vantadb-server
    # (.github/workflows/release-binaries.yml "Package binaries").
    # ponytail: if vantadb-mcp joins the release assets, add it here.
    bin.install "vanta-cli"
    bin.install "vantadb-server"

    # Shell completions generated from the installed binary (Homebrew standard:
    # no checked-in scripts, so nothing can go stale between releases).
    # `vanta-cli completions --shell <sh>` accepts clap's `--shell=<sh>` form.
    generate_completions_from_executable(bin/"vanta-cli", "completions",
                                         shell_parameter_format: :flag)
  end

  test do
    system "#{bin}/vanta-cli", "--version"
  end
end
