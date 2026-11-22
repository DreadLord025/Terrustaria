// W4-d35 sheets phase: real .xnb dims for the 33 kinds' sheets, via the
// pinned LZX harness (_d35_atlas_sim_pinned.js is a copy of b4's
// tools/atlas_sim.js; we re-require its decoder by extracting sheetDims).
// Simpler: inline-load the pinned harness's functions by eval of its source
// up to main(), then call sheetDims directly.
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d35_atlas_sim_pinned.js'), 'utf8');
const cut = src.indexOf('function main()');
const head = src.slice(0, cut);
const sandbox = {};
const fn = new Function('require', '__dirname', head + `
  const ids = [73,74,75,77,78,79,80,82,83,84,85,86,93,94,95,96,97,98,99,100,101,102,103,104,109,110,112,116,120,121,122,137,140];
  const out = {};
  for (const id of ids) {
    const d = sheetDims('NPC_' + id);
    out[id] = d;
  }
  return out;
`);
const dims = fn(require, __dirname);
let area = 0;
for (const [id, d] of Object.entries(dims)) {
  if (!d) { console.log(`NPC_${id}: MISSING/UNDECODABLE`); continue; }
  area += (d.w + 2) * (d.h + 2);
  console.log(`NPC_${id}: ${d.w}x${d.h}`);
}
console.log('total padded area:', (area / 1e6).toFixed(3), 'Mpx');
fs.writeFileSync(path.join(__dirname, '_d35_dims.json'), JSON.stringify(dims, null, 1));
