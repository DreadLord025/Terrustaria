// W4-d35 ATLAS-REPACK extension phase: BG-STYLE demand measurement.
// Extends research/113 with (a) the full bg-sheet inventory (every style
// variant of every family, REAL retail .xnb dims via the pinned harness),
// (b) the page budget WITH bg included on the merged-W4 worst-case base,
// (c) the per-world SELECTIVE-loading budget (the C# AssetInitializer
// registers all 344 Background_* + 14 Underworld assets DoNotLoad and
// Main.LoadBackground fires per DRAWN layer only - the port contract is
// load-on-demand style sheets, not all-407).
//
// Style tables transcribed VERBATIM from WorldGen.setBG (WorldGen.cs:
// 7215-7637) + SetForestBGSet (:7639-7768); the cavern sets from
// Main.DrawBackground_UpdateBackgroundStyles (Main.cs:53812-54015);
// the animation families from DoUpdate_AnimateBackgrounds_UpdateForest
// (Main.cs:19658-19735) and the 173 frame swap (Main.cs:61371-61376).
//
// Run: node tools/_d35_bg_inventory.js   (resume-safe: cached scenarios
// in _d35_bg_inventory.json are skipped). Every layout: verify() +
// double-run skyline determinism. No cargo, no Rust touched.
'use strict';
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '_d35_atlas_sim_pinned.js'), 'utf8');
const head = src.slice(0, src.indexOf('function main()'));
const pushed = JSON.parse(fs.readFileSync(path.join(__dirname, '_d35_pushed_sheets.json'), 'utf8'));

const D40_IDS = [430,431,432,433,434,435,436,437,438,440,442,443,444,445,446,
  447,448,449,450,451,452,453,454,455,456,457,458,459,460,461,462,463,464,465,
  466,467,468,469,470,472,473,474,475,476,477,478,479,480,481,482,483,484,485,
  486,487,488];
const D35_DEFERRED = [79,80,84,85,86,103,104,109,110,120,122,140];

// --- setBG style tables (WorldGen.cs:7215-7637), -1 = layer never drawn ---
// forest: SetForestBGSet (:7639-7768). mnt default [7,8].
const FOREST = {
  0:  { mnt: [7, 8],    tree: [9, 10, 11] },
  1:  { mnt: [7, 8],    tree: [50, 51, 52] },
  2:  { mnt: [7, 8],    tree: [53, 54, 55] },
  3:  { mnt: [7, 90],   tree: [91, -1, 92] },
  31: { mnt: [7, 90],   tree: [91, -1, 11] },
  4:  { mnt: [93, 94],  tree: [-1, -1, -1] },
  5:  { mnt: [93, 94],  tree: [-1, -1, 55] },
  51: { mnt: [93, 94],  tree: [-1, -1, 11] },
  6:  { mnt: [171, 172], tree: [173, -1, -1] },
  7:  { mnt: [176, 177], tree: [178, -1, -1] },
  71: { mnt: [176, 177], tree: [178, -1, 11] },
  72: { mnt: [176, 177], tree: [178, -1, 52] },
  73: { mnt: [176, 177], tree: [178, -1, 55] },
  8:  { mnt: [179, 180], tree: [184, -1, -1] },
  9:  { mnt: [277, 278], tree: [279, -1, -1] },
  10: { mnt: [280, 281], tree: [282, -1, -1] },
  11: { mnt: [7, 331],  tree: [330, 329, 328] },
  12: { mnt: [7, 336],  tree: [335, 334, 333] },
  13: { mnt: [7, -1],   tree: [343, 342, 341] },
};
// The update-side animation families (Main.cs:19660-19734): the 94 hill
// cycle swaps mnt[1] 94/114/115/116 AND mnt[0] 93/168/169/170; the 180
// cycle swaps mnt[1] 180..183 (mnt[0] stays 179). The 173 tree line draws
// frames 251..254 (Main.cs:61371-61376). g70 packs these whole.
function forestSet(style) {
  const s = FOREST[style] || FOREST[0];
  const ids = [...s.mnt, ...s.tree];
  if (s.mnt[1] === 94) ids.push(114, 115, 116, 168, 169, 170);
  if (s.mnt[1] >= 180 && s.mnt[1] <= 183) ids.push(181, 182, 183);
  if (s.tree[0] === 173) ids.push(251, 252, 253, 254);
  return ids;
}
const CORRUPT = { 0: [12,13,14], 1: [56,57,58], 2: [211,212,213], 3: [225,226,227],
  4: [240,241,242], 51: [324,323,322], 52: [324,226,322] };
const JUNGLE = { 0: [15,16,17], 1: [59,60,61], 2: [222,223,224], 3: [237,238,239],
  4: [284,285,286], 5: [271,272,273], 6: [302,301,300] };
// snow: mnt default [35,36] (WorldGen.cs:7319-7320), overridden per style.
const SNOW = {
  0:  { mnt: [35, 36], bg: [37, 38, 39] },
  1:  { mnt: [35, 36], bg: [97, 96, 95] },
  2:  { mnt: [98, 99], bg: [-1, -1, -1] },
  21: { mnt: [98, 99], bg: [95, 96, 97] },
  22: { mnt: [98, 99], bg: [37, 38, 39] },
  3:  { mnt: [98, 100], bg: [-1, -1, -1] },
  31: { mnt: [98, 100], bg: [95, 96, 97] },
  32: { mnt: [98, 100], bg: [37, 38, 39] },
  4:  { mnt: [98, 101], bg: [-1, -1, -1] },
  41: { mnt: [98, 101], bg: [95, 96, 97] },
  42: { mnt: [98, 101], bg: [37, 38, 39] },
  5:  { mnt: [-1, -1], bg: [258, 259, 260] },
  6:  { mnt: [-1, -1], bg: [263, 264, 265] },
  7:  { mnt: [269, 270], bg: [267, 266, 268] },
  8:  { mnt: [35, 36], bg: [299, 298, -1] },
};
const HALLOW = { 0: [29,30,31], 1: [102,103,104], 2: [219,220,221],
  3: [243,244,245], 4: [-1,261,262], 5: [327,326,325] };
const CRIMSON = { 0: [43,44,45], 1: [105,106,107], 2: [174,-1,175],
  3: [214,215,216], 4: [-1,229,230], 5: [255,256,257], 6: [339,338,337] };
// desert: the four BackgroundVariant rows (WorldGen.cs:7501-7546).
const DESERT = {
  0: { Pure: [21,20,-1] }, 1: { Pure: [108,109,-1] }, 2: { Pure: [207,208,-1] },
  3: { Pure: [217,218,-1] }, 4: { Pure: [248,249,250] },
  51: { Pure: [306,303,-1], Corrupt: [310,307,-1], Hallow: [314,311,-1], Crimson: [318,315,-1] },
  52: { Pure: [306,304,-1], Corrupt: [310,308,-1], Hallow: [314,312,-1], Crimson: [318,316,-1] },
  53: { Pure: [306,305,-1], Corrupt: [310,309,-1], Hallow: [314,313,-1], Crimson: [318,317,-1] },
};
const OCEAN = { 0: 28, 1: 110, 2: 111, 3: 209, 4: 210, 5: 283, 6: 332, 7: 340 };
const MUSHROOM = { 0: [46,47,48], 1: [231,232,233], 2: [234,235,236],
  3: [287,288,289], 4: [321,320,319] };
// underworld: five TextureAssets.Underworld indices per style
// (WorldGen.cs:7609-7635). Sheet names "Backgrounds/Underworld <n>".
const UNDERWORLD = { 0: [0,1,2,3,4], 1: [5,6,7,8,9], 2: [10,11,12,13,9] };

// --- cavern sets: DrawBackground_UpdateBackgroundStyles (Main.cs:
// 53812-54015). 7 slots; [5] = 125+hell, [6] = 185+hell appended for ALL
// styles (:53996-54008). worldID mods pick the deep-cavern row.
function cavernSet(style, ice, jungle, hell, worldID) {
  let a;
  switch (style) {
    case 0: a = [1,2,4,3,6,5]; break;
    case 1: a = ice === 0 ? [40,33,34,32] : ice === 1 ? [160,118,161,117]
      : ice === 2 ? [164,165,166,167] : [162,120,163,119];
      a.push(128 + hell); break;
    case 2: a = [62,63,64,65,143 + hell]; break;
    case 3: a = [66,67,68,69,128 + hell]; break;
    case 4: a = [70,71,68,72,128 + hell]; break;
    case 5: a = [73,74,75,76,131 + hell]; break;
    case 6: a = [77,78,79,80,134 + hell]; break;
    case 7: a = [77,81,79,82,134 + hell]; break;
    case 8: a = [83,84,85,86,137 + hell]; break;
    case 9: a = [83,87,88,89,137 + hell]; break;
    case 10: a = [121,122,123,124,140 + hell]; break;
    case 11: a = jungle === 0 ? [153,147,148,149,150 + hell]
      : [146,154,155,156,157 + hell]; break;
    case 12: a = [66,67,68,193 + worldID % 4,128 + hell]; break;
    case 13: a = [66,67,68,188 + worldID % 5,128 + hell]; break;
    case 14: a = [66,67,68,197 + worldID % 3,128 + hell]; break;
    case 15: a = [40,33,34,200,128 + hell]; break;
    case 16: a = [40,33,34,201 + worldID % 2,128 + hell]; break;
    case 17: a = [40,33,34,203 + worldID % 4,128 + hell]; break;
    case 18: a = [290,291]; break;
    case 19: a = [292,293]; break;
    case 20: a = [294,295]; break;
    case 21: a = [296,297]; break;
    default: a = [1,2,4,3,6,5];
  }
  if (style !== 0) a.push(125 + hell, 185 + hell);
  return a;
}

// The fixed always/conditionally-loaded ids OUTSIDE the style tables:
// 0/49 eager (AssetInitializer.cs:422-423); 112/113 cloudBG (Main.cs:
// 66676, loaded :59376-59377); 18/19 hallow shimmer (:59487-59488);
// 24/25/42 far mountains (:59858/:59914/:59931); 246/247 hallow-3 hills
// (:59876/:59937); 23/22 desert mountains (:59852/:59908, skipped at
// desert 4); 26/27 GoodEvilDesert fallback (:60885/:60907).
const FIXED_ALWAYS = [0, 49, 112, 113];
const FIXED_CONDITIONAL = [18, 19, 24, 25, 42, 246, 247, 26, 27];

const bgName = id => 'Background_' + id;
const uwName = n => 'Backgrounds/Underworld ' + n;

const fn = new Function('require', '__dirname', 'pushed', 'D40_IDS', 'D35_DEFERRED',
  'FOREST', 'forestSet', 'CORRUPT', 'JUNGLE', 'SNOW', 'HALLOW', 'CRIMSON',
  'DESERT', 'OCEAN', 'MUSHROOM', 'UNDERWORLD', 'cavernSet',
  'FIXED_ALWAYS', 'FIXED_CONDITIONAL', 'bgName', 'uwName', head + `
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
  const boundaryTwoPage = (sheets, cap) => {
    const full = skylinePack(sheets, 'waste');
    const cur0 = [], cur1 = [], keep = [];
    for (let i = 0; i < sheets.length; i++) {
      const p = full.layout[i], s = sheets[i];
      if (p.y + s.h <= cap - PAD) { cur0.push(s); keep.push(p); }
      else cur1.push(s);
    }
    const h0 = keep.length ? Math.max(...keep.map((p, k) => p.y + cur0[k].h)) + PAD : PAD;
    const r0 = { height: h0, layout: keep };
    const r1 = cur1.length ? skylinePack(cur1, 'waste') : { height: 0, layout: [] };
    return { h0, h1: r1.height, n0: cur0.length, n1: cur1.length, r0, r1, cur0, cur1 };
  };
  // N-page boundary recursion: the merger contract asserts EVERY page
  // <= 8192 and recurses the tail split (research/113 section 2).
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
      const r = { height: h, layout: keep };
      verify(cur, r);
      pages.push({ height: h, n: cur.length });
      rest = tail;
    }
    return pages;
  };
  const areaOf = names => {
    let a = 0, n = 0, missing = [];
    for (const nm of new Set(names)) {
      const d = sheetDims(nm);
      if (!d) { missing.push(nm); continue; }
      a += (d.w + PAD) * (d.h + PAD); n++;
    }
    return { sheets: n, areaMpx: +(a / 1e6).toFixed(3), missing };
  };

  // ---- 1. the full inventory: every Background_0..343 + Underworld 0..13
  const inv = [];
  for (let id = 0; id < 344; id++) {
    const d = sheetDims(bgName(id));
    inv.push({ name: bgName(id), id, w: d ? d.w : null, h: d ? d.h : null });
  }
  for (let n = 0; n < 14; n++) {
    const d = sheetDims(uwName(n));
    inv.push({ name: uwName(n), id: 'U' + n, w: d ? d.w : null, h: d ? d.h : null });
  }
  const invMissing = inv.filter(r => r.w === null).map(r => r.name);
  const invArea = inv.reduce((a, r) => a + (r.w ? (r.w + PAD) * (r.h + PAD) : 0), 0);

  // ---- 2. per-style tables with real dims (the doc appendix)
  const styleRows = [];
  const pushStyle = (family, style, names) => {
    const uniq = [...new Set(names.filter(n => n !== null))];
    const a = areaOf(uniq);
    const dims = uniq.map(nm => { const d = sheetDims(nm); return nm + (d ? ' ' + d.w + 'x' + d.h : ' MISSING'); });
    styleRows.push({ family, style, sheets: a.sheets, areaMpx: a.areaMpx, names: uniq, dims, missing: a.missing });
  };
  for (const s of Object.keys(FOREST)) pushStyle('forest', +s, forestSet(+s).filter(i => i >= 0).map(bgName));
  for (const s of Object.keys(CORRUPT)) pushStyle('corrupt', +s, CORRUPT[s].filter(i => i >= 0).map(bgName));
  for (const s of Object.keys(JUNGLE)) pushStyle('jungle', +s, JUNGLE[s].filter(i => i >= 0).map(bgName));
  for (const s of Object.keys(SNOW)) pushStyle('snow', +s, [...SNOW[s].mnt, ...SNOW[s].bg].filter(i => i >= 0).map(bgName));
  for (const s of Object.keys(HALLOW)) pushStyle('hallow', +s, HALLOW[s].filter(i => i >= 0).map(bgName));
  for (const s of Object.keys(CRIMSON)) pushStyle('crimson', +s, CRIMSON[s].filter(i => i >= 0).map(bgName));
  for (const s of Object.keys(DESERT)) pushStyle('desert', +s, Object.values(DESERT[s]).flat().filter(i => i >= 0).map(bgName));
  for (const s of Object.keys(OCEAN)) pushStyle('ocean', +s, [bgName(OCEAN[s])]);
  for (const s of Object.keys(MUSHROOM)) pushStyle('mushroom', +s, MUSHROOM[s].filter(i => i >= 0).map(bgName));
  for (const s of Object.keys(UNDERWORLD)) pushStyle('underworld', +s, UNDERWORLD[s].map(uwName));
  // cavern: one row per style at ice=0/jungle=0/hell=0/worldID=0 (the
  // variant axes are enumerated separately below).
  for (let s = 0; s <= 21; s++) pushStyle('cavern', s, cavernSet(s, 0, 0, 0, 0).filter(i => i >= 0).map(bgName));
  for (let ice = 1; ice <= 3; ice++) pushStyle('cavern-ice', ice, cavernSet(1, ice, 0, 0, 0).filter(i => i >= 0).map(bgName));
  pushStyle('cavern-jungle', 1, cavernSet(11, 0, 1, 0, 0).filter(i => i >= 0).map(bgName));

  // ---- 3. the merged-W4 worst-case base (font + pushed + d40 + deferred)
  const union = [...new Set(Object.values(pushed).flat())];
  const base = buildSheetList({ font: true });
  let merged = addNew(base, union);
  merged = addNew(merged, D40_IDS.map(id => 'NPC_' + id));
  merged = addNew(merged, D35_DEFERRED.map(id => 'NPC_' + id));
  const baseNames = new Set(base.map(s => s.name));

  // world-set builders. A "world" = the rolled styles of every family.
  // The port packs the ROLLED set (a superset of what the C# would have
  // loaded at any one instant: LoadBackground fires per drawn layer).
  const surfaceWorld = (r) => {
    const ids = [];
    for (const f of r.forest) ids.push(...forestSet(f));
    ids.push(...CORRUPT[r.corrupt], ...JUNGLE[r.jungle],
      ...SNOW[r.snow].mnt, ...SNOW[r.snow].bg,
      ...HALLOW[r.hallow], ...CRIMSON[r.crimson],
      ...Object.values(DESERT[r.desert]).flat(),
      OCEAN[r.ocean], ...MUSHROOM[r.mushroom]);
    // conditional fixed ids: 18/19 + 246/247 ride hallow presence/3;
    // 23/22 skip at desert 4; 26/27 ride the GoodEvilDesert fallback
    // (non-variant desert); 24/25/42 always reachable far rows.
    ids.push(...FIXED_ALWAYS, 24, 25, 42, 18, 19);
    if (r.hallow === 3) ids.push(246, 247);
    if (r.desert < 51) ids.push(26, 27);
    return { bg: ids.filter(i => i >= 0).map(bgName), uw: UNDERWORLD[r.underworld].map(uwName) };
  };
  const cavernWorld = (c) => {
    const ids = [];
    for (const s of c.bands) ids.push(...cavernSet(s + 3, c.ice, c.jungle, c.hell, c.worldID));
    ids.push(...cavernSet(1, c.ice, c.jungle, c.hell, c.worldID));   // snow cave
    ids.push(...cavernSet(11, c.ice, c.jungle, c.hell, c.worldID));  // jungle cave
    ids.push(...cavernSet(2, c.ice, c.jungle, c.hell, c.worldID));   // glowshroom
    for (const s of c.extra) ids.push(...cavernSet(s, c.ice, c.jungle, c.hell, c.worldID));
    ids.push(...cavernSet(0, 0, 0, 0, 0)); // the boot style-0 set (5 incl.)
    return ids.filter(i => i >= 0).map(bgName);
  };

  // g71's seed-42 rolled world (research/92-doc-sweep-bg section 2, the
  // independent Python xorshift64 reference): forest [2,7,51,11],
  // corrupt 51, jungle 5, snow 5, hallow 4, crimson 4, desert 4, ocean 2,
  // mushroom 2, underworld 2; caveBackStyle [1,4,3,0], ice 3, hell 2,
  // jungle 0. worldID mod 0 assumed (the deep rows' worst-area variant
  // is covered by the worst-case scenario). Non-drunk hardmode corrupt
  // world: beach 18/19/20, deep 12/14/15/17 reachable.
  const seed42surface = { forest: [2, 7, 51, 11], corrupt: 51, jungle: 5, snow: 5,
    hallow: 4, crimson: 4, desert: 4, ocean: 2, mushroom: 2, underworld: 2 };
  const seed42cavern = { bands: [1, 4, 3, 0], ice: 3, hell: 2, jungle: 0, worldID: 0,
    extra: [18, 19, 20, 12, 14, 15, 17] };

  // ---- 4. scenarios
  const outFile = path.join(__dirname, '_d35_bg_inventory.json');
  let results = [], done = new Set();
  try {
    const prev = JSON.parse(fs.readFileSync(outFile, 'utf8'));
    results = prev.scenarios || [];
    for (const r of results) done.add(r.scenario);
  } catch (e) {}
  const save = () => fs.writeFileSync(outFile, JSON.stringify({
    inventory: inv, inventoryMissing: invMissing,
    inventoryAreaMpx: +(invArea / 1e6).toFixed(2),
    inventoryAreaBound: Math.ceil(invArea / (MAX_WIDTH - PAD)) + PAD,
    styleRows, scenarios: results }, null, 1));

  const measure = (label, sheets, note) => {
    if (done.has(label)) { console.log(label + ': (cached)'); return; }
    const sky = skylinePack(sheets, 'waste');
    const sky2 = skylinePack(sheets, 'waste');
    verify(sheets, sky);
    if (JSON.stringify(sky.layout) !== JSON.stringify(sky2.layout)) throw new Error('nondeterministic: ' + label);
    const area = sheets.reduce((a, s) => a + (s.w + PAD) * (s.h + PAD), 0);
    const row = { scenario: label, note: note || '', sheets: sheets.length,
      missing: sheets.missing || [],
      areaMpx: +(area / 1e6).toFixed(2),
      areaBound: Math.ceil(area / (MAX_WIDTH - PAD)) + PAD,
      skyline: sky.height, skyOver: Math.max(0, sky.height - 8192) };
    const pages = boundaryNPages(sheets, 8192);
    row.pages = pages;
    console.log(label + ': ' + sheets.length + ' sheets, area ' + row.areaMpx +
      ' Mpx (bound ' + row.areaBound + ') | skyline ' + sky.height +
      (row.skyOver ? ' OVER+' + row.skyOver : '') +
      ' | pages ' + pages.map(p => p.height + '(' + p.n + ')').join(' / '));
    if (row.missing.length) console.log('  missing: ' + row.missing.join(', '));
    results.push(row); save();
  };

  measure('merged-base', merged, 'font + pushed(339) + d40(56) + d35deferred(12) - the research/113 worst case');

  // g71 table reproduction ON THE MERGED BASE (their baseline was 7847
  // on the g71 branch asset set; ours is the 9904-sheet merged wave).
  const g71adds = [
    ['corrupt4', CORRUPT[4].map(bgName)],
    ['snow7', [...SNOW[7].mnt, ...SNOW[7].bg].map(bgName)],
    ['hallow5', HALLOW[5].map(bgName)],
    ['snow5', [...SNOW[5].mnt, ...SNOW[5].bg].filter(i => i >= 0).map(bgName)],
    ['jungle6', JUNGLE[6].map(bgName)],
    ['crimson6', CRIMSON[6].map(bgName)],
    ['desert51', Object.values(DESERT[51]).flat().filter(i => i >= 0).map(bgName)],
    ['desert4', DESERT[4].Pure.map(bgName)],
    ['mushroom4', MUSHROOM[4].map(bgName)],
    ['ocean5', [bgName(OCEAN[5])]],
  ];
  for (const [tag, names] of g71adds) {
    const fresh = names.filter(n => !baseNames.has(n));
    measure('merged+' + tag, addNew(merged, names),
      'g71 single-style add-on: +' + fresh.length + ' new sheets (' + fresh.join(',') + ')');
  }
  // g71's one-each rolled world (their 12345 px scenario): the 8 biome
  // styles they measured, surface only.
  const oneEach = [].concat(CORRUPT[4], [...SNOW[7].mnt, ...SNOW[7].bg], HALLOW[5],
    JUNGLE[6], CRIMSON[6], Object.values(DESERT[51]).flat(), MUSHROOM[4], [OCEAN[5]])
    .filter(i => i >= 0).map(bgName);
  measure('merged+one-each-8biomes', addNew(merged, oneEach), 'g71 one-each rolled world, surface styles only');

  // the seed-42 realistic world: surface + cavern + underworld.
  const s42 = surfaceWorld(seed42surface);
  const c42 = cavernWorld(seed42cavern);
  measure('merged+seed42-surface', addNew(merged, [...s42.bg, ...s42.uw]),
    'g71 seed-42 rolls, surface + underworld, no cavern machine');
  measure('merged+seed42-full', addNew(merged, [...s42.bg, ...s42.uw, ...c42]),
    'g71 seed-42 rolls + the full per-world cavern set (bands/ice/jungle/glowshroom/beach/deep)');

  // worst-case realistic world: heaviest style per family (padded area
  // of the sheets NOT already in the base), 4 distinct forest styles,
  // 4 distinct cave bands, every reachable beach/deep row, hell rows.
  const weight = names => names.filter(n => !baseNames.has(n))
    .reduce((a, n) => { const d = sheetDims(n); return a + (d ? (d.w + PAD) * (d.h + PAD) : 0); }, 0);
  const heaviest = (table, toNames) => Object.keys(table)
    .map(s => ({ s: +s, w: weight(toNames(+s)) }))
    .sort((a, b) => b.w - a.w || a.s - b.s);
  const fOrder = heaviest(FOREST, s => forestSet(s).filter(i => i >= 0).map(bgName));
  const worstSurface = {
    forest: fOrder.slice(0, 4).map(r => r.s),
    corrupt: heaviest(CORRUPT, s => CORRUPT[s].map(bgName))[0].s,
    jungle: heaviest(JUNGLE, s => JUNGLE[s].map(bgName))[0].s,
    snow: heaviest(SNOW, s => [...SNOW[s].mnt, ...SNOW[s].bg].filter(i => i >= 0).map(bgName))[0].s,
    hallow: heaviest(HALLOW, s => HALLOW[s].filter(i => i >= 0).map(bgName))[0].s,
    crimson: heaviest(CRIMSON, s => CRIMSON[s].filter(i => i >= 0).map(bgName))[0].s,
    desert: heaviest(DESERT, s => Object.values(DESERT[s]).flat().filter(i => i >= 0).map(bgName))[0].s,
    ocean: heaviest(OCEAN, s => [bgName(OCEAN[s])])[0].s,
    mushroom: heaviest(MUSHROOM, s => MUSHROOM[s].map(bgName))[0].s,
    underworld: heaviest(UNDERWORLD, s => UNDERWORLD[s].map(uwName))[0].s,
  };
  const bandOrder = [0,1,2,3,4,5,6,7].map(b => ({ b, w: weight(cavernSet(b + 3, 3, 1, 2, 0).filter(i => i >= 0).map(bgName)) }))
    .sort((a, b) => b.w - a.w || a.b - b.b);
  const worstCavern = { bands: bandOrder.slice(0, 4).map(r => r.b), ice: 3, hell: 2, jungle: 1,
    worldID: 0, extra: [18, 19, 20, 21, 12, 13, 14, 15, 16, 17] };
  const ws = surfaceWorld(worstSurface);
  const wc = cavernWorld(worstCavern);
  measure('merged+worst-world', addNew(merged, [...ws.bg, ...ws.uw, ...wc]),
    'heaviest rolled style per family: forest ' + JSON.stringify(worstSurface.forest) +
    ' corrupt ' + worstSurface.corrupt + ' jungle ' + worstSurface.jungle + ' snow ' + worstSurface.snow +
    ' hallow ' + worstSurface.hallow + ' crimson ' + worstSurface.crimson + ' desert ' + worstSurface.desert +
    ' ocean ' + worstSurface.ocean + ' mushroom ' + worstSurface.mushroom + ' uw ' + worstSurface.underworld +
    '; bands ' + JSON.stringify(worstCavern.bands) + ' + all beach/deep rows (drunk hardmode upper bound)');

  // underworld alone on the merged base (g72's 8424 blocker context).
  for (const s of [0, 1, 2]) {
    measure('merged+underworld' + s, addNew(merged, UNDERWORLD[s].map(uwName)),
      'g72 setBG(9) style ' + s + ' alone');
  }
  measure('merged+underworld-all', addNew(merged, [0,1,2].flatMap(s => UNDERWORLD[s]).map(uwName)),
    'all three underworld styles (14 distinct sheets, 9 shared)');

  // the FULL-VARIANT upper bound: every surface style + every cavern
  // style/axis + all underworld - what an all-407 loader would pack.
  const allNames = [];
  for (const s of Object.keys(FOREST)) allNames.push(...forestSet(+s).filter(i => i >= 0).map(bgName));
  for (const t of [CORRUPT, JUNGLE, HALLOW, CRIMSON, MUSHROOM])
    for (const s of Object.keys(t)) allNames.push(...t[s].filter(i => i >= 0).map(bgName));
  for (const s of Object.keys(SNOW)) allNames.push(...[...SNOW[s].mnt, ...SNOW[s].bg].filter(i => i >= 0).map(bgName));
  for (const s of Object.keys(DESERT)) allNames.push(...Object.values(DESERT[s]).flat().filter(i => i >= 0).map(bgName));
  for (const s of Object.keys(OCEAN)) allNames.push(bgName(OCEAN[s]));
  for (let s = 0; s <= 21; s++)
    for (const ice of [0,1,2,3]) for (const jungle of [0,1]) for (const hell of [0,1,2])
      allNames.push(...cavernSet(s, ice, jungle, hell, 0).filter(i => i >= 0).map(bgName));
  // worldID-mod rows: every reachable remainder
  for (const [st, mod] of [[12,4],[13,5],[14,3],[16,2],[17,4]])
    for (let m = 0; m < mod; m++)
      allNames.push(...cavernSet(st, 0, 0, 0, m).filter(i => i >= 0).map(bgName));
  allNames.push(...[0,1,2].flatMap(s => UNDERWORLD[s]).map(uwName));
  allNames.push(...FIXED_ALWAYS.map(bgName), ...FIXED_CONDITIONAL.map(bgName));
  measure('merged+ALL-VARIANTS', addNew(merged, allNames),
    'every style of every family + all cavern axes + all underworld - the load-everything upper bound');

  // the inventory alone (no merged base): how big is the bg corpus itself
  const invSheets = [];
  for (const r of inv) if (r.w) invSheets.push({ name: r.name, w: r.w, h: r.h });
  measure('bg-inventory-alone', invSheets, 'all 344 Background_* + 14 Underworld sheets, nothing else');

  // the bg-styles PAGE form: merged base stays on pages 0/1 (the 113
  // boundary split), the world's rolled bg set packs into its OWN page
  // (page 2), split recursively if it ever exceeds 8192. This is the
  // load-on-demand alternative to mixing bg sheets into the union pack.
  const pageForm = (label, bgNames, note) => {
    if (done.has(label)) { console.log(label + ': (cached)'); return; }
    const fresh = [...new Set(bgNames)].filter(n => !merged.some(s => s.name === n));
    const bgSheets = [];
    const missing = [];
    for (const n of fresh) {
      const d = sheetDims(n);
      if (!d) { missing.push(n); continue; }
      bgSheets.push({ name: n, w: d.w, h: d.h });
    }
    const bgPages = bgSheets.length ? boundaryNPages(bgSheets, 8192) : [];
    const area = bgSheets.reduce((a, s) => a + (s.w + PAD) * (s.h + PAD), 0);
    const row = { scenario: label, note, sheets: merged.length + bgSheets.length,
      newBgSheets: bgSheets.length, missing,
      areaMpx: +((74.72e6 + area) / 1e6).toFixed(2),
      basePages: [{ height: 8192, n: 963 }, { height: 1298, n: 8941 }],
      bgPages, totalPages: 2 + bgPages.length,
      bgAreaMpx: +(area / 1e6).toFixed(2) };
    console.log(label + ': +' + bgSheets.length + ' bg sheets (' + row.bgAreaMpx +
      ' Mpx) on ' + bgPages.length + ' own page(s): ' +
      bgPages.map(p => p.height + '(' + p.n + ')').join(' / ') +
      ' | total pages ' + row.totalPages);
    if (missing.length) console.log('  missing: ' + missing.join(', '));
    results.push(row); save();
  };
  pageForm('pageform+seed42-full', [...s42.bg, ...s42.uw, ...c42],
    'seed-42 world bg set on its own page(s), merged base untouched on pages 0/1');
  pageForm('pageform+worst-world', [...ws.bg, ...ws.uw, ...wc],
    'worst-case world bg set on its own page(s)');
  for (const s of [0, 1, 2]) {
    pageForm('pageform+underworld' + s, UNDERWORLD[s].map(uwName),
      'g72 style-' + s + ' underworld set on its own page');
  }
  pageForm('pageform+ALL-VARIANTS', allNames,
    'the load-everything upper bound on its own pages (NOT the contract)');

  save();
  return results;
`);
const results = fn(require, __dirname, pushed, D40_IDS, D35_DEFERRED,
  FOREST, forestSet, CORRUPT, JUNGLE, SNOW, HALLOW, CRIMSON,
  DESERT, OCEAN, MUSHROOM, UNDERWORLD, cavernSet,
  FIXED_ALWAYS, FIXED_CONDITIONAL, bgName, uwName);
console.log('wrote tools/_d35_bg_inventory.json (' + results.length + ' scenarios)');
