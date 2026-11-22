// W4-d40 sheets phase: second greedy pass over the deferred sheets in id
// order — keep iff BOTH main and font stay <= 8192 (the d35 bound rule).
// The first pass (_d40_headroom.js) packed 34; this pass tries the 22
// deferred against the packed layout and keeps any that still fit.
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d40_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));
const dims = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_dims.json'), 'utf8'));
let picked = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_picked_solo.json'), 'utf8'));

const fn = new Function('require', '__dirname', 'dims', 'picked', head + `
  const addNew = (base, names) => {
    const seen = new Set(base.map(s => s.name));
    const out = base.slice();
    for (const n of names) {
      if (seen.has(n)) continue;
      const d = sheetDims(n);
      if (!d) continue;
      seen.add(n);
      out.push({ name: n, w: d.w, h: d.h });
    }
    out.dupDropped = base.dupDropped;
    return out;
  };
  const base = buildSheetList({});
  const baseFont = buildSheetList({ font: true });
  const H = (sheets) => skylinePack(sheets, 'waste').height;
  const all = Object.keys(dims).map(Number).sort((a,b)=>a-b);
  const deferred = all.filter(i => !picked.includes(i));
  for (const id of deferred) {
    const trial = picked.concat([id]).sort((a,b)=>a-b);
    const names = trial.map(i => 'NPC_' + i);
    const hm = H(addNew(base, names)), hf = H(addNew(baseFont, names));
    if (hm <= 8192 && hf <= 8192) { picked = trial; console.log('pass2 pick NPC_' + id + ' -> main ' + hm + ' font ' + hf); }
    else console.log('pass2 SKIP NPC_' + id + ' (main ' + hm + ' font ' + hf + ')');
  }
  picked.sort((a,b)=>a-b);
  const names = picked.map(i => 'NPC_' + i);
  const sm = addNew(base, names), sf = addNew(baseFont, names);
  const r1 = skylinePack(sm, 'waste'), r2 = skylinePack(sm, 'waste');
  const rf = skylinePack(sf, 'waste');
  verify(sm, r1); verify(sf, rf);
  const det = JSON.stringify(r1.layout) === JSON.stringify(r2.layout) && r1.height === r2.height;
  console.log('FINAL ' + picked.length + ' sheets: main ' + r1.height + ' font ' + rf.height + ' det=' + det);
  console.log('PICKED:', JSON.stringify(picked));
  console.log('DEFERRED:', JSON.stringify(all.filter(i => !picked.includes(i))));
  return picked;
`);
const out = fn(require, __dirname, dims, picked);
fs.writeFileSync(path.join(__dirname, '_d40_picked_final.json'), JSON.stringify(out));
