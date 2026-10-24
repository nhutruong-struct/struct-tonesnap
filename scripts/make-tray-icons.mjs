// Draws the icons of the tray menu (a gear, a download arrow and a power symbol) into src-tauri/icons/tray/.
// Run with `node scripts/make-tray-icons.mjs` after changing a shape or a size. The menu library
// (muda) scales every bitmap to a fixed size: 16 × 16 pixels on Windows and 18 points tall on macOS,
// which is 36 pixels on a Retina screen. Each platform gets a bitmap of exactly that size, since a
// bitmap scaled to a size that is not a whole multiple of its own comes out blurred. Windows cannot
// tint a menu icon for a dark or light menu, so the strokes use one neutral grey that reads on both.

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { crc32, deflateSync } from "node:zlib";

/**
 * Folder ⇄ bitmap size and stroke width, in pixels; `tray.rs` picks the folder for its platform.
 * At 16 pixels the 2-unit stroke of the grid would be 1.33 pixels and come out faint, so it is thicker.
 */
const SIZES = {
  windows: { size: 16, stroke: 1.6 },
  macos: { size: 36, stroke: 3 },
};
const SUPERSAMPLING = 8;
const GREY = [0x85, 0x85, 0x85];
/** Shapes are drawn on a 24 × 24 grid; strokes have round ends. */
const GRID = 24;

const out = join(dirname(fileURLToPath(import.meta.url)), "..", "src-tauri", "icons", "tray");

/** Distance from a point to a segment. */
function toSegment([px, py], [ax, ay], [bx, by]) {
  const [dx, dy] = [bx - ax, by - ay];
  const t = Math.max(0, Math.min(1, ((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)));
  return Math.hypot(px - (ax + t * dx), py - (ay + t * dy));
}

/** Distance from a point to a closed polygon's outline. */
function toPolygon(point, vertices) {
  return Math.min(...vertices.map((v, i) => toSegment(point, v, vertices[(i + 1) % vertices.length])));
}

const polar = (cx, cy, r, degrees) => {
  const a = (degrees * Math.PI) / 180;
  return [cx + r * Math.sin(a), cy - r * Math.cos(a)];
};

/** Eight trapezoid teeth around a root circle. */
function gearOutline() {
  const [rootRadius, toothRadius, teeth] = [7.4, 10, 8];
  const vertices = [];
  for (let k = 0; k < teeth; k++) {
    const middle = (k * 360) / teeth;
    for (const [radius, offset] of [
      [rootRadius, -13],
      [toothRadius, -9],
      [toothRadius, 9],
      [rootRadius, 13],
    ]) {
      vertices.push(polar(12, 12, radius, middle + offset));
    }
  }
  return vertices;
}

const GEAR = gearOutline();

/** Distance to the gear: its outline plus the circle in the middle. */
const gear = (point) => Math.min(toPolygon(point, GEAR), Math.abs(Math.hypot(point[0] - 12, point[1] - 12) - 3.2));

/** Distance to the power symbol: a bar over an arc that is open at the top. */
function power(point) {
  const [cx, cy, radius, from, to] = [12, 13, 8, 40, 320];
  const [dx, dy] = [point[0] - cx, point[1] - cy];
  const angle = ((Math.atan2(dx, -dy) * 180) / Math.PI + 360) % 360;
  const arc =
    angle >= from && angle <= to
      ? Math.abs(Math.hypot(dx, dy) - radius)
      : Math.min(
          Math.hypot(point[0] - polar(cx, cy, radius, from)[0], point[1] - polar(cx, cy, radius, from)[1]),
          Math.hypot(point[0] - polar(cx, cy, radius, to)[0], point[1] - polar(cx, cy, radius, to)[1]),
        );
  return Math.min(arc, toSegment(point, [12, 2.5], [12, 11.5]));
}

/** Distance to the download arrow: a shaft ending in a head, over a base line. */
const download = (point) =>
  Math.min(
    toSegment(point, [12, 3], [12, 15.5]),
    toSegment(point, [6.5, 10], [12, 15.5]),
    toSegment(point, [17.5, 10], [12, 15.5]),
    toSegment(point, [4.5, 20.5], [19.5, 20.5]),
  );

/** RGBA pixels: the share of each pixel's samples within the stroke becomes its opacity. */
function render(distance, size, stroke) {
  const halfStroke = (stroke / 2) * (GRID / size);
  const pixels = Buffer.alloc(size * size * 4);
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      let covered = 0;
      for (let j = 0; j < SUPERSAMPLING; j++) {
        for (let i = 0; i < SUPERSAMPLING; i++) {
          const point = [
            ((x + (i + 0.5) / SUPERSAMPLING) * GRID) / size,
            ((y + (j + 0.5) / SUPERSAMPLING) * GRID) / size,
          ];
          if (distance(point) <= halfStroke) covered++;
        }
      }
      const at = (y * size + x) * 4;
      pixels.set(GREY, at);
      pixels[at + 3] = Math.round((255 * covered) / (SUPERSAMPLING * SUPERSAMPLING));
    }
  }
  return pixels;
}

function chunk(type, data) {
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length);
  const checksum = Buffer.alloc(4);
  checksum.writeUInt32BE(crc32(body));
  return Buffer.concat([length, body, checksum]);
}

function png(pixels, size) {
  const header = Buffer.alloc(13);
  header.writeUInt32BE(size, 0);
  header.writeUInt32BE(size, 4);
  header.set([8, 6, 0, 0, 0], 8); // 8 bits, RGBA, no interlacing
  const rows = Buffer.alloc(size * (1 + size * 4));
  for (let y = 0; y < size; y++) {
    pixels.copy(rows, y * (1 + size * 4) + 1, y * size * 4, (y + 1) * size * 4);
  }
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", header),
    chunk("IDAT", deflateSync(rows)),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

for (const [folder, { size, stroke }] of Object.entries(SIZES)) {
  const dir = join(out, folder);
  mkdirSync(dir, { recursive: true });
  for (const [name, distance] of Object.entries({ settings: gear, update: download, quit: power })) {
    writeFileSync(join(dir, `${name}.png`), png(render(distance, size, stroke), size));
  }
}
