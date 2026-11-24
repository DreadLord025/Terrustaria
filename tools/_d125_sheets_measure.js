// W4-d125 remainder 1: NPC_636 / NPC_661 atlas-headroom measurement.
// Reuses the pinned b4/d35 harness (_d125_atlas_sim_pinned.js — a copy of
// d35's tip tools/_d35_atlas_sim_pinned.js, whose LZX decoder reads the REAL
// retail .xnb dims and whose L-8b fallback survives the wave-2 purge) by
// eval'ing its source up to main(). Scenarios: HEAD base (this branch adds
// no sheets yet), + boot font, + each sheet solo, + both, + the d35
// re-extracted pushed union (tools/_d125_pushed_sheets.json) for the
// overflow-ledger context (research/113 on task/W4-d35-npc-kinds-71-140).
// Every layout: verify() (no overlap incl. PAD, in-bounds) + double-run
// determinism. Run: node tools/_d125_sheets_measure.js  (repo root).
//
// NOTE (draw phase): the harness reads THIS branch's assets.rs, so after
// the sheets+draw phases landed the "base" scenarios already include the
// three packed sheets (NPC_636/NPC_661/Extra_188) — they re-measure the
// FINAL state (8141/8141, 51 px headroom). The pre-landing base numbers
// (8115/8119) and the per-sheet solo deltas are recorded in
// docs/research/309-empress-sheets-ledger.md from the first run.
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d125_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));
const pushed = JSON.parse(fs.readFileSync(path.join(__dirname, '_d125_pushed_sheets.json'), 'utf8'));

const fn = new Function('require', '__dirname', 'pushed', head + `
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
  const union = [...new Set(Object.values(pushed).flat())];
  const mine = ['NPC_636', 'NPC_661'];
  // The draw phase's halo ring (the only overlay strip that fits solo).
  const halo = ['Extra_188'];
  const scenarios = [
    ['base (d125 HEAD, no sheet adds)', base],
    ['base + boot font', baseFont],
    ['base + NPC_661 solo', addNew(base, ['NPC_661'])],
    ['base + font + NPC_661 solo', addNew(baseFont, ['NPC_661'])],
    ['base + NPC_636 solo', addNew(base, ['NPC_636'])],
    ['base + font + NPC_636 solo', addNew(baseFont, ['NPC_636'])],
    ['base + both (636 then 661)', addNew(base, mine)],
    ['base + font + both', addNew(baseFont, mine)],
    ['base + both + Extra_188 (the draw phase)', addNew(base, mine.concat(halo))],
    ['base + font + both + Extra_188', addNew(baseFont, mine.concat(halo))],
    ['base + pushed union (' + union.length + ') + both', addNew(addNew(base, union), mine)],
    ['base + font + pushed union + both', addNew(addNew(baseFont, union), mine)],
  ];
  const results = [];
  for (const [label, sheets] of scenarios) {
    const sky = skylinePack(sheets, 'waste');
    const sky2 = skylinePack(sheets, 'waste');
    verify(sheets, sky);
    const det = JSON.stringify(sky.layout) === JSON.stringify(sky2.layout) && sky.height === sky2.height;
    if (!det) throw new Error('non-deterministic: ' + label);
    const over = sky.height > 8192;
    console.log(label + ': ' + sheets.length + ' sheets -> skyline ' + sky.height +
      ' px ' + (over ? 'OVERFLOW (+' + (sky.height - 8192) + ')' : '(headroom ' + (8192 - sky.height) + ')'));
    results.push({ label, sheets: sheets.length, height: sky.height, overflow: over, deterministic: det });
  }
  for (const n of mine.concat(halo)) {
    const d = sheetDims(n);
    console.log(n + ': ' + d.w + 'x' + d.h + ' (padded ' + (d.w + 2) + 'x' + (d.h + 2) + ')');
  }
  // The five deferred overlay strips (the draw phase's census): each
  // overflows at its greedy turn on the both-packed base.
  let keep = [];
  for (const n of ['Extra_157', 'Extra_158', 'Extra_159', 'Extra_160', 'Extra_187', 'Extra_188']) {
    const trial = keep.concat([n]);
    const hm = skylinePack(addNew(base, mine.concat(trial)), 'waste').height;
    const hf = skylinePack(addNew(baseFont, mine.concat(trial)), 'waste').height;
    if (hm <= 8192 && hf <= 8192) { keep = trial; console.log('overlay PICK ' + n + ' -> main ' + hm + ' font ' + hf); }
    else console.log('overlay SKIP ' + n + ' (main ' + hm + ' font ' + hf + ')');
  }
  return results;
`);
const results = fn(require, __dirname, pushed);
fs.writeFileSync(path.join(__dirname, '_d125_sheets_measure.json'), JSON.stringify(results, null, 1));
