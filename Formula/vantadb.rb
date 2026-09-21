class Vantadb < Formula
  desc "VantaDB: an embedded persistent memory and vector retrieval engine for local-first AI applications"
  homepage "https://vantadb.dev"
  license "Apache-2.0"
  # SHA256s verified 2026-09-03 (MKT-18h): computed locally from the v0.5.0
  # release tarballs and cross-checked against the *.tar.gz.sha256 sidecars
  # uploaded by .github/workflows/release-binaries-63.yml.
  # On each new release, refresh the version above and the SHA256s from:
  #   for plat in x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu; do
  #     curl -sL "https://github.com/ness-e/Vantadb/releases/download/v$VERSION/vantadb-$plat.tar.gz.sha256"
  #   done
  version "0.5.0"

  livecheck do
    url :stable
    strategy :github_latest
  end

  on_macos do
    on_intel do
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-x86_64-apple-darwin.tar.gz"
      sha256 "a892ef6eccdc4b670684579ecb93981e562cd5d950f95f8c6e378a90ff7d52b8"
    end
    on_arm do
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-aarch64-apple-darwin.tar.gz"
      sha256 "77547c2991c322d50321223f4ef6b792f69043821b6aeb7f9b2d146acb6ae1a4"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "bb221673e49fa10e7d61bc4f06130697a4645f6a7f14a54fee8a6fa21132e6ae"
    end
    on_arm do
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "f2948bdce182a3854f20364f64b047165e2695704731716dbc9bbf27bd441027"
    end
  end

  def install
    # Release tarballs ship exactly vanta-cli + vantadb-server
    # (.github/workflows/release-binaries-63.yml "Package binaries").
    # ponytail: if vantadb-mcp joins the release assets, add it here.
    bin.install "vanta-cli"
    bin.install "vantadb-server"
  end

  test do
    system "#{bin}/vanta-cli", "--version"
  end
end
