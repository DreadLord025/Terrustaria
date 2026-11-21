// W4-d40 sheets phase: CUMULATIVE atlas-headroom measurement (the d35
// _d35_headroom.js shape, re-derived against the CURRENT pushed claims).
// Reuses the pinned b4 harness (_d40_atlas_sim_pinned.js, whose LZX decoder
// reads the REAL retail .xnb dims) by eval'ing its source up to main(), then
// runs skyline scenarios:
//   base            — HEAD's enumeration (main test world: gore landed, no font)
//   base + font     — the boot-font full set
//   + pushed        — every sheet-name row the in-flight W4 branches ADD
//                     (tools/_d40_pushed_sheets.json, deduped union — 403
//                     names incl. d35's sheets-phase 21 and d38's 35)
//   + pushed + d40  — plus this task's 56 NPC_<id> sheets, cumulative batches
//   SOLO            — d40 alone on HEAD, cumulative + id-order greedy
// Reports skyline heights vs the 8192 strict-GPU bound, verifies no overlap
// and determinism for each scenario (the harness's verify()).
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d40_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));
const pushed = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_pushed_sheets.json'), 'utf8'));
const dims = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_dims.json'), 'utf8'));

const fn = new Function('require', '__dirname', 'pushed', 'dims', head + `
  const union = [...new Set(Object.values(pushed).flat())];
  const ids = Object.keys(dims).map(Number).sort((a,b)=>a-b);
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
  const H = (sheets) => skylinePack(sheets, 'waste').height;
  const scenarios = [];
  scenarios.push(['base (HEAD, gore landed)', base]);
  scenarios.push(['base + boot font', baseFont]);
  scenarios.push(['base + pushed union (' + union.length + ')', addNew(base, union)]);
  scenarios.push(['base + font + pushed union', addNew(baseFont, union)]);
  // d40 ALONE on HEAD, cumulative in id order (the dispatch's batch order).
  let accSolo = [];
  for (const id of ids) {
    accSolo.push('NPC_' + id);
    scenarios.push(['SOLO base + d40 through NPC_' + id + ' (' + accSolo.length + ')',
      addNew(base, accSolo)]);
  }
  // CUMULATIVE: pushed union + d40 in id order.
  let acc = [];
  const pushedBase = addNew(base, union);
  const pushedFontBase = addNew(baseFont, union);
  for (const id of ids) {
    acc.push('NPC_' + id);
    scenarios.push(['+ pushed + d40 through NPC_' + id + ' (' + acc.length + ')',
      addNew(pushedBase, acc)]);
  }
  scenarios.push(['+ font + pushed + ALL d40 (56)', addNew(pushedFontBase, ids.map(i => 'NPC_' + i))]);
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
  // id-order greedy SOLO on HEAD (the d35 method): keep iff both <= 8192.
  let picked = [];
  for (const id of ids) {
    const trial = picked.concat([id]);
    const names = trial.map(i => 'NPC_' + i);
    const hm = H(addNew(base, names)), hf = H(addNew(baseFont, names));
    if (hm <= 8192 && hf <= 8192) { picked = trial; console.log('SOLO idorder pick NPC_' + id + ' -> main ' + hm + ' font ' + hf); }
    else console.log('SOLO idorder SKIP NPC_' + id + ' (main ' + hm + ' font ' + hf + ')');
  }
  console.log('SOLO IDORDER RESULT:', JSON.stringify(picked));
  // per-sheet dims table (the witness record)
  for (const id of ids) {
    const d = dims[id];
    console.log('NPC_' + id + ': ' + d.w + 'x' + d.h);
  }
  return { results, picked };
`);
const { results, picked } = fn(require, __dirname, pushed, dims);
fs.writeFileSync(path.join(__dirname, '_d40_headroom.json'), JSON.stringify(results, null, 1));
fs.writeFileSync(path.join(__dirname, '_d40_picked_solo.json'), JSON.stringify(picked));
