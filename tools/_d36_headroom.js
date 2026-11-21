// W4-d36 sheets phase: atlas-headroom measurement for the 141-210 band.
// Reuses the pinned b4 harness (_d36_atlas_pinned.js, whose LZX decoder
// reads the REAL retail .xnb dims) by eval'ing its source up to main(),
// then runs skyline scenarios:
//   base            — HEAD's enumeration (main test world: gore landed, no font)
//   base + font     — the boot-font full set
//   + d36 batch N   — this task's NPC_<id> sheets, per batch 1..8
// Reports skyline heights vs the 8192 strict-GPU bound, verifies no overlap
// and determinism for each scenario (the harness's verify()).
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d36_atlas_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));

const fn = new Function('require', '__dirname', head + `
  const batches = {
    1: [141,142,148,149,150],
    2: [151,152,153,154,155],
    3: [156,158,159,160,162],
    4: [163,166,167,168,170],
    5: [171,172,173,176,178],
    6: [180,181,185,197,198],
    7: [199,201,202,203,204],
    8: [205,206,209],
  };
  const addNew = (base, names) => {
    const seen = new Set(base.map(s => s.name));
    const out = base.slice();
    for (const n of names) {
      if (seen.has(n)) continue;
      const d = sheetDims(n);
      if (!d) { console.log('  (missing file, skipped: ' + n + ')'); continue; }
      seen.add(n);
      out.push({ name: n, w: d.w, h: d.h });
    }
    out.dupDropped = base.dupDropped;
    return out;
  };
  const base = buildSheetList({});
  const baseFont = buildSheetList({ font: true });
  const scenarios = [];
  scenarios.push(['base (HEAD, gore landed)', base]);
  scenarios.push(['base + boot font', baseFont]);
  let acc = [];
  for (const [b, ids] of Object.entries(batches)) {
    acc = acc.concat(ids.map(id => 'NPC_' + id));
    scenarios.push(['base + d36 through batch ' + b + ' (' + acc.length + ' sheets)',
      addNew(base, acc)]);
    scenarios.push(['base + font + d36 through batch ' + b,
      addNew(baseFont, acc)]);
  }
  const results = [];
  for (const [label, sheets] of scenarios) {
    const sky = skylinePack(sheets, 'waste');
    const sky2 = skylinePack(sheets, 'waste');
    verify(sheets, sky);
    const det = JSON.stringify(sky.layout) === JSON.stringify(sky2.layout) && sky.height === sky2.height;
    if (!det) throw new Error('skyline is not deterministic: ' + label);
    const over = sky.height > 8192;
    console.log(label + ': ' + sky.height + ' px' + (over ? ' OVERFLOW +' + (sky.height - 8192) : '') + ' (verified, deterministic)');
    results.push({ label, height: sky.height, overflow: over });
  }
  return results;
`);
const results = fn(require, __dirname);
fs.writeFileSync(path.join(__dirname, '_d36_headroom.json'), JSON.stringify(results, null, 1));
