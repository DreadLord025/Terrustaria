// W4-d38 sheets/atlas phase — CUMULATIVE headroom measurement.
// Reuses tools/atlas_sim.js's REAL-dims census + packers (the LZX/XNB reader
// and the skyline/maxrects twins) without touching that file: the purged
// S1-L-8b branch's git-show block is stubbed in memory (the pinned additions
// all landed on main already — the base sim reports "L-8b additions (0)").
//
// Scenarios (all over the working tree's assets.rs STATIC_SHEET_NAMES):
//   A  HEAD as-is (d38's 35 NPC sheets already packed)
//   B  HEAD minus d38's 35 sheets (the pre-d38 baseline)
//   C  HEAD + d35's 33 deferred sheets (the CUMULATIVE wave measurement)
//   D  B + d35's 33 (cumulative without d38 — attribution)
//   E  C + boot font (the full runtime set)
// Run: node tools/_d38_atlas_cumulative.js
'use strict';
const fs = require('fs');
const path = require('path');
const ROOT = path.join(__dirname, '..');
let src = fs.readFileSync(path.join(ROOT, 'tools', 'atlas_sim.js'), 'utf8');
// Stub the L-8b git-show block (branch purged; its 25 additions landed).
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
console.log('IMAGES =', IMAGES, '| exists:', fs.existsSync(IMAGES));
console.log('Gore_1 dims:', JSON.stringify(sheetDims('Gore_1')), '| Gore files:',
  fs.readdirSync(IMAGES).filter(f => /^Gore_\d+\.xnb$/.test(f)).length);

// d38's 35 (this branch, packed in STATIC_SHEET_NAMES).
const D38 = [281,282,283,284,285,286,287,289,290,291,292,293,294,295,296,297,298,299,
  300,301,302,303,304,316,317,318,322,323,324,328,333,334,335,336,337].map(i => 'NPC_' + i);
// d35's 33 DEFERRED sheets (origin/task/W4-d35-npc-kinds-71-140's draw-arm
// comments: "NPC_<id> are NOT packed ... the packing rides the atlas-repack
// wave"): batches A-G.
const D35 = [73,74,75,77,78,79,80,82,83,84,85,86,93,94,95,96,97,98,99,100,
  101,102,103,104,109,110,112,116,120,121,122,137,140].map(i => 'NPC_' + i);
for (const n of D38) if (!STATIC_NAMES.includes(n)) throw new Error('d38 sheet not packed: ' + n);
for (const n of D35) if (STATIC_NAMES.includes(n)) throw new Error('d35 sheet already packed?! ' + n);
const missing = D35.filter(n => !sheetDims(n));
console.log('d35 deferred sheets:', D35.length, '| undecodable:', missing);
// c4's four (origin/task/W4-c4-style44-flying-fish's STYLE44_SHEETS): the
// branch's own measurement 7711->7757 static / 8119->8143 full — the sibling
// wave whose baseline our numbers extend.
const C4 = [224,509,581,587].map(i => 'NPC_' + i);
for (const n of C4) if (STATIC_NAMES.includes(n)) throw new Error('c4 sheet already packed?! ' + n);
const withoutD38 = STATIC_NAMES.filter(n => !D38.includes(n));

const scenarios = {
  'A: HEAD static-only (d38 35 packed)': buildSheetList({ gores: false, staticNames: STATIC_NAMES }),
  'B: HEAD static-only minus d38 (pre-d38 baseline)': buildSheetList({ gores: false, staticNames: withoutD38 }),
  'C: HEAD static-only + d35 33': buildSheetList({ gores: false, staticNames: STATIC_NAMES.concat(D35) }),
  'A+gore: HEAD (E-4b landed set)': buildSheetList({ gores: true, staticNames: STATIC_NAMES }),
  'B+gore: pre-d38 baseline': buildSheetList({ gores: true, staticNames: withoutD38 }),
  'C+gore: HEAD + d35 33 (CUMULATIVE)': buildSheetList({ gores: true, staticNames: STATIC_NAMES.concat(D35) }),
  'D+gore: pre-d38 + d35 33 (attribution)': buildSheetList({ gores: true, staticNames: withoutD38.concat(D35) }),
  'E+gore: CUMULATIVE + boot font': buildSheetList({ gores: true, staticNames: STATIC_NAMES.concat(D35), font: true }),
  'F: c4 baseline check (HEAD minus d38 plus c4 4)': buildSheetList({ gores: false, staticNames: withoutD38.concat(C4) }),
  'F+gore: c4 baseline check': buildSheetList({ gores: true, staticNames: withoutD38.concat(C4) }),
  'G+gore: d38 + d35 + c4 (three-wave cumulative)': buildSheetList({ gores: true, staticNames: STATIC_NAMES.concat(D35, C4) }),
};
for (const [label, sheets] of Object.entries(scenarios)) {
  const bf = shelfPack(sheets, true);
  const sky = skylinePack(sheets, 'waste');
  const sky2 = skylinePack(sheets, 'waste');
  const mr = maxrectsPack(sheets);
  verify(sheets, sky);
  verify(sheets, mr);
  const det = JSON.stringify(sky.layout) === JSON.stringify(sky2.layout);
  const area = sheets.reduce((s, x) => s + (x.w + 2) * (x.h + 2), 0);
  const tag = h => h > 8192 ? `OVERFLOW(+${h - 8192})` : `fits(${8192 - h} headroom)`;
  console.log(`\n${label}: ${sheets.length} sheets, padded ${(area / 1e6).toFixed(2)} Mpx`);
  console.log(`  best-fit shelves: ${bf.height} ${tag(bf.height)}`);
  console.log(`  skyline (waste):  ${sky.height} ${tag(sky.height)} (det:${det}, verified)`);
  console.log(`  maxrects BAF:     ${mr.height} ${tag(mr.height)} (verified)`);
}
