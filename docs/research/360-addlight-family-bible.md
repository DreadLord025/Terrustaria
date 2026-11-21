# 360 — THE ADDLIGHT-FAMILY BIBLE (W4-g18 SEAL): the git-log census, the d54-bible reconcile, the c29 gems-premise refutation, the e6 compose, the family declaration

> W4-g18 · 2026-11-21 · code-only (no cargo/tests/rustfmt/graphify run) ·
> branch `task/W4-g18-addlight-main-1`, base `a725acb04` (main at claim).
> Number **360**: registry-150 (`origin/task/W4-d54-dust-sites-world`,
> `docs/research/150-research-number-registry.md`) §8.7 AMENDMENT-3 FROZEN
> TABLE — "New docs take ≥360 after a registry check"; the §8/§8.7 blocks
> consume 151–351 (winners 237–245 disjoint). A fresh 362-tip `ls-tree`
> sweep at this seal finds NO `docs/research/` file ≥352 at any tip and no
> `research/360` ref; 360 is the first free number ≥360. NO self-pick into
> any reserved block (the §8.7 enforcement rule).
>
> This is the g18 lane's SEAL: the Main.cs batch-1 claim/audit/oracle
> (`547154708`/`a99dfa964`/`22d529c5c`) is unchanged; this round adds the
> family-wide census + the four cross-lane receipts + the bible. Witnesses
> WRITTEN NOT RUN.

## 1. The git-log census — what landed (every AddLight lane, at its pushed tip)

The S1-E-5 wave-4 AddLight family. Tips re-fetched at this seal; every
accounting quoted from the lane's OWN final record (not re-derived from
commit messages). "Covered" = ported + audit-landed + cited-with-a-port;
"absent" = no port (unrowed kind / rand-gated / cited skip).

| Lane | Branch tip | File(s) | Sites | Final accounting | Witnesses |
| --- | --- | --- | --- | --- | --- |
| **c19** proj first wave | `5400e0374` | `proj_lights_a.rs` + sim arms (`ai_typhoon`/`ai_truffle_spore`/`whip_dust_lights` → `drain_ai_lights`) | 1-67 (`:16655`…`:54071`) | **61 ported (57 feed + 4 sim) / 67 contributions / 2 audit-landed / 4 absent** (round-4 SEAL; 465/731/phaseblade-pair/756 task-seeds) | 45 (35 `proj_lights_a` + 9 `projectile.rs` + 1 `world_sim`) |
| **e6** proj second wave | `c8b072b69` | `proj_ai_lights.rs` | 68-107 (`:54173`…`:78556`) | **20 ported / 23 contributions / 20 absent** (19 unrowed kinds + the `:78556` rand-gated electrification) | 15 |
| **f7** proj census close | `67110ccfb` | `tests/proj_addlight_census.rs` | the 107-site freeze | remainder EMPTY (67+40=107); the verbatim 107-row table + 51 structure pins + the negative census (FloodFillTile 2 / PlotTileLine 44 / CastLight 40 / SpreadLight 1) | 5 |
| **c20** NPC first half | `7f68a69d1` | `npc_lights_a.rs` | 1-40 (`:21307`…`:48641`) | **all 40 landed** (FED / TWIN / DEDUP dispositions; the TWINs feed once at their aiStyle wiring) | 27 |
| **e7** NPC second half + Player gap | `c59d68bf6` | `npc_lights_b.rs` + `player_lights_b.rs` + `snapping_stone.rs` | NPC 41-80 (`:49929`…`:92412`); Player 46 | NPC: 34 ported + Horseman-67 audit-landed + 6 cited (43 Hungry-II-116 no-kind, 61/62 ore/herb, 76/77/78 CastLights `Next(28,42)` flicker). Player: 32 landed + 10 gap-batch + 3 Snapping-Stone (`5ae32127e`) = 45 covered, wing-27 `:36026` cited (f3 stealth producer) | 17 NPC + 18 Player + 7 Snapping + 3 oscillator |
| **f8** NPC indirect casts | `8ce3395b8` | `npc_lights_c.rs::dd2_body_cast_light` | the 2 INDIRECT `PlotTileLine(Top,Bottom,width,CastLightOpen)` (`:63543` AI_107 566/567, `:65829` AI_109 564/565) | both ported (one shared fn; the NPC-side CastLightOpen copy — merger keeps ONE with e8's delegate) | 8 |
| **e8** tile-draw family | `58606f3a4` | `tile_lights.rs` | Main.cs 5 (`:29016/29020/29024/29028/:29186`) + TileDrawing 6 (`:8127/8132/8172/8177/8220/8225`) + the 5 `DelegateMethods.CastLight*`/`SpreadLight` delegates | all ported (the 332/488 hook chains, the mushroom-tree flicker `treeStyle==14`, the five casts' kept quirks) | 11 |
| **f9** Main census close | `c27407842` | (docs) | Main.cs 5 | remainder EMPTY (third tile batch); the five-site verdict g18/g19 re-derive | — |
| **g18** Main batch 1 (THIS) | `22d529c5c` | `tests/main_addlight_batch1.rs` | Main.cs sites 1-4 (the 332 chain) | distrust-verbatim audit (all faithful in e8's port) + the f32 behavioural oracle; NO duplicate port | 5 |
| **g19** Main batch 2 | `9733c82a7` | `tests/main_addlight_census.rs` | Main.cs 5 (whole-file freeze) | the census freeze (5 sites + the negative light-write census + the 332/488 arm pins + the Lighting.cs funnel) | 5 |
| **g20** item shimmer | `b43297746` | `entity_lights.rs::shimmer_light` | WorldItem.cs `:482` | the item family's last RNG-free site (69 total: 68 in `UpdateItem_VisualEffects` landed S1-E-5 + `:482` here) | 2 |
| **d50** dust type-table | `e84e6bd76` | `dust_physics.rs` (chain) | Dust.cs physics | the 94-site chain's physics mutations (the lights interleave; the feed is post-update reads) | — |
| **g13** dust second-chain lights | `3fe509158` | `dust_lights_b.rs` | Dust.cs 52 (`:1061`…`:2312`) | **52 ported + WIRED** (`feed_dust_chain_lights`); with main's `dust_lights.rs` 42 = **94/94** (research/308 §3 fresh census) | 23 + 1 SEAL |

**The pre-wave landed base (origin/main `a725acb04`):** `entity_lights.rs`
(the ONE `EntityLights` pair ≈ `LightingEngine._perFrameLights`, 9 witnesses),
`proj_lights.rs` (the ProjLight chain, 12), `npc_lights.rs` (the aiStyle-8
tail, 6), `player_lights.rs` (32 Player sites, 18), `item_lights.rs` (the
rand/essScale head, 5), `dust_lights.rs` (the 42 FIRST-ladder arms, 15).
These are the S1-E-5 first-wave landed feeds the wave-4 lanes extend.

## 2. The d54-bible reconcile — AddLight 94/94 + proj-107 + NPC + Main

The d54 master bible (`origin/task/W4-d54-dust-sites-world`
`docs/research/131-dust-master-status.md`, tip `1c462c4fe`) is the
cross-lane authority. Its AddLight rows, re-verified at this seal:

* **Dust.cs 94/94.** 131 §7.2's G1 row flips CLOSED (g13 `a234107ac`):
  "dust_lights_b.rs 52-arm collector WIRED via feed_dust_chain_lights in
  draw.rs; 23 witnesses; 94/94 family (research/69 §7)". The s8.2
  family-complete line: "d50 chain + d54 table incl. G3/G4 + g13 lights_b
  (52) + main dust_lights (42) = **94/94**". g13's SEAL (research/308 §3,
  tip `3fe509158`) re-counts FRESH from Dust.cs: **94 exactly** — 42
  FIRST-ladder (`:485`…`:1046`) + 52 SECOND-chain (`:1061`…`:2312`), the
  `w4_g13_seal_the_52_line_census_covers_every_addlight_site` witness.
  The `:2275` customData-Color second light is **LIVE in C#** (d54 G5
  CORRECTED `c5a0e646`: `ParticleOrchestrator.cs:1171/1184/1199`
  `SpawnLightningExplosionDust` sets `customData = dust.color` with NO
  `noLightEmittance`; `Projectile.cs:39844` + `StormLightningParticle.cs:215`
  double-set `noLight = (noLightEmittance = true)` so THEY never fire it).
  The port drops it wholesale (the `Dust` record has no customData) — a
  CITED gap, not an uncounted site; the merger reconstructs from
  `num83 * d.color.rgb / 255` gated `!d.no_light_emittance` (at EVERY Color
  producer `customData == dust.color`, CloneDust copies both `Dust.cs:280/:284`).
* **Projectile.cs 107 — 86 covered / 21 absent (rand-gated).** The family
  is CLOSED (c19 67 + e6 40 = 107, the f7 freeze). The **86/21** is the
  family-level MERGE-STATE census (the d54-bible cross-lane view), derived
  here from the lanes' final registers + the pushed row-activations:
  - c19 round-4 SEAL: 61 ported + 2 audit-landed = **63 covered**, 4 absent
    (465/731/phaseblade-pair/756 — task-seed rows, research/189).
  - e6: 20 ported, 20 absent (19 unrowed kinds + the `:78556` rand gate).
  - **6 of e6's 19 unrowed kinds are NOW ROWED on pushed lanes** (verified
    by `git grep '<CONST> =>' <tip> -- projectile/data.rs` at this seal):
    435 MartianTurretBolt → c11 `data.rs:1924`; 576 NebulaLaser → d11
    `data.rs:2517`; 606 ScutlixLaser → d8 `data.rs:1772` (the research/97 §5
    activation cross-note); 707 MonkStaffT3 → d14 `data.rs:2664` / d15
    `:4066`; 708 MonkStaffT3_Alt → d15 `:4072`; 709 MonkStaffT3_AltShot →
    d15 `:4081`. Each activates its e6 site at the merge → **+6 covered**.
  - **86 covered = 63 (c19) + 20 (e6) + 6 (activations); 21 absent = 4
    (c19 task-seeds) + 13 (e6 still-unrowed: 467/468/440/593/592/585/577/
    682/684/312/324/595/735) + 1 (e6 `:78556` rand-gated).** 86+21=107. ✓
  - The c19 round-4 activations (502 Meowmere → d75 `b59e3a874`, 1092
    LibrarianSkeletonBook → g5 `96f074959`, 466 CultistBossLightningOrbArc →
    d53 `c0dc7d37f`) are ALREADY in c19's 63; they are NOT double-counted.
  - **The e6 activation obligation:** e6's `proj_ai_lights` is an if-chain
    over the 20 ported-kind consts ONLY — the 6 now-rowed kinds have NO
    code arm (register-only "no row. Absent."). Unlike c19's row-gated
    collector (which auto-activates), e6 needs the 6 sub-ifs ADDED at the
    merge (the d8-606 contract: "proj_ai_lights.rs gains the 606 sub-if
    (AddLight(Center, 0.7, 0.3, 0.3), the Vector2 quotient-truncation
    anchor), the f7 census test's 'absent' assertion for 606 flips to
    'present'"). The 6 sites' colours/anchors are pinned in the e6 register
    + the f7 freeze; the rows stay the lanes' to merge (keep-one).
* **NPC.cs 80.** c20 (1-40) + e7 (41-80) + f8 (the 2 indirect casts). The
  80 direct sites: 74 covered (c20 40 + e7 34) + Horseman-67 audit-landed +
  6 cited skips (43/61/62/76/77/78); the 3 CastLights flicker arms
  (`:92382/:92395/:92404`, type 160 / 254-261+634+635 / the third) draw
  `Next(28,42)` UNCONDITIONALLY per tick — cited until a sim-side feed
  lands (the e7 RNG rule). f8's 2 indirect casts are the light-map-WRITE
  layer ported as EntityLights walks (the PlotLine end-point-drop +
  right-first quirks kept).
* **Main.cs 5.** e8 ports all five (the 332/488 hook chains); g18 audits
  sites 1-4 + the oracle; g19 freezes the whole file; f9 closes the
  remainder EMPTY. **NO update-side/ambient AddLight in Main.cs** — tile
  ambience is scanner-seeded (`TileLightScanner`, `tr-core/tile_light`),
  the laser/hook tile-cast delegates live in `DelegateMethods.cs`/`Utils.cs`
  (e8 scope), the mushroom-tree flicker is TileDrawing's (e8). The g18
  negative census (seven light-write forms all 0) re-verified at this seal.

## 3. The c29 gems-NEVER-emit-light premise — REFUTED (re-confirmed)

The c29 SEAL (`origin/task/W4-c29-deadmans-chest-biome` `f0643602d`,
research/260 Item 1 + the gem-family bible) refutes the "gem tiles 63-68
emit light via AddLight/tileLighted" premise. Re-derived independently
here against the C# (direct-path ≤100-line windows, no junction recursion):

1. **No `tileLighted` write for gems.** `Main.tileLighted[]` is written for
   ~100 tile ids (Main.cs:7066-8861+); a whole-tree grep for
   `tileLighted[(63|64|65|66|67|68|178|566|211)]` returns **ZERO** matches.
   The gem ids are absent from the lighted set.
2. **The gem sparkle is DUST, not light.** `TileDrawing.DrawTiles_EmitParticles`'
   `Main.tileShine` arm (`TileDrawing.cs:7413-7529`): the 178 special
   (`:7453-7482`) picks the colour by `tileFrameX / 18` (0 magenta, 1|6
   yellow, 2 blue, 3 green, 4 red, 5 white) then
   `Dust.NewDust(new Vector2(i*16, j*16), 16, 16, 43, 0f, 0f, 254, newColor, 0.5f)`
   + `velocity *= 0f` (`:7479-7480`). The gem-stone colours (`:7483-7506`):
   63 blue, 64 red, 65 green, 66 yellow, 67 magenta, 68 white. **Dust type
   43, no AddLight.** The shine table: `tileShine[178]=500` (Main.cs:7516),
   `[63..68]=900` (`:7523-7528`), `[566]=900` (`:7529`) — a sparkle CADENCE,
   not a light amount.
3. **No TileLightScanner arm for gems.** `ApplyTileLight`'s top-level
   `switch (tile.type)` (`TileLightScanner.cs:347`) has NO case 63-68/178/
   566/211 (a `^\t\t\tcase (6[3-8]|178|566|211):` grep returns zero). The
   `case 63-68` hits at `:1489/:1826/:2134/:2408/:2848` are NESTED
   frame-index sub-switches (`tile.frameY/22`, `frameY/36`,
   `frameY/54 + 37*(frameX/108)`) — chandelier/lantern STYLE rows, not tile
   kinds (the c29 note's exact clarification). The scanner's outer gate is
   `if (Main.tileLighted[tile.type])` (`:345`) — gems never enter it.
4. **The generated tables carry no gem light.** `tr-core/tiledata/sets.rs`
   `TILE_LIGHTED` bitset: tiles 63-68/178/566/211 all **false** (decoded;
   torch-4/campfire-215/chandelier-34/lantern-42 true as the sanity check).
   `emissive.rs` `EMISSIVE` rows 63-68 (`:90-95`), 178 (`:205`), 566 (`:593`)
   all `[0.0, 0.0, 0.0]`.
5. **Main.cs has no ambient AddLight** (§2 above) — the gem sparkle cannot
   be an entity-list light either.

**The g18 negative census AGREES.** g18's batch-1 negative census
(ANALYSIS "The Main.cs AddLight census, batch 1": Main.cs has NO
update-side/ambient AddLight, the seven light-write forms all 0) is the
same verdict from the Main.cs side: the gem tiles' only "glow" is the
dust-43 sparkle (d60's `tile_draw_particles.rs`, UNWIRED pending the
wiring wave) + the `tileShine` cadence — NEVER a light-map source. The
c29 gem-family bible's "gems NEVER emit light — REFUTED premise (no
tileLighted[63..68/178/566], no scanner case, no Lighting AddLight)" row
is confirmed verbatim.

## 4. The e6-addlight-proj-2 compose (mutual receipt)

c19's SEAL (research/189 "Compose receipts") already publishes the
e6-disjoint + g18-disjoint verdicts; this is the g18-side mutual receipt,
re-verified at the current tips (c19 `5400e0374`, e6 `c8b072b69`, both
fork `a725acb04`):

* **Site-disjoint by construction.** The f7 census freeze splits at
  `:54071`/`:54173`: c19's `proj_lights_a.rs` (sites 1-67) vs e6's
  `proj_ai_lights.rs` (sites 68-107) — ZERO shared kind arms (e6's
  436/634/635/459/442/449/438/350/325/819/1094/1113/613/445/663/665/667/
  691/692/693 appear in NO c19 arm). The ONE `EntityLights` pair max-merges,
  so feed-loop order does not change pixels.
* **The merge hunks (keep-both unions):**
  - `main.rs`: e6 `mod proj_ai_lights;` BEFORE `mod proj_lights;`, c19
    `mod proj_lights_a;` AFTER it — adjacent-line union (alphabetical:
    proj_ai_lights, proj_lights, proj_lights_a).
  - `app/frame/draw.rs`: BOTH feed loops insert AFTER the ProjLight feed
    at the same seam (`@@ -569`); keep BOTH, e6's first (its sites follow
    the AI order after the first wave's). c19's loop carries the
    mounted-centre + Chillet-mount + round-4 `self.disco` arg; e6's carries
    the 445 hand anchor (`rotated_relative_point`). Both pure.
  - `anchors.txt`: one row each (`fn proj_ai_lights` / `fn proj_lights_a`)
    — union, no collision.
  - `projectile.rs` / `world_sim/projectiles.rs` / `projectile/data.rs`:
    e6 touches NONE of these (its true diff is 6 files: proj_ai_lights.rs,
    draw.rs, main.rs, anchors, ANALYSIS, ROADMAP). c19's sim arms
    (`pending_ai_lights`/`drain_ai_lights`/`set_mouse_text_color` + the
    typhoon/spore/whip machines) are c19-exclusive; no e6 overlap.
* **The craft.rs 340-line "deletion" in BOTH branches' `origin/main..`
  diffs is NOT a deletion** — it is main's LATER S1-L-11g addition
  (`03711f456`, post-`a725acb04`, the craft freeze guards) absent from both
  fork trees. The merge keeps main's copy; neither branch touches craft.rs
  (`git diff a725acb04..HEAD --stat` lists no craft.rs on either). NO
  keep-one work owed on craft.rs. (g18's true diff is likewise 4 files:
  tests/main_addlight_batch1.rs, ANALYSIS, ROADMAP, anchors.)
* **The 6-kind activation obligation (§2)** is the ONE e6 merge task: add
  the 435/576/606/707/708/709 sub-ifs to `proj_ai_lights` (the rows land on
  c11/d11/d8/d14/d15), flip the f7 census "absent" assertions to "present".
  The d8-606 contract (research/97 §5) is the template; the other five
  follow the same shape (the e6 register pins each colour/anchor).
* **g18 × e6 / g18 × c19: DISJOINT.** g18 is Main.cs batch 1 (the 332-chain
  audit + oracle); its file set {tests/main_addlight_batch1.rs, ANALYSIS,
  ROADMAP, anchors} shares ONLY the three doc ledgers with c19/e6
  (append-union: g18's "Main.cs census batch 1" ANALYSIS section + its
  ROADMAP S1-E-5 tail sentence + its one anchors row; the merger
  concatenates, neither edits the other's text). Main.cs's 5 sites are
  outside Projectile.cs's 107 — NO kind/site overlap. g19 (`9733c82a7`,
  the Main census freeze) is likewise file-disjoint.

## 5. The addlight-family bible (SEAL declaration)

**ONE family, the per-frame `EntityLights` list pair, fed by the C#
`Lighting.AddLight` call sites.** The funnel: every overload funnels into
`Lighting.AddLight(int i, int j, float r, float g, float b)`
(`Lighting.cs:157-163`) behind `!Main.gamePaused && Main.netMode != 2`,
appending to the active engine's per-frame list (`LightingEngine.cs:51-54`),
max-merged onto the working light map one frame late (`:208-235`). The
anchor vocabulary: `(int)Center.X / 16` truncates the coordinate FIRST
(`XAnchor::CenterTruncated`); the `AddLight(Vector2, …)` overload
(`Lighting.cs:140-143`) truncates the QUOTIENT (`XAnchor::Center`); the two
agree for every real coordinate (the e6 anchor-agreement invariant). The
torch-id overload (`:145-154`) reads `TorchID.TorchColor`.

**The core family (the S1-E-5 wave-4 scope), per file:**

| C# file | Sites | Owner lane(s) | Port module | State |
| --- | --- | --- | --- | --- |
| `Projectile.cs` | 107 | c19 (1-67) + e6 (68-107) + f7 (freeze) | `proj_lights_a.rs` + `proj_ai_lights.rs` + sim arms | CLOSED — 86 covered / 21 absent at merge (§2) |
| `NPC.cs` | 80 (+2 indirect) | c20 (1-40) + e7 (41-80) + f8 (indirect) | `npc_lights_a.rs` + `npc_lights_b.rs` + `npc_lights_c.rs` | 74 covered + 1 audit + 6 cited; the 2 indirect casts ported |
| `Dust.cs` | 94 | main (42) + g13 (52) + d50 (chain) | `dust_lights.rs` + `dust_lights_b.rs` | **94/94 CLOSED** (the `:2275` customData light a cited reconstruct-from-d.color gap) |
| `Player.cs` | 46 | main (32) + e7 (10 gap + 3 Snapping) | `player_lights.rs` + `player_lights_b.rs` | 45 covered, wing-27 cited (f3 stealth) |
| `WorldItem.cs` | 69 | main (68) + g20 (`:482`) | `entity_lights.rs::item_light` + `shimmer_light` | 69 covered (the 116 arm a draw-free skip, the 75/BossBag dust halves with the dust pass) |
| `Main.cs` | 5 | e8 (port) + g18 (audit/oracle) + g19 (freeze) + f9 (close) | `tile_lights.rs` | 5 covered, remainder EMPTY |
| `TileDrawing.cs` | 6 | e8 | `tile_lights.rs::mushroom_tree_light` | 6 covered (the `treeStyle==14` flicker, `_rand.Next(28,42)` draw-pass stream) |
| `Item.cs` / `TileObject.cs` / `Wiring.cs` / `NetMessage.cs` / `WorldGen.cs` | 0 | e7 (census) | — | EMPTY (WorldGen's one PlotTileLine `:72966` is a CheckStopForSolids solid-probe, not a light) |

**The two feed rules (e6 precedent, all lanes):** an arm whose kind has no
`proj_data`/roster row is absent wholesale (no slot can carry it); a
rand-gated light is absent FROM THE PURE FEED. The documented exception
(c19 round 3, the `item_lights` named-stream precedent): the four
rand-gated proj sites arm IN THE SIM on the `UpdateProjectiles` stream,
queueing intents the world-sim pass drains (`drain_ai_lights`) — the
draw-pass feeds stay pure, so their placement cannot move any stream. The
NPC CastLights flicker arms (76/77/78) stay cited until a sim-side feed.

**The standing light-map-WRITE deferral (ONE task, several registers cite
it):** the 870 FloodFillTile spread (proj site 1's skipped half), the
hooks' PlotTileLine rope spreads (`:52613`/`:52644`), d13's pet casts
(`:35008-35043`, research/243 §4), the Player CastLightOpen/FloodFillTile
delegate lane (yoraiz0r-eye `:12555/:12559`, Magiluminescence
`:14561/:14562`, fairy-glowstick `:50135`), the LaserDrill `>=30f`
PlotTileLine flood half (`:66264-66271`). These write the light map
directly (not the EntityLights list); when the light-map-write task lands
it consumes all these registers together. f8's NPC DD2 casts are the ONE
exception ported (as EntityLights walks, the PlotLine quirks kept).

**The peripheral AddLight files (whole-tree census, NOT the core family —
censused here for completeness, owned by their own lanes / cited skips):**
`Mount.cs` 14 (the mount lights — the Scutlix eye-glow `:5413` d8-cited,
the rest the mount layer), `WaterfallManager.cs` 11 (the per-fall AddLight
fn `:1067-1112`, g73/g74/g75 falls family — the disco fall wired
`d118f7ba`), `Gore.cs` 4 (the gore light layer, S1-E-4 follow-up),
`DelegateMethods.cs` 5 (the CastLight*/SpreadLight delegates, e8),
`ParticleOrchestrator.cs` 4 + `StormLightningParticle.cs` 1 +
`GasParticle.cs` 1 + `BloodyExplosionParticle.cs` 1 (the particle lights,
the dust/orchestrator lanes), the 5 LeashedEntities files 8 (the leashed
critter lights — the firefly/snail/butterfly/fairy aiStyle outcomes feed
once at wiring, c20/e7 DEDUP-TWIN), `ArmorSetBonuses.cs` 1 (the Shroomite
stealth light, the f3 stealth producer), `VoidLensHelper.cs` 1 (g56/g85
void-lens), `PotionOfReturnGateHelper.cs` 1 (the potion-of-return gate).
`Lighting.cs`'s 3 `Lighting.AddLight(` tokens are the funnel's OWN
overload-delegation calls (`:137/:142/:154`), NOT call sites. The
whole-tree token total is 462 across 24 files; the core family is 407
(107+80+94+46+69+5+6) + the 2 NPC indirect casts.

**The gems-NEVER-emit-light verdict (§3) is a FAMILY boundary:** the gem
tiles (63-68/178/566) are NOT AddLight sources — their sparkle is dust-43
(d60) + the tileShine cadence; the scanner-seeded ambience (torches/lava/
glowing blocks) is the `tr-core/tile_light` lane, NOT this family. The
g18 negative census + the c29 refutation agree from both sides.

**Merge order:** the core lanes are site-disjoint (c19 × e6 × f7 proj;
c20 × e7 × f8 NPC; main × g13 × d50 dust; e8 × g18 × g19 × f9 Main/tile;
g20 item). Either lands first within each file family; the doc ledgers
append-union. The d-lane rows land on their own lanes (keep-one — the
collectors copy NO rows). The ONE owed merge tasks: the e6 6-kind
activation sub-ifs (§2/§4), the `:2275` customData reconstruct (§2), the
light-map-write deferral task (above). The craft.rs S1-L-11g addition is
main's, kept at merge (no lane touches it).

**SEAL**: no further census/arm/row work is owed on the Main.cs batch-1
sites by this lane (g18 ports nothing — e8 keeps the single copy). The
family bible above is the g18-side authority; the per-file authorities are
c19 research/189 (proj first wave), e6 ANALYSIS (proj second wave), f7
`proj_addlight_census.rs` (the 107 freeze), g13 research/308 (dust 94/94),
c29 research/260 (the gems refutation), g19 `main_addlight_census.rs` (the
Main 5 freeze). The four proj task-seeds (465/731/phaseblade/756) belong to
the row-owning lanes; the light-map-write layer is its own task.

## 6. Verification record (code-only)

* The git-log census: every lane tip re-fetched (`git fetch --prune`), the
  accountings quoted from each lane's final ANALYSIS/research record; the
  witness counts machine-counted (`#[test]` grep) at the pushed tips.
  c19 45 = 35 proj_lights_a + 9 projectile.rs + 1 world_sim (the round-4
  SEAL's "45 total"); e6 15; f7 5; c20 27; e7 17 NPC + 18 Player; f8 8;
  e8 11; g18 5; g19 5; g20 2; g13 23 + 1 SEAL; main base 9+12+6+18+5+15.
* The d54-bible rows re-read at tip `1c462c4fe` (131 §7.2 G1 / s8.2); the
  g13 94/94 fresh census re-read at `3fe509158` (research/308 §3).
* The proj 86/21 derived from c19 round-4 (63 covered/4 absent) + e6
  (20/20) + the 6 row-activations (each `git grep '<CONST> =>'` verified at
  the rowing lane's tip); the 13 still-unrowed e6 kinds + the 4 c19
  task-seeds re-confirmed rowless by whole-lane `data.rs` grep.
* The gems refutation re-derived from the C# (the tileLighted whole-tree
  grep, the TileDrawing.cs:7453-7506 dust-43 window, the TileLightScanner
  top-level-case grep, the Lighting.cs funnel) + the generated TILE_LIGHTED
  bitset decode + the EMISSIVE gem rows; the c29 research/260 Item 1 + bible
  row read at `f0643602d`.
* The e6 compose re-verified at the current tips (the true diffs
  `git diff a725acb04..<tip> --stat`, the draw.rs/main.rs/anchors hunks, the
  craft.rs S1-L-11g provenance `03711f456`).
* The registry number 360 per §8.7 (new docs ≥360) + a fresh 362-tip
  ls-tree sweep (no docs ≥352, no research/360 ref).
* **No cargo/rustc/rustfmt/tests/dotnet/graphify invoked.** The witnesses
  (g18's 5 oracle tests + the family-census witness
  `tests/addlight_family_census.rs`) are WRITTEN, NOT RUN.
