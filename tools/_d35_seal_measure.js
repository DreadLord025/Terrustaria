// W4-d35 SEAL-VERIFY: re-measure the merged worst case on the SEAL tips.
// The pushed extraction grew 504 -> 630 (65 branches; d43's 63-row band,
// g10's 11 pets, e23's 9 MapBGOverlay, f48 shields, e19 cursors, d125 now
// PUSHED); the full union is 562 -> 678 (+116 genuinely-new, 0 gone) +
// d42's 29 NO-PACK forecast names = the 707-name census union.
// Mirrors _d35_retire_measure.js: merged-final-630 / worst / lazy unions /
// +Glow362 / per-world NPC own pages (census + the new-claimant kinds, now
// including d43/g10's band) / total stacks / GPU-limit check. Every layout
// verify() + double-run; boundary split recursed. Fresh output
// _d35_seal_measure.json (no resume). Run: node tools/_d35_seal_measure.js.
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
// d42 FINAL 32-kind band (research/239 §4), NO-PACK verdict (research/238:
// eager 8382 +190 OVER) - forecast rows like d40's 56; 580/630 pushed
// by f42 and 619 by d11 already, these 29 are the unpacked remainder.
const D42_FORECAST = [578,579,582,583,584,585,586,590,591,593,594,610,611,612,
  613,614,616,617,618,620,621,622,623,624,625,626,627,628,629];

const fn = new Function('require', '__dirname', 'pushed', 'pushed339', 'census', 'bgInv',
  'D40_IDS', 'D35_DEFERRED', 'D125_TIP', 'D125_DEFERRED', 'D42_FORECAST', head + `
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

  // ---- the merged base (section 7's exact input) then the SEAL pushed union.
  const base = buildSheetList({ font: true });
  const union339 = [...new Set(Object.values(pushed339).flat())];
  let merged = addNew(base, union339);
  merged = addNew(merged, D40_IDS.map(id => 'NPC_' + id));
  merged = addNew(merged, D35_DEFERRED.map(id => 'NPC_' + id));
  measure('merged-407-base', merged, 'font + pushed(339) + d40(56) + d35deferred(12) - the section-7 base');

  const union630 = [...new Set(Object.values(pushed).flat())];
  let merged630 = addNew(merged, union630);
  measure('merged-final-630', merged630,
    'the SEAL re-extraction (65 branches / 630 names) on the 407 base - d125 tip now PUSHED (in the 630)');
  // d42's NO-PACK forecast (29 names, research/238/239): the eager worst
  // includes them; the lazy filter pulls them onto the per-world NPC page.
  merged630 = addNew(merged630, D42_FORECAST.map(id => 'NPC_' + id));
  measure('merged-final-630+d42forecast29', merged630,
    '+ d42 FINAL-band 29 forecast names (the NO-PACK verdict, research/238 §4)');
  const d125def = addNew(merged630, D125_DEFERRED);
  const worst = measure('merged-final-630+d42f29+deferred5', d125def,
    'd125 deferred overlay strips admitted (the SEAL merged WORST case)');

  // ---- the lazy union: pull ALL NPC_<id> sheets out. NPC_Head_* /
  // NPC_Head_Boss_* are eager UI/head rows (f29 receipt §11.3), they STAY.
  const isNpcKind = n => /^NPC_\\d+$/.test(n);
  const noNpc630 = merged630.filter(s => !isNpcKind(s.name));
  const lazy630 = measure('lazy-union-630-no-npc', noNpc630,
    'the 630 base minus every NPC_<id> (the section-10 lazy contract on the SEAL tips)');
  const noNpcWorst = d125def.filter(s => !isNpcKind(s.name));
  const lazyWorst = measure('lazy-union-worst-no-npc', noNpcWorst,
    'the WORST base (630+deferred5) minus every NPC_<id> (lazy contract, worst union)');
  const withGlow362 = addNew(noNpcWorst, ['Glow_362']);
  measure('lazy-union-worst+Glow362', withGlow362,
    'the lazy worst union + Glow_362 (the g31 table-pass load obligation, section 16.3)');

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
  ownPage('ownpage+town-ambient', townAmb, noNpcWorst, 'fresh-world NPC set, own page(s)');
  ownPage('ownpage+long-lived', longLived, noNpcWorst, 'long-lived NPC set, own page(s)');
  ownPage('ownpage+bestiary-all', bestiaryAll, noNpcWorst, 'bestiary-opened NPC set, own page(s)');

  // ---- the POST-MERGE augmented NPC sets: retire's 31 new-claimant kinds
  // PLUS the SEAL arrivals' spawnable kinds (d43 band incl. g10 pets, d41
  // completion, d36/d39/d6/e21/f18/g8 additions - every genuinely-new
  // NPC_<id> in the pushed union not already in the landed census sets).
  const RETIRE_CLAIMANT_NETS = [172,356,366,367,377,454,455,456,457,458,459,494,495,496,497,516,521,546,580,595,596,597,598,599,600,601,619,630,666,676,693];
  const pushedNpcIds = [...union630, ...D42_FORECAST.map(id => 'NPC_' + id)]
    .filter(isNpcKind).map(n => +n.slice(4));
  const censusIds = new Set([...townAmb, ...longLived, ...bestiaryAll].map(n => +n.slice(4)));
  const sealNets = pushedNpcIds.filter(id => !censusIds.has(id));
  const newNames = netNames([...RETIRE_CLAIMANT_NETS, ...sealNets]);
  console.log('post-merge augmented roster: retire 31 + SEAL census-new ' +
    sealNets.length + ' kinds = ' + newNames.length + ' names');
  ownPage('ownpage+long-lived+newclaimants', [...longLived, ...newNames], noNpcWorst,
    'long-lived NPC set + retire-31 + SEAL census-new kinds (post-merge roster)');
  ownPage('ownpage+bestiary+newclaimants', [...bestiaryAll, ...newNames], noNpcWorst,
    'bestiary-opened NPC set + retire-31 + SEAL census-new kinds (post-merge lifetime superset)');

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
  stack('TOTAL-lazy630-fresh-seed42', lazy630, 'ownpage+town-ambient', bg42);
  stack('TOTAL-lazy630-longlived-seed42', lazy630, 'ownpage+long-lived', bg42);
  stack('TOTAL-lazy630-bestiary-seed42', lazy630, 'ownpage+bestiary-all', bg42);
  stack('TOTAL-lazyWorst-fresh-seed42', lazyWorst, 'ownpage+town-ambient', bg42);
  stack('TOTAL-lazyWorst-longlived-seed42', lazyWorst, 'ownpage+long-lived', bg42);
  stack('TOTAL-lazyWorst-bestiary-seed42', lazyWorst, 'ownpage+bestiary-all', bg42);
  stack('TOTAL-lazyWorst-longlived-worstbg', lazyWorst, 'ownpage+long-lived', bgWorst);
  stack('TOTAL-lazyWorst-bestiary-worstbg', lazyWorst, 'ownpage+bestiary-all', bgWorst);
  stack('TOTAL-lazyWorst-longlivedNEW-seed42', lazyWorst, 'ownpage+long-lived+newclaimants', bg42);
  stack('TOTAL-lazyWorst-bestiaryNEW-seed42', lazyWorst, 'ownpage+bestiary+newclaimants', bg42);
  stack('TOTAL-lazyWorst-bestiaryNEW-worstbg', lazyWorst, 'ownpage+bestiary+newclaimants', bgWorst);

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
  const eager630 = results.find(r => r.scenario === 'merged-final-630');
  stackEager('TOTAL-eager630-seed42', eager630, bg42);
  stackEager('TOTAL-eager630-worstbg', eager630, bgWorst);

  // ---- the GPU-limit check + min-spec (section 15, re-run on the SEAL union).
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
    ' MB, font array ' + fontMB + ' MB, render targets ' + targetsMB + ' MB');
  for (const b of gpu.budgets) {
    console.log('  ' + b.scenario + ': ' + b.totalPages + ' pages, atlas ' +
      b.atlasMB + ' MB, max page ' + b.maxPageMB + ' MB (' +
      (b.maxPageUnderCap ? 'OK' : 'OVER') + '), total VRAM ' + b.totalVRAMMB + ' MB');
  }
  fs.writeFileSync(path.join(__dirname, '_d35_seal_gpu.json'), JSON.stringify(gpu, null, 1));
  fs.writeFileSync(path.join(__dirname, '_d35_seal_measure.json'),
    JSON.stringify({ scenarios: results }, null, 1));
  console.log('\\nwrote _d35_seal_measure.json (' + results.length + ' scenarios) + _d35_seal_gpu.json');
  return results;
`);
fn(require, __dirname, pushed, pushed339, census, bgInv, D40_IDS, D35_DEFERRED, D125_TIP, D125_DEFERRED, D42_FORECAST);
