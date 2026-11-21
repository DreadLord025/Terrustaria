// W4-c4 headroom probe: measure the skyline height delta of each candidate
// NPC sheet against the CURRENT main static list (the 7757/8143 baseline),
// individually and cumulatively, using atlas_sim.js's own decoder/packer.
// Run: node tools/_w4c4_headroom.js   (from the repo root, assets junction)
'use strict';
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Pull the sim's internals without running main(): read the source, cut the
// trailing `main();` call, eval in a sandbox that exposes the helpers.
const simSrc = fs.readFileSync(path.join(__dirname, 'atlas_sim.js'), 'utf8');
const cut = simSrc.replace(/\nmain\(\);\s*$/, '\nmodule.exports = { buildSheetList, skylinePack, sheetDims, verify };\n');
const Module = require('module');
const m = new Module('atlas_sim_probe');
m._compile(cut, path.join(__dirname, 'atlas_sim_probe.js'));
const { buildSheetList, skylinePack, sheetDims, verify } = m.exports;

const CANDIDATES = [57, 157, 241, 465, 592, 607, 615, 688, 692];

function heights(extraNames) {
  const main = buildSheetList({});
  const full = buildSheetList({ gores: true, font: true });
  for (const list of [main, full]) {
    const seen = new Set(list.map(s => s.name));
    for (const n of extraNames) {
      if (seen.has(n)) continue;
      const d = sheetDims(n);
      if (d) { list.push({ name: n, w: d.w, h: d.h }); seen.add(n); }
    }
  }
  const hm = skylinePack(main, 'waste');
  const hf = skylinePack(full, 'waste');
  verify(main, hm); verify(full, hf);
  return { main: hm.height, full: hf.height };
}

const base = heights([]);
console.log(`baseline: main ${base.main} / full ${base.full} (bound 8192)`);
for (const id of CANDIDATES) {
  const d = sheetDims('NPC_' + id);
  const h = heights(['NPC_' + id]);
  console.log(`NPC_${id}: dims ${d ? d.w + 'x' + d.h : 'MISSING'} -> main ${h.main} (+${h.main - base.main}) / full ${h.full} (+${h.full - base.full})${h.full > 8192 ? ' OVERFLOW' : ''}`);
}
// cumulative in id order
let acc = [];
for (const id of CANDIDATES) {
  acc.push('NPC_' + id);
  const h = heights(acc);
  console.log(`cumulative through ${id}: main ${h.main} / full ${h.full}${h.full > 8192 ? ' OVERFLOW' : ''}`);
}
