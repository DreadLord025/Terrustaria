// W4-d35 sheets phase: natural-id-order packing measurement.
// The loader packs STATIC_SHEET_NAMES rows in list order, but Atlas::build
// sorts by (height desc, width desc, input index) — so the PACK order is
// deterministic regardless of where the rows sit in the const. This measures
// the 33 kinds in ascending id order (the batch A..G order), cumulatively,
// on the HEAD base (main + font), to pick the largest prefix/subset that fits.
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d35_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));
const dims = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_dims.json'), 'utf8'));

const fn = new Function('require', '__dirname', 'dims', head + `
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
  const ids = Object.keys(dims).map(Number).sort((a,b)=>a-b);
  // cumulative in id order
  let acc = [];
  for (const id of ids) {
    acc.push(id);
    const names = acc.map(i => 'NPC_' + i);
    console.log('cum NPC_' + id + ' (' + acc.length + '): main ' + H(addNew(base, names)) + ' font ' + H(addNew(baseFont, names)));
  }
  // greedy in id order: keep a sheet iff both sets stay <= 8192
  let picked = [];
  for (const id of ids) {
    const trial = picked.concat([id]);
    const names = trial.map(i => 'NPC_' + i);
    const hm = H(addNew(base, names)), hf = H(addNew(baseFont, names));
    if (hm <= 8192 && hf <= 8192) { picked = trial; console.log('idorder pick NPC_' + id + ' -> main ' + hm + ' font ' + hf); }
    else console.log('idorder SKIP NPC_' + id + ' (main ' + hm + ' font ' + hf + ')');
  }
  console.log('IDORDER RESULT:', JSON.stringify(picked));
  const names = picked.map(i => 'NPC_' + i);
  const sm = addNew(base, names), sf = addNew(baseFont, names);
  const r1 = skylinePack(sm, 'waste'), r2 = skylinePack(sm, 'waste');
  verify(sm, r1);
  verify(sf, skylinePack(sf, 'waste'));
  console.log('final main', r1.height, 'font', H(sf), 'deterministic:', JSON.stringify(r1.layout) === JSON.stringify(r2.layout));
  return picked;
`);
const picked = fn(require, __dirname, dims);
fs.writeFileSync(path.join(__dirname, '_d35_picked_idorder.json'), JSON.stringify(picked));
