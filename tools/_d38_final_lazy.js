// W4-d38 FINAL sheet 1 - the lazy-load re-projection of the crossnote's
// 35+56 sets against d35's research/113 §10 (tip e5558b0a9, read 2026-11-21):
// the NPC corpus leaves the eager union; d38's 35 + d40's forecast 56 ride
// the per-world NPC page instead. Reuses the _d38_crossnote.js harness
// technique (atlas_sim real-dims census + packer twins, purged L-8b stubbed
// IN MEMORY, verify() + double-run determinism on EVERY layout).
//
// Scenarios (runtime = the landed E-4b gore enumeration included):
//   E35  HEAD eager minus d38's 35 NPC rows (the lazy union WITHOUT d38's
//        sheets in the eager half) - must drop the crossnote HEAD row
//        (8236 skyline) below the bound on its own if the 35 were the only
//        overflow driver (they are not - the base was already 8115).
//   L91  the per-world NPC page candidate: d38's 35 + d40's forecast 56
//        (the bestiary-superset slice these two waves add) ALONE.
//   X    HEAD eager minus d38 35 PLUS L91 packed as a second (NPC) page -
//        the two-page lazy form for the three-wave subset.
//   each with a +boot-font twin.
// Run: node tools/_d38_final_lazy.js
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

// d38's 35 (protocol == vanilla, tools/_d38_final_census.json extraction).
const D38 = [281,282,283,284,285,286,287,289,290,291,292,293,294,295,296,
  297,298,299,300,301,302,303,304,316,317,318,322,323,324,328,333,334,335,336,337];
// d40's 56 forecast - VERBATIM the _d38_crossnote.js D40 list (== 113 D40_IDS).
const D40 = [430,431,432,433,434,435,436,437,438,440,442,443,444,445,446,
  447,448,449,450,451,452,453,454,455,456,457,458,459,460,461,462,463,464,465,
  466,467,468,469,470,472,473,474,475,476,477,478,479,480,481,482,483,484,485,
  486,487,488];
const names = ids => ids.map(i => 'NPC_' + i);
for (const n of names(D38)) {
  if (!STATIC_NAMES.includes(n)) throw new Error('d38 sheet not on HEAD?! ' + n);
}
for (const n of names(D40)) {
  if (STATIC_NAMES.includes(n)) throw new Error('d40 already packed?! ' + n);
  if (!sheetDims(n)) throw new Error('undecodable: ' + n);
}
console.log('d38:', D38.length, 'on HEAD | d40 forecast:', D40.length, 'decodable, not on HEAD');

const headNo38 = STATIC_NAMES.filter(n => !names(D38).includes(n));
const scenarios = {
  'E35: HEAD eager MINUS d38 35 (lazy union, d38 rows out)': { staticNames: headNo38 },
  'L91: the d38 35 + d40 56 NPC-page candidate ALONE': { staticNames: names([...D38, ...D40]), noBase: true },
};
const out = [];
function measure(label, sheets, font) {
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
for (const font of [false, true]) {
  // E35: the eager union with d38's rows pulled out (everything else stays).
  measure('E35: HEAD eager MINUS d38 35 (lazy union, d38 rows out)',
    buildSheetList({ gores: true, staticNames: headNo38, font }), font);
  // L91: JUST the 91 NPC sheets (the per-world page candidate) - no tiles/
  // walls/etc: a bare sheet list through the same packer twins.
  const npc = names([...D38, ...D40]).map(n => {
    const d = sheetDims(n);
    if (!d) throw new Error('no dims ' + n);
    return { name: n, w: d.w, h: d.h };
  });
  if (font) {
    const f = buildSheetList({ gores: false, staticNames: [], font: true });
    // buildSheetList with empty statics still adds the fixed furniture;
    // take ONLY its font rows (the boot-font twin's one extra sheet).
    const fontRows = f.filter(s => /font/i.test(s.name));
    npc.push(...fontRows);
  }
  measure('L91: the d38 35 + d40 56 NPC-page candidate ALONE', npc, font);
}
fs.writeFileSync(path.join(ROOT, 'tools', '_d38_final_lazy.json'), JSON.stringify(out, null, 1) + '\n');
console.log('\nwrote tools/_d38_final_lazy.json (' + out.length + ' rows)');
