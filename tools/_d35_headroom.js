// W4-d35 sheets phase: CUMULATIVE atlas-headroom measurement.
// Reuses the pinned b4 harness (_d35_atlas_sim_pinned.js, whose LZX decoder
// reads the REAL retail .xnb dims) by eval'ing its source up to main(), then
// runs skyline scenarios:
//   base            — HEAD's enumeration (main test world: gore landed, no font)
//   base + font     — the boot-font full set (the c4 "full" scenario)
//   + pushed        — every sheet-name row the in-flight W4 branches ADD
//                     (tools/_d35_pushed_sheets.json, deduped union)
//   + pushed + d35  — plus this task's 33 NPC_<id> sheets, per batch A..G
// Reports skyline heights vs the 8192 strict-GPU bound, verifies no overlap
// and determinism for each scenario (the harness's verify()).
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d35_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));
const pushed = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_pushed_sheets.json'), 'utf8'));
const dims = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_dims.json'), 'utf8'));

const fn = new Function('require', '__dirname', 'pushed', 'dims', head + `
  const union = [...new Set(Object.values(pushed).flat())];
  const d35ids = Object.keys(dims).map(Number).sort((a,b)=>a-b);
  const batches = {
    A: [73,74,75,77,78], B: [79,80,82,83,84], C: [85,86,93,94,95],
    D: [96,97,98,99,100], E: [101,102,103,104,109],
    F: [110,112,116,120,121], G: [122,137,140],
  };
  const extra = (names) => (sheets) => {
    for (const n of names) {
      const d = sheetDims(n);
      if (d) sheets.push({ name: n, w: d.w, h: d.h });
    }
    return sheets;
  };
  // dedup the extras against the base list by name (the loader's HashMap
  // keeps one rect per name; a re-pack is UV-neutral dead weight).
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
  const pushedNames = union;
  const d35Names = d35ids.map(id => 'NPC_' + id);
  const scenarios = [];
  scenarios.push(['base (HEAD, gore landed)', base]);
  scenarios.push(['base + boot font', baseFont]);
  scenarios.push(['base + pushed union (' + pushedNames.length + ')', addNew(base, pushedNames)]);
  scenarios.push(['base + font + pushed union', addNew(baseFont, pushedNames)]);
  // d35 ALONE on HEAD (no pushed union): does the 33-kind wave fit the
  // CURRENT main's headroom, batch by batch?
  let accSolo = [];
  for (const [b, ids] of Object.entries(batches)) {
    accSolo = accSolo.concat(ids.map(id => 'NPC_' + id));
    scenarios.push(['SOLO base + d35 through batch ' + b + ' (' + accSolo.length + ')',
      addNew(base, accSolo)]);
    scenarios.push(['SOLO base + font + d35 through batch ' + b,
      addNew(baseFont, accSolo)]);
  }
  let acc = [];
  for (const [b, ids] of Object.entries(batches)) {
    acc = acc.concat(ids.map(id => 'NPC_' + id));
    scenarios.push(['+ pushed + d35 through batch ' + b + ' (' + acc.length + ' sheets)',
      addNew(addNew(base, pushedNames), acc)]);
    scenarios.push(['+ font + pushed + d35 through batch ' + b,
      addNew(addNew(baseFont, pushedNames), acc)]);
  }
  const results = [];
  for (const [label, sheets] of scenarios) {
    const sky = skylinePack(sheets, 'waste');
    const sky2 = skylinePack(sheets, 'waste');
    verify(sheets, sky);
    const det = JSON.stringify(sky.layout) === JSON.stringify(sky2.layout);
    if (!det) throw new Error('non-deterministic: ' + label);
    const over = sky.height > 8192;
    console.log(label + ': ' + sheets.length + ' sheets -> skyline ' + sky.height +
      ' px ' + (over ? 'OVERFLOW (+' + (sky.height - 8192) + ')' : '(headroom ' + (8192 - sky.height) + ')'));
    results.push({ label, sheets: sheets.length, height: sky.height, overflow: over });
  }
  // per-sheet dims table for the 33 (the witness record)
  for (const id of d35ids) {
    const d = dims[id];
    console.log('NPC_' + id + ': ' + d.w + 'x' + d.h);
  }
  return results;
`);
const results = fn(require, __dirname, pushed, dims);
fs.writeFileSync(path.join(__dirname, '_d35_headroom.json'), JSON.stringify(results, null, 1));
