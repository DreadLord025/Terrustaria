// W4-d35 RETIRE phase 3: the font-array VRAM census. The main atlas is not
// the only GPU texture array - the five retail DynamicSpriteFonts pack into
// 8192x8192 BC2 layers (font_atlas.rs, 1 byte/px). This decompresses the
// retail font XNBs (full multi-frame LZX, a port of crates/tr-content/src/
// lzx.rs reusing the pinned harness's verified primitives), walks the font
// structure (tr-content/font.rs read_font) for the page dims, and replicates
// FontAtlas::build's shelf pack to count the layers. The total-VRAM min-spec
// (atlas + font array) is then measured, not guessed.
// Run: node tools/_d35_font_vram.js. No cargo, no Rust touched.
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d35_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));

const fn = new Function('require', '__dirname', head + `
  // ---- full multi-frame LZX decompress (port of lzx.rs decompress_into +
  // decompress_block, reusing Input/BitBuffer/HuffTable/readLengths/EB/PB).
  function lzxDecompress(data, outputLength) {
    const input = new Input(data);
    const windowSize = 1 << 16;
    const window = Buffer.alloc(windowSize, 0xdc);
    let windowPos = 0, r0 = 1, r1 = 1, r2 = 1;
    const pre = new HuffTable(20, 6);
    const main = new HuffTable(MAINTREE_MAX, 12);
    const length = new HuffTable(NUM_SECONDARY_LENGTHS + 1, 12);
    const aligned = new HuffTable(8, 7);
    let blockType = 0, blockLength = 0, blockRemaining = 0;
    let readHeader = true, intelFileSize = 0;
    const out = [];
    const endPosition = data.length;
    let pos = 0;
    while (input.pos < endPosition) {
      input.pos = pos;
      const hi = input.readByte(), lo = input.readByte();
      let blockSize = (hi << 8) | lo, frameSize = 0x8000;
      if (hi === 0xff) {
        const hi2 = lo, lo2 = input.readByte();
        frameSize = (hi2 << 8) | lo2;
        const hi3 = input.readByte(), lo3 = input.readByte();
        blockSize = (hi3 << 8) | lo3;
        pos += 5;
      } else pos += 2;
      if (blockSize === 0 || frameSize === 0) break;
      // decompress_block
      const startpos = input.pos, endpos = startpos + blockSize;
      const buf = new BitBuffer(input);
      if (readHeader) {
        if (buf.read(1) === 1) { const h = buf.read(16), l = buf.read(16); intelFileSize = (h << 16) | l; }
        readHeader = false;
      }
      let togo = frameSize;
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
        if (buf.input.pos > startpos + blockSize) {
          if (buf.input.pos > endpos + 2 || buf.remaining < 16) throw new Error('LZX block overran its input');
        }
        while (blockRemaining > 0 && togo > 0) {
          let thisRun = Math.min(blockRemaining, togo);
          togo -= thisRun; blockRemaining -= thisRun;
          windowPos &= windowSize - 1;
          if (windowPos + thisRun > windowSize) throw new Error('window wrap straddle');
          if (blockType === 3) {
            buf.input.readInto(window.subarray(windowPos, windowPos + thisRun));
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
                  if (matchOffset !== 3) matchOffset = PB[matchOffset] - 2 + buf.read(EB[matchOffset]);
                  else matchOffset = 1;
                } else {
                  let extra = EB[matchOffset];
                  matchOffset = PB[matchOffset] - 2;
                  if (extra > 3) { extra -= 3; matchOffset += buf.read(extra) << 3; matchOffset += aligned.readHuffSym(buf); }
                  else if (extra === 3) matchOffset += aligned.readHuffSym(buf);
                  else if (extra > 0) matchOffset += buf.read(extra);
                  else matchOffset = 1;
                }
                r2 = r1; r1 = r0; r0 = matchOffset;
              } else if (matchOffset === 0) matchOffset = r0;
              else if (matchOffset === 1) { matchOffset = r1; r1 = r0; r0 = matchOffset; }
              else { matchOffset = r2; r2 = r0; r0 = matchOffset; }
              let rundest = windowPos;
              thisRun -= matchLength;
              let runsrc;
              if (windowPos >= matchOffset) runsrc = rundest - matchOffset;
              else {
                runsrc = rundest + (windowSize - matchOffset);
                let copyLength = matchOffset - windowPos;
                if (copyLength < matchLength) {
                  matchLength -= copyLength; windowPos += copyLength;
                  while (copyLength-- > 0) window[rundest++] = window[runsrc++];
                  runsrc = 0;
                }
              }
              windowPos += matchLength;
              while (matchLength-- > 0) window[rundest++] = window[runsrc++];
            }
          }
        }
      }
      if (togo !== 0) throw new Error('togo != 0');
      let startWindowPos = windowPos === 0 ? windowSize : windowPos;
      startWindowPos -= frameSize;
      if (startWindowPos >= 0) out.push(Buffer.from(window.subarray(startWindowPos, startWindowPos + frameSize)));
      else {
        out.push(Buffer.from(window.subarray(windowSize + startWindowPos, windowSize)));
        out.push(Buffer.from(window.subarray(0, windowPos)));
      }
      pos += blockSize;
    }
    const full = Buffer.concat(out);
    return full.subarray(0, outputLength);
  }

  // ---- XNB header -> decompressed body (xnb.rs read).
  function xnbBody(raw) {
    if (raw[0] !== 0x58 || raw[1] !== 0x4e || raw[2] !== 0x42) throw new Error('not XNB');
    const compressed = (raw[5] & 0x80) !== 0;
    const compressedSize = raw.readInt32LE(6);
    const decompressedSize = compressed ? raw.readInt32LE(10) : compressedSize;
    const bodyStart = compressed ? 14 : 10;
    const body = compressed ? lzxDecompress(raw.subarray(bodyStart, compressedSize), decompressedSize)
                            : raw.subarray(bodyStart, compressedSize);
    return body;
  }

  // ---- font structure walk (font.rs read_font / read_page) -> page dims.
  function fontPages(body) {
    let p = 0;
    const seven = () => { let v = 0, sh = 0, b; do { b = body[p++]; v |= (b & 0x7f) << sh; sh += 7; } while (b & 0x80); return v; };
    const i32 = () => { const v = body.readInt32LE(p); p += 4; return v; };
    // reader table
    const nreaders = seven();
    for (let i = 0; i < nreaders; i++) { const len = seven(); p += len + 4; }
    const shared = seven();
    const typeId = seven();
    // font payload
    p += 4; // spacing f32
    p += 4; // line spacing
    seven(); // default char
    const pageCount = i32();
    const pages = [];
    for (let pg = 0; pg < pageCount; pg++) {
      seven(); // texture reader index (2)
      const fmt = i32();
      const w = i32(), h = i32();
      const mips = i32();
      if (fmt !== 5) throw new Error('font page format ' + fmt);
      const mip0 = i32();
      if (mip0 !== w * h) throw new Error('mip0 ' + mip0 + ' != ' + w * h);
      p += mip0;
      for (let m = 1; m < mips; m++) { const sz = i32(); p += sz; }
      // glyph rect list
      seven(); const cnt = i32(); p += cnt * 16;
      // padding rect list
      seven(); const pcnt = i32(); if (pcnt !== cnt) throw new Error('padding count'); p += pcnt * 16;
      // char list (UTF-8, cnt chars)
      seven(); const ccnt = i32(); if (ccnt !== cnt) throw new Error('char count');
      for (let c = 0; c < ccnt; c++) { const b = body[p]; p += b < 0x80 ? 1 : (b < 0xe0 ? 2 : (b < 0xf0 ? 3 : 4)); }
      // kerning vector3 list
      seven(); const kcnt = i32(); if (kcnt !== cnt) throw new Error('kerning count'); p += kcnt * 12;
      pages.push([w, h]);
    }
    return pages;
  }

  // ---- FontAtlas::build shelf pack (block units, PAD 1 block) -> layer count.
  const BLOCKS = 8192 / 4;
  function packLayers(pages) {
    const order = pages.map((_, i) => i).sort((a, b) =>
      pages[b][1] - pages[a][1] || pages[b][0] - pages[a][0] || a - b);
    const layers = []; // [x_blocks, y_blocks, row_height_blocks]
    for (const pi of order) {
      const [w, h] = pages[pi];
      const bw = w / 4, bh = h / 4;
      let placed = false;
      for (const L of layers) {
        let [x, y, rh] = L;
        if (x + bw + 1 > BLOCKS) { x = 1; y += rh + 1; rh = 0; }
        if (y + bh + 1 > BLOCKS) continue;
        L[0] = x + bw + 1; L[1] = y; L[2] = Math.max(rh, bh);
        placed = true; break;
      }
      if (!placed) layers.push([1 + bw + 1, 1, bh]);
    }
    return layers.length;
  }

  const FONTS = path.join(__dirname, '..', 'assets', 'Content', 'Fonts');
  const NAMES = ['Mouse_Text', 'Item_Stack', 'Death_Text', 'Combat_Text', 'Combat_Crit'];
  const allPages = [];
  const perFont = {};
  for (const n of NAMES) {
    const raw = fs.readFileSync(path.join(FONTS, n + '.xnb'));
    const body = xnbBody(raw);
    const pages = fontPages(body);
    perFont[n] = pages.length;
    for (const pg of pages) allPages.push(pg);
    const dims = [...new Set(pages.map(p => p.join('x')))].sort();
    console.log(n + ': ' + pages.length + ' pages, dims ' + dims.join(', '));
  }
  const nlayers = packLayers(allPages);
  const fontMB = nlayers * 8192 * 8192 * 1 / 1e6; // BC2 = 1 byte/px
  console.log('\\ntotal font pages: ' + allPages.length);
  console.log('font array layers: ' + nlayers + ' x 8192x8192 BC2 = ' + fontMB.toFixed(1) + ' MB');
  fs.writeFileSync(path.join(__dirname, '_d35_font_vram.json'), JSON.stringify({
    perFontPages: perFont, totalPages: allPages.length,
    fontLayers: nlayers, fontMB: +fontMB.toFixed(1) }, null, 1));
  console.log('wrote _d35_font_vram.json');
`);
fn(require, __dirname);
