# 247 — W4-c3 RESEARCH FINAL: the cursed-skull-family bible (SEALED)

**STATUS: FINAL — SEALED.** No further W4-c3 rounds planned. Lane
`task/W4-c3-style10-cursed-skull`, worktree TR-W4-c3, tip at seal
`9c54f4c0e` (base `a725acb04`). Code-only mode throughout: no
cargo/rustc/rustfmt/test runs; every witness WRITTEN NOT RUN; verification
is C# re-reads (whole-arm line-by-line) + hand-traced arithmetic + the
origin-tip probes cited below.

**Number receipt (research/150 registry)**: 150 is the collision registry
(`38bfd1d22`, d54); its free block 151–236 is fully consumed. The 237+
first-free scan at seal time (`git log --remotes=origin --diff-filter=A
--name-only -- docs/research/`, first-add per number) finds 237–246
claimed — the max is **246** `246-temple-family-seal.md`, first add
`682f1b504` 2026-11-21T15:10:56+02:00. **247 is the first free number**;
this doc claims it (first-pushed-wins per the registry rules; if another
lane pushes a 247 first, this doc renumbers to the next free and every
on-branch ref follows — INDEX row, ANALYSIS FINAL pointer, PORT_MAP row,
ROADMAP addendum, the two anchors rows).

---

## 1. The family bible — SEALED declaration

**One paragraph**: the aiStyle-10 cursed-skull family is the three-kind
machine `NPC.AI()`'s `if (this.aiStyle == 10)` arm (`NPC.cs:21572-21918`)
— CursedSkull 34 (`SetDefaults :9150-9165`), GiantCursedSkull 289
(`:12753-12768`), WaterBoltMimic 694 (`:17737-17751`) — ported as the pure
`npc_ai_style10::style10_ai` step + `Style10Out` side-effect record
(S1-D-3-s10), WIRED on c3 through the aiStyle-axis **10** claim
(`npc.rs::update_rng_dust`, `npc.rs:4730-4769` at the seal tip) with the
kb/Item8/WaterSphere drains, the 299 Shadowflames bolt row/arm/on-hit
debuff, the faithful FindFrame arms, the GetAlpha draw-tint overrides,
the dungeon spawn band (`NPC.cs:2626-2793`) and the 299 spawn-side
census. At merge the duplicate CursedSkull/GiantCursedSkull kind rows
DROP per the d37 research/108 E8 work order (keep d34@267 / d38@289);
WaterBoltMimic 694 KEEPS c3 (mutual receipt: d43's research/244 §2d,
`6fbe06250`, adjudicates 694→c3 for the full anchor + axis claim). The
wiring — axis arm, drains, band, tints, 299 row — is the lane's lasting
contribution and MUST survive the merge (§4 hazards). No unported
style-10 behaviour remains in scope: the arm serves exactly these three
types (census `npc/data.rs:2488-2491` — `aiStyle == 10` in SetDefaults is
exactly :9154/:12757/:17741), and the only other `AI_FindNearbyBook`
caller (`NPC.cs:21254`, the 693 LibrarianSkeleton's style-8 teleport
book-scan) belongs to the g5/d43 caster lanes, not this family.

### 1.1 The three SetDefaults rows (verbatim, C# → port)

| field | 34 CursedSkull (`:9150-9165`) | 289 GiantCursedSkull (`:12753-12768`) | 694 WaterBoltMimic (`:17737-17751`) |
| --- | --- | --- | --- |
| box | 26×28 | 44×44 | 44×44 |
| aiStyle | 10 | 10 | 10 |
| damage | 35 | 60 | 20 |
| defense | 6 | 20 | 4 |
| lifeMax | 40 | 400 | 60 |
| HitSound | NPCHit2 | NPCHit2 | DD2_BookStaffCast (`dd2_book_staff_cast_0`) |
| DeathSound | NPCDeath2 | NPCDeath2 | NPCDeath52 (`NPC_Killed_52`) |
| noGravity / noTileCollide | true / true | true / true | true / true |
| value | 150 | 150 | 150 |
| knockBackResist | 0.2 | 0.2 | 0.2 (live 0/1/0.2 writes ride the override) |
| npcSlots | 0.75 | 0.75 | (default 1 — the 694 row writes none) |
| buff immunities | 20/31 (`npc_buffs.rs:490`, C# `:6287-6291` region) | 20/31 | 20/31 |

Protocol ids on c3: CursedSkull **355** (vanilla 34 is the Merchant's
protocol slot), GiantCursedSkull **289**, WaterBoltMimic **694**. Post-
merge: CursedSkull rides **d34@267** (`5f3435a8`, ancestor-verified in
`origin/task/W4-d34-npc-kinds-1-70`), GiantCursedSkull **d38@289**
(`6f9dd02d`, ancestor-verified in `origin/task/W4-d38-npc-kinds-281-350`),
694 unchanged. The spawner/bestiary/sheet code keys on `NpcKind`, so the
renumber is invisible to the wiring (sheets stay NPC_34/NPC_289/NPC_694 —
`texture_net_id`/`cs_id` keyed).

### 1.2 The machine (module `npc_ai_style10.rs`)

Whole arm in C# order with per-block citations (module header + inline):
flags `:21574-21600`; closed/wake book states 3/4 `:21601-21627` (park +
kb 0→1, the 80-tick wake); the 600/650 enrage cycle + <100 retreat /
<250 wobble chain `:21633-21678`; distance bands `:21679-21698`; dead-
target flee `:21709-21713`; charge damp/14-snap `:21714-21730`; X/Y
approach `:21731-21749`; aim rotation + 694 flip `:21750-21766`; the 289
volley machine `:21767-21823` (≤500 band, 120-tick arm → 40-tick dash
window, the tick-20 bolt `:21804-21811`: kind 299, damage 25, 6-speed aim
from the PRE-move centre `center3` re-read at `:21774`); the 694
charge/dash cycle `:21828-21917` (the tick-17 Item8 cue `:21896` + Water
Sphere 33 summon `:21899` at the truncated centre+10, the −300 cooldown).
Integration tail: `UpdateCollision`'s noTileCollide half (`:92538-92540`,
`:92605-92607`); c3's module version adds the dead-band X snap
(`:90473-90476`) ahead of the integration — **the merger keeps c3's
module copy** (superset of the base S1-D-3-s10 module d34/d38/d43 carry).
The axis arm stamps `oldPosition`/`oldDirection` (`:90496-90497`) before
the call (`npc.rs:4737-4739`).

**RNG by construction**: the arm's ONLY `Main.rand` draw is `Next(3)` at
`:21851`, gated `flag19 && (!flag20 || …)` — exactly the both-bands host
tick (100..300 charge AND ≤500 dash, `ai[2] >= 0`, `ai[3] == 0`); the
34/289 paths and the client (`netMode == 1`, early return `:21847-21850`)
draw nothing. The oracle witness
(`rng_witness_only_the_both_bands_mimic_tick_draws`) mirrors the gate on
a second stream over 300 ticks.

**The drains** (c3 axis arm `npc.rs:4749-4768`):
- `Style10Out::knock_back_resist` → `Npc::kb_resist_override` (0 state 3
  `:21606`, 1 state 4 `:21619`, 0.2 mimic `:21630`; the skulls never write).
- `play_item8_sound` → `wants_sound` (`NpcSoundCue::Item8` at
  `base.position`, the S1-E-21g queue).
- `spawn_water_sphere` → `wants_sphere` (`WaterSphere` 33, the sim's
  sphere drain Bottom-places it, `NPC.NewNPC` `:80770`, ai3 arg 0).
- the 289 bolt → `Npc::wants_hostile_shot` pushed IN-module
  (`npc_ai_style10.rs:434`), drained by `world_sim/npcs.rs:5869` →
  `spawn_hostile` (`:5934`).
- the dead `dontTakeDamage = ai[3] == 4` write (`:21631`) carried verbatim
  (state 4 already returned at `:21626` — always false; witness pins it).
- `justHit` = the one-tick `self.just_hit` flag; `netMode` folds 0 (the
  single-player host — the module's client gate is inert).

### 1.3 The 299 Shadowflames bolt (row + arm + debuff + census)

- **Row**: `projectile/data.rs:473-477` (`SHADOWFLAMES: u16 = 299`) +
  `:1998-2002` — `Projectile.cs:3893-3905`: 6×6, aiStyle 1, hostile,
  penetrate −1, extraUpdates 2, alpha 255, magic, ignoreWater,
  tileCollide false.
- **Arm**: `projectile_ai001_a.rs`'s `AI_001` 299 case
  (`Projectile.cs:56631-56656` — the `localAI[0]` counter, the Item8 cue
  at 6, the flag3-rest straight flight; dust arms = documented cosmetic
  skip). Base sprite alpha-255 invisible exactly as the C# (generic
  `GetAlpha` fade, `Projectile.cs:79744-79757`); the glowmask/dust visual
  is the recorded skip.
- **On-hit**: Cursed 23 / 240 ticks in `damage_player`
  (`projectile.rs:23439-23445`, `StatusPlayer` `Projectile.cs:12280-12283`,
  gated on the accepted hit).
- **Spawn-side census (gap 1, `9299046f7`)**: the whole decompiled tree
  carries exactly ONE `NewProjectile(…, 299, …)` site — `NPC.cs:21808-21810`
  (the 289 volley). The Lead's GoblinSummoner/ShadowFlameBow premise does
  not hold: item 3014 = ClingerStaff (shoots 482, `Item.cs:27882`),
  ShadowFlameBow = item **3052** (shoots arrow 495, `Item.cs:28223`);
  GoblinSummoner 471 is aiStyle-3 spawning NPC 30 (`:56514`). No unclaimed
  spawn; the witness `w4_c3_the_299_spawn_drain_lands_the_row_hostile`
  pins the drain seam (row lands hostile, slice-A dispatch flies it).

### 1.4 FindFrame + draw tints + anchors

- **FindFrame**: 34/289 reuse the faithful wave-B arms
  (`npc_findframe_b::find_frame_b` — case 34 the 4-tick 3-row strip
  `NPC.cs:77055-77066`, case 289 the 8-tick cycle + ai[3]==1 spin bank
  `:72652-72675`); the 694 book strip (`:77202-77289`, 30 rows: closed
  pins 0, wake climbs 0..16 at 5 ticks, charge 17..23, dash 24..29, idle
  cycle) ported inline in `npc.rs::find_frame` (`:6790`). Post-merge the
  canonical d34/d38 kinds carry their own equivalent case arms (d34
  `npc.rs:6904` region, d38 `:9533` region) — c3's 34/289 arms drop, the
  694 arm stays (d43's 694 FindFrame is equivalent; keep ONE — c3's).
- **GetAlpha tint overrides (gap 4, `c0bc6c0de`)** through the
  `draw_npcs` tint seam (`render/entities.rs:3309-3334`, the d70
  `GetMoonLordColoring` co-lane pattern):
  - 34: `Main.cs:23120-23123` sets `npcColor = White` for
    `aiStyle == 10 && type != 694` BEFORE GetAlpha; no type arm → the
    generic fade rides White → `[fade; 4]`.
  - 289: `NPC.cs:93609-93611` returns the literal (250,250,250,50) EARLY
    — no fade.
  - 694: `:93417-93427` early return — ai[3] 3 → `newColor` (light),
    ai[3] 4 → `Lerp(newColor, White, clamp(frame/17))`, else White.
  These are DRAW-VISIBLE self-illumination (the house light-fold would
  hide the skulls in the dark dungeon). The 694 tint is frame-determined
  (book-state ↔ frame coupled) so the G6 `render_key`'s `frame` term
  already disambiguates — no new key term.
- **The 694 anchor terms** (`num45` X + `num46` Y, `Main.cs:23100-23114`):
  `npc_anchor_nudge` (`entities.rs:933`, the ai[3]==3/4 offsets) + the
  new `npc_anchor_nudge_x` (`:1017`). **d43 has NO 694 anchor** — its
  research/244 §2d names c3's anchor the keeper. Merger keeps both fns'
  694 arms.

### 1.5 The dungeon spawn band (gaps 2+3, `cb5cef764` + `9c54f4c0e`)

`spawner.rs::spawn_dungeon_band` (`:2037`) ports `Spawner.SpawnAnNPC`'s
`else if (this.ZoneDungeon)` arm (`NPC.cs:2626-2793`) in roll order:

| C# | roll | outcome | port |
| --- | --- | --- | --- |
| `:2628-2642` | wall pick + RollLuck(7)==0 → Next(3) | num40: slab 94/96/98→1, tile 95/97/99→2 (tile check second, wins mixed) | verbatim (RollLuck = plain Next, the spawner's luck fold) |
| `:2643-2652` | !downedBoss3 | 68 Guardian | None (unported; consumes nothing) |
| `:2653-2657` | Next(5)==0 + guards | 123 bound Mechanic | `bound_mech_ok && below_deep_dungeon` |
| `:2658-2662` | hardDungeon && Next(30)==0 | 287 BoneLee | consume-None |
| `:2663-2677` | hardDungeon && num40 0/1/2 && Next(15)==0 | 293/291/292 | consume-None |
| `:2678-2682` | hardDungeon && !AnyNPCs(290) && num40 0 && Next(35)==0 | 290 Paladin | consume-None (AnyNPCs unmodelled) |
| `:2683-2687` | hardDungeon && num40 1/2 && Next(30)==0 | **289 GiantCursedSkull** | `spawn_hover` |
| `:2688-2705` | hardDungeon && Next(20)==0 → Next(2) | 281-286 casters | consume-both-None |
| `:2706-2719` | hardDungeon && Next(3)!=0 → Next(4) | 269-280 armored bones | consume-both-None |
| `:2720-2724` | RollLuck(35)==0 | 71 DungeonSlime | `spawn_caster` |
| `:2725-2729` | num40 1 && Next(3)==0 && !NearSpikeBall | 70 SpikeBall | consume-None (scan unmodelled) |
| `:2730-2734` | num40 2 && Next(5)==0 | 72 BlazingWheel | `spawn_hover` |
| `:2735-2739` | num40 0 && Next(7)==0 | **34 CursedSkull** | `spawn_hover` |
| `:2740-2744` | Next(7)==0 | 32 DarkCaster | `spawn_caster` |
| `:2745-2753` | Next(8)==0 → AI_FindNearbyBook | **694 WaterBoltMimic** at (bookX·16+8, bookY·16), ai3=3 | Bottom-placed closed book |
| `:2755-2763` | else Next(10)==0 → book-find | 693 LibrarianSkeleton | flag13 consume-None (d43/g5 lane) |
| `:2764-2768` | Next(5) ALWAYS draws; flag13 → return | num43 | verbatim order |
| `:2769-2792` | num43 0/1/2 → 294/295/296; else Next(4)==0 / Next(5)==0 → −14/−13/31 angry bones | consume-None | verbatim |

`hardDungeon = downedPlantBoss && hardMode` (`NPC.cs:380`) wired into the
live ctx (`world_sim/npcs.rs:6620`). The band sits after the water ladder,
before the surface/moon ladders. The unmodelled pre-dungeon arms (wall-244
spider/gnome chain, bartender/spider/UD/golfer specials, evil-water
hardmode arms, beach/angler chains, granite/marble SetSpawnFlagsForChosenTile
draws, surface critter arms) are the documented consume-skip deviation.
**g61 cross-note**: natural SpawnNPC/NewNPC family — NO TeleportEffect
(research/238 seam-6 receipt); the book-find and skull rolls draw no
teleport dust.

**AI_FindNearbyBook** (`NPC.cs:62308-62364`, port `spawner.rs:2169`):
32×32 scan at (spawnTile−16, spawnTile−16), closest active book (type 50)
passing `CheckNotSpawningOnScreen`; distance from the scan's TOP-LEFT
corner; the closestBook path writes no `_nearbyBooks` entry and draws no
RNG. **The return gate** (`:62349-62357`, faithfulness fix `9c54f4c0e`):
`vector` starts AT the search position — a book exactly at the origin
(or no book) compares equal and returns false; the port filters
`best != (sx, sy)`.

### 1.6 Presentation

Bestiary ENTRIES (`bestiary.rs:2178-2201`,
`BestiaryDatabaseNPCsPopulator.cs:1112/:1559/:1561`), portraits
(`ui/bestiary.rs:1584-1601`), sheets NPC_34/289/694 (atlas + bestiary
tables, `assets.rs`), cue registrations `NPC_Killed_52` (`audio.rs:176`)
+ `dd2_book_staff_cast_0` (`:438/:507`). Net relay seeds ride the hover
placement (`app/netglue.rs:3435-3440` — all three are noTileCollide
flyers).

---

## 2. The protocol-355 adjudication (gap 5) — RECEIPT

d37's pushed band **354-399** (`750d8e44`; re-verified at current tip
`2ea22ce3e`: `npc/data.rs:598` CyanBeetle→355, `:977` 355→CyanBeetle)
collides with c3's CursedSkull 355 (and c4's FlyingFish 354). d37's
research/108 roster-master-reconcile (77 contested ids, 18 claimants;
354-399 the most-squatted corridor) adjudicates at wave scope:

- **CursedSkull: keep d34@267, drop c3@355** (d34 kinds-1-70 band,
  `5f3435a8`; also drops g8@375).
- **GiantCursedSkull: keep d38@289, drop c3@289** (d38 kinds-281-350
  band, `6f9dd02d`).
- **WaterBoltMimic 694: c3 KEEPS** (vanilla-free, `NPCID.cs:12065`).
  **Mutual receipt**: d43 grew a 694 port AFTER 108's census (batch 13
  `0253bde90`, tip probe: kind row + `update_water_bolt_mimic`
  `npc.rs:12723` — a kind-keyed style-10 call WITH drains but NO axis
  claim, NO anchor terms, NO tint override, NO spawn band, NO 299 row);
  d43's own retire doc research/244 (`6fbe06250`, §2d) drops it onto c3:
  "keep c3's wired skull-family member … c3's `npc_anchor_nudge` 694 arm
  (num45 X + num46 Y) is the keeper — d43 has NO 694 anchor". Both lanes
  agree; no dispute remains.

The Lead's earlier "c3/c4 keep 353-355, d37 shifts" recommendation is the
OPPOSITE of 108 R8's receipt (d37 ZERO-move, the 46-kind band
byte-identical to its witness-pinned port commit). c3 accepts the E8 work
order AT MERGE per the dropped-lane rule (rows stay on-branch; the drop
executes in the merge wave — the d42 `49346dba6` precedent inverted, here
pre-published as hunks).

## 3. E8 drop-hunks (c3 side, at seal tip `9c54f4c0e`) — for the merger

Drop ONLY the two duplicate kind identities; every wiring block stays,
keyed on the canonical variants (same names — `NpcKind::CursedSkull` /
`NpcKind::GiantCursedSkull` resolve to d34/d38's rows post-merge).

| file | hunks (seal-tip lines) | action |
| --- | --- | --- |
| `npc.rs` | enum docs+variants `:1089-1104` | drop both; keep WaterBoltMimic `:1105-1112` |
| `npc.rs` | kb arm `:3560`, defense `:3943-3944`, value `:4320`, unreachable arm `:6229-6231` | strip the two variants from the shared arms (keep 694) |
| `npc.rs` | FindFrame arm `:6757-6789` (34/289 → find_frame_b) | drop (canonical d34/d38 case arms own it) |
| `npc.rs` | `to_u16` `:18810-18811` (+comment repoint `:18805-18809`) | drop (d34@267/d38@289 own) |
| `npc.rs` | ALL_KINDS `:24161-24162`; witness rows `:24296-24297`, `:24319-24322`, `:24326-24327`, lists `:24334`/`:24353-24354`, `:31504-31505` | drop the two-kind entries; re-pin the defaults/axis witnesses onto the canonical protocol ids (267/289) |
| `npc/data.rs` | `to_u16` `:596-597`, `from_u16` `:932-933`, hit/death sound `:1812`/`:2120`, noGravity `:2195-2196`, ai_style `:2491`, size `:3024-3025`, cs_id `:3106-3107`, hp `:3676-3677`, dmg `:4045-4046` | strip the two variants (d34/d38 rows own every field; keep 694 in the shared arms) |
| `bestiary.rs` | ENTRIES `:2178-2193` (two defs) | drop (d34 `bestiary.rs:2178` net_id 34 / d38 `:2440` net_id 289 own); keep the 694 def |
| `ui/bestiary.rs` | portraits `:1584-1595` | drop both arms (canonical lanes own); keep 694 |
| `render/entities.rs` | draw arms `:3235-3240`, tint arms `:3310-3316`, sheet whitelist `:3445-3446`, witness `bare(CursedSkull)` half `:3940-3953` | drop the DRAW arms (d34/d38 draw their sheets); **KEEP the tint arms** re-keyed onto the canonical variants (d34/d38 have NO GetAlpha overrides — verified: zero (250,250,250) hits in d38 entities.rs); keep the 694 tint + anchor + witness |
| `npc_buffs.rs` | `:490` shared arm | strip the two variants (d34 `:489` / d38 `:525` own) |
| `app/netglue.rs` | relay lines `:3435-3436` | strip (d34 netglue `:3578` / d38 `:3518` own) |
| `anchors.txt` | `:899` row text | repoint (drop "protocol 355"/three-kinds wording; the marker `unreachable!("aiStyle 10 rides the style axis (W4-c3)")` SURVIVES on the 694-only arm — do not delete the row) |

**STAYS whole**: `npc_ai_style10.rs` (c3's superset copy — dead-band X
snap), the axis `10 =>` arm + drains (`npc.rs:4730-4769`), the 299
row/arm/debuff, `spawn_dungeon_band` + `find_nearby_book` + the 4 spawner
witnesses (kind-keyed — renumber-transparent), the 694 kind row +
FindFrame + anchor + tint, sheets/cues/694 bestiary, the two new anchors
rows (this commit).

## 4. MERGE HAZARDS — re-verified at CURRENT origin tips (seal-time probe)

Tips probed: d34 `5f3435a87` (06:12:31), d38 `ff4b8ed32` (15:02:51),
d37 `2ea22ce3e` (15:02:36), d43 `0253bde90`+`6fbe06250` (15:55:04).

1. **d34/d38 DISCARD `Style10Out`.** d34 `update_cursed_skull`
   (`npc.rs:9654`, dispatch `:6499`): `let _ = style10_ai(…)` — no
   drains, no oldPosition/oldDirection stamps. d38
   `update_giant_cursed_skull` (`npc.rs:12296`, dispatch `:6555`):
   `let _ = out;` (`:12309`) with a comment claiming the 289 writes
   nothing — true for kb/sphere/Item8, but the wrapper ALSO lacks the
   `:90496-90497` stamps c3's axis arm carries.
2. **d34/d38 have NO `10` axis arm.** Both style axes enumerated at
   tips: 5,21,9,27,28,29,49,56,72,30,31,43,47,55,50,69,70,71,75 — no 10.
   Their kinds route through `update_kind_ai` wrappers instead.
3. **d38 (and d43) have NO 299 proj_data row.** `proj_data(299)` →
   None → `spawn_with_data_ai_owned`'s `let data = proj_data(kind)?`
   (d38 `projectile.rs:15974`) → None → `spawn_hostile(299)` SILENTLY
   FAILS. The 289's in-module `wants_hostile_shot` push drains into a
   no-op: the Giant Cursed Skull would never fire.
4. **d34/d38/d43 carry the BASE style10 module** (no dead-band X snap)
   and **no GetAlpha tint overrides / no 694 anchor / no spawn band**.

**Merger instructions (blocking)**:
- Route the canonical d34@267 / d38@289 kinds through **c3's axis `10`
  arm** (their `ai_style()` already maps 10 — d34 data.rs `:2555`, d38
  `:2661` — so adding c3's arm claims them automatically); drop or
  unreachable-mark the d34/d38 wrapper dispatch arms. This restores the
  drains AND the `:90496-90497` stamps.
- Keep **c3's `npc_ai_style10.rs`** copy (superset).
- Keep **c3's 299 row + AI_001 arm + Cursed 23/240 debuff** — without
  them hazard 3 fires.
- Keep **c3's tint arms** re-keyed onto the canonical variants, the 694
  anchor fns, the spawn band, the 694 kind row (d43's 694 drops per its
  own research/244).
- Post-merge run the catalog command + the re-pinned
  `w4_c3_the_style10_defaults_match_the_csharp_rows` /
  `w4_c3_the_style10_axis_claims_the_three_kinds` witnesses (protocol
  267/289/694) as the family's activation gate.

## 5. Lane ledger + witness register

Commits (base `a725acb04`): `f6b0468d9` wiring (kinds + axis + drains +
299 row/arm/debuff + FindFrame + presentation) → `8d988f8be` docs+anchor
→ `9ea0ee90d` Item8 pre-move-position witness fix → `9299046f7` gap 1
census → `cb5cef764` gaps 2+3 spawn band → `c0bc6c0de` gap 4 tints →
`b6571f899` gaps 1-5 docs → `9c54f4c0e` gap 2 book-origin gate → this
FINAL.

Witnesses — **24, all WRITTEN NOT RUN** (code-only):
- module 15 (`npc_ai_style10.rs:598-1054`): enrage 600/650, distance
  bands, wobble wrap, dead-target flee, rotation+flip, 289 tick-20
  volley, hit-disarm, charge damp/snap, closed-book wake 80, mimic
  retreat, tick-17 sphere, −300 cooldown, the RNG oracle, client skip,
  kb write + dead dontTakeDamage.
- wiring 9: `w4_c3_the_style10_defaults_match_the_csharp_rows`
  (npc.rs:24293), `w4_c3_the_style10_axis_claims_the_three_kinds`
  (:24349), `w4_c3_the_shadowflames_bolt_row_and_arm`
  (projectile_ai001_a.rs:821), `w4_c3_the_299_spawn_drain_lands_the_row_hostile`
  (:871), `w4_c3_the_giant_skull_needs_hard_dungeon_and_a_slab_wall`
  (spawner.rs:2839), `w4_c3_the_cursed_skull_needs_the_default_wall`
  (:2870), `w4_c3_the_water_bolt_mimic_spawns_at_the_closest_book_closed`
  (:2892), `w4_c3_the_dungeon_band_is_terminal_and_deterministic`
  (:2926), `w4_c3_the_style10_get_alpha_overrides`
  (render/entities.rs:3936).

Standing deviations (recorded in ANALYSIS "The aiStyle-10 family" +
"W4-c3 wiring"): the consume-skip spawn arms (§1.5), the unmodelled
AnyNPCs/NearSpikeBall scans, RollLuck→Next luck fold, the 299 glowmask/
dust cosmetic skip, `GetSpawnSourceForProjectileNPC` host bookkeeping,
netMode-0 fold. None open a follow-up on this lane.

**SEAL**: gaps 1-5 landed; the 355 collision adjudicated (d37 keeps,
c3 drops two at merge, 694 keeps with the d43 mutual receipt); the
merge hazards published with tip-verified evidence; the family bible
sealed. No further c3 commits planned after this doc + its refs.
