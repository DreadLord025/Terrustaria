# 68 — Main dust residuals doc-sweep (W4-g13, lane 41): sibling re-probe, residual census, Dust.cs internals audit, lavaBubbles consumers

Status: DONE + FINAL (W4-g13 sweep phase, code-only — one small port
landed `1af79847`, 2 witnesses WRITTEN NOT RUN; the rest is census/flags;
§5 is the final-phase family-complete declaration for the d54f-131
master, sibling tips re-probed at g11 `9a9ac1fd` / g12 `f24dd76a`).
Probed tips (all fetched from origin, base `a725acb0`): g11 `13a1c48e`
(+ docs-only tip `d892d8d4`), g12 `665cd1e8`, d50 `2e243a2d`,
f6 `5daa1d70`, d54 `c792cf37` (research/66 EXECUTED), this branch
`15fd99c9` → `1af79847`.

## 1. Sibling re-probe (post-push): NO site collision with g13

The pre-push probe left g11/g12's overlap open; both are now pushed and
read in full (`git show` of their modules + diffs vs `a725acb0`).

| Lane | Module | Sites | vs g13's 17-23 + 4 feeds |
| ---- | ------ | ----- | ------------------------ |
| g11 `13a1c48e` | `main_dust.rs` (1305 ln) | 1-8: `snowing` :13088 + `teleport_effect` styles 0/1/2/3/4/5/7 (:13243-13358) + `player_teleport`/`npc_teleport` replays, WIRED (player tick + wiring pads) | disjoint — no g13 line touched |
| g12 `665cd1e8` | `main_dust_b.rs` (817 ln) | 9-16: `teleport_effect_case10/11/13` (:13390/13438/13452, incl the :13410 CloneDust), `skeletron_hand_bone_dust` :22744, `prime_limb_bone_dust` :22820, `moon_lord_core_dust` :24736/24748/24752 | disjoint — the module doc + `moon_lord_core_dust` both cite "`:24756` is g13's site 17"; the boundary holds |

- **g12 does NOT carry the Tavernkeep** (sites 18/19, :27381/27384) —
  the "ale g12 may have sites 9-16 incl tavernkeep" worry is resolved
  NO: g12's DrawNPCDirect/Extras coverage stops at the flag8 trio;
  `tavernkeep_ale_sparkle`/`tavernkeep_ale_feed` are g13-only.
- **g11 tip `d892d8d4` is docs-only** (research/83 weather-reconcile +
  research/116 teleport-reconcile + INDEX; `git diff 13a1c48e..d892d8d4
  --stat` = 3 doc files). The snowing budget-hoist fix (research/66 §4.3
  / research/83 fold F4) is STILL owed: at the tip, `main_dust.rs:140`
  re-reads `minecart::live_type(pool, 76)` INSIDE the loop. g11b owns it.
- **Filename collision CONFIRMED** (research/66 §2 already records the
  resolve): g11 and g13 both create `crates/tr-app/src/main_dust.rs`
  and both add `mod main_dust;` to `main.rs` at different spots (g11
  after `map_layers`, g13 after `render`). Union = ONE file: g13's
  bodies + g11's `snowing` (budget-hoisted); g61's teleport body wins
  the 4-way keep-one (research/116), g11's seams stay the dispatcher,
  g12's case10/11/13 + d54's latent bodies drop to witness supplements.
  Merge-mechanics notes for the union: the two `tests` modules' helpers
  collide by name with different shapes (g13 `rig() -> (DustPool, World,
  SpawnCtx)` / `live(&DustPool)` vs g11 `rig(seed) -> App` /
  `live(&App)`); g11's half is App-coupled (`crate::app`, `RngKey`,
  `Audio`) while g13's is pure-pool — the union keeps both import sets
  and renames one helper pair.
- **`app/frame/draw.rs` overlap is hunk-disjoint**: g11's only draw.rs
  delta is the `snow_count` argument at ~:2009 (`rain.snow_count()` →
  `minecart::live_type(dust, 76)`); g13's feeds sit at ~:1187. No
  textual conflict; semantically g11's read depends on g11's snowing
  spawning into the shared pool (the Snowflake stand-in retired).
- **Double-feed risk at the union (flag)**: g12's `moon_lord_core_dust`
  (sites 14-16) and g13's `moon_lord_tentacle_feed` (site 17) both
  resolve the SAME type-398 arm geometry (the 397-hand scan +
  `vector32`/`vector33`). Both are dead (`flag8 = false`, :24703) so
  nothing double-spawns today, but the wiring wave must fuse them into
  ONE caller pass (g12's anchor/pull/center + g13's tip maths), never
  two independent feeds walking the NPC list twice.

## 2. Main.cs residual census: ZERO unclaimed spawn windows remain

Full `Dust.` reference census of `decompiled/Terraria/Main.cs` (every
family, not just NewDust): 17 `NewDust` + 5 `NewDustDirect` + 1
`NewDustPerfect` = the 23-site census, ALL claimed (g11 1-8, g12 9-16,
g13 17-23); the one `CloneDust` (:13410) rides g12's case-10 arm. The
remaining `Dust.` windows are non-spawn machinery:

| Window | C# | Port status |
| ------ | -- | ----------- |
| pool init | :6687-6688 | `DustPool::new` (S1-E-1) |
| `UpdateWorld_Dust` + crash-recovery re-init | :18046-18070 | `minecart::tick`'s single `update_dust` pass (S1-E-3, ANALYSIS :19164); the :18063-18067 catch re-init is crash recovery, not per-tick |
| `DrawDust` (the generic draw + per-type tint arms) | :38828-39027 | PARTIAL by design: `minecart::draw` (type 213) + `sand::draw` (type 268) cover the live types; the all-type generic draw is the dust DRAW lane's remainder (d50's doc: "the pool's generic draw pass is the draw lane's"), not a Main spawn site |
| `DrawCapture` snow cull | :56735-56744 | **UNCLAIMED → PORTED this sweep** (`1af79847`): `main_dust::capture_snow_cull`, the four-negation containment OR verbatim (integer `*16` products, the saved PRE-capture view `vector`/`num2`/`num3` :56694-56696), full 6001-slot walk, live type-76 only, RNG-free; staged behind the S1-G-13 `CaptureRenderer` seam; 2 witnesses |
| `Dust.lavaBubbles` read | :57785 | g13's retro feed (wired) |

The d54 KillTile / e4 orch-41-61 / f6 mount / g66 player-trails lanes
own OTHER files (WorldGen/Orchestrator/Mount/Player) — none carries a
Main.cs window. **Nothing left to port in Main.cs** (the "smallest ≤20
sites" instruction resolves to the single 10-line cull above; the
23-site spawn census is closed).

## 3. Dust.cs internals audit vs d50 `dust_physics` (2e243a2d)

Arm-by-arm census of `Dust.UpdateDust` (`Dust.cs:420-2423`) against
`origin/task/W4-d50-dust-type-table:crates/tr-core/src/dust_physics.rs`
(2154 ln, `pre_move` + `type_chain`, 38 witnesses):

- **Pre-move** (:468-476): the 299/300/301 (`(uint)(type-299)<=2`) + 305
  shrink/lift and the 35 `lavaBubbles++` — BOTH in `pre_move` (:162-173).
- **FIRST-ladder standalone ifs** (:478-814): 258, 309, 325, 86-92/286
  (gem switch), 262, 240, 329, 259/6/158/135, 263/264, 230, 154/218,
  172, 182, 261, 254, 255, 211, 284, 213/260, 157, 206, 163, 205, 170,
  156, 234, 175, 174 — every physics mutation ported; the light-only
  arms (325/86-92/262/263-264/172/254/255/157/163/205/170/156/234) and
  the customData followers are CITED skips (S1-E-5 / field drop).
  Spot-verified against the C# (172 :656-664, 67/92 :1915-1927, 185
  :1928-1940, 107 :1941-1952 are AddLight-only — no hidden physics).
- **Big else-if chain** (:815-1829): 235 (dead vector3 draws kept),
  torch family (228/279/229/6/242/135/127/187/75/169/29/59-65/158/
  293-298/302/307/310), 306, 269, 159, 164, 173, 304, 184, 160/162,
  168, 139-142, 326/327/328/14/16/31/46/124/186/188/303 (31's
  noGravity arm reachable BECAUSE the NPC customData branch drops —
  cited), 32, 244-247, 43 (PRE-ramp light terms + dark kill), 15/57/
  58/274/292, 204, 110-114/311-313 (light-only), 66, 267, 20/21/231,
  27/45, 55/56/73/74, 71/72, 76 (snowDust++ cited drop), 270, 271,
  268 (SandStormCount via env), generic gravity else-if (107 damp) —
  ALL present, statement order matches.
- **Standalone ifs** (:1830-2129): 5/273, bubble family (308/33/52/
  266/98-105/123/288), 35/152 noGravity sub-chain + 67/92 + 185 + 107
  + 34/35/152 wet arm (the Next(2)/Next(-10,10) draws), 68/70
  (light-only), 41/44/air-drag, 322, rotation rebuild (79/268/304
  exempt), fadeIn ramp (235 homing / 46 / 213/260 / generic + burn-out)
  — ALL present.
- **SECOND chain** (:2130-2334): 130-134 (131 sub-arm), 278 (SolidTile
  damp), 219-223, 226, 331, 272 — ALL present, each returns `true`
  (suppresses the :2335 gravity tail exactly as the C# else-if does).
- **Head/tail** (:426-466, :2343-2422): the skeleton's (S1-E-3,
  `particles.rs::update_dust`), not d50's — verified present on main.

**Verdict: NO missing physics type-arm in d50's port.** Every C# arm is
ported, cited-light, cited-customData, or cited-snowDust. Three FLAGS
for the d50 lane / wiring wave:

1. **The 52 second-chain light arms are ORPHANED (circular deferral).**
   `UpdateDust` carries 94 `Lighting.AddLight` sites: 42 in the FIRST
   ladder (≤:1053) — ported in `tr-app::dust_lights` — and 52 in the
   second chain (:1099-2312). `dust_lights.rs:41-46` says the 52 "land
   with S1-E-3's chain port"; d50's chain port skips them ALL as
   "S1-E-5 (`dust_lights`)". Neither lane owns them. The 52 (159/164/
   173/304-region/160/162/168/244-247/43/15-57-58/110-114/311-313/66/
   267/20-21-231/27-45/55-56-73-74/71-72/35/67-92/185/107/68/70/41/44/
   130-134/278/219-223/226/331/272 lights) need an owner — the d50
   wiring wave or a lights follow-up lane.
2. **lavaBubbles DOUBLE-COUNT at d50 wiring.** d50's `pre_move`
   (:170-172) increments `lava_bubbles` for type 35, AND g13's landed
   inline increment (`particles.rs:359`, witnessed
   `w4_g13_lava_bubbles_counts_the_live_type35_population`) does the
   same inside `update_dust`. d50's ANALYSIS wiring note ("the generic
   loop calls pre_move ... then update_dust") would count every 35
   TWICE per pass, halving the <200 bubble budget. Resolve at wiring:
   keep the inline increment (landed + witnessed), strip the 35 arm
   from `pre_move` (it keeps the 299-305 shrink/lift, which the
   skeleton LACKS and the wiring must add).
3. **dust_lights local-decrement retire** (206/174/230, plus 309/261
   post-mutation reads): d50's ANALYSIS already flags it — once
   `type_chain` is wired the feed's compensating decrements
   double-count; retire them in the same wave.

## 4. lavaBubbles consumer census: BOTH readers accounted, none unclaimed

`Dust.lavaBubbles` (Dust.cs:14; reset :427, increment :475 — both
maintained by `DustPool::update_dust`, g13's increment landed
`150e7204`) has exactly TWO C# readers (full-tree grep):

| Reader | C# | Scope | Port |
| ------ | -- | ----- | ---- |
| retro `oldDrawWater` | `Main.cs:57785` | `FocusHelper.AllowMiscDustEffects && < 200`, the :57789/57793 pair | g13 `lava_bubble_pair` + `retro_liquid_dust_feed`, WIRED (`35404f8a`) |
| modern `LiquidRenderer.InternalPrepareDraw` | `LiquidRenderer.cs:556` | per visible lava cell (`VisibleType == 1 && HasVisibleLiquid`) in the draw rect, `< 200`, independent `Next(700)`/`Next(350)` gates (:558/:562) + the big-bubble post chain (:565-572) | **f6 batch-4** `world_dust.rs::lava_bubbles` (`680ac9d8`, sites 560/564), STAGED |

- **It is NOT an Underworld bg lava-lake spawner**: the block walks
  EVERY visible lava cell of the liquid draw cache (any biome), inside
  `InternalPrepareDraw` (:161, called by `PrepareDraw` :849 → the
  liquid draw path, e.g. `Main.cs:56754`). The retro path's
  `value.Y`-gated surface bubble is the g13 one; the modern path has
  no surface gate — both gates roll per cell.
- **Stream**: the modern consumer rolls the renderer's PRIVATE
  `_random` (`LiquidRenderer.cs:132`, `new UnifiedRandom()` — a
  distinct, non-swap stream; f6's claim doc + emitter doc cite it).
  The port has no such stream yet — its identity/seeding is f6's
  wiring-wave decision, NOT a `rngs.global()` slot.
- **Verdict: CLAIMED (f6) → not wired here.** Wiring it would steal
  f6's seam and force the `_random` decision early; the port seam is
  the App-side liquid draw (`render/liquids.rs` over
  `tr_core::liquid_draw::build`, the `InternalPrepareDraw` port — a
  pure tr-core fn with no pool/rng access, so the consumer must ride
  the App draw caller, f6's staged-emitter contract). g13's maintained
  counter feeds BOTH consumers correctly; no unclaimed reader remains.

## 5. FINAL — the Main.cs dust-family status declaration (for d54f-131 master)

Final-phase re-probe (base `daf7b780`): g11 ADVANCED `d892d8d4` →
`9a9ac1fd` (union 1-3 EXECUTED: the g61 widest-11 teleport body adapted,
the App dispatcher + 6 gap closures, research/116 executed + research/83
F1-F7 published as exact patch-hunks for d130); g12 ADVANCED `665cd1e8` →
`f24dd76a` (render-lane doc-sweep + the wave-A slime scales — NO dust-site
change; the `:24756` g13 boundary comment still present at
`main_dust_b.rs:18`/`:413`). d50 `2e243a2d` UNCHANGED (research/69 §6);
d54 `c89aea9f` (research/131 FINAL); g67 `30b39fceb` (the DrawDust cull
`2c452edd` ancestor-verified, research/70 §1).

**The family-complete declaration:**

| Column | Sites | Owner + evidence | Status |
| ------ | ----- | ---------------- | ------ |
| Main.cs spawn windows | 23 NewDust-family + 1 CloneDust | g11 1-8 (`9a9ac1fd`, snowing body DEPRECATED for d130's snow.rs per research/83 keep-one; teleport rides the g61 body), g12 9-16 (`f24dd76a`), g13 17-23 (`main_dust.rs`, WIRED `35404f8a`+) | **ZERO unclaimed** (§2 stands; the g11/g12 advances touch no g13 site) |
| Main.cs non-spawn machinery | pool init / UpdateWorld_Dust / DrawCapture cull / lavaBubbles read | S1-E-1 / S1-E-3 / g13 `1af79847` / g13 wired feed | CLOSED (§2 table) |
| DrawDust cull + else-kill | `:38831-38847`, `:39020-39022` | g67 `2c452edd` (rain.rs, ancestor of `30b39fceb`); the d130 fold is research/134 R3 | PORTED |
| DrawDust tint column | GetAlpha/GetColor all consumers | S1-E-2 table + g13's 157/327 closures (`daf7b780`, research/70); minecart/sand draws live | TABLE-COMPLETE (the trail walks = the DRAW lane's remainder, not a Main gap) |
| Dust.UpdateDust lights | 94 = 42 + 52 | `dust_lights.rs` (S1-E-5) + g13 `dust_lights_b.rs` (`a234107a`, WIRED) | **94/94 CLOSED** (research/69) |
| Dust.UpdateDust physics | every type-arm | d50 `dust_physics.rs` staged (3 wiring flags → contracts, research/69 §4-6) | PORTED, wiring owed to d50's wave |
| lavaBubbles consumers | 2 readers | g13 retro (wired) + f6 modern (staged, `_random` seam f6's) | BOTH ACCOUNTED (§4) |

**Residual notes refreshed at the final tips:**
- The g11 snowing budget-hoist (§1's "STILL owed") is SUPERSEDED at
  `9a9ac1fd`: the body is DEPRECATED wholesale for d130's `snow.rs`
  (research/83 keep-one); the in-loop `live_type` read survives only in
  the deprecated body the merger drops. No hoist work remains for g11.
- The `main_dust.rs` filename union (§1) now resolves THREE-way: g13's
  bodies + g11's dispatcher/seams (its snowing half DROPS for d130) +
  g12's `main_dust_b.rs` stays separate. The moon-lord double-feed fuse
  flag (§1) stands — both halves still dead (`flag8`), one caller pass
  at the wiring wave.
- **Verdict for the d54f-131 master: the Main.cs dust family is
  COMPLETE — zero unclaimed spawn windows, zero unclaimed non-spawn
  machinery, the 94-light family closed, the tint column table-complete.
  Everything remaining is wiring (d50's contracts, the DRAW lane's
  generic walk, the union merge mechanics), not porting.**

## Delivered this sweep

- `1af79847` — `main_dust::capture_snow_cull` (the §2 residual) + 2
  witnesses, staged (`#[allow(dead_code)]`, the S1-G-13 seam).
- This doc + the ANALYSIS sweep section + the PORT_MAP/anchors/
  research-65 touch-ups. No other code touched; g11/g12/d50/f6's files
  are READ-ONLY probes (their lanes own the fixes flagged above).