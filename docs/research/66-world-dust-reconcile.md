# 66 — World-dust RECONCILE (W4-d54 merger assist): arm diffs, KEEP/DROP, witness union

Status: **FINAL — EXECUTED on `task/W4-d54-dust-sites-world`** (union-exec
round + final-audit round, 2026-11-21 — §6 landed hashes; residuals ZERO
per the final audit: all 11 anchors markers re-verified present at the
audited tip, no stale pre-union prose left, the branch diff vs
origin/main lists ONLY task files — 24 files, the main.rs/lib.rs mod
rows included; checklist 1-3/7 EXECUTED here, 4-5 cross-doc stand, 6 is
the merge-verify wave's; research/131 §6/§7 carry the final record).
Originally a RECONCILE doc (code-only merge-assist round,
2026-11-21). Seed: research/67's overlap note. Filename note: e4's
branch carries a different doc at the same number
(`66-dust-world-sites-2-claims.md`); the two coexist (distinct
filenames).

## In-flight probe (2026-11-21 02:28 / 02:30, 2× ≥90 s apart)

Worktree `TR-W4-d54`, branch `task/W4-d54-dust-sites-world` @ `8cb5d5a4`
(ahead/behind 0). Both probes: no `.git/index.lock`, no cargo/rustc/git/
dotnet/python process with this worktree in its command line, clean
`git status --porcelain`. The prior live writer is FINISHED (its last
writes 01:59-02:14 match the landed tip). Proceeded.

Branch tips reconciled here (all at base `a725acb0` = origin/main):

| Lane | Tip | Code landed |
| ---- | --- | ----------- |
| d54 (this) | `8cb5d5a4` | world_dust{,_main,_tile,_wall}.rs + orch_dust{,_lightning,_boulder,_town}.rs + LIVE editing seams |
| g14 | `a35efec2` | tr-core worldgen_dust{,_tile,_wall}.rs (all 29 WorldGen sites, UNWIRED) |
| g11 | `25e180d4` | tr-app main_dust.rs (snowing + TeleportEffect styles 0-7, UNWIRED) |
| g12 | `665cd1e8` | tr-app main_dust_b.rs (Main sites 9-16: TeleportEffect 10/11/13 staged + the DrawNPCDirect bone-arm tip puffs 22744/22820 + the Moon Lord flag8 trio 24736/24748/24752, UNWIRED, 7 witnesses) — **the original `73a85c01` claim-only row was STALE; batch 2 landed 02:47, after this doc's 02:28 probe** |
| g13 | `dca09bed` | tr-app main_dust.rs (Main sites 17-23, UNWIRED) |
| g61 | `a9628928` | tr-app npc_spawn_visuals.rs (spawn/teleport-in family, UNWIRED) + tr-core particles.rs DustRng generalisation |
| e4 | `2a79551a` | tr-app orchestrator_dust.rs (orch 41-61) + mount_dust.rs (Mount 1-19), UNWIRED |
| f6 | `9dec91b0` | tr-app mount_dust.rs (ALL 27 Mount sites; batch-4 small files NOT landed), UNWIRED |

## 1. KillTile_MakeTileDust arm-by-arm fidelity diff

d54 `crates/tr-app/src/world_dust_tile.rs` (2343 ln; the ~250-arm
overwrite-if chain as ONE fn `tile_dust_type` :117-1697 + `make_tile_dust`
:1714-1846) vs g14 `crates/tr-core/src/worldgen_dust_tile.rs` (2368 ln;
the same chain in 8 C#-order chunks `tile_ladder_head/switch1/middle/
frames1/frames2/walks_ores/furniture_herbs/tail` + `kill_tile_make_tile_dust`
:1511-1739). C# source: `WorldGen.cs:66687-69169`.

### Ladder (the type chain, :66689-69058)

Mechanical census (this round): BOTH ladders carry exactly **35 genRand
draw sites + 1 Main.rand draw site** (the type-19 row-30 `Next(3)`,
`:67402`), and the per-call multisets are IDENTICAL (Compare-Object on
the grouped `gen.next_max(N)`/`next_range` call shapes: empty diff).
Chunk/statement order matches the C# (head → big switch :66846 → moss/
529/742-750 → frames → 5/323 column walks :68160-68260 → furniture/herbs
→ tail vetoes + flag2). The C# dead writes are kept in BOTH: the 216
duplicate (:66698), the 187 second `<=1132` (:67603), the 129 `fx>=324`
overwrite (:68992), the tree-walk duplicated 44/132-176 arm (:68183).
The 583-589 seven `Next(10)` arms, 633 `Next(6)`, 634 `Next(10)+Next(12)`
pair, 385 `next_range(68,71)`, 27/37/370/641 `Next(2)` arms all match
arm-for-arm (spot-verified both sides). **Verdict: the ladders are
equivalent; either body is faithful. g14's 8-chunk split is the cleaner
tr-core home.**

### Spawn tail (:69085-69167) — 14 arms

| # | C# | Arm | d54 | g14 | Faithful |
| - | ---- | --- | --- | --- | -------- |
| 1 | 69089 | 627/628/184-col-10 disco, noGravity+noLightEmittance | :1734-1743 | :1538-1557 | BOTH (identical) |
| 2 | 69101 | 518 liquid lift, SolidTile(i,j-1) cap 8 | :1745-1754, local `solid_tile` :50-64 | :1559-1578, `crate::edit::solid_tile` | **g14 on the OOB edge** (below) |
| 3 | 69105 | 352 num==5: scale 1.5, vel ×1.65, fadeIn 1.6 | :1756-1764 | :1580-1590 | BOTH |
| 4 | 69114 | 160/692 disco-66 alpha 100 scale 0.75 | :1766-1773 | :1592-1609 | BOTH |
| 5 | 69120 | 323 frameY x-offset | :1775-1780 | :1611-1626 | BOTH |
| 6 | 69124 | 314 track: 2 gen speed args BEFORE internals, fadeIn roll AFTER, duplicated noGravity | :1784-1793 | :1630-1641 | BOTH |
| 7 | 69132 | flag2 gem family alpha 75 scale 0.75 noLight | :1795-1799 (`flag2_kind(kind)` recomputed) | :1643-1649 (flag2 threaded from ladder tail :1487-1494) | BOTH (the 178/440 base OR the switch set = the same 20-kind union) |
| 8 | 69138 | 193/18-num4/19-num4 Color(0,80,255,100) | :1801-1806 | :1651-1666 | BOTH |
| 9 | 69142 | 197 Color(97,200,255,100) | :1808-1813 | :1668-1683 | BOTH |
| 10 | 69146 | 185 num 86..=91 noLight | :1815-1819 | :1685-1691 | BOTH |
| 11 | 69152 | 4-num66 torch disco, **float Color ctor** | :1823-1830 `(b/255f*255f).clamp(0,255) as u8` | :1696-1717 `(b as f32/255.0) as u8` | **d54 ONLY** (below) |
| 12 | 69158 | num==139: `Type: num + Main.rand.Next(4)` arg FIRST | :1834-1837 | :1721-1726 | BOTH |
| 13 | 69162 | 722 velocity ×0.4 | :1839-1843 | :1728-1734 | BOTH |
| 14 | 69166 | plain tail | :1845 | :1736-1738 | BOTH |

**Arm 11 is the one real fidelity divergence.** The C# is
`new Color((float)Main.DiscoR / 255f, …)` — XNA/MonoGame's float ctor
multiplies each channel back by 255 and truncates (`(byte)(v*255f)`),
so the disco bytes round-trip (modulo the C#'s own f32 divide/multiply
rounding, which d54 keeps verbatim). The repo's established convention
agrees everywhere else: `dust_table.rs:66` (`mul_rgb`'s `/255*255`
truncation), `render/gore.rs:80-83` + its witness :306 (the 1225 float
ctor keeps 155), e4's `orchestrator_dust.rs:115-122` (`color_f` =
`*255` then cast), g61's pylon colour (`npc_spawn_visuals.rs:826`).
g14 instead truncates the [0,1] value DIRECTLY (every channel lands 0
except a full-255 one → 1) and its ANALYSIS note + witness
`w4_g14_tile_torch_disco_float_ctor_truncation` (:1970-1984, asserts
(200,100,255)→(0,0,1)) pin the WRONG semantics. **Merge action: keep
g14's module, replace its arm-11 colour maths with d54's round-trip
(`world_dust_tile.rs:1824`), and fix/replace that one witness with
d54's torch assertion (:2329-2336).**

**Arm 2 edge:** C# `SolidTile(i,j)` (`WorldGen.cs:70111-70131`) returns
TRUE for a NULL tile (:70115-70118) but FALSE for an out-of-bounds index
(the try/catch swallows the throw). The port's `World::tile` returns
`None` only for OOB (there are no null tiles). d54's local copy maps
`None → true` (faithful to the null arm, wrong for OOB); g14 reuses
`edit::solid_tile` (`edit/kill.rs:580-590`, `None → false`, the OOB
reading). In-world behaviour is identical; at j=0 with lift>8 g14 is
the faithful one. **Merge action: g14's call stands; drop d54's local
`solid_tile` copy.** (g14's `crate::edit::solid_tile` resolves — the
`pub(crate) use kill::{solid_tile, …}` re-export, `edit.rs:33-35.)

### Around the tail

- **Amount gate**: d54 ports `KillTile_GetTileDustAmount` as
  `tile_dust_amount(fail, kind, frame_x)` (:70) — a THIRD copy of a table
  already on main (`collision.rs:2383 tile_dust_amount(kind, fail,
  frame_x)` + `projectile_ai_misc.rs:588`'s fail-only variant). g14
  deliberately leaves it to collision.rs (its claim doc). **Merge: keep
  collision.rs's copy, drop d54's, re-point the orch callers (argument
  order differs!).**
- **Burst wrapper**: d54's `tile_break_burst` (:1852-1868, the
  `WorldGen.cs:63699-63703` count loop) has no g14 equivalent — KEEP it
  in tr-app, calling g14's body.
- **Return shape**: d54 `make_tile_dust` returns the slot; g14
  `kill_tile_make_tile_dust` likewise (sentinel on num<0). Signatures
  differ only in parameter order + `disco` tuple-vs-array — the seam
  re-point is mechanical.

## 2. KEEP/DROP per module

### WorldGen lane (d54 batches 1-3 vs g14)

| Module | Verdict | Reason |
| ------ | ------- | ------ |
| g14 `tr-core/worldgen_dust.rs` | **KEEP** (canonical) | ash-shake consumes the gore sibling's ARGUMENT draws (`RandomVector2(-40,40)` + `61+Next(3)`, :146-149, C# :29669) — d54's `ash_shake` DROPS them (:268 "g16 skip"), breaking the !hitTree stream by 3 draws/row. g14 faithful. |
| g14 `worldgen_dust.rs` paint colour | **KEEP + 1-line FIX** | g14's `paint_color` (:193-196) forces alpha 255; the C# `paintColor` color-30 arm sets `white.A = 150` (`WorldGen.cs:44026-44032`). d54's `paint_color_full` (:64-67) has it right — port that one conditional into g14. |
| g14 `tr-core/worldgen_dust_wall.rs` | **KEEP** (canonical) | ladder 11-gen multiset identical to d54's (verified); 4-arm tail identical (348 damp / 44-346 disco / 76 blue / generic + the 116-135 silent band). g14 returns the slot (witness convenience), d54 void — no behaviour delta. |
| g14 `tr-core/worldgen_dust_tile.rs` | **KEEP** (canonical) + arm-11 fix | §1. |
| d54 `world_dust.rs` | **DROP bodies, KEEP seams** | `cutter_burst` is LIVE (`editing/wiring.rs:132/:154` — actuator + 4 wire colours, drop-then-dust order per C# :58030-58035/:58130-58135). Re-point the seam at `tr_core::worldgen_dust::{kill_actuator_dust, kill_wire_dust}` (identical five-type-50 body). unlock-door/paint/coat/ash bodies: g14's win. |
| d54 `world_dust_wall.rs` | **DROP ladder+make_wall_dust, KEEP wrappers** | `wall_break_burst` (:478) + `place_tile_torch_dust` (:500) are LIVE (`editing.rs:5312` break_wall sound→dust→drop, `:5398` place_wall_route dust→sound→drop — the C# orders :60385-60394 / :62545-62551 — and `:7614` PlaceTile 22/140 after the sound). Keep the wrappers, call g14's `kill_wall_make_wall_dust` / `place_tile_ore_dust`. d54's internal 22/140 gate vs g14's caller-side gate: equivalent, pick one at the seam. |
| d54 `world_dust_tile.rs` | **DROP ladder+make_tile_dust+amount+solid_tile, KEEP tile_break_burst** | §1; the LIVE seam `editing.rs:5693` (break_tile, pre-kill snapshot, after sounds before drops) stays, re-pointed. |
| d54 `orch_dust*.rs` tile-sweep delegations | **KEEP, RE-POINT** | `orch_dust.rs:345-355`, `orch_dust_boulder.rs:346-357`, `orch_dust_lightning.rs:125-135`, `orch_dust_town.rs:121-132` call `world_dust_tile::{tile_dust_amount, make_tile_dust}` → become `collision::tile_dust_amount` (arg order!) + `tr_core::worldgen_dust_tile::kill_tile_make_tile_dust`. The DownDash gate variant stays distinct (research/67 note 4). |

### Orchestrator lane (d54 1-40 vs e4 41-61)

**DISJOINT — verified, no overlap.** d54's four modules cover sites 1-40
(lines 423-2582, research/67's ledger); e4's `orchestrator_dust.rs`
covers 41-61 (2653-3920: ShimmerArrow…WoFGoatMountFlames, 21 sites, 9
witnesses). Boundary 2582/2653 holds in both. **KEEP both.** Shared
helper duplication (`next_vector2_circular`, `lerp_color` ×3,
`to_rotation*`) is cosmetic — fold into one helper module at the wiring
wave, not a fidelity issue. Two wiring-wave reconciles to record:
- `prefix.rs:678-716` consumes RainbowRodHit/BestReforge draws
  COUNT-ONLY on the reforge tail; e4's `rainbow_rod_hit` (:489) is the
  FULL emitter (its witness pins the 272-draw contract). The wiring
  wave folds the count-only drains into the real emitter — never both
  (e4's claim note; double-draw risk).
- `Spawn_BestReforge` (:3069-3087) has NO dust site (8× circular = 16
  draws, CreativeSacrificeParticle only) — prefix.rs's count-only drain
  stays canonical for it; neither d54 nor e4 owes a body.

### Main lane (d54 batch 4 vs g11/g12/g13 vs g61)

| Module | Verdict |
| ------ | ------- |
| g13 `main_dust.rs` (sites 17-23) | **KEEP whole** — no overlap with anyone (the dead flag8 tentacle puff, Tavernkeep ale pair, DrawItem sparkle, retro oldDrawWater trio; 14 witnesses, offline-pinned seeds). |
| g11 `main_dust.rs` snowing | **KEEP with one fix** (§4). **SUPERSEDED by research/83/131: d130's snow.rs is canonical; g11's body retires, the F1-F7 folds land on d130.** |
| g11 `main_dust.rs` teleport_effect (styles 0-7) | **DROP** — g61's is wider and byte-exacter (§4). Keep its witnesses as supplements. |
| d54 `world_dust_main.rs` | **DROP both bodies** (§4), keep the 9 witnesses as supplements. |
| g61 `npc_spawn_visuals.rs` | **KEEP whole** — the widest teleport port (§4) + the spawn family (PoofOfSmoke, BigMimicSpawnSmoke, TransformVisuals, NPC.Teleport driver, UpdateNPC_TeleportVisuals, alt-texture transitions). 18 witnesses. |
| g61 `tr-core/particles.rs` delta | **KEEP** — `new_dust_perfect/direct/clone_dust` generalised `&mut UnifiedRandom` → `R: DustRng` (behaviour identical; `UnifiedRandom` implements the trait, so every d54/e4/f6/g14 call site still compiles). |
| g12 `main_dust_b.rs` (sites 9-16) | **KEEP whole** — `665cd1e8` landed 817 lines + 7 witnesses AFTER this doc's probe (the matrix row was stale). **Main sites 12-16 (`:22744/:22820/:24736/:24748/:24752`) ARE covered**: the two DrawNPCDirect bone-arm tip puffs (type 5 hand / type 6 Prime limbs, the two-hop tether geometry + AllowMiscDustEffects gate) + the Moon Lord core arm's three flag8 puffs (DEAD-gate census: `Main.cs:24703` never set true in 1.4.5.8 — the fourth puff `:24756` is g13's site 17). The §2 "GAP" note below is SUPERSEDED — no follow-up batch owed; the census closes at the merge-verify wave. Sites 9-11 (13390/13438/13452) are triple-covered (d54 + g61 + g12): g61's teleport wins per §4, g12's cases 10/11/13 witnesses supplement. |

**Filename collision:** g11 and g13 BOTH create
`crates/tr-app/src/main_dust.rs` with different contents (968 vs 976
lines) and both add `mod main_dust;` to main.rs at different spots.
The merge resolves to ONE file: g13's six bodies + g11's snowing (with
the §4 fix); one `mod` declaration. **(Crosslane update, research/131:
the snowing half retires to d130 per research/83; the one file is
g13's bodies + g11's teleport SEAMS dispatcher over g61's body, per
research/116.)**

### Mount lane (e4 1-19 vs f6 all-27) — flagged overlap

e4's `mount_dust.rs` (19 sites, 10 witnesses) and f6's `mount_dust.rs`
(ALL 27 sites incl. e4's 19, 24 witnesses) collide on the SAME filename.
f6's claim ("d54/e4 dead lanes") predates e4's push and is superseded,
but f6's code is the superset (adds 5099 lava-cart, 5149 pixie, 6339
failed-dismount, 6382-6493 DoSpawnDust wolf/chillet/generic + the
FairyQueen colour helper). Bodies spot-checked equivalent (drill pair,
dash_ring_20, golf smoke, gem sparkle). **Recommendation: KEEP f6's
file as the base**, with two e4-derived fixes:
- f6's `drill_beam_dust` computes `d.y.atan2(d.x)` in F32; C#
  `Utils.ToRotation` is `(float)Math.Atan2` in DOUBLE. e4's shape
  (caller passes the aim, computed with the double atan2 —
  `orchestrator_dust.rs:76-78` precedent) is the faithful one.
- e4 misnames mounts 44/45 "wolf/witch"; MountID 44 = PirateShip, 45 =
  SpookyWood (`MountID.cs:141/:143`) — f6's names are right; keep them.
Keep e4's 10 mount witnesses as supplements where they pin draws f6's
24 don't. f6's batch-4 small files are NOT landed; its CLAIMS on
`Utils.PoofOfSmoke` (Utils.cs:1089) and `TeleportPylonsSystem.
SpawnInWorldDust` (:390) are already ported by g61 (`poof_of_smoke`,
`pylon_spawn_in_world_dust`) — **g61's landed bodies win; f6's follow-up
must skip those two rows** (claim reconcile, no code conflict today).
**(Crosslane update, research/131 §2: f6's batch 4 LANDED at `680ac9d8`
— world_dust.rs, 13 sites, 8 witnesses — and it DID port the poof +
pylon rows plus a rain_splash that duplicates d130's EXECUTED splash
union; the keep-one stands: g61's poof/pylon + d130's splash win, f6's
unique six fold. The double-atan2 drill fix is STILL owed at f6 tip
`5daa1d70`.)**

## 3. Witness-supplement union lists

All WRITTEN NOT RUN (every lane was code-only). The merge-verify wave
compiles the union; duplicates noted. Counts: d54 48, g14 24, g11 11,
g13 14, g61 18, e4 19, f6 24.

### KillTile (keep g14's 10 + d54's 6, minus 1 conflict)

- g14 `worldgen_dust_tile.rs::tests`: plain_stone_no_ladder_draws,
  negative_num_returns_sentinel, 627_moss_disco, 314_track_gen_order,
  139_plus_rand4, 352_gen_draw_always, 19_row30_main_rand_draw,
  **torch_disco_float_ctor_truncation (DROP/REWRITE — pins the wrong
  arm-11 semantics, §1)**, 518_liquid_lift_cap, sapling_short_circuit.
- d54 `world_dust_tile.rs::tests` (supplement): amount-table pin
  (re-point at collision.rs's copy), type-table pin (~90 kinds),
  random-arms oracle (the 583-589/633/634/27/37/370/641 gen arms),
  **make_tile_dust_draws_match_the_oracle (10 arms × 40 iterations,
  full-stream peek equality — the widest spawn-tail census either lane
  has)**, post-fields (627/314/722/**torch round-trip :2329-2336 — the
  replacement for g14's wrong witness**), break-burst count.

### KillWall + WorldGen small (g14's 14 + d54's 12)

- g14 wall: plain_spawn, 348_damps, 44_disco, 76_blue, band_silence_
  and_144_roll, genrand_arms_roll_once, negative_silent (7).
- g14 worldgen_dust: unlock_door_twelve_type11, paint_two_stream_order,
  paint_coat_pick_order, hardware_breaks_five_type50, place_tile_ore_
  three_type14, ash_shake_full_draw_census, ash_shake_hit_tree_counts_
  and_no_gore (7 — the ash census INCLUDES the gore argument draws d54
  dropped; it is the authority).
- d54 supplement: wall type-table pin + genrand-arms oracle + spawn
  oracle + post-fields + place-torch draws (5); cutter/unlock/paint/
  coat/ash oracles (7 — the ash ones lack the gore draws, adapt or drop
  in favour of g14's; **add a color-30 alpha-150 paint witness** when
  the g14 fix lands — neither lane has one).

### Main snowing/teleport (keep g11's 11 + d54's 9 + g61's teleport 8)

- g11: styles 0/1 ladder-after-cue, 2 soundless GhostWhite, 3 full
  shrink ladder, 4 portal pair, 5/7 cue-after-column, g12-boundary
  (REWRITE: styles 10/11/13 now exist via g61), sentinel post-writes,
  snowing gates/dry-mirror/storm-Deerclops-4-seed/live-76-budget-break
  (the budget witness must adapt to the hoisted read, §4).
- d54 supplement: snowing oracle + gates, style0 ladder, style3 oracle,
  styles5/7 writes+draws, **style10 oracle (float-triple + clone),
  style11 oracle (Next(4) colour first), style13 oracle (position
  floats then Next(1,3))** — the g12-third coverage g11 lacks,
  hsl/portal byte pins.
- g61: teleport style0/3/4/9/10/11 slot-sequence witnesses (RecordingRng
  — every draw pinned by name), npc_teleport two-sided latch,
  teleport-visuals gates/style12 clock hue, alt-texture transitions.
- g13: all 14 KEEP as-is (no overlap).

### Orch + mount + spawn

- d54 orch 21 (7+3+5+6) KEEP — sites 1-40, incl. the corrected
  RainbowBoulder sparkle draw counts (b3ce6a44/73f1ffcc).
- e4 orch 9 KEEP — sites 41-61 (the 272-draw RainbowRodHit pin is the
  prefix.rs fold authority).
- e4 mount 10 / f6 mount 24: union into the kept f6 file; drop exact
  duplicates (drill/ring/golf/gem/confetti/broom/drill-trail shapes
  overlap), keep f6's lava/pixie/dismount/wolf/chillet/generic tails +
  e4's basilisk-charge-fold and witch-puff-type31 pins.
- g61 spawn 18 KEEP whole.

## 4. snowing / teleport keep-one decisions

### snowing() — KEEP g11's body, with ONE d54-derived fix

Both ports keep the double `Math.Pow`/`Math.Sqrt`, the double
`cloudAlpha*0.2` scale add, the budget-BREAK vs probability-CONTINUE
shape, the else-if edge pushes. Three deltas:

1. **Open-air gate**: C# `!tile.nactive()` (:13086) = `!(active &&
   !inActive)` (`Tile.cs:240-247`). g11 has it right
   (`!(t.active() && !t.inactive())`, main_dust.rs:167); d54's
   `!t.active()` (world_dust_main.rs:127) misses ACTUATED tiles.
   → g11 wins.
2. **maxScreenW**: the runtime value under HiDef is 4096
   (`Main.cs:3709`; :66715's 1920 is the static initializer the profile
   switch overwrites). g11 pins 4096 with the citation; d54's env field
   is caller-fed but its doc + witnesses pin 1920 (the sand.rs
   deviation, not the runtime value). → g11 wins.
3. **Budget counter**: C# `Main.snowDust` is a static incremented only
   in `Dust.UpdateDust` (`Dust.cs:1747`, reset :428) — it is CONSTANT
   for the whole snowing() call. d54's once-per-call `env.snow_dust`
   snapshot matches that; g11 re-reads `minecart::live_type(pool, 76)`
   INSIDE the loop (:133), so freshly spawned flakes count against the
   budget mid-call and break it early — a deviation. → **fix g11: hoist
   the live_type read above the loop** (one edit), adapt its
   budget-break witness. d54's body then DROPs.

### TeleportEffect — KEEP g61's (widest), DROP d54's + g11's

Coverage: g61 = ALL eleven C# arms (0/1/2/3/4/5/7/9/10/11/13) + the
pylon style-9 body + `NPC.Teleport` two-sided driver +
`UpdateNPC_TeleportVisuals` (styles 0/4/12/13 per-tick) + the
alt-texture transitions. d54 = ten styles (no 9, no drivers); g11 =
seven (0-7). g61 is the strict superset.

Fidelity tie-breaks (all three agree on the load-bearing parts — the
style-10 fadeIn/scale-local-first float triple, the dead scale-ladder
rolls, the clone guard on the SPAWNED dustIndex, the style-11 Next(4)
colour pick before the position floats, the style-13 Next(1,3) scale
argument):
- **hsl bytes**: g61 rides `ui::color::hsl_to_rgb` (f64 walk,
  `round_ties_even` — byte-exact `Main.hslToRgb`, Main.cs:47890-47917);
  d54's local `hsl_to_rgb_bytes` is the same walk (equivalent); g11's
  portal colour goes through `tile_light::hsl_to_rgb`'s NORMALISED
  floats then `×255 .round()` — Rust f32 round is half-AWAY-from-zero,
  not C#'s banker's `Math.Round`, plus a float-narrowing step: a
  potential one-byte drift. → g61/d54 exact, g11 deviates.
- **Sound placement**: g61 records `play_sound(id, style, x, y)` AT the
  C# call position (the pitch draw resolves in the wiring callback
  without re-ordering — the style75 convention); g11 plays the cue
  inline on the pass stream (also faithful, but App-coupled); d54
  defers sounds entirely to the caller (ordering risk at wiring).
  g61's env-generic shape serves Player/NPC/Wiring callers alike.
- **Style 4 alpha**: g61 pre-sets A=255 before the loop, d54/g11
  post-write per dust like the C# — observation-equal (no draws
  between), cosmetic.
- Minor g61 doc nit: the style-5/7 comment says "float halves" for the
  sound centre; the CODE is integer halves (`rect.0 + rect.2 / 2`,
  correct per `Rectangle.Center`) — fix the comment at merge.

**Merge shape**: one `teleport_effect` (g61's), called from the
recall_teleport / NPC-teleport / wiring seams with the caller's pass
key; d54's `world_dust_main.rs` and g11's teleport half DROP, their
witnesses supplement g61's (§3). The pylon style-9 body stays g61's
(f6's unlanded claim yields, §2).

## 5. Merge checklist (ordered)

1. Land g14's three tr-core modules + lib.rs rows; apply the arm-11
   torch round-trip fix + the paint color-30 alpha fix; fix/replace the
   one wrong g14 witness.
2. Re-point d54's LIVE seams (editing.rs:5312/:5398/:5693/:7614,
   wiring.rs:132/:154) at the tr-core bodies; keep the burst wrappers;
   drop d54's world_dust{,_tile,_wall} bodies + the duplicate
   tile_dust_amount/solid_tile copies.
3. Re-point the four orch modules' tile sweeps (amount arg order!).
4. Resolve main_dust.rs to g13's bodies + g11's snowing (budget hoist);
   drop both other teleport ports; keep g61's module + particles.rs
   generalisation whole.
5. Resolve mount_dust.rs to f6's superset + the double-atan2 fix; keep
   e4's orchestrator_dust.rs (disjoint); record the prefix.rs fold.
6. Compile + run the witness UNION (§3) in the merge-verify wave;
   ~~record the Main sites 12-16 gap (g12 never landed code)~~
   **SUPERSEDED: g12's `665cd1e8` landed main_dust_b.rs covering
   sites 9-16 incl. 12-16 — no gap, no follow-up owed** (§2 Main row).
7. anchors.txt: keep one row per surviving module; the dropped d54
   world_dust* rows re-point at the tr-core markers (never delete a
   feature row without its replacement marker landing).

WRITTEN NOT RUN (code-only directive): no cargo/tests/graphify this
round; the mechanical censuses above were produced by read-only
git/grep comparisons, quoted per section.

## 6. EXECUTED — union-exec round on `task/W4-d54-dust-sites-world` (2026-11-21)

The checklist's d54-owned steps ran on this branch (base `b878ef70`,
the reconcile tip). Landed hashes, in order:

| Step | Hash | What landed |
| ---- | ---- | ----------- |
| 1a | `ce1276ba` | g14 claim doc cherry-pick (`46181d09` → research/65-dust-worldgen-sites-claims.md; coexists with d54's same-number claims doc, distinct filenames) |
| 1b | `8f8471cb` | g14's three tr-core modules + lib.rs rows, cherry-picked clean (`2f8931a7`/`d9b9690d`/`6ab0c7ec`): worldgen_dust{,_wall,_tile}.rs, all 29 WorldGen sites |
| 1c | `74ba4a2e` | the 2 one-line fixes on the landed copies: arm-11 `:69152` torch `*255` round-trip (d54's maths, XNA `(byte)(v*255f)` convention) + `paint_color` color-30 alpha 150 (`:44026-44032`); g14's wrong-semantics torch witness REWRITTEN to the round-trip assertion; NEW color-30 alpha-150 paint witness (neither lane had one, §3) |
| 1d | `462a21ff` | g14 docs cherry-pick (`a35efec2`, union-resolved trailing-append conflicts in anchors.txt/ANALYSIS.md — both lanes' rows kept); the ANALYSIS torch-ctor bullet corrected to the round-trip semantics + the paint fix noted; PORT_MAP's all-29 row as g14 wrote it |
| 2+3 | `e41a90f0` | LIVE seams re-pointed + duplicates dropped: wiring.rs cutter → `kill_actuator_dust`/`kill_wire_dust`; `tile_break_burst` kept, amount gate → `collision::tile_dust_amount` (kind-first!), body → `kill_tile_make_tile_dust` (`edit::solid_tile` OOB-false — d54's local `None→true` copy and duplicate amount table DROPPED per checklist 1's drop-copy option); `wall_break_burst` + `place_tile_torch_dust` (caller-side 22/140 gate kept) → tr-core bodies; world_dust.rs bodies dropped (g14's ash-shake consumes the gore argument draws); 4 orch tile sweeps re-pointed; editing.rs seams unchanged (wrapper signatures kept); 18 d54 witnesses kept as supplements re-pointed (ash oracle adapted to the gore draws; the type-table/arms oracles read the tr-core body's slot type — 454 pins the 139..=142 family, the 352 forced-scale noted); anchors rows re-pointed at surviving markers, never deleted (checklist 7) |
| 4+5 | (this doc commit) | the g11 budget-hoist cross-doc note (below) + the Main 12-16 matrix correction (§2) |

**Checklist items NOT owned by this branch (cross-doc, no code here):**

- **Step 4 — g11 snowing budget-hoist** (g11b owns the wiring): g11's
  `main_dust.rs:133` re-reads `minecart::live_type(pool, 76)` INSIDE
  the snowing loop; C# `Main.snowDust` is constant for the whole call
  (`Dust.cs:1747` increments only in `UpdateDust`, reset `:428`).
  **Fix owed by g11b: hoist the live_type read above the loop** (one
  edit) + adapt the live-76-budget-break witness (§4.3). d54's
  once-per-call snapshot body (`world_dust_main.rs`) still stands on
  THIS branch pending the main-lane resolve — the DROP lands when
  g11's fixed snowing + g61's teleport merge (the §2 filename-collision
  resolve: one main_dust.rs = g13's bodies + g11's snowing).
  **SUPERSEDED (crosslane round, research/131 §5 row 2):** research/83
  (d130 EXECUTED `88cfbf38` + the g11 addendum `d892d8d4`) rules d130's
  `snow.rs` the snowing canonical — g11's snowing body RETIRES whole,
  so the hoist target is d130's once-per-call counter param (fold F4),
  not g11's loop; the §4 "KEEP g11's body" decision and the
  g13+g11-snowing merge shape above yield to 83/131. The g11c owed
  work is the F1-F7 fold list (global stream / HiDef 4096 / pause
  gate / counter / int loop / env re-point / witness re-pins).
- **Step 5 — Main 12-16**: VERIFIED covered — g12 `665cd1e8`
  main_dust_b.rs EXISTS (817 lines, sites 9-16, 7 witnesses). The
  matrix's claim-only row was stale (g12 pushed 02:47, after the
  02:28/02:30 probes); §2 corrected, checklist 6's gap note
  superseded. No ROADMAP gap row owed.
- **Steps 4-5 main/mount resolves** (g13+g11 merge shape, g61 keep,
  f6 superset + double-atan2 fix, e4 orch keep): merger-wave actions
  on the OTHER lanes' files — this branch does not carry those files;
  the decisions stand as §2/§4 record them.

**Witness union state on this branch**: g14's 24 (one torch witness
rewritten, +1 new paint alpha witness = 25) + d54's 18 batch-1/2/3
supplements re-pointed + d54's batch-4/5-8 witnesses untouched (48
total d54 → the 9 main + 21 orch remain as landed). All WRITTEN NOT
RUN — the compile + union run stays owed to the merge-verify wave
(checklist 6). No cargo/tests/graphify invoked this round either.
