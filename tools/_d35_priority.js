// W4-d35 sheets phase: overflow prioritization measurement.
// SOLO (HEAD base, no pushed union) per-batch and per-sheet marginal skyline
// heights, to pick the subset of the 33 kinds that fits the 8192 bound.
// Also measures the cheap-kinds-first greedy order (ascending padded area).
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
  console.log('base main', H(base), '| base font', H(baseFont));
  // per-sheet marginal cost on the base (main set)
  const marginal = [];
  for (const id of ids) {
    const h = H(addNew(base, ['NPC_' + id]));
    const d = dims[id];
    marginal.push({ id, h, delta: h - H(base), area: (d.w+2)*(d.h+2) });
  }
  for (const m of marginal) console.log('solo NPC_' + m.id + ': height ' + m.h + ' (delta ' + m.delta + ', area ' + m.area + ')');
  // greedy: add sheets in ascending marginal delta, keep while <= 8192 on
  // BOTH main and font sets. Recompute the running height each step (the
  // packer is order-sensitive; the greedy order is the pack input order).
  const order = marginal.slice().sort((a,b) => a.delta - b.delta || a.area - b.area);
  let picked = [];
  for (const m of order) {
    const trial = picked.concat([m.id]);
    const names = trial.map(id => 'NPC_' + id);
    const hm = H(addNew(base, names));
    const hf = H(addNew(baseFont, names));
    if (hm <= 8192 && hf <= 8192) {
      picked = trial;
      console.log('greedy pick NPC_' + m.id + ' -> main ' + hm + ' font ' + hf);
    } else {
      console.log('greedy SKIP NPC_' + m.id + ' (would be main ' + hm + ' font ' + hf + ')');
    }
  }
  console.log('GREEDY RESULT:', JSON.stringify(picked.sort((a,b)=>a-b)));
  const names = picked.map(id => 'NPC_' + id);
  console.log('final main', H(addNew(base, names)), 'font', H(addNew(baseFont, names)));
  return picked;
`);
const picked = fn(require, __dirname, dims);
fs.writeFileSync(path.join(__dirname, '_d35_picked.json'), JSON.stringify(picked));
