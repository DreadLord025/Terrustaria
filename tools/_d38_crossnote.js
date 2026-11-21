// W4-d38 CROSSNOTE phase — the cumulative atlas projection UPDATED with the
// sibling branches' latest pushed tips (read 2026-11-21):
//   d35 tip 7d131be3 (ATLAS-REPACK doc consistency; sheets phase 0668fba7
//     packed 21 of its 33, 12 stay deferred) — its research/113 plan already
//     chose the boundary-split 2-page skyline form.
//   d40 tip 8fe1d6d3 (data-only, 56 kinds 430-488; the d40b sheets phase is
//     in flight, NOT pushed) — its 56 sheets are FORECAST here exactly as
//     d35's _d35_repack_measure.js D40_IDS does (same id list, verified).
//
// Reuses the _d38_atlas_cumulative.js technique: tools/atlas_sim.js's real-
// dims census + packer twins, the purged L-8b git-show stubbed IN MEMORY
// (atlas_sim.js untouched). Every layout verify()'d + skyline double-run
// determinism. "area bound" = ceil(padded area / 8190) + 2 — the height NO
// single-page packer can beat (d35 research/113's formula).
//
// Scenarios (runtime = the landed E-4b gore enumeration included):
//   P   HEAD + d35's 21 PACKED (the d35 tip's actual state) — RECONCILIATION:
//       must equal d35's "base" row (9518 sheets / 8188 skyline): their base
//       is their sheets-phase HEAD, i.e. a725acb0 + the 21, NOT bare main.
//   R   HEAD + d35 21 + d40 forecast 56 (d40b lands, the 12 still deferred)
//   Q   HEAD + d35 33 (21 + the deferred 12) + d40 forecast 56 (full wave)
//   S   Q + c4's 4 (224/509/581/587 — the three-wave+ check)
//   each with a +boot-font twin.
// Run: node tools/_d38_crossnote.js
'use strict';
const fs = require('fs');
const path = require('path');
const ROOT = path.join(__dirname, '..');
let src = fs.readFileSync(path.join(ROOT, 'tools', 'atlas_sim.js'), 'utf8');
const a = src.indexOf('// The L-8b branch');
const b = src.indexOf('// Projectile kinds:');
src = src.slice(0, a) +
  'const L8B_ADDITIONS=[],L8B_REMOVED=[],L8B_STATIC=[];\n' +
  src.slice(b, src.indexOf('function main()'));
const module_ = { exports: {} };
new Function('module', 'exports', 'require', '__dirname', src +
  '\nmodule.exports={buildSheetList,sheetDims,skylinePack,maxrectsPack,verify,STATIC_NAMES};')
  (module_, module_.exports, require, path.join(ROOT, 'tools'));
const { buildSheetList, sheetDims, skylinePack, maxrectsPack, verify, STATIC_NAMES } = module_.exports;

// d35 sheets phase 0668fba7: the 21 PACKED (its STATIC_SHEET_NAMES additions).
const D35_PACKED = [73,74,75,77,78,82,83,93,94,95,96,97,98,99,100,101,102,112,116,121,137];
// d35's 12 DEFERRED (the atlas-repack wave admits them onto page 1).
const D35_DEFERRED = [79,80,84,85,86,103,104,109,110,120,122,140];
// d40's 56 kinds — VERBATIM the D40_IDS list in d35's _d35_repack_measure.js
// (origin/task/W4-d35-npc-kinds-71-140 at 7d131be3), cross-checked against
// d40's own batch commits 8b83863f..8fe1d6d3 (ALL_KINDS 270->326, +56).
const D40 = [430,431,432,433,434,435,436,437,438,440,442,443,444,445,446,
  447,448,449,450,451,452,453,454,455,456,457,458,459,460,461,462,463,464,465,
  466,467,468,469,470,472,473,474,475,476,477,478,479,480,481,482,483,484,485,
  486,487,488];
const C4 = [224,509,581,587];
const names = ids => ids.map(i => 'NPC_' + i);
for (const n of names([...D35_PACKED, ...D35_DEFERRED, ...D40, ...C4])) {
  if (STATIC_NAMES.includes(n)) throw new Error('already packed on HEAD?! ' + n);
  if (!sheetDims(n)) throw new Error('undecodable/missing sheet: ' + n);
}
console.log('d35 packed:', D35_PACKED.length, '| d35 deferred:', D35_DEFERRED.length,
  '| d40 forecast:', D40.length, '| all decodable, none on HEAD');

const scenarios = {
  'P: HEAD + d35 21 packed (d35-tip reconciliation)': [...D35_PACKED],
  'R: HEAD + d35 21 + d40 56 (d40b lands)': [...D35_PACKED, ...D40],
  'Q: HEAD + d35 33 + d40 56 (FULL wave forecast)': [...D35_PACKED, ...D35_DEFERRED, ...D40],
  'S: Q + c4 4 (four-wave check)': [...D35_PACKED, ...D35_DEFERRED, ...D40, ...C4],
};
const out = [];
for (const [label, ids] of Object.entries(scenarios)) {
  for (const font of [false, true]) {
    const sheets = buildSheetList({ gores: true, staticNames: STATIC_NAMES.concat(names(ids)), font });
    const sky = skylinePack(sheets, 'waste');
    const sky2 = skylinePack(sheets, 'waste');
    const mr = maxrectsPack(sheets);
    verify(sheets, sky);
    verify(sheets, mr);
    const det = JSON.stringify(sky.layout) === JSON.stringify(sky2.layout);
    const area = sheets.reduce((s, x) => s + (x.w + 2) * (x.h + 2), 0);
    const areaBound = Math.ceil(area / 8190) + 2;
    const tag = h => h > 8192 ? `OVERFLOW(+${h - 8192})` : `fits(${8192 - h} headroom)`;
    const full = label + (font ? ' + boot font' : '');
    console.log(`\n${full}: ${sheets.length} sheets, padded ${(area / 1e6).toFixed(2)} Mpx, area bound ${areaBound} ${tag(areaBound)}`);
    console.log(`  skyline (waste): ${sky.height} ${tag(sky.height)} (det:${det}, verified)`);
    console.log(`  maxrects BAF:    ${mr.height} ${tag(mr.height)} (verified)`);
    out.push({ scenario: full, sheets: sheets.length, areaMpx: +(area / 1e6).toFixed(2),
      areaBound, skyline: sky.height, maxrects: mr.height, deterministic: det });
  }
}
fs.writeFileSync(path.join(ROOT, 'tools', '_d38_crossnote.json'), JSON.stringify(out, null, 1) + '\n');
console.log('\nwrote tools/_d38_crossnote.json (' + out.length + ' rows)');
