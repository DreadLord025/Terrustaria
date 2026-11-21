// W4-d35 RETIRE phase 2: re-measure the merged worst case on the NEW tips.
// The pushed extraction grew 424 -> 488 (comment-tolerant regex recovered
// e23's 26 MapBG backdrops + the new claimant waves); the full union is
// 490 -> 552. This script measures, on the REAL .xnb dims:
//   - merged-final-504 (the new pushed union on the 407 base)
//   - + d125 full tip (NPC_636/661/Extra_188) + deferred 5 (worst case)
//   - the lazy union (minus every NPC_) -> ONE page?
//   - the per-world NPC own pages (fresh/long-lived/bestiary) re-measured
//   - the realistic total stacks (lazy union + seed-42/worst bg + NPC)
// Every layout: verify() + double-run skyline determinism; boundary split
// recursed to N pages. Fresh output _d35_retire_measure.json (no resume).
// Run: node tools/_d35_retire_measure.js. No cargo, no Rust touched.
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d35_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));
const pushed = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_pushed_sheets.json'), 'utf8'));
const pushed339 = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_pushed_sheets_339.json'), 'utf8'));
const census = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_npc_census.json'), 'utf8'));
const bgInv = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_bg_inventory.json'), 'utf8'));

const D40_IDS = [430,431,432,433,434,435,436,437,438,440,442,443,444,445,446,
  447,448,449,450,451,452,453,454,455,456,457,458,459,460,461,462,463,464,465,
  466,467,468,469,470,472,473,474,475,476,477,478,479,480,481,482,483,484,485,
  486,487,488];
const D35_DEFERRED = [79,80,84,85,86,103,104,109,110,120,122,140];
const D125_TIP = ['NPC_636','NPC_661','Extra_188'];
const D125_DEFERRED = ['Extra_157','Extra_158','Extra_159','Extra_160','Extra_187'];

const fn = new Function('require', '__dirname', 'pushed', 'pushed339', 'census', 'bgInv',
  'D40_IDS', 'D35_DEFERRED', 'D125_TIP', 'D125_DEFERRED', head + `
  const addNew = (base, names) => {
    const seen = new Set(base.map(s => s.name));
    const out = base.slice();
    const missing = [];
    for (const n of names) {
      if (seen.has(n)) continue;
      const d = sheetDims(n);
      if (!d) { missing.push(n); continue; }
      seen.add(n);
      out.push({ name: n, w: d.w, h: d.h });
    }
    out.missing = missing;
    return out;
  };
  const boundaryNPages = (sheets, cap) => {
    const pages = [];
    let rest = sheets;
    while (rest.length) {
      const full = skylinePack(rest, 'waste');
      const keep = [], cur = [], tail = [];
      for (let i = 0; i < rest.length; i++) {
        const p = full.layout[i], s = rest[i];
        if (p.y + s.h <= cap - PAD) { cur.push(s); keep.push(p); }
        else tail.push(s);
      }
      if (!cur.length) throw new Error('no progress: sheet taller than cap');
      const h = Math.max(...keep.map((p, k) => p.y + cur[k].h)) + PAD;
      verify(cur, { height: h, layout: keep });
      pages.push({ height: h, n: cur.length });
      rest = tail;
    }
    return pages;
  };
  const areaOf = sheets => sheets.reduce((a, s) => a + (s.w + PAD) * (s.h + PAD), 0);
  const sheetsOf = names => {
    const out = [], missing = [];
    for (const n of new Set(names)) {
      const d = sheetDims(n);
      if (!d) { missing.push(n); continue; }
      out.push({ name: n, w: d.w, h: d.h });
    }
    out.missing = missing;
    return out;
  };

  const results = [];
  const measure = (label, sheets, note) => {
    const sky = skylinePack(sheets, 'waste');
    const sky2 = skylinePack(sheets, 'waste');
    verify(sheets, sky);
    if (JSON.stringify(sky.layout) !== JSON.stringify(sky2.layout)) throw new Error('nondeterministic: ' + label);
    const area = areaOf(sheets);
    const row = { scenario: label, note: note || '', sheets: sheets.length,
      missing: sheets.missing || [],
      areaMpx: +(area / 1e6).toFixed(2),
      areaBound: Math.ceil(area / (MAX_WIDTH - PAD)) + PAD,
      skyline: sky.height, skyOver: Math.max(0, sky.height - 8192) };
    row.pages = boundaryNPages(sheets, 8192);
    row.atlasMB = +(row.pages.reduce((a,p)=>a+p.height,0) * MAX_WIDTH * 4 / 1e6).toFixed(1);
    console.log(label + ': ' + sheets.length + ' sheets, ' + row.areaMpx +
      ' Mpx (bound ' + row.areaBound + ') | skyline ' + sky.height +
      (row.skyOver ? ' OVER+' + row.skyOver : '') +
      ' | pages ' + row.pages.map(p => p.height + '(' + p.n + ')').join(' / ') +
      ' | ' + row.atlasMB + ' MB');
    if (row.missing.length) console.log('  missing: ' + row.missing.join(', '));
    results.push(row);
    return row;
  };

  // ---- the merged base (section 7's exact input) then the NEW pushed union.
  const base = buildSheetList({ font: true });
  const union339 = [...new Set(Object.values(pushed339).flat())];
  let merged = addNew(base, union339);
  merged = addNew(merged, D40_IDS.map(id => 'NPC_' + id));
  merged = addNew(merged, D35_DEFERRED.map(id => 'NPC_' + id));
  measure('merged-407-base', merged, 'font + pushed(339) + d40(56) + d35deferred(12) - the section-7 base');

  const union488 = [...new Set(Object.values(pushed).flat())];
  const merged488 = addNew(merged, union488);
  measure('merged-final-504', merged488,
    'the RETIRE re-extraction (54 branches / 504 names) on the 407 base - d125 NOT in it');
  const d125full = addNew(merged488, D125_TIP);
  measure('merged-final-504+d125-full', d125full, 'd125 full sheets+draw tip: +NPC_636/661/Extra_188');
  const d125def = addNew(d125full, D125_DEFERRED);
  const worst = measure('merged-final-504+d125-full+deferred5', d125def,
    'd125 deferred overlay strips admitted (the NEW merged WORST case)');

  // ---- the lazy union: pull ALL NPC_ sheets out.
  const noNpc488 = merged488.filter(s => !/^NPC_\\d+$/.test(s.name));
  const lazy504 = measure('lazy-union-504-no-npc', noNpc488,
    'the 504 base minus every NPC_<id> (the section-10 lazy contract on the NEW tips)');
  const noNpcWorst = d125def.filter(s => !/^NPC_\\d+$/.test(s.name));
  const lazyWorst = measure('lazy-union-worst-no-npc', noNpcWorst,
    'the WORST base (504+d125full+deferred5) minus every NPC_<id> (lazy contract, worst union)');

  // ---- the per-world NPC working sets (own pages after the lazy union).
  const netNames = nets => [...new Set(nets)].map(n => 'NPC_' + n);
  const townAmb = netNames([...census.townNets, ...Object.values(census.ambientNets)]);
  const longLived = netNames([...census.townNets, ...Object.values(census.spawnNets),
    ...Object.values(census.moonNets), ...census.drawExtraNets]);
  const bestiaryAll = netNames([...census.entryNets, ...census.texOverrideNets,
    ...Object.values(census.moonNets), ...census.drawExtraNets]);
  const ownPage = (label, names, unionNoNpc, note) => {
    const fresh = sheetsOf(names.filter(n => !unionNoNpc.some(s => s.name === n)));
    const pages = fresh.length ? boundaryNPages(fresh, 8192) : [];
    const row = { scenario: label, note, newSheets: fresh.length,
      missing: fresh.missing, areaMpx: +(areaOf(fresh) / 1e6).toFixed(2),
      ownPages: pages,
      atlasMB: +(pages.reduce((a,p)=>a+p.height,0) * MAX_WIDTH * 4 / 1e6).toFixed(1) };
    console.log(label + ': +' + fresh.length + ' NPC sheets (' + row.areaMpx +
      ' Mpx) on ' + pages.length + ' own page(s): ' +
      pages.map(p => p.height + '(' + p.n + ')').join(' / ') + ' | ' + row.atlasMB + ' MB');
    results.push(row);
    return row;
  };
  // own pages measured against the WORST lazy union (the superset - any NPC
  // already in the union is not re-packed; the worst union has the most NPCs
  // pulled out, so the own-page sets are the largest realistic demand).
  ownPage('ownpage+town-ambient', townAmb, noNpcWorst, 'fresh-world NPC set, own page(s)');
  ownPage('ownpage+long-lived', longLived, noNpcWorst, 'long-lived NPC set, own page(s)');
  ownPage('ownpage+bestiary-all', bestiaryAll, noNpcWorst, 'bestiary-opened NPC set, own page(s)');

  // ---- the POST-MERGE augmented NPC sets: the new claimant waves add
  // spawnable kinds the landed census does not yet carry (f42 desert, d33
  // lunar, d39 critter, g5 caster, d11 probe, f47 shimmer). After the merge
  // their spawn arms land, so their sheets join the per-world NPC page.
  const NEW_CLAIMANT_NETS = [172,356,366,367,377,454,455,456,457,458,459,494,495,496,497,516,521,546,580,595,596,597,598,599,600,601,619,630,666,676,693];
  const newNames = NEW_CLAIMANT_NETS.map(n => 'NPC_' + n);
  ownPage('ownpage+long-lived+newclaimants', [...longLived, ...newNames], noNpcWorst,
    'long-lived NPC set + the 31 new-claimant spawnable kinds (post-merge roster)');
  ownPage('ownpage+bestiary+newclaimants', [...bestiaryAll, ...newNames], noNpcWorst,
    'bestiary-opened NPC set + the 31 new-claimant kinds (post-merge lifetime superset)');

  // ---- the realistic total stacks: lazy union + bg + NPC own pages.
  const bg42 = bgInv.scenarios.find(r => r.scenario === 'pageform+seed42-full');
  const bgWorst = bgInv.scenarios.find(r => r.scenario === 'pageform+worst-world');
  const stack = (label, unionRow, npcLabel, bgRow) => {
    const npcRow = results.find(r => r.scenario === npcLabel);
    const unionPages = unionRow.pages;
    const bgPages = bgRow ? bgRow.bgPages : [];
    const npcPages = npcRow.ownPages;
    const allH = [...unionPages, ...bgPages, ...npcPages].reduce((a,p)=>a+p.height,0);
    const row = { scenario: label, unionPages, bgPages, npcPages,
      totalPages: unionPages.length + bgPages.length + npcPages.length,
      atlasMB: +(allH * MAX_WIDTH * 4 / 1e6).toFixed(1) };
    console.log(label + ': union ' + unionPages.map(p=>p.height).join('/') +
      ' + bg ' + bgPages.map(p=>p.height).join('/') +
      ' + npc ' + npcPages.map(p=>p.height).join('/') +
      ' = ' + row.totalPages + ' pages | ' + row.atlasMB + ' MB');
    results.push(row);
    return row;
  };
  stack('TOTAL-lazy504-fresh-seed42', lazy504, 'ownpage+town-ambient', bg42);
  stack('TOTAL-lazy504-longlived-seed42', lazy504, 'ownpage+long-lived', bg42);
  stack('TOTAL-lazy504-bestiary-seed42', lazy504, 'ownpage+bestiary-all', bg42);
  stack('TOTAL-lazyWorst-fresh-seed42', lazyWorst, 'ownpage+town-ambient', bg42);
  stack('TOTAL-lazyWorst-longlived-seed42', lazyWorst, 'ownpage+long-lived', bg42);
  stack('TOTAL-lazyWorst-bestiary-seed42', lazyWorst, 'ownpage+bestiary-all', bg42);
  stack('TOTAL-lazyWorst-longlived-worstbg', lazyWorst, 'ownpage+long-lived', bgWorst);
  stack('TOTAL-lazyWorst-bestiary-worstbg', lazyWorst, 'ownpage+bestiary-all', bgWorst);
  // the post-merge augmented NPC roster (the new claimants' kinds spawnable)
  stack('TOTAL-lazyWorst-longlivedNEW-seed42', lazyWorst, 'ownpage+long-lived+newclaimants', bg42);
  stack('TOTAL-lazyWorst-bestiaryNEW-seed42', lazyWorst, 'ownpage+bestiary+newclaimants', bg42);
  stack('TOTAL-lazyWorst-bestiaryNEW-worstbg', lazyWorst, 'ownpage+bestiary+newclaimants', bgWorst);

  // ---- the eager forms (NPCs stay in the union) for the budget comparison.
  const stackEager = (label, unionRow, bgRow) => {
    const unionPages = unionRow.pages;
    const bgPages = bgRow ? bgRow.bgPages : [];
    const allH = [...unionPages, ...bgPages].reduce((a,p)=>a+p.height,0);
    const row = { scenario: label, unionPages, bgPages, npcPages: [],
      totalPages: unionPages.length + bgPages.length,
      atlasMB: +(allH * MAX_WIDTH * 4 / 1e6).toFixed(1) };
    console.log(label + ': eager union ' + unionPages.map(p=>p.height).join('/') +
      ' + bg ' + bgPages.map(p=>p.height).join('/') +
      ' = ' + row.totalPages + ' pages | ' + row.atlasMB + ' MB');
    results.push(row);
    return row;
  };
  const eager504 = results.find(r => r.scenario === 'merged-final-504');
  stackEager('TOTAL-eager504-seed42', eager504, bg42);
  stackEager('TOTAL-eager504-worstbg', eager504, bgWorst);

  // ---- the GPU-limit check + min-spec (section 12.1). The atlas is RGBA8
  // (4 B/px); the font array is BC2 (1 B/px, 4 x 8192x8192 layers measured by
  // _d35_font_vram.js); the render-target family is 7 world-layer 2304x1584
  // + 3 full-screen 1920x1080 RGBA8 (targets.rs, HiDef 1920x1200 view + 192
  // offScreenRange). wgpu exposes Limits::max_texture_dimension_2d (8192 on
  // the target adapters) but NO total-VRAM limit - the budget is an
  // allocation the driver must satisfy, so the min-spec is a recommendation.
  const fontVram = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_font_vram.json'), 'utf8'));
  const fontMB = fontVram.fontMB;
  const layerPx = (1920 + 384) * (1200 + 384);
  const targetsMB = +((7 * layerPx + 3 * 1920 * 1080) * 4 / 1e6).toFixed(1);
  const gpu = { fontArrayMB: fontMB, fontLayers: fontVram.fontLayers,
    renderTargetsMB: targetsMB, maxTextureDimension2d: 8192,
    perPageCapMB: +(8192 * 8192 * 4 / 1e6).toFixed(1), budgets: [] };
  for (const r of results) {
    if (!r.scenario.startsWith('TOTAL')) continue;
    const perPage = [...r.unionPages, ...r.bgPages, ...r.npcPages].map(p =>
      +(p.height * MAX_WIDTH * 4 / 1e6).toFixed(1));
    const maxPage = Math.max(...perPage);
    gpu.budgets.push({ scenario: r.scenario, totalPages: r.totalPages,
      atlasMB: r.atlasMB, perPageMB: perPage, maxPageMB: maxPage,
      pageUnder8192: r.unionPages.concat(r.bgPages, r.npcPages).every(p => p.height <= 8192),
      maxPageUnderCap: maxPage <= gpu.perPageCapMB,
      atlasPlusFontMB: +(r.atlasMB + fontMB).toFixed(1),
      totalVRAMMB: +(r.atlasMB + fontMB + targetsMB).toFixed(1) });
  }
  console.log('\\nGPU-limit check: per-page cap ' + gpu.perPageCapMB +
    ' MB (8192x8192 RGBA8), font array ' + fontMB + ' MB (' + fontVram.fontLayers +
    ' BC2 layers), render targets ' + targetsMB + ' MB');
  for (const b of gpu.budgets) {
    console.log('  ' + b.scenario + ': ' + b.totalPages + ' pages, atlas ' +
      b.atlasMB + ' MB, max page ' + b.maxPageMB + ' MB (' +
      (b.maxPageUnderCap ? 'OK' : 'OVER') + '), total VRAM ' + b.totalVRAMMB + ' MB');
  }
  fs.writeFileSync(path.join(__dirname, '_d35_retire_gpu.json'), JSON.stringify(gpu, null, 1));
  console.log('wrote _d35_retire_gpu.json');

  fs.writeFileSync(path.join(__dirname, '_d35_retire_measure.json'),
    JSON.stringify({ scenarios: results }, null, 1));
  console.log('\\nwrote _d35_retire_measure.json (' + results.length + ' scenarios)');
  return results;
`);
fn(require, __dirname, pushed, pushed339, census, bgInv, D40_IDS, D35_DEFERRED, D125_TIP, D125_DEFERRED);
