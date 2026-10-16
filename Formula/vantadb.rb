class Vantadb < Formula
  desc "VantaDB: an embedded persistent memory and vector retrieval engine for local-first AI applications"
  homepage "https://vantadb.dev"
  license "Apache-2.0"
  # SHA256s verified 2026-10-16 (FIND-342): fetched from the v0.8.0
  # release tarball sidecars (*.tar.gz.sha256) uploaded by
  # .github/workflows/release-binaries.yml.
  # On each new release, refresh the version above and the SHA256s from:
  #   for plat in x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu; do
  #     curl -sL "https://github.com/ness-e/Vantadb/releases/download/v$VERSION/vantadb-$plat.tar.gz.sha256"
  #   done
  version "0.8.0"

  livecheck do
    url :stable
    strategy :github_latest
  end

  on_macos do
    on_intel do
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-x86_64-apple-darwin.tar.gz"
      sha256 "a5ffd5da6d6269ee676f2333e4d581e61c82ca18e0d18c81af508f4026a021cc"
    end
    on_arm do
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-aarch64-apple-darwin.tar.gz"
      sha256 "37dca6e6a207f3c7a8d2cb7cc3b4ad56f86821f5699a47c4568bf90eee37633a"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "f067b62642ff932b797130a257c5e75575373a25f01725fc30a1a02fdeee869c"
    end
    on_arm do
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "52d5d67cad602943fe48f05b01d21885e2fd112459006919427d0db46cbc5cdc"
    end
  end

  def install
    # Release tarballs ship exactly vanta-cli + vantadb-server
    # (.github/workflows/release-binaries.yml "Package binaries").
    # ponytail: if vantadb-mcp joins the release assets, add it here.
    bin.install "vanta-cli"
    bin.install "vantadb-server"
  end

  test do
    system "#{bin}/vanta-cli", "--version"
  end
end
