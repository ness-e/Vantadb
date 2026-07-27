import { ImageResponse } from "next/og";
import { readFile } from "node:fs/promises";
import path from "node:path";

export const runtime = "nodejs";
export const alt = "VantaDB — Embedded Rust Engine for Local-First Hybrid Retrieval";
export const size = { width: 1200, height: 630 };
export const contentType = "image/png";

export default async function OpengraphImage() {
  // Read the mascot image from public/assets
  let mascotData: Buffer | null = null;
  try {
    mascotData = await readFile(
      path.join(process.cwd(), "public", "assets", "mascota_gato.png")
    );
  } catch {
    // mascot not available, proceed without it
  }

  return new ImageResponse(
    (
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          flexDirection: "row",
          alignItems: "center",
          justifyContent: "space-between",
          backgroundColor: "#FBF9F5",
          backgroundImage:
            "linear-gradient(to right, rgba(0,0,0,0.04) 1px, transparent 1px), linear-gradient(to bottom, rgba(0,0,0,0.04) 1px, transparent 1px)",
          backgroundSize: "32px 32px",
          padding: "60px",
          fontFamily: "sans-serif",
        }}
      >
        {/* Left side — text */}
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            flex: 1,
            maxWidth: 680,
            gap: 20,
          }}
        >
          {/* Badge */}
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
            }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                border: "4px solid #000",
                backgroundColor: "#FF5500",
                padding: "8px 16px",
                fontSize: 20,
                fontWeight: 800,
                color: "#000",
                textTransform: "uppercase",
                letterSpacing: 2,
              }}
            >
              v0.1 · MVP
            </div>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 8,
                border: "3px solid #000",
                backgroundColor: "#000",
                padding: "8px 16px",
                fontSize: 16,
                fontWeight: 700,
                color: "#FBF9F5",
                textTransform: "uppercase",
                letterSpacing: 2,
              }}
            >
              <div
                style={{
                  width: 10,
                  height: 10,
                  backgroundColor: "#FF5500",
                  borderRadius: 0,
                }}
              />
              Embedded · Rust
            </div>
          </div>

          {/* Title */}
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              fontSize: 110,
              fontWeight: 900,
              lineHeight: 0.9,
              color: "#000",
              textTransform: "uppercase",
              letterSpacing: -3,
            }}
          >
            <div>Vanta</div>
            <div style={{ color: "#FF5500" }}>DB</div>
          </div>

          {/* Tagline */}
          <div
            style={{
              display: "flex",
              fontSize: 24,
              color: "#333",
              maxWidth: 580,
              lineHeight: 1.4,
            }}
          >
            Embedded Rust engine for durable local memory and hybrid vector
            retrieval. BM25 + HNSW via RRF. Zero network. 1.2ms latency.
          </div>

          {/* Stats row */}
          <div
            style={{
              display: "flex",
              gap: 0,
              border: "4px solid #000",
              marginTop: 10,
            }}
          >
            {[
              { v: "1.2ms", l: "Latency" },
              { v: "5,400", l: "Vec/s" },
              { v: "100%", l: "Recall@10" },
              { v: "0", l: "Network" },
            ].map((s, i) => (
              <div
                key={i}
                style={{
                  display: "flex",
                  flexDirection: "column",
                  backgroundColor: i === 0 ? "#FF5500" : "#FBF9F5",
                  padding: "12px 16px",
                  borderRight: i < 3 ? "2px solid #000" : "none",
                  minWidth: 110,
                }}
              >
                <div
                  style={{
                    fontSize: 32,
                    fontWeight: 900,
                    color: "#000",
                    lineHeight: 1,
                  }}
                >
                  {s.v}
                </div>
                <div
                  style={{
                    fontSize: 12,
                    fontWeight: 700,
                    color: "#000",
                    textTransform: "uppercase",
                    letterSpacing: 1,
                    marginTop: 4,
                  }}
                >
                  {s.l}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Right side — mascot */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            width: 380,
            height: 380,
            border: "6px solid #000",
            backgroundColor: "#FBF9F5",
            boxShadow: "12px 12px 0 0 #000",
          }}
        >
          {mascotData ? (
            <img
              src={`data:image/png;base64,${mascotData.toString("base64")}`}
              alt="VantaDB mascot"
              style={{
                width: "100%",
                height: "100%",
                objectFit: "cover",
              }}
            />
          ) : (
            <div
              style={{
                fontSize: 180,
                fontWeight: 900,
                color: "#FF5500",
              }}
            >
              🐱
            </div>
          )}
        </div>
      </div>
    ),
    { ...size }
  );
}
