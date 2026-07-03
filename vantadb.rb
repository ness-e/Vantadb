class Vantadb < Formula
  desc "Embedded cognitive memory for AI agents - HNSW + BM25 hybrid vector database"
  homepage "https://vantadb.dev"
  version "0.1.5"
  license "Apache-2.0"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    else
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  elsif OS.linux?
    url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  end

  def install
    bin.install "vanta-cli"
    bin.install "vantadb-server"
  end

  test do
    system "#{bin}/vanta-cli", "--version"
  end
end
