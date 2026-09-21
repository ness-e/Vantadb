// WEB-03 — regenera los placeholders de la mascota "shadow cat" (assets que
// nunca existieron en git; las 4 refs usaban fallbacks silenciosos).
// Pixel-art 64x64 brutalist: silueta negra + ojos de fuego #FF5500, fondo
// transparente. Encoder PNG puro Node (zlib) — cero dependencias.
//
// Uso: node scripts/generate-mascot.mjs   (escribe public/assets/*.png)
import { deflateSync } from "node:zlib";
import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const W = 64;
const H = 64;
const px = new Uint8Array(W * H * 4); // RGBA, transparente

const BLACK = [0, 0, 0, 255];
const ORANGE = [255, 85, 0, 255]; // #FF5500 — accent del sistema visual

function set(x, y, c) {
  if (x < 0 || x >= W || y < 0 || y >= H) return;
  const i = (y * W + x) * 4;
  px[i] = c[0];
  px[i + 1] = c[1];
  px[i + 2] = c[2];
  px[i + 3] = c[3];
}

// Orejas triangulares: apex arriba, base ancha a la altura de la cabeza.
function ear(apexX, baseHalf) {
  for (let y = 4; y <= 18; y++) {
    const t = (y - 4) / (18 - 4);
    const half = Math.max(1, Math.round(baseHalf * t));
    for (let x = apexX - half; x <= apexX + half; x++) set(x, y, BLACK);
  }
}

// Cabeza cuadrada brutalist (la "sombra").
for (let y = 18; y <= 56; y++) for (let x = 12; x <= 51; x++) set(x, y, BLACK);

ear(19, 7); // oreja izquierda (base 12..26)
ear(45, 7); // oreja derecha (base 38..52)

// Ojos cuadrados de fuego.
for (let y = 27; y <= 34; y++) {
  for (let x = 22; x <= 27; x++) set(x, y, ORANGE);
  for (let x = 37; x <= 42; x++) set(x, y, ORANGE);
}

// Nariz.
for (let y = 40; y <= 43; y++) for (let x = 30; x <= 33; x++) set(x, y, ORANGE);

// ---- encoder PNG (RGBA8) ----
function crc32(buf) {
  let c = 0xffffffff;
  for (const b of buf) {
    c ^= b;
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return c ^ 0xffffffff;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const td = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(td) >>> 0);
  return Buffer.concat([len, td, crc]);
}

function encodePng() {
  const sig = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(W, 0);
  ihdr.writeUInt32BE(H, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 6; // color type RGBA
  const raw = Buffer.alloc(H * (1 + W * 4));
  for (let y = 0; y < H; y++) {
    raw[y * (1 + W * 4)] = 0; // filter none
    px.subarray(y * W * 4, (y + 1) * W * 4).forEach(
      (v, i) => (raw[y * (1 + W * 4) + 1 + i] = v),
    );
  }
  const idat = deflateSync(raw, { level: 9 });
  return Buffer.concat([sig, chunk("IHDR", ihdr), chunk("IDAT", idat), chunk("IEND", Buffer.alloc(0))]);
}

const out = join(dirname(fileURLToPath(import.meta.url)), "..", "public", "assets");
mkdirSync(out, { recursive: true });
const png = encodePng();
for (const name of ["mascota_gato.png", "avatar_gato.png"]) {
  writeFileSync(join(out, name), png);
  console.log(`wrote public/assets/${name} (${png.length} bytes)`);
}
