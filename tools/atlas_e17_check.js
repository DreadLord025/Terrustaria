// W4-e17 atlas measurement: the boss-head set + MapDeath + Extra_173/175/176
// additions vs the branch's baseline, through the landed skyline packer.
// W4-f29 F4: the Coin_0..3 animation strips join the additions
// (research/140 §5 — re-measure after the coin block lands).
// Reuses tools/atlas_sim.js's decoder/packer by monkey-loading it with a
// stubbed main (the sim's own main needs a branch that is not fetched here).
'use strict';
const fs = require('fs');
const path = require('path');
const Module = require('module');

const simPath = path.join(__dirname, 'atlas_sim.js');
let src = fs.readFileSync(simPath, 'utf8');
// Drop the L-8b branch fetch (not present in this worktree's refs) and main().
src = src.replace(/const \{ execSync \}[\s\S]*?const L8B_REMOVED = [^\n]+\n/,
  'const L8B_STATIC = STATIC_NAMES; const L8B_ADDITIONS = []; const L8B_REMOVED = [];\n');
src = src.replace(/\nmain\(\);\s*$/, '\n');
src += '\nmodule.exports = { buildSheetList, skylinePack, verify, sheetDims, enumIds, STATIC_NAMES };\n';
const m = new Module(simPath, null);
m.filename = simPath;
m.paths = Module._nodeModulePaths(path.dirname(simPath));
m._compile(src, simPath);
const sim = m.exports;

const extra = ['MapDeath', 'Extra_173', 'Extra_175', 'Extra_176', 'Coin_0', 'Coin_1', 'Coin_2', 'Coin_3'];
for (let i = 0; i < 40; i++) if (i !== 7) extra.push('NPC_Head_Boss_' + i);

function measure(label, sheets) {
  const sky = sim.skylinePack(sheets, 'waste');
  sim.verify(sheets, sky);
  console.log(`${label}: ${sheets.length} sheets -> ${sky.height} px ${sky.height > 8192 ? 'OVERFLOW' : 'OK'}`);
  return sky.height;
}

const base = sim.buildSheetList({ gores: true, staticNames: sim.STATIC_NAMES.filter(n => !extra.includes(n)) });
const baseH = measure('branch baseline (static list + gore, minus e17 additions)', base);

// The additions ride STATIC_SHEET_NAMES (loader order: after Extra_305).
const withAdds = sim.buildSheetList({ gores: true });
// Rebuild with the additions injected into the static list position: the
// packer sorts by (h desc, w desc, input order), so append order only breaks
// exact size ties — matching the real list edit (mid-list) closely enough
// for a bound check; ties among equal-size boss heads keep id order either way.
const seen = new Set(withAdds.map(s => s.name));
for (const name of extra) {
  const d = sim.sheetDims(name);
  if (!d) { console.log('  missing file:', name); continue; }
  if (seen.has(name)) continue;
  withAdds.push({ name, w: d.w, h: d.h });
}
const addH = measure('branch + e17 map-icon sheets', withAdds);
console.log(`delta: +${addH - baseH} px (bound 8192)`);
const dims = extra.map(n => [n, sim.sheetDims(n)]).filter(([, d]) => d);
console.log('added sheet dims:', dims.map(([n, d]) => `${n}=${d.w}x${d.h}`).join(' '));
