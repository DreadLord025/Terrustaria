// S1-G-12 atlas-packer measurement harness (code-only mode: no compiler).
//
// Reads the REAL sheet dimensions out of the retail `.xnb` files (an LZX
// decoder ported line-for-line from crates/tr-content/src/lzx.rs — only the
// first frame is decoded, the texture header sits in its first bytes),
// mirrors the assets.rs enumeration (plus the two SKIP-blocked branches'
// additions: task/S1-E-4b-gore-draw's Gore_* set and task/S1-L-8b-mount-data's
// STATIC_SHEET_NAMES additions), and simulates the candidate packers:
//   - best-fit decreasing-height shelves (the landed P4 packer, main)
//   - first-fit decreasing-height shelves (the E-4b branch's variant)
//   - skyline bottom-left, waste tie-break (the chosen packer — mirrors
//     skyline_pack in crates/tr-render/src/atlas.rs exactly)
//   - skyline bottom-left, narrowest-span tie-break (a candidate)
//   - maxrects best-area-fit (comparison only)
//
// Run: node tools/atlas_sim.js   (from the repo root, with the assets junction)
// Prints measured heights per sheet-set scenario and verifies the chosen
// layout (no overlaps incl. padding, in-bounds, deterministic double run).

'use strict';
const fs = require('fs');
const path = require('path');

const IMAGES = path.join(__dirname, '..', 'assets', 'Content', 'Images');
const PAD = 2;
const MAX_WIDTH = 8192;

// ---------------------------------------------------------------------------
// LZX (XMemCompress) decoder — port of crates/tr-content/src/lzx.rs.
// ---------------------------------------------------------------------------
const NUM_CHARS = 256, NUM_SECONDARY_LENGTHS = 249;
const MAINTREE_MAX = NUM_CHARS + 50 * 8;

function extraBits() {
  const eb = new Array(52).fill(0);
  let j = 0;
  for (let i = 0; i <= 50; i += 2) {
    eb[i] = j; eb[i + 1] = j;
    if (i !== 0 && j < 17) j++;
  }
  return eb;
}
function positionBase() {
  const eb = extraBits(), pb = new Array(51).fill(0);
  let j = 0;
  for (let i = 0; i <= 50; i++) { pb[i] = j; j += 1 << eb[i]; }
  return pb;
}
const EB = extraBits(), PB = positionBase();

class Input {
  constructor(data) { this.data = data; this.pos = 0; }
  readByte() { const b = this.pos < this.data.length ? this.data[this.pos] : 0; this.pos = Math.min(this.pos + 1, this.data.length); return b; }
  readInto(dst) {
    const n = Math.min(dst.length, Math.max(0, this.data.length - this.pos));
    for (let i = 0; i < n; i++) dst[i] = this.data[this.pos + i];
    for (let i = n; i < dst.length; i++) dst[i] = 0;
    this.pos += n;
  }
}

class BitBuffer {
  constructor(input) { this.input = input; this.remaining = 0; this.bits = 0; }
  reset() { this.remaining = 0; this.bits = 0; }
  ensure(n) {
    while (this.remaining < n) {
      const lo = this.input.readByte(), hi = this.input.readByte();
      // Rust wrapping_shl: JS << masks the shift to 5 bits, same semantics.
      this.bits = (this.bits | (((hi << 8) | lo) << (16 - this.remaining))) | 0;
      this.remaining += 16;
    }
  }
  peek(n) { return (this.bits >>> (32 - n)) >>> 0; }
  remove(n) { this.bits = (this.bits << n) | 0; this.remaining -= n; }
  read(n) { if (n <= 0) return 0; this.ensure(n); const r = this.peek(n); this.remove(n); return r; }
}

class HuffTable {
  constructor(maxSymbols, tableBits) {
    this.maxSymbols = maxSymbols; this.tableBits = tableBits;
    this.table = new Int16Array((1 << tableBits) + (maxSymbols << 1));
    this.length = new Uint8Array(maxSymbols + 64);
  }
  makeDecodeTable() {
    let bitNum = 1, pos = 0;
    let tableMask = 1 << this.tableBits;
    let bitMask = tableMask >>> 1;
    let nextSymbol = bitMask;
    const max = this.maxSymbols;
    while (bitNum <= this.tableBits) {
      for (let sym = 0; sym < max; sym++) {
        if (this.length[sym] === bitNum) {
          let leaf = pos;
          pos += bitMask;
          if (pos > tableMask) return;
          for (let fill = bitMask; fill > 0; fill--) { this.table[leaf] = sym; leaf++; }
        }
      }
      bitMask >>>= 1; bitNum++;
    }
    if (pos !== tableMask) {
      for (let sym = pos; sym < tableMask; sym++) this.table[sym] = 0;
      pos <<= 16; tableMask <<= 16; bitMask = 1 << 15;
      while (bitNum <= 16) {
        for (let sym = 0; sym < max; sym++) {
          if (this.length[sym] === bitNum) {
            let leaf = pos >>> 16;
            for (let fill = 0; fill < bitNum - this.tableBits; fill++) {
              if (this.table[leaf] === 0) {
                const base = nextSymbol << 1;
                this.table[base] = 0; this.table[base + 1] = 0;
                this.table[leaf] = nextSymbol; nextSymbol++;
              }
              leaf = this.table[leaf] << 1;
              if (((pos >>> (15 - fill)) & 1) === 1) leaf++;
            }
            this.table[leaf] = sym;
            pos += bitMask;
            if (pos > tableMask) return;
          }
        }
        bitMask >>>= 1; bitNum++;
      }
    }
  }
  readHuffSym(buf) {
    buf.ensure(16);
    let i = this.table[buf.peek(this.tableBits)];
    if (i >= this.maxSymbols) {
      let j = 1 << (32 - this.tableBits);
      do {
        j >>>= 1;
        i = (i << 1) | ((buf.bits & j) !== 0 ? 1 : 0);
        if (j === 0) throw new Error('ReadHuffSym: decode error');
        i = this.table[i];
      } while (i >= this.maxSymbols);
    }
    buf.remove(this.length[i]);
    return i;
  }
}

function readLengths(pre, lens, first, last, buf) {
  for (let x = 0; x < 20; x++) pre.length[x] = buf.read(4);
  pre.makeDecodeTable();
  let x = first;
  while (x < last) {
    const z = pre.readHuffSym(buf);
    if (z === 17) { let y = buf.read(4) + 4; while (y-- > 0) lens[x++] = 0; }
    else if (z === 18) { let y = buf.read(5) + 20; while (y-- > 0) lens[x++] = 0; }
    else if (z === 19) {
      let y = buf.read(1) + 4;
      let zz = lens[x] - pre.readHuffSym(buf);
      if (zz < 0) zz += 17;
      while (y-- > 0) lens[x++] = zz;
    } else {
      let zz = lens[x] - z;
      if (zz < 0) zz += 17;
      lens[x++] = zz;
    }
  }
}

// Decodes the FIRST frame only (0x8000 bytes unless the 0xFF header says
// otherwise) — enough for the XNB reader table + texture header.
function lzxFirstFrame(data) {
  const input = new Input(data);
  const window = Buffer.alloc(1 << 16, 0xdc);
  let windowPos = 0, r0 = 1, r1 = 1, r2 = 1;
  const pre = new HuffTable(20, 6);
  const main = new HuffTable(MAINTREE_MAX, 12);
  const length = new HuffTable(NUM_SECONDARY_LENGTHS + 1, 12);
  const aligned = new HuffTable(8, 7);
  let blockType = 0, blockLength = 0, blockRemaining = 0;

  const hi0 = input.readByte(), lo0 = input.readByte();
  let blockSize = (hi0 << 8) | lo0, frameSize = 0x8000;
  if (hi0 === 0xff) {
    const hi = lo0, lo = input.readByte();
    frameSize = (hi << 8) | lo;
    const hi2 = input.readByte(), lo2 = input.readByte();
    blockSize = (hi2 << 8) | lo2;
  }
  if (blockSize === 0 || frameSize === 0) return Buffer.alloc(0);

  const buf = new BitBuffer(input);
  if (buf.read(1) === 1) { buf.read(16); buf.read(16); } // Intel header (never set by game assets)
  let togo = frameSize;
  const startpos = input.pos, endpos = startpos + blockSize;

  while (togo > 0) {
    if (blockRemaining === 0) {
      if (blockType === 3) {
        if ((blockLength & 1) === 1) buf.input.readByte();
        buf.reset();
      }
      const t = buf.read(3);
      if (t > 3) throw new Error('invalid block type ' + t);
      blockType = t;
      const a = buf.read(16), b = buf.read(8);
      blockLength = (a << 8) | b;
      blockRemaining = blockLength;
      if (blockType === 2) {
        for (let i = 0; i < 8; i++) aligned.length[i] = buf.read(3);
        aligned.makeDecodeTable();
      }
      if (blockType === 2 || blockType === 1) {
        readLengths(pre, main.length, 0, NUM_CHARS, buf);
        readLengths(pre, main.length, NUM_CHARS, NUM_CHARS + (16 << 4), buf);
        main.makeDecodeTable();
        readLengths(pre, length.length, 0, NUM_SECONDARY_LENGTHS, buf);
        length.makeDecodeTable();
      } else if (blockType === 3) {
        buf.ensure(16);
        if (buf.remaining > 16) buf.input.pos -= 2;
        const raw32 = () => {
          const x0 = buf.input.readByte(), x1 = buf.input.readByte(),
            x2 = buf.input.readByte(), x3 = buf.input.readByte();
          return (x0 | (x1 << 8) | (x2 << 16) | (x3 << 24)) | 0;
        };
        r0 = raw32(); r1 = raw32(); r2 = raw32();
      } else throw new Error('unknown block type');
    }
    if (buf.input.pos > endpos + 2 || (buf.input.pos > startpos + blockSize && buf.remaining < 16)) {
      throw new Error('LZX block overran its input');
    }
    while (blockRemaining > 0 && togo > 0) {
      let thisRun = Math.min(blockRemaining, togo);
      togo -= thisRun; blockRemaining -= thisRun;
      windowPos &= (1 << 16) - 1;
      if (blockType === 3) {
        const dst = window.subarray(windowPos, windowPos + thisRun);
        buf.input.readInto(dst);
        windowPos += thisRun;
        continue;
      }
      const isAligned = blockType === 2;
      while (thisRun > 0) {
        let mainElement = main.readHuffSym(buf);
        if (mainElement < NUM_CHARS) {
          window[windowPos] = mainElement; windowPos++; thisRun--;
        } else {
          mainElement -= NUM_CHARS;
          let matchLength = mainElement & 7;
          if (matchLength === 7) matchLength += length.readHuffSym(buf);
          matchLength += 2;
          let matchOffset = mainElement >> 3;
          if (matchOffset > 2) {
            if (!isAligned) {
              if (matchOffset !== 3) {
                matchOffset = PB[matchOffset] - 2 + buf.read(EB[matchOffset]);
              } else matchOffset = 1;
            } else {
              let extra = EB[matchOffset];
              matchOffset = PB[matchOffset] - 2;
              if (extra > 3) {
                extra -= 3;
                matchOffset += buf.read(extra) << 3;
                matchOffset += aligned.readHuffSym(buf);
              } else if (extra === 3) {
                matchOffset += aligned.readHuffSym(buf);
              } else if (extra > 0) {
                matchOffset += buf.read(extra);
              } else matchOffset = 1;
            }
            r2 = r1; r1 = r0; r0 = matchOffset;
          } else if (matchOffset === 0) {
            matchOffset = r0;
          } else if (matchOffset === 1) {
            matchOffset = r1; r1 = r0; r0 = matchOffset;
          } else {
            matchOffset = r2; r2 = r0; r0 = matchOffset;
          }
          let rundest = windowPos;
          thisRun -= matchLength;
          let runsrc;
          if (windowPos >= matchOffset) {
            runsrc = rundest - matchOffset;
          } else {
            runsrc = rundest + ((1 << 16) - matchOffset);
            let copyLength = matchOffset - windowPos;
            if (copyLength < matchLength) {
              matchLength -= copyLength;
              windowPos += copyLength;
              while (copyLength-- > 0) { window[rundest++] = window[runsrc++]; }
              runsrc = 0;
            }
          }
          windowPos += matchLength;
          while (matchLength-- > 0) { window[rundest++] = window[runsrc++]; }
        }
      }
    }
  }
  // First frame: windowPos == frameSize (no wrap yet), copy it out.
  return Buffer.from(window.subarray(0, frameSize));
}

// ---------------------------------------------------------------------------
// XNB header → texture dimensions (Texture2DReader FormatFirst).
// ---------------------------------------------------------------------------
function read7(r, state) {
  let result = 0, shift = 0;
  for (;;) {
    const b = r[state.pos++];
    result |= (b & 0x7f) << shift;
    if ((b & 0x80) === 0) return result;
    shift += 7;
    if (shift > 28) throw new Error('bad 7-bit int');
  }
}

const dimCache = new Map();
function sheetDims(name) {
  if (dimCache.has(name)) return dimCache.get(name);
  const file = path.join(IMAGES, name + '.xnb');
  let dims = null;
  try {
    const raw = fs.readFileSync(file);
    if (raw[0] !== 0x58 || raw[1] !== 0x4e || raw[2] !== 0x42) throw new Error('not XNB');
    const compressed = (raw[5] & 0x80) !== 0;
    const compressedSize = raw.readInt32LE(6);
    let body;
    if (compressed) {
      body = lzxFirstFrame(raw.subarray(14, compressedSize));
    } else {
      body = raw.subarray(10, compressedSize);
    }
    const state = { pos: 0 };
    const readerCount = read7(body, state);
    for (let i = 0; i < readerCount; i++) {
      const len = read7(body, state);
      state.pos += len + 4; // name bytes + reader version i32
    }
    read7(body, state); // shared resources
    read7(body, state); // type id
    // payload: format i32, width i32, height i32, mipCount i32
    const width = body.readInt32LE(state.pos + 4);
    const height = body.readInt32LE(state.pos + 8);
    if (width < 1 || width > 4096 || height < 1 || height > 4096) throw new Error('bad dims');
    dims = { w: width, h: height };
  } catch (e) {
    dims = null; // missing/undecodable file: the loader skips it too
  }
  dimCache.set(name, dims);
  return dims;
}

// ---------------------------------------------------------------------------
// Census — mirrors assets.rs::load_world_sheets (empty test world) +
// load_static_sheets, plus the two blocked branches' additions.
// ---------------------------------------------------------------------------
const assetsSrc = fs.readFileSync(path.join(__dirname, '..', 'crates', 'tr-app', 'src', 'assets.rs'), 'utf8');

function constStringList(src, constName) {
  const start = src.indexOf('const ' + constName);
  if (start < 0) throw new Error('const not found: ' + constName);
  const open = src.indexOf('= &[', start) + 2;
  let depth = 0, end = -1;
  for (let i = open + 1; i < src.length; i++) {
    if (src[i] === '[') depth++;
    else if (src[i] === ']') { depth--; if (depth === 0) { end = i; break; } }
  }
  const block = src.slice(open, end);
  return [...block.matchAll(/"([^"]+)"/g)].map(m => m[1]);
}

const STATIC_NAMES = constStringList(assetsSrc, 'STATIC_SHEET_NAMES');
const GLOW_NAMES = constStringList(assetsSrc, 'GLOW_SHEET_NAMES');

// The L-8b branch's STATIC_SHEET_NAMES (renamed minecart rows + 25 additions).
// The branch was deleted in the wave-2 purge; when the ref is gone, fall back
// to the pinned 25-name additions list — the SAME list the assets.rs witness
// `the_full_gore_and_mount_sheet_set_fits_the_strict_gpu_bound` simulates.
// The renamed-away rows had no file on main, so the loader skipped them and
// the rename is pixel-neutral (L8B_REMOVED stays empty in the fallback).
const { execSync } = require('child_process');
const L8B_PINNED_ADDITIONS = [
  'Mount_Glow_DrillRing', 'Mount_Glow_DrillSeat', 'Mount_Glow_DrillDiode',
  'Glow_283', 'Glow_284', 'Glow_285', 'Glow_370',
  'Mount_MinecartMech', 'Mount_MinecartMechGlow', 'Mount_MinecartWood',
  'Mount_MinecartMineCarp', 'Mount_MinecartDesert', 'Mount_Turtle',
  'Extra_96', 'Extra_97', 'Extra_149', 'Extra_167', 'Extra_168',
  'Extra_253', 'Extra_254', 'Extra_271', 'Extra_272', 'Extra_273',
  'Extra_301', 'Extra_302',
];
let L8B_STATIC, L8B_ADDITIONS, L8B_REMOVED;
try {
  const l8bSrc = execSync('git show origin/task/S1-L-8b-mount-data:crates/tr-app/src/assets.rs',
    { cwd: path.join(__dirname, '..'), maxBuffer: 64 * 1024 * 1024, stdio: ['ignore', 'pipe', 'ignore'] }).toString();
  L8B_STATIC = constStringList(l8bSrc, 'STATIC_SHEET_NAMES');
  L8B_ADDITIONS = L8B_STATIC.filter(n => !STATIC_NAMES.includes(n));
  L8B_REMOVED = STATIC_NAMES.filter(n => !L8B_STATIC.includes(n));
} catch (e) {
  console.log('note: origin/task/S1-L-8b-mount-data is gone (wave-2 purge) — using the pinned 25-name additions list');
  L8B_ADDITIONS = L8B_PINNED_ADDITIONS.filter(n => !STATIC_NAMES.includes(n));
  L8B_REMOVED = [];
  L8B_STATIC = STATIC_NAMES.concat(L8B_ADDITIONS);
}

// Projectile kinds: proj_data's match arms over the module's u16 consts.
const projDir = path.join(__dirname, '..', 'crates', 'tr-app', 'src', 'projectile');
const consts = new Map();
for (const f of fs.readdirSync(projDir)) {
  if (!f.endsWith('.rs')) continue;
  const s = fs.readFileSync(path.join(projDir, f), 'utf8');
  for (const m of s.matchAll(/pub const (\w+): u16 = (\d+)/g)) consts.set(m[1], +m[2]);
}
const dataSrc = fs.readFileSync(path.join(projDir, 'data.rs'), 'utf8');
const PROJ_KINDS = (() => {
  const i = dataSrc.indexOf('pub fn proj_data');
  const lines = dataSrc.slice(i).split('\n');
  const end = lines.findIndex(l => /^}/.test(l));
  const body = lines.slice(0, end).join('\n');
  const mstart = body.indexOf('match kind');
  let depth = 0, mend = -1;
  for (let k = mstart + 5; k < body.length; k++) {
    if (body[k] === '{') depth++;
    else if (body[k] === '}') { depth--; if (depth === 0) { mend = k; break; } }
  }
  const matchBody = body.slice(mstart, mend);
  const kinds = new Set();
  const addTok = (t) => {
    if (t === '_') return; // the wildcard arm
    const range = t.match(/^(\d+)\.\.=(\d+)$/);
    if (range) {
      for (let id = +range[1]; id <= +range[2]; id++) if (id <= 1200) kinds.add(id);
      return;
    }
    const id = /^\d+$/.test(t) ? +t : consts.get(t);
    if (id === undefined) throw new Error('unknown projectile const ' + t);
    if (id <= 1200) kinds.add(id); // spawnable_kinds probes 0..=1200
  };
  // arms may span lines, carry ranges (133..=144) and an `if` guard
  for (const m of matchBody.matchAll(/(?:^|\n)\s+((?:[A-Z0-9_]+|\d+(?:\.\.=\d+)?)(?:\s*\|\s*(?:[A-Z0-9_]+|\d+(?:\.\.=\d+)?))*)\s*(?:if\s+([^\n]+?))?\s*=>/g)) {
    for (const tok of m[1].split('|')) addTok(tok.trim());
    if (m[2]) {
      // the only guard in the table: `kind != 779 && kind != 783`
      for (const g of m[2].matchAll(/kind != (\d+)/g)) kinds.delete(+g[1]);
    }
  }
  kinds.add(734); // the void lens sheet, packed outside spawnable_kinds
  return kinds;
})();

function enumIds(prefix, exclude = []) {
  const ids = [];
  for (const f of fs.readdirSync(IMAGES)) {
    const m = f.match(new RegExp('^' + prefix + '(\\d+)\\.xnb$'));
    if (m && !exclude.includes(+m[1])) ids.push(+m[1]);
  }
  ids.sort((a, b) => a - b);
  return [...new Set(ids)];
}

// World half of the content_lookups test: empty tile/wall sets, the five
// default (style 0) surface backgrounds, the Tiles_0 fallback.
const WORLD_BG_DEFAULT = [12, 13, 14, 15, 16, 17, 29, 30, 31, 43, 44, 45, 46, 47, 48];

function buildSheetList({ gores = true, mountAdditions = false, font = false, staticNames = STATIC_NAMES, dedup = true } = {}) {
  const sheets = []; // {name, w, h} in loader order
  const seen = new Set(); // name dedup: a duplicate decode packs twice but the
  // sheets HashMap keeps one rect — the later copy's pixels are dead weight.
  // Dropping the later copies is UV-neutral for every lookup.
  let dupDropped = 0;
  const add = (name) => {
    const d = sheetDims(name);
    if (!d) return;
    if (dedup && seen.has(name)) { dupDropped++; return; }
    seen.add(name);
    sheets.push({ name, w: d.w, h: d.h });
  };
  for (const id of WORLD_BG_DEFAULT) add('Background_' + id);
  add('Tiles_0');
  for (const name of staticNames) add(name);
  // W4-c4: candidate sheets under measurement BEFORE they are committed to
  // STATIC_SHEET_NAMES — `EXTRA_SHEETS=NPC_57,NPC_157,... node tools/atlas_sim.js`
  // appends them exactly where the static list packs, so the reported height is
  // the real post-add skyline height (the headroom gate).
  for (const name of (process.env.EXTRA_SHEETS || '').split(',').map(s => s.trim()).filter(Boolean)) add(name);
  for (const kind of [...PROJ_KINDS].sort((a, b) => a - b)) add('Projectile_' + kind);
  for (const id of enumIds('Item_')) add('Item_' + id);
  for (const id of enumIds('Buff_')) add('Buff_' + id);
  // E-4b landed on main: load_static_sheets now ALWAYS enumerates the gore
  // set (assets.rs:1353, enumerate_gore_sheet_ids — 910/911 excluded, they
  // ride STATIC_SHEET_NAMES). The `gores` flag is kept for the historical
  // pre-E-4b scenarios but defaults on.
  if (gores) for (const id of enumIds('Gore_', [910, 911])) add('Gore_' + id);
  for (let style = 0; style < 32; style++)
    for (const prefix of ['Tree_Tops_', 'Tree_Branches_']) add(prefix + style);
  for (let i = 0; i < 42; i++) add('Cloud_' + i);
  for (let i = 0; i < 15; i++) add('Misc/water_' + i);
  for (const n of ['Spine', 'Shoulder', 'Torso', 'Head', 'Explosion']) add('Misc/MoonExplosion/' + n);
  for (let i = 0; i < 15; i++) for (const p of ['Liquid_', 'LiquidSlope_']) add(p + i);
  for (let i = 0; i < 28; i++) add('Waterfall_' + i);
  for (let i = 0; i < 18; i++) add('Flame_' + i);
  for (const n of ['WiresNew', 'Actuator', 'TileCracks']) add(n);
  for (const n of GLOW_NAMES) add(n);
  if (mountAdditions) for (const n of L8B_ADDITIONS) add(n);
  if (font) sheets.push({ name: '__font', w: 768, h: 8 });
  sheets.push({ name: '__white', w: 4, h: 4 }); // Atlas::build's synthetic sheet
  sheets.dupDropped = dupDropped;
  return sheets;
}

// ---------------------------------------------------------------------------
// Packers. All pack the PAD-inflated rects (w+PAD, h+PAD) into a bin of width
// MAX_WIDTH-PAD and report height = used + PAD, matching build_ref's geometry
// (PAD border on every side, >= PAD between sheets).
// ---------------------------------------------------------------------------
function sorted(sheets) {
  // Stable sort, height desc then width desc — build_ref's order.
  return sheets.map((s, i) => ({ ...s, i }))
    .sort((a, b) => (b.h - a.h) || (b.w - a.w) || (a.i - b.i));
}

function shelfPack(sheets, bestFit, maxWidth = MAX_WIDTH) {
  const order = sorted(sheets);
  const shelves = []; // {x (from PAD), y, h}
  const layout = [];
  for (const s of order) {
    const need = s.w + PAD;
    let best = -1;
    // exact replica of the Rust scan: s.x + need <= max_width with x starting at PAD
    for (let i = 0; i < shelves.length; i++) {
      const sx = shelves[i].x + PAD;
      if (sx + need <= maxWidth) {
        if (bestFit) {
          if (best < 0 || shelves[i].x > shelves[best].x) best = i;
        } else { best = i; break; }
      }
    }
    if (best >= 0) {
      layout[s.i] = { x: shelves[best].x + PAD, y: shelves[best].y + PAD };
      shelves[best].x += need;
    } else {
      const y = shelves.length ? shelves[shelves.length - 1].y + shelves[shelves.length - 1].h + PAD : 0;
      layout[s.i] = { x: PAD, y: y + PAD };
      shelves.push({ x: need, y, h: s.h });
    }
  }
  const last = shelves[shelves.length - 1];
  const height = last ? last.y + last.h + PAD + PAD : PAD;
  return { height, layout };
}

// Skyline bottom-left. Nodes: [x, y, width] sorted by x, covering [0, W).
// Candidate: every node's left edge; y = max skyline height over the span.
// tieBreak 'waste': score (y, wasted area under the rect, x) — the chosen
// packer, mirrors skyline_pack in tr-render/src/atlas.rs.
// tieBreak 'span':  score (y, span width of the nodes touched, x).
function skylinePack(sheets, tieBreak = 'waste', maxWidth = MAX_WIDTH) {
  const order = sorted(sheets);
  const W = maxWidth - PAD;
  let nodes = [[0, 0, W]];
  const layout = [];
  for (const s of order) {
    const w = s.w + PAD, h = s.h + PAD;
    if (w > W) throw new Error('sheet wider than the bin: ' + s.name);
    let best = null; // {x, y, tie, idx}
    for (let i = 0; i < nodes.length; i++) {
      const x = nodes[i][0];
      if (x + w > W) continue;
      let y = 0, span = 0, j = i;
      while (span < w && j < nodes.length) {
        y = Math.max(y, nodes[j][1]);
        span += nodes[j][2];
        j++;
      }
      if (span < w) continue;
      let tie;
      if (tieBreak === 'waste') {
        tie = 0;
        let acc = 0;
        for (let k = i; acc < w; k++) {
          const segW = Math.min(nodes[k][2], w - acc);
          tie += segW * (y - nodes[k][1]);
          acc += segW;
        }
      } else {
        tie = span;
      }
      if (!best || y < best.y || (y === best.y && (tie < best.tie || (tie === best.tie && x < best.x)))) {
        best = { x, y, tie, idx: i };
      }
    }
    if (!best) throw new Error('no skyline placement for ' + s.name);
    layout[s.i] = { x: best.x + PAD, y: best.y + PAD };
    // Insert the new node [x, y+h, w], trimming covered nodes.
    const newNode = [best.x, best.y + h, w];
    const next = [];
    for (let k = 0; k < nodes.length; k++) {
      const [nx, ny, nw] = nodes[k];
      const nend = nx + nw;
      if (nend <= best.x || nx >= best.x + w) { next.push(nodes[k]); continue; }
      if (nx < best.x) next.push([nx, ny, best.x - nx]);
      if (nend > best.x + w) next.push([best.x + w, ny, nend - (best.x + w)]);
    }
    let at = next.findIndex(n => n[0] > best.x);
    if (at < 0) at = next.length;
    next.splice(at, 0, newNode);
    // merge adjacent equal-height nodes
    const merged = [];
    for (const n of next) {
      const last = merged[merged.length - 1];
      if (last && last[1] === n[1] && last[0] + last[2] === n[0]) last[2] += n[2];
      else merged.push([...n]);
    }
    nodes = merged;
  }
  const height = Math.max(...nodes.map(n => n[1])) + PAD;
  return { height, layout, nodes };
}

// Maxrects best-area-fit (comparison only): score = (free-rect area − sheet
// area, short-side leftover, y, x). Free rects split on every placement,
// contained rects pruned; fixed total-order tie-breaks.
function maxrectsPack(sheets) {
  const order = sorted(sheets);
  const W = MAX_WIDTH - PAD, H = 1 << 20;
  let free = [{ x: 0, y: 0, w: W, h: H }];
  const layout = [];
  let used = 0;
  for (const s of order) {
    const w = s.w + PAD, h = s.h + PAD;
    let best = null, bestScore = null;
    for (const f of free) {
      if (f.w < w || f.h < h) continue;
      const areaFit = f.w * f.h - w * h;
      const short = Math.min(f.w - w, f.h - h);
      const score = [areaFit, short, f.y, f.x];
      const better = !bestScore || score.some((v, k) => {
        for (let j = 0; j < k; j++) if (score[j] !== bestScore[j]) return false;
        return v < bestScore[k];
      });
      if (better) { best = f; bestScore = score; }
    }
    if (!best) throw new Error('no maxrects placement for ' + s.name);
    const p = { x: best.x, y: best.y, w, h };
    layout[s.i] = { x: p.x + PAD, y: p.y + PAD };
    used = Math.max(used, p.y + h);
    const next = [];
    for (const f of free) {
      if (f.x >= p.x + p.w || f.x + f.w <= p.x || f.y >= p.y + p.h || f.y + f.h <= p.y) {
        next.push(f); continue;
      }
      if (p.x > f.x) next.push({ x: f.x, y: f.y, w: p.x - f.x, h: f.h });
      if (p.x + p.w < f.x + f.w) next.push({ x: p.x + p.w, y: f.y, w: f.x + f.w - (p.x + p.w), h: f.h });
      if (p.y > f.y) next.push({ x: f.x, y: f.y, w: f.w, h: p.y - f.y });
      if (p.y + p.h < f.y + f.h) next.push({ x: f.x, y: p.y + p.h, w: f.w, h: f.y + f.h - (p.y + p.h) });
    }
    free = next.filter((a, i) => !next.some((b, j) => i !== j &&
      b.x <= a.x && b.y <= a.y && b.x + b.w >= a.x + a.w && b.y + b.h >= a.y + a.h &&
      (j < i || b.w > a.w || b.h > a.h || b.x < a.x || b.y < a.y)));
  }
  return { height: used + PAD, layout };
}

// ---------------------------------------------------------------------------
// Verification of a layout: in-bounds, no overlaps (with the PAD reservation).
// ---------------------------------------------------------------------------
function verify(sheets, result) {
  const { height, layout } = result;
  for (let i = 0; i < sheets.length; i++) {
    const s = sheets[i], p = layout[i];
    if (!p) throw new Error('unplaced sheet ' + s.name);
    if (p.x < PAD || p.y < PAD) throw new Error('border violated: ' + s.name);
    if (p.x + s.w > MAX_WIDTH - PAD) throw new Error('width overflow: ' + s.name);
    if (p.y + s.h > height - PAD) throw new Error('height overflow: ' + s.name);
  }
  // pairwise overlap on the PAD-inflated rects (grid-bucketed for speed)
  const cell = 256, grid = new Map();
  for (let i = 0; i < sheets.length; i++) {
    const s = sheets[i], p = layout[i];
    const r = { x: p.x, y: p.y, w: s.w + PAD, h: s.h + PAD, i };
    const x0 = Math.floor(r.x / cell), x1 = Math.floor((r.x + r.w) / cell);
    const y0 = Math.floor(r.y / cell), y1 = Math.floor((r.y + r.h) / cell);
    for (let gx = x0; gx <= x1; gx++) for (let gy = y0; gy <= y1; gy++) {
      const key = gx + ',' + gy;
      const bucket = grid.get(key) || [];
      for (const o of bucket) {
        if (r.x < o.x + o.w && o.x < r.x + r.w && r.y < o.y + o.h && o.y < r.y + r.h) {
          throw new Error('overlap: ' + sheets[r.i].name + ' vs ' + sheets[o.i].name);
        }
      }
      bucket.push(r);
      grid.set(key, bucket);
    }
  }
}

// ---------------------------------------------------------------------------
function main() {
  const scenarios = {
    // E-4b landed: main's enumeration now includes the gore set by default.
    'main (test world, E-4b gore landed)': buildSheetList({}),
    'main + boot font': buildSheetList({ font: true }),
    'main pre-E-4b (no gore, historical)': buildSheetList({ gores: false }),
    'main + mount (L-8b additions)': buildSheetList({ mountAdditions: true }),
    'main + mount + font (L-8b)': buildSheetList({ mountAdditions: true, font: true }),
    'full: L-8b static list + gore + font': buildSheetList({ gores: true, font: true, staticNames: L8B_STATIC }),
    'main WITHOUT dedup (fidelity check vs the 8121 baseline)': buildSheetList({ gores: false, dedup: false }),
  };
  console.log('L-8b additions (' + L8B_ADDITIONS.length + '):', L8B_ADDITIONS.join(', '));
  console.log('L-8b removed (renamed, no file on main):', L8B_REMOVED.join(', '));
  const goreCount = enumIds('Gore_', [910, 911]).length;
  console.log('gore sheets enumerated:', goreCount, '| projectile kinds:', PROJ_KINDS.size);
  const full = scenarios['full: L-8b static list + gore + font'];
  console.log('duplicate decodes dropped in the full set:', full.dupDropped);

  for (const [label, sheets] of Object.entries(scenarios)) {
    const t0 = Date.now();
    const bf = shelfPack(sheets, true);
    const ff = shelfPack(sheets, false);
    const skyW = skylinePack(sheets, 'waste');
    const skyS = skylinePack(sheets, 'span');
    const sky = skylinePack(sheets, 'waste');
    const sky2 = skylinePack(sheets, 'waste');
    const mr = maxrectsPack(sheets);
    verify(sheets, sky);
    const det = JSON.stringify(sky.layout) === JSON.stringify(sky2.layout) && sky.height === sky2.height;
    if (!det) throw new Error('skyline is not deterministic');
    const area = sheets.reduce((a, s) => a + (s.w + PAD) * (s.h + PAD), 0);
    console.log(`\n${label}: ${sheets.length} sheets (${sheets.dupDropped} duplicate decodes dropped), padded area ${(area / 1e6).toFixed(1)} Mpx`);
    console.log(`  best-fit shelves (main):   ${bf.height} px ${bf.height > 8192 ? 'OVERFLOW' : ''}`);
    console.log(`  first-fit shelves (E-4b):  ${ff.height} px ${ff.height > 8192 ? 'OVERFLOW' : ''}`);
    console.log(`  skyline (waste tie-break): ${skyW.height} px ${skyW.height > 8192 ? 'OVERFLOW' : ''} (deterministic: ${det}, verified: no overlap/in-bounds)`);
    console.log(`  skyline (span tie-break):  ${skyS.height} px ${skyS.height > 8192 ? 'OVERFLOW' : ''}`);
    console.log(`  maxrects BAF:              ${mr.height} px ${mr.height > 8192 ? 'OVERFLOW' : ''}`);
    console.log(`  skyline occupancy: ${(100 * area / (MAX_WIDTH * sky.height)).toFixed(1)}%  (${Date.now() - t0} ms)`);
  }
}

main();
