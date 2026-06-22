class Vantadb < Formula
  desc "VantaDB: an embedded persistent memory and vector retrieval engine for local-first AI applications"
  homepage "https://vantadb.dev"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vanta-cli-macos-amd64"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"

      resource "server" do
        url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-server-macos-amd64"
        sha256 "0000000000000000000000000000000000000000000000000000000000000000"
      end

      resource "mcp" do
        url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-mcp-macos-amd64"
        sha256 "0000000000000000000000000000000000000000000000000000000000000000"
      end
    end
  end

  on_linux do
    if Hardware::CPU.intel? && Hardware::CPU.is_64_bit?
      url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vanta-cli-linux-amd64"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"

      resource "server" do
        url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-server-linux-amd64"
        sha256 "0000000000000000000000000000000000000000000000000000000000000000"
      end

      resource "mcp" do
        url "https://github.com/ness-e/Vantadb/releases/download/v#{version}/vantadb-mcp-linux-amd64"
        sha256 "0000000000000000000000000000000000000000000000000000000000000000"
      end
    end
  end

  def install
    bin.install "vanta-cli-macos-amd64" => "vanta-cli" if OS.mac? && Hardware::CPU.intel?
    bin.install "vanta-cli-linux-amd64" => "vanta-cli" if OS.linux?
    resource("server").stage { bin.install "vantadb-server-macos-amd64" => "vantadb-server" } if OS.mac? && Hardware::CPU.intel?
    resource("server").stage { bin.install "vantadb-server-linux-amd64" => "vantadb-server" } if OS.linux?
    resource("mcp").stage { bin.install "vantadb-mcp-macos-amd64" => "vantadb-mcp" } if OS.mac? && Hardware::CPU.intel?
    resource("mcp").stage { bin.install "vantadb-mcp-linux-amd64" => "vantadb-mcp" } if OS.linux?
  end

  test do
    system "#{bin}/vanta-cli", "--version"
  end
end
