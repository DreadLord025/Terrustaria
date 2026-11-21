// W4-d42 SHEETS phase — the NPCID 578-630 roster's real-dims census + the
// pack/headroom measurement + the d35 union-forecast inventory delta.
// Reuses tools/atlas_sim.js's REAL-dims reader (LZX/XNB `sheetDims`) and its
// packers WITHOUT touching that file: the purged S1-L-8b branch's git-show
// block is stubbed in memory (the d38 `_d38_atlas_cumulative.js` technique —
// the pinned additions all landed on main already).
//
// The roster after the research/108 vanilla-id renumber: 43 kinds (the eight
// sibling-owned kinds 581/587 c4, 592/607/615 c4b, 604/605 e26b, 606 c8 are
// NOT ours; 588/589 were already packed before this wave).
//
// Scenarios (working-tree assets.rs STATIC_SHEET_NAMES):
//   A  HEAD static-only
//   B  HEAD + gore (the E-4b landed set)
//   C  HEAD + the 43 roster sheets (static-only)
//   D  HEAD + gore + the 43 (the full runtime set)
//   E  D + boot font
//   F  HEAD + gore + the 39 union-delta sheets (the 4 c10-owned already in
//      the union ride their own lane's pack; attribution scenario)
// Run: node tools/_d42_sheets.js [--json <out-path>]
'use strict';
const fs = require('fs');
const path = require('path');
const ROOT = path.join(__dirname, '..');
let src = fs.readFileSync(path.join(ROOT, 'tools', 'atlas_sim.js'), 'utf8');
// Stub the L-8b git-show block (branch purged; its additions landed).
const a = src.indexOf('// The L-8b branch');
const b = src.indexOf('// Projectile kinds:');
src = src.slice(0, a) +
  'const L8B_ADDITIONS=[],L8B_REMOVED=[],L8B_STATIC=[];\n' +
  src.slice(b, src.indexOf('function main()'));
const module_ = { exports: {} };
new Function('module', 'exports', 'require', '__dirname', src +
  '\nmodule.exports={buildSheetList,sheetDims,shelfPack,skylinePack,maxrectsPack,verify,STATIC_NAMES,IMAGES};')
  (module_, module_.exports, require, path.join(ROOT, 'tools'));
const { buildSheetList, sheetDims, shelfPack, skylinePack, maxrectsPack, verify, STATIC_NAMES, IMAGES } = module_.exports;

// The 43 roster kinds (vanilla ids; the renumber's kept set).
const D42_IDS = [578, 579, 580, 582, 583, 584, 585, 586, 590, 591, 593, 594,
  595, 596, 597, 598, 599, 600, 601, 602, 603, 608, 609, 610, 611, 612, 613,
  614, 616, 617, 618, 619, 620, 621, 622, 623, 624, 625, 626, 627, 628, 629, 630];
// The four c10-owned sheets already rowed in d35's union list (their pack
// rides c10's lane; ours measures them for the shared-kind keep-one).
const C10_OWNED = [602, 603, 608, 609];
// The seven dragonfly sheets d33's tip ALREADY packs (style-114 wiring wave,
// origin/task/W4-d33-npc-ai-styles-95-127 assets.rs) but d35's §11
// re-extraction predates — shared ownership in the union delta.
const D33_PACKED = [595, 596, 597, 598, 599, 600, 601];
const D42 = D42_IDS.map(i => 'NPC_' + i);
const DELTA = D42_IDS.filter(i => !C10_OWNED.includes(i)).map(i => 'NPC_' + i);
for (const n of D42) if (STATIC_NAMES.includes(n)) throw new Error('roster sheet already packed?! ' + n);

// 1. The real-dims inventory (the union-forecast input). Owners: the four
// waterfowl sheets are c10's union rows (shared-kind keep-one); the seven
// dragonfly sheets are packed on d33's tip (post-d35-extraction arrival).
const ownersFor = id => {
  const o = ['W4-d42-npc-kinds-561-630'];
  if (C10_OWNED.includes(id)) o.unshift('W4-c10-style68-waterfowl');
  if (D33_PACKED.includes(id)) o.unshift('W4-d33-npc-ai-styles-95-127');
  return o;
};
const inventory = [];
const missing = [];
for (const id of D42_IDS) {
  const name = 'NPC_' + id;
  const d = sheetDims(name);
  if (!d) { missing.push(name); inventory.push({ name, owners: ownersFor(id), w: 0, h: 0, padded_area: 0, missing: true }); continue; }
  inventory.push({ name, owners: ownersFor(id), w: d.w, h: d.h, padded_area: (d.w + 2) * (d.h + 2), missing: false });
}
console.log('IMAGES =', IMAGES, '| exists:', fs.existsSync(IMAGES));
console.log('roster sheets:', D42.length, '| undecodable:', JSON.stringify(missing));
let area = 0;
for (const e of inventory) {
  area += e.padded_area;
  console.log(`  ${e.name.padEnd(8)} ${String(e.w).padStart(4)}x${String(e.h).padEnd(4)} padded ${e.padded_area}`);
}
console.log('roster padded area:', area, 'px^2 =', (area / 1e6).toFixed(3), 'Mpx');

// 2. The pack scenarios.
const scenarios = {
  'A: HEAD static-only': buildSheetList({ gores: false, staticNames: STATIC_NAMES }),
  'B: HEAD + gore': buildSheetList({ gores: true, staticNames: STATIC_NAMES }),
  'C: HEAD static + roster 43': buildSheetList({ gores: false, staticNames: STATIC_NAMES.concat(D42) }),
  'D: HEAD + gore + roster 43': buildSheetList({ gores: true, staticNames: STATIC_NAMES.concat(D42) }),
  'E: D + boot font': buildSheetList({ gores: true, staticNames: STATIC_NAMES.concat(D42), font: true }),
  'F: HEAD + gore + union-delta 39': buildSheetList({ gores: true, staticNames: STATIC_NAMES.concat(DELTA) }),
  // G: the roster-43 ALONE — the §10 lazy-form own-page cost estimate (the
  // per-world NPC page grows by at most this when every roster kind is
  // spawnable/hovered; the real rolls dedup against the resident set).
  'G: roster-43 solo page': D42.map(n => { const d = sheetDims(n); return { name: n, w: d.w, h: d.h }; }),
};
const results = {};
for (const [label, sheets] of Object.entries(scenarios)) {
  const bf = shelfPack(sheets, true);
  const sky = skylinePack(sheets, 'waste');
  const sky2 = skylinePack(sheets, 'waste');
  const mr = maxrectsPack(sheets);
  verify(sheets, sky);
  verify(sheets, mr);
  const det = JSON.stringify(sky.layout) === JSON.stringify(sky2.layout);
  const padded = sheets.reduce((s, x) => s + (x.w + 2) * (x.h + 2), 0);
  const tag = h => h > 8192 ? `OVERFLOW(+${h - 8192})` : `fits(${8192 - h} headroom)`;
  console.log(`\n${label}: ${sheets.length} sheets, padded ${(padded / 1e6).toFixed(2)} Mpx`);
  console.log(`  best-fit shelves: ${bf.height} ${tag(bf.height)}`);
  console.log(`  skyline (waste):  ${sky.height} ${tag(sky.height)} (det:${det}, verified)`);
  console.log(`  maxrects BAF:     ${mr.height} ${tag(mr.height)} (verified)`);
  results[label] = { sheets: sheets.length, padded, shelf: bf.height, skyline: sky.height, maxrects: mr.height, deterministic: det };
}

// 3. Optional JSON dump (inventory + scenario heights) for the research note.
const ji = process.argv.indexOf('--json');
if (ji > 0) {
  const out = { roster_ids: D42_IDS, c10_owned: C10_OWNED, inventory, missing, scenarios: results };
  fs.writeFileSync(process.argv[ji + 1], JSON.stringify(out, null, 1) + '\n');
  console.log('\nwrote', process.argv[ji + 1]);
}
