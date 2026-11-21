# W4-g85 — the shadow orb / crimson heart draw census + port (code-only)

Worker W4-g85 (`task/W4-g85-shadow-orb-draw`, base `a725acb0`). Code-only
mode: **no cargo/rustc/tests/graphify invoked**; every witness below is
WRITTEN NOT RUN. The task: the orb family's draw-side behaviour per the
C# `TileDrawing` census (research/09 row "Particles/dust from tiles —
MISSING") — the glow pulse and the hammer-break stages.

## 1. Census — where the family's draw behaviour lives

The family is tile 31 (ShadowOrbs) + 696 (ShadowOrbsReplica); the crimson
heart is NOT a separate tile — it is the **`frameX >= 36` half of the
same 31 sheet** (`WorldGen.AddShadowOrb`, `WorldGen.cs:32095-32132`:
`num = 0; if (crimsonHeart) num += 36;` then the 2×2 writes
`frameX = num / 18+num`, `frameY = 0 / 18`). The replica 696 mirrors the
31 animation frame (`Main.cs:18588`). Every draw-side site keys on the
same stage bit:

| Site | C# | Behaviour | Port state at base |
| --- | --- | --- | --- |
| Glow pulse (light seed) | `TileLightScanner.cs:2965-2983` | one `FastRandom.Next(-5,6)*0.0025f` per tile; orb stage purple `(0.31+n, 0.1, 0.44+2n)`, heart stage orange `(0.5+2n, 0.2+n, 0.1)` | **LANDED** — `tile_light/scanner.rs` `misc_lights` `26\|31\|695\|696` arm (:1051-1061); this slice adds 2 stage/jitter witnesses, no second copy |
| Animation row | `TileDrawing.cs:5433-5440` (`addFrY = Main.tileFrame[type]*36`), animator `Main.cs:18578-18588` (counter `>= 8`, 4 rows, 696 frame-only mirror) | 4-row pulse cycle, 8 ticks/row | **LANDED** — `drawdata.rs` :739-741 group `12\|31\|96\|639\|665\|696` + `drawdata_anim.rs` `step_ge_ge(31, 8, 4)` + the 696 mirror (:289-290), WIRED S1-G-1f |
| Ambient draw dust | `TileDrawing.DrawTiles_EmitParticles` `:7335-7347` | `(31\|\|696) && _rand.Next(20)==0`: heart stage dust 5 alpha 100 + `velocity.Y=0; velocity.X*=0.3`; orb stage dust 14 alpha 100 | **MISSING on main** — ported here (`render/orb_draw.rs::orb_ambient_dust`), UNWIRED |
| Shine sparkle colours | same tail `:7413-7432` gate + `:7511-7518` + `:7527-7528` | Heart 12 red / ManaCrystal 639 blue dust 43 alpha 254 scale 0.5 velocity 0; `Next(tileShine=300)` drawn before the `>= max/30` miss test | **MISSING on main** — ported here (`orb_shine_sparkle`), UNWIRED |
| Hammer-break dust stages | `WorldGen.KillTile` `:63699-63703` → `KillTile_GetTileDustAmount` `:66601-66606` (default 3 fail / 10 break) → `KillTile_MakeTileDust` `:68966-68969` + plain tail `:69166` | heart stage dust 5 (NO draw); orb stage `genRand.Next(2)!=0 ? 1 : 14` (one `Main.rand` draw); plain `NewDust(pos,16,16,num)` alpha 0 scale 1 | **MISSING on main** — ported here (`orb_break_dust_type` / `orb_break_burst`), UNWIRED |
| Break reaction (sim, not draw) | `WorldGen.CheckOrb` `:54169-54344` | 2×2 integrity cascade, drops, third-break summon | LANDED (M4-EVENT-3c, `edit/kill.rs::check_orb`); the break-tick sound tail rides W4-c14 (`5dc20f8c`, in flight) — NOT touched here |

### Census corrections / pins

1. **`:7511`'s 665 colour row is dead code.** The line reads
   `if (typeCache == 12 || typeCache == 665) newColor = red;` but
   `Main.tileShine[665]` is NEVER written (grep of `Main.cs`: only
   `[12] = 300` `:7518` and `[639] = 300` `:9652`), so the `:7413` gate
   `Main.tileShine[typeCache] > 0` never admits 665. The port's
   `orb_shine_sparkle` returns false for 665 with zero draws; a witness
   pins it. (d60's whole-battery `shine_arm` keeps the row inside its
   `tile_shine` table gate — same observable behaviour.)
2. **The altar pair 26/695 is the g86 lane.** Its ambient arm
   (`:7348-7361`, threshold `frameX >= 54`, dust 5 scale 1.5 noGravity
   velocity×0.75) and its break arm (`:68970-68973`) share the shape but
   NOT the threshold; `orb_break_stage` returns `None` for 26/695 so the
   two slices cannot double-fire.
3. **No `MakeTileDust` re-write.** A full scan of `:66687-69169` shows
   the ONLY 31/696 mentions are `:68966`/`:68968`; no special spawn arm
   (`:69085-69165`) matches the family, so the burst tail is the plain
   `:69166` `NewDust` (alpha 0, scale 1, default colour).
4. **The `:7335` arm's chain position draws nothing before it for orb
   kinds.** The preceding else-if arms (22 `:7319`, 23/24/32 `:7323`,
   25 `:7327`, 112 `:7331`) fail their TYPE test before the `&&` gate,
   so an orb cell reaches the arm with zero prior `_rand` draws — the
   slice function is stream-exact standalone.

## 2. RNG census (S1-A-10 — three streams, never mixed)

- `TileDrawing._rand` (`UnifiedRandom`, `TileDrawing.cs:191`): the
  ambient `Next(20)` (once per orb cell per emit tick, miss included)
  and the shine `Next(300)` (drawn BEFORE the `>= num42` miss test;
  the dark-light early return `:7415-7418` draws NOTHING).
- `Main.rand`: every `Dust.NewDust` internal (position pair, velocity
  pair, frame row, scale — `Dust.cs:153-193` via
  `tr_core::particles::DustPool::new_dust`) AND the break stage's
  `Next(2)` (`genRand` IS `Main.rand`, `WorldGen.cs:4445`). Per break
  dust the order is: stage `Next(2)` (orb stage only) THEN the NewDust
  internals; the heart stage is draw-free outside NewDust.
- The scanner's per-tile `FastRandom` (`WithModifier(x,y)`): the glow
  pulse's single `Next(-5,6)` — a third stream, untouched by this slice.

The emit entry gate (`TileDrawing.cs:965-968`:
`_isActiveAndNotPaused && (!Lighting.UpdateEveryFrame ||
FastRandom(TileFrameSeed).WithModifier(x,y).Next(4)==0)`) is the wiring
wave's obligation (the d60 module doc records the same seam).

## 3. What landed

- `crates/tr-app/src/render/orb_draw.rs` (new, UNWIRED —
  `render/special_tiles.rs` convention): `orb_break_stage`,
  `orb_ambient_dust`, `orb_shine_sparkle`, `orb_break_dust_type`,
  `orb_break_burst` + 10 witnesses (stage bit, gate-once/miss-zero-draw,
  heart velocity clamp vs a mirrored plain spawn, orb plain dust 14,
  shine dark-return/roll-before-miss/colours/665-dead, break type
  draw census, burst counts + stream-order mirror).
- `crates/tr-core/src/tile_light/scanner.rs`: 2 glow-pulse witnesses
  (`w4_g85_orb_glow_stage_flip_matches_the_jitter_recipe`,
  `w4_g85_orb_glow_jitter_is_per_tile_and_bounded`) — the landed arm is
  NOT modified.
- `render/mod.rs`: the module registration.
- anchors.txt / PORT_MAP / ANALYSIS / ROADMAP rows.

## 4. Keep-one boundaries (for the merger)

- **W4-d60** (`tile_draw_particles.rs`, in flight) ports the WHOLE
  `DrawTiles_EmitParticles` battery; its `ambient_dust_tail` carries the
  same `:7335-7347` arm and its `shine_arm` the same colour rows. Keep
  ONE body per arm at merge — this slice's functions are the orb-family
  verbatim subset; the witnesses pin the contract either way.
- **W4-d54** (`world_dust_tile.rs`, in flight) ports the WHOLE
  `KillTile_MakeTileDust` table including the `:68966` arm and a
  `tile_break_burst`. Same keep-one rule; `orb_break_burst` is the
  family slice with the fail-3/break-10 count fold.
- **W4-g86** (altar draw) owns 26/695; `orb_break_stage` excludes them.
- **W4-c14** owns the CheckOrb break-tick SOUND tail; this slice is
  dust/light only.

## 5. Verification

Code-only: every ported line re-read against the cited C# windows
(`TileDrawing.cs:7335-7347`/`:7413-7529`, `TileLightScanner.cs:2965-2983`,
`WorldGen.cs:32095-32132`/`:63699-63703`/`:66601-66606`/`:68966-68969`/
`:69166`, `Main.cs:7518`/`:9652`/`:18578-18588`). 12 witness tests
WRITTEN NOT RUN — no cargo/rustc/rustfmt/tests/dotnet/graphify invoked.

## 6. The wiring round (2026-11-21, same branch, code-only)

Three seams, one commit each:

### Seam 1 — the emit entry gate + the ambient/shine call site (`3be55964`)

* `orb_draw::emit_entry_gate` ports `TileDrawing.cs:965-968` verbatim:
  `_isActiveAndNotPaused && (!Lighting.UpdateEveryFrame ||
  FastRandom(TileFrameSeed).WithModifier(x,y).Next(4) == 0)`. The
  quarter-roll is a FRESH per-tile derivation (`with_modifier` XORs into
  a copy — no stream advance, a miss costs nothing). `UpdateEveryFrame`
  (`Lighting.cs:77-87`) = `!RenderTargetsRequired && !NotRetro`: true
  only in Retro/Trippy at zoom 1; the port's frame path never binds the
  target family (S1-G-13 allocation-only), so `RenderTargetsRequired`
  stands false — recorded deviation (the zoom > 1 capture path is
  unported).
* The call site is a family-kind-gated block in `render/tiles.rs` at the
  C# slot — BEFORE the dark-cell merge (`:975-991` comes after the
  battery in C#; a dark orb cell still emits, the shine dark-return is
  inside the body). Gates ride `leaf_rng` (the pinned per-rebuild `_rand`
  stand-in) through the `GateRand` adapter; the `NewDust` internals ride
  the `frame_rng` clone snapshot written back after the pass (the d54
  break-seam convention).
* **Chain-position census (stream-exactness).** Every battery arm before
  `:7335` fails its TYPE/SET test for 31/696/12/639 before any `_rand`
  draw: 718/244/565/215/592/406/452/192/384/666/712/711, the
  `SpawnsNatureFlies` set (3,20,24,27,73,201,80,110,529,530,595,615,637),
  the `MakesRubbleDust` set, 238/139/463/497/165/42, the torch/lamp arms
  4/93/100/98/49/372/646/34/83, then 22/23/24/32/25/112/26/695/71/72/
  17/77/133/405/37/58/76/684/61/703. The family reaches its arm with
  ZERO prior draws. The pre-entry `DrawSingleTile` arms are the potion
  seams: dangerSense `:911 Next(30)` (the family is not dangerous —
  `IsTileDangerous`'s sets exclude it), spelunker `:930 Next(60)`
  (**12/639 ARE `tileSpelunker`** — `Main.cs:9424`/`:9651` — so a
  findTreasure buff WOULD draw before the entry; the port has not wired
  the spelunker draw arm, constants-off, zero draws today), biomeSight
  `:955 Next(480)` (the family is in no BiomeSight set). When the potion
  draw arms land they must draw BEFORE this block — contract note.
* **Contested seam.** `render/tiles.rs` + `app/frame/draw.rs` are touched
  by ~12 in-flight draw lanes (d58/d59/d69/g33/g34/g68/g75/g78/g84/f14/
  f15/c39/d127); the block is family-gated (zero draws for every other
  kind) so d60's whole-battery keep-one replaces it wholesale at merge.

### Seam 2 — the hammer FAIL-tick dust + the d54/g14 keep-one census (`2fcc9439`)

* **d54/g14 table census.** BOTH pushed lanes carry the `:68966` arm:
  d54 `world_dust_tile.rs:1595-1605` and g14
  `worldgen_dust_tile.rs:1381-1391` — verbatim-identical to
  `orb_break_dust_type` (the `25|31|696` gate, the `(31|696) && fx>=36`
  short-circuit 5 with NO roll, else `Next(2)!=0?1:14`). Keep ONE at
  merge; the witnesses here pin the contract either way. d54's
  `break_tile` wiring (`tile_break_burst` between the sounds and the
  drops, the `frame_rng` snapshot for the type chain + `UpdatePlayers`
  for the spawns) owns the 10-dust BREAK seam — NOT duplicated here.
* **The unclaimed half: the fail tick.** `hammer_tile_hit`'s sub-100 arm
  is `KillTile(fail: true)` (`Player.cs:45692`); the C# dust loop
  (`WorldGen.cs:63699-63703`) runs BEFORE the fail return (`:63736`), so
  every sub-100 hammer hit puffs 3 stage dusts. `orb_hammer_fail_dust`
  wires exactly that (the `CanKillTile` refusal + vanished-cell guards
  precede the draws; `UpdatePlayers` stream — `genRand` IS `Main.rand`,
  `:4445`). The fail SOUND stays c14's `play_kill_fail_cue` lane (its
  call lands at the same site — keep-one comment).
* **Hammer-break stage progression census.** `Main.tileHammer[31]/[696]`
  (`Main.cs:7206-7207`) route the orb to the hammer arm; neither is
  `tileNoFail`. There is NO orb-specific damage accumulation in
  `KillTile` — the progression is the i31 `HitTile` book: an Iron Hammer
  (40) needs 3 hits (40+40+20 ≥ 100), i.e. puff-puff-break. The witness
  pins it.

### Seam 3 — the CheckOrb loot census fix (`200674ec`)

* **The drop switch is FOUR cases, not one table** (`WorldGen.cs:
  :54226-54333`): 696 → the replica item 5469/5470 by the `fx>=36` bit
  (the dispatch arm `:85251` was missing 696 — added); 12 is the
  Heart/Life Crystal OBJECT → 29 (the pre-fix port wrongly rolled the
  5-way table for it — the crimson heart is 31's `fx>=36` half, NOT tile
  12); 639 → 109. All three: NO roll, NO flags. ONLY case 31 rolls the
  5-way tables and raises `shadowOrbSmashed`/`shadowOrbCount`
  (`:54292-54293` sits INSIDE case 31). The `Next(5)` draws even on the
  forced first smash (`:54240`/`:54267` draw-then-override).
* **The case-0 Musket Ball pair** (the documented ObjectDrop follow-up,
  unclaimed): the gun (96/800) spawns WITH a 97 stack of
  `genRand.Next(100, 101)` ≡ 100 — one dead-range draw kept. Resolved
  app-side in `break_drop_events` (gun id + `tile_kind == 31` gate),
  `UpdatePlayers` stream.
* **Meteorite spawn trigger census.** `CheckOrb` sets NO `spawnMeteor`.
  The triggers: the orb-boss deaths (`NPC.cs:79453-79456` — guaranteed
  first / `Next(2)` after; landed `npcs.rs:5174` via
  `meteor::boss_death_roll`) and the nightly `Next(50)` after downedBoss2
  (`Main.cs:65230`; landed `weather.rs:367` via `meteor::night_roll`);
  the 16200-tick resolution landed `meteor.rs:244`. NOTHING to wire.
* **Goblin-army unlock census.** `shadowOrbSmashed` arms (a) the dawn
  invasion roll (`Main.cs:65448-65461` — 1/3 pre-defeat, 1/30 classic /
  1/60 hardmode after; landed `world.rs:970 roll_morning_invasion` +
  the `npcs.rs:4052` arm resolve) and (b) the Goblin Scout spawn gate
  (`NPC.cs:4478` — `!downedGoblins && shadowOrbSmashed && Next(7)==0`,
  the second-chance arm after the `Next(15)`). (b) rides the UNPORTED
  day-surface ladder (`spawner.rs`'s day arm falls through to the basic
  slime; 6 lanes contest spawner.rs) — UNWIRED contract note: when the
  day ladder lands, the scout arm draws `Next(15)` then (on miss, flag
  gates) `Next(7)`, in that order, on the SpawnNPC stream.
* The break-tick sound tail stays c14/g24 (both carry the `CheckOrb`
  `:54335-54342` tail — their own keep-one); the third-break summon
  stays the landed sim edge (`npcs.rs:5699`).

### Wiring-round verification

Code-only: every ported line re-read against the cited C# windows
(`TileDrawing.cs:864-968`/`:6688-7432`, `Lighting.cs:63-87`,
`Main.cs:2585-2595`/`:7206-7207`/`:9424`/`:9651`, `WorldGen.cs:
:54169-54344`/`:63537-63736`/`:66597-66606`/`:68966-68969`,
`Player.cs:45646-45782`, `NPC.cs:4478`/`:79443-79458`). 6 new witness
tests WRITTEN NOT RUN (2 entry-gate, 2 fail-tick/progression, 2
loot-switch/gun-pair) — no cargo/rustc/rustfmt/tests/dotnet/graphify
invoked. 18 witnesses total on the branch.

## 7. Wiring round 2 (2026-11-21, same branch, code-only) — the four contract seams

### 7.1 The spelunker :930 pre-entry draw — WIRED (family slice)

**Claim census first.** The contract's "buff-11?" is a misread: the
Spelunker Potion is **buff 9** (`Item.cs:4914-4927` — item 296,
`buffType = 9`, `buffTime = 18000`; buff 11 is Shine, a player-light
effect with no flag). The potion-use arm + the flag are ALREADY landed
on main: `potions.rs::use_potion_item` grants `buff_type` generically
(`Player.cs:42876-42883`), the UpdateBuffs arm 9 sets `find_treasure`
(`player.rs:6464`, S1-E-7a wave A, `Player.cs:9981-9984`), and
`ResetEffects` clears it (`player.rs:5980`, `:18367-18374`). The
`tileSpelunker` bitset + `IsTileSpelunkable` are W4-d9's in-flight
`projectile_ai_style90.rs` (`667e1f0c2` — the MagicLantern 10-tick scan,
49 ids + the 185/186 frame exceptions) — claimed, NOT duplicated here.
The DRAW GATE was the unclaimed half: d60's module doc explicitly defers
the `DrawSingleTile` prologue arms `:911-962` ("also out of this body's
scope").

**The family verdict.** `Main.tileSpelunker[12] = true` (`Main.cs:9424`)
and `[639] = true` (`:9651`); **31/696 are NOT in the set** — the shadow
orbs themselves are not treasure-highlighted (only the Heart/LifeCrystal
object 12 and the ManaCrystal 639 are). So the :930 seam activates for
exactly 12/639, and the §6 contract note's "a findTreasure buff WOULD
draw before the entry" applies to those two kinds only.

**The port** (`orb_draw.rs`, three functions + 2 witnesses):

* `spelunker_family(kind)` — `matches!(kind, 12 | 639)`, the family slice
  of `IsTileSpelunkable` (`Main.cs:21783-21798`). Keep-one: d9's full-set
  function at merge; the family verdict is identical either way.
* `spelunker_light_lift(light)` — `:922-929`: `R = max(R, 200)`,
  `G = max(G, 170)`, **B untouched** (no B clause — unlike dangerSense's
  `:907-910` B≥50). The lift is draw-free and runs whether or not the
  sparkle roll hits.
* `spelunker_dust(...)` — `:930-936`: `_isActiveAndNotPaused &&
  _rand.Next(60) == 0` → `Dust.NewDust(pos, 16, 16, 204, 0, 0, 150,
  default, 0.3)` then `fadeIn = 1`, `velocity *= 0.1`, `noLight = true`.
  **NO noGravity** — the dangerSense arm `:917` sets it, this one does
  not (a witness pins the difference).

**The wiring** (`render/tiles.rs`): per cell, `spelunk = active &&
player.find_treasure && spelunker_family(kind)`; `tile_light =
spelunker_light_lift(raw_l)` when admitted, else `raw_l`. The sprite
light `l` ghosts `tile_light` (the C# `GetFinalLight` `:4380` applies
`actColor` AFTER the potion lift); the emit battery receives
`tile_light` (`:967` passes `tileDrawInfo.tileLight`) — so a buff-lit
12/639 cell is never dark to the shine arm's `:7415` early return. The
sparkle call sits BEFORE the `:965` entry block and is NOT gated by the
quarter-roll (the C# arm order); it rides `leaf_rng` (the `_rand`
stand-in) + `orb_main_rand` (the NewDust internals).

**Stream order.** For 12/639 the preceding prologue arms draw nothing:
dangerSense `:911 Next(30)` — the family is in no `IsTileDangerous` set
(`:4437-4446`: the explicit ids, `Boulders`, `TouchDamageImmediate`,
`TouchDamageHot`, the 483/482/481/51/229/162 tail all exclude 12/639);
biomeSight `:955 Next(480)` — no BiomeSight set member (`TileID.cs:578/
:582/:587`). So the `Next(60)` is the FIRST `_rand` draw for an admitted
cell, then the battery's ambient type test fails (12/639 are not 31/696)
and the shine `Next(300)` follows — stream-exact with the C#.

### 7.2 The Goblin Scout day-surface ladder — SPAWN CONTRACT (unclaimed, doc-only)

**Claim census.** Six in-flight lanes touch `crates/tr-app/src/spawner.rs`
(c38 moon-wave rosters, c40 sandstorm, c41 town-spawn invaders, c6
pumpking blade, f32 luck consumers, g17 gore-world sites — re-derived by
`git diff --name-only origin/main...<branch>` over all 353 W4 branches).
NONE carries scout / `NPC.cs:4478` / day-surface-ladder content (grep
`scout|4478|day_surface` over each lane's spawner.rs: zero hits). W4-g61
(npc-spawn-dust) ports the spawn/teleport-in DUST visuals, not the ladder.
The scout gate is therefore UNCLAIMED. It is NOT self-contained/small:
the arm sits ~15 else-if rungs deep in the day-surface chain, each
preceding rung drawing on `Main.rand` — porting it alone would desync the
stream. Per the contract this lane publishes the EXACT spawn-contract for
the spawner-lane owner; the arm itself stays UNWIRED.

**The enclosing gates (all must hold to reach the day ladder).**
`SpawnAnNPC` (`NPC.cs:1282`) dispatches by zone; the surface ladder is the
`else if (this.surfaceSpawn)` arm (`:4165`), reached only when no
tower/corruption/crimson/jungle/dungeon/meteor/etc. zone arm matched and
`!this.invaders` (the invaders ladder `:1424` is a SIBLING — an active
goblin army spawns invasion enemies, NEVER the scout). Inside surfaceSpawn:
the graveyard `:4172`, snow-rain `:4184`, rain `:4189`, martian-probe
`:4194` specials return early; then `if (!this.ZoneGraveyard &&
Main.dayTime)` (`:4199`) opens the DAY ladder. So the scout's full
precondition: **surface band (`spawnTileY <= Main.worldSurface`), day
(`Main.dayTime`), not graveyard, no invasion active, no zone special.**

**The day-ladder draw order (`NPC.cs:4202-4501`, `num45 =
Math.Abs(spawnTileX - Main.spawnTileX)`).** Each rung is an `else if`;
the `&&` operand order is the DRAW order — a term left of the
`Main.rand` roll gates it draw-free, a term RIGHT of the roll is tested
only after the draw is consumed (and a failed right-term still leaves
the draw spent). The scout is rung 11. The rungs and their exact draw
positions:

| # | C# | Gate terms LEFT of the first draw (draw-free if false) | First draw | Terms RIGHT of the draw / follow-ups |
| - | -- | ---------------------------------------------------- | ---------- | ------------------------------------ |
| 1 | `:4202` | `!waterTile && num45 < maxTilesX/2` | `Next(15)` | `tileType ∈ {2,477,109,492,147,161}` AFTER the roll; hit → the critter sub-ladder (snow 148/149 `Next(2)`, stinkbug/butterfly/gold/holiday/bird rolls) |
| 2 | `:4302` | `!waterTile && beachDistance < spawnTileX < maxTilesX-beachDistance` | `Next(12)` | `tileType==53` AFTER; hit → `Next(366,368)` |
| 3 | `:4306` | `(tileType ∈ {2,477,53}) && !tooWindy && !raining && dayTime` | `Next(3)` (`!=0`) | `(spawnTileY<=worldSurface \|\| remix) && FindCattailTop` AFTER; hit → goldCritter + dragonfly rolls |
| 4 | `:4325` | `!waterTile && num45 < maxTilesX/3 && dayTime && time<18000 && (tileType ∈ {2,477,109,492})` | `Next(4)` | `spawnTileY<=worldSurface && CountNPCS(74)+CountNPCS(297)+CountNPCS(298)<6` AFTER; hit → `Next(4)` + goldCritter |
| 5 | `:4346` | `!waterTile && num45 < maxTilesX/3` | `Next(15)` | `tileType ∈ {2,477,109,492,147}` AFTER; hit → `Next(4)` + goldCritter |
| 6 | `:4367` | `!waterTile && num45 > maxTilesX/3 && tileType==2` | `Next(300)` | `!AnyNPCs(50)` AFTER; hit → SpawnOnPlayer(50) |
| 7 | `:4371` | `!waterTile && num45 > maxTilesX/8 && (tileType ∈ {2,147,60,161})` | `Next(160)` | `!AnyNPCs(696) && !AnyNPCs(695)` AFTER; hit → HasItem(5663/5664) + RollLuck(100) |
| 8 | `:4386` | `!xRange && tileType==53 && (spawnTileX<beachDistance \|\| spawnTileX>maxTilesX-beachDistance)` | `Next(10)` | the beach water chain (`!waterTile` → 602; water → the 50-row scan + `Next(10)` + `Next(4)`) |
| 9 | `:4470` | `!waterTile && tileType==53` | `Next(5)` | `Spawning_SandstoneCheck` AFTER; hit → 69 |
| 10 | `:4474` | `tileType==53 && !waterTile` | none | → 61 (vulture), terminal for sand |
| 11 | `:4478` | **THE SCOUT** — `!waterTile && (num45 > maxTilesX/3 \|\| remixWorld)` | `Next(15)` | on miss: `!downedGoblins && shadowOrbSmashed` then `Next(7)` — see below |
| 12 | `:4482` | `raining` | `Next(4)` | hit → 224 |
| 13 | `:4486` | `!waterTile && raining` | `Next(2)` | hit → 225 |
| 14 | `:4490` | `!waterTile && wallType==0 && IsItAHappyWindyDay && isSpawningInWindDirection` | `Next(3)` (`!=0`) | hit → 594 |
| 15 | `:4494` | `!waterTile && wallType==0 && (tileType ∈ {2,477}) && happyWindyDay && windDir` | `Next(10)` (`!=0`) | `Main.tile[spawnTileX,spawnTileY].type==tileType` AFTER; hit → 628 |
| 16 | `:4498` | `!waterTile` | none | → `GetBasicSlimeToSpawn(surface:true, tileType, num45)` |

**The scout arm verbatim (`NPC.cs:4478-4481`):**
```csharp
else if (!this.waterTile && (num45 > Main.maxTilesX / 3 || Main.remixWorld)
    && (Main.rand.Next(15) == 0
        || (!NPC.downedGoblins && WorldGen.shadowOrbSmashed && Main.rand.Next(7) == 0)))
{
    this.SpawnNPC(spawnTileX * 16 + 8, spawnTileY * 16, 73);
}
```
**Draw contract for the spawner-lane owner:**
1. Static gate FIRST (draw-free): `!waterTile && (num45 > maxTilesX/3 ||
   remixWorld)`. If it fails, the rung draws NOTHING and falls through.
2. `Main.rand.Next(15)` — drawn ALWAYS when the static gate passes.
   `== 0` → spawn 73 (GoblinScout), return.
3. On a `Next(15)` MISS, evaluate `!NPC.downedGoblins &&
   WorldGen.shadowOrbSmashed` (both draw-free flags). ONLY if both hold,
   draw `Main.rand.Next(7)`; `== 0` → spawn 73, return. The `Next(7)` is
   short-circuited away when either flag is false (a miss with
   `downedGoblins` or `!shadowOrbSmashed` consumes ONLY the `Next(15)`).
4. Both miss → fall through to rung 12 (the rain arms).

So the scout's per-attempt draw budget is **1 (`Next(15)`) + conditionally
1 (`Next(7)`)** — the second draw exists ONLY while the goblin army has
not been downed AND an orb has been smashed. This is the "second-chance"
gate the §6 census named. The spawn is `SpawnNPC(spawnTileX*16+8,
spawnTileY*16, 73)` → `NewNPC`'s `Bottom=(X,Y)` placement (feet on the
found tile's top edge, X centred 8 px in), the same machinery the port's
invaders/moon arms use. `NpcKind::GoblinScout` (73) is W4-d35's in-flight
data wave (`npc.rs`, `NPC.cs:9732-9746`) — the kind lands there; this
contract is the SPAWN gate only.

**Port insertion point.** `spawner.rs:1155-1160` — the day arm currently
falls straight to `spawn_basic_slime` ("the day ladder's
critter/water/wind specials all sit behind unmodelled tile+weather
gates"). The ladder owner replaces that fallthrough with rungs 1-16 in
the table order; the scout rung reads `ctx.world.shadow_orb_smashed` +
`ctx.world.downed_goblins` (both landed fields, `world.rs:374`/`:213`)
and `ctx.world.seeds.contains(SeedFlags::REMIX)` for the remix term;
`num45` is `(x - ctx.world.spawn_x).abs()` (the `spawn_basic_slime` dist
convention). The stream is the same `UnifiedRandom` the night ladder
already rides (the `SpawnNPC` swap, `Main.rand`).

**Why the flags matter (the progression chain).** `shadowOrbSmashed` is
raised ONLY by case-31 orb smashes (`WorldGen.cs:54292`, the g85 seam-3
fix — 12/639/696 never arm it). `downedGoblins` clears the gate once the
army is defeated. So the scout spawns in the window **after the first
shadow-orb smash, before the goblin army is downed** — exactly the
pre-invasion scouting phase. The dawn invasion roll (`Main.cs:65448-65461`,
landed `world.rs::roll_morning_invasion`) is the OTHER `shadowOrbSmashed`
consumer; both read the same flag the g85 loot fix now sets correctly.

### 7.3 The progression-7 chain — VERIFIED + the case-31 gate reconciled

**The contract question:** does c36's landed `orb-smash-7` read the
case-31-only flags the g85 seam-3 fix produces? **Verdict: the chain is
correct AFTER this round's reconciliation; c36's in-flight copy alone
would mis-fire.**

* **The C# position.** `AchievementsHelper.NotifyProgressionEvent(7)`
  (`WorldGen.cs:54331`) sits INSIDE `case 31` — after the `:54322-54330`
  chat arm, before the `:54332` `break`. The 12/639/696 cases break at
  `:54230`/`:54233`/`:54236` and NEVER reach it. So SMASHING_POPPET fires
  on a 31 smash only (orb OR crimson-heart half — both are case 31).
* **c36's copy (`6c8eb75dd`, in flight, NOT on main).** Its `check_orb`
  sets `world.orb_smash_notify_pending = true` UNCONDITIONALLY (its base
  predates the four-case fix — its `check_orb` still rolls the 5-way table
  for every kind and raises the flags for every kind). Merged AS-IS with
  the g85 seam-3 fix, the 12/639/696 breaks would queue a notify the C#
  never fires. The g85 seam-3 fix corrected the FLAGS but c36's notify
  rode the old unconditional shape.
* **The reconciliation (this round).** The flag set moves INSIDE the
  `if kind == 31` block in `check_orb` (`kill.rs`), beside the
  `shadow_orb_smashed`/`shadow_orb_count` writes — the same case-31 gate.
  The `World.orb_smash_notify_pending` field + the `break_tile` drain
  (`editing.rs`, `std::mem::take` → `notify_progression_net(7)`) are
  ported here with the c36 keep-one note; at merge keep ONE field/drain
  and THIS gate.
* **The full chain, verified end to end:** case-31 smash →
  `check_orb` sets `orb_smash_notify_pending` (and the smash flags) →
  `break_tile` drains it → `notify_progression_net(7)`
  (`netglue.rs:2974`: host broadcasts `S2c::ProgressionEvent`, SP/client
  fires locally) → `achievements.notify_progression(7)` →
  `SMASHING_POPPET` (`achievements_data.rs:163-168`,
  `Cond::Progression(7)`, `AchievementInitializer.cs:49-51`). The
  `netMode != 1 && !generatingWorld` gate (`:54224`) is the port's
  standing single-player/host reduction (the whole drop switch already
  rides it).
* **Drain coverage (a c36-lane observation, NOT changed here).** The
  drain sits in `break_tile` — the player-mining path. Other
  `edit::kill_tile` callers (projectile tile-breaks `projectile.rs`,
  collision `collision.rs:973`, touch-damage, the player wall-break
  `player.rs:1364`) can also trigger `check_orb` on a 31 cell; their
  notify stays pending until the next `break_tile`. That is c36's drain
  placement (kept verbatim for keep-one); widening it to every kill path
  is the c36/achievement lane's call, not this draw lane's.

The four-case witness (`w4_g85_the_orb_drop_switch_is_four_cases_not_one_table`)
now also pins the notify: 12/639/696 leave `orb_smash_notify_pending`
false, the 31 smash sets it, and the `mem::take` drain takes it once.

### 7.4 The orb/heart LIGHT emit — VERIFIED scanner-seeded (NOT AddLight)

**The contract question:** is the shadow-orb/crimson-heart light wired
through `AddLight` (the g18-oracle co-lane)? **Verdict: the family's light
is SCANNER-seeded, not entity-list `AddLight` — already landed, nothing
to wire; g18's census independently confirms the AddLight remainder is
empty.**

* **The orb family (31/696 + the 26/695 altars).** The glow pulse is
  `TileLightScanner.ApplyTileLight`'s `case 26/31/695/696`
  (`TileLightScanner.cs:2965-2983`): the per-tile
  `FastRandom.Next(-5,6)*0.0025f` jitter, purple `(0.31+n, 0.1, 0.44+2n)`
  for the orb stage (`fx<36`), orange `(0.5+2n, 0.2+n, 0.1)` for the
  heart/altar stage. LANDED in `tile_light/scanner.rs::misc_lights`
  (`:1051-1061`, the `26|31|695|696` arm) — the g85 slice added 2
  stage/jitter witnesses there, NOT a second copy. The scanner writes the
  light into the `LightMap` directly (`apply_tile_light`), NOT through
  `Lighting.AddLight`.
* **The Heart object (12) / ManaCrystal (639).** Neither has a top-level
  scanner `case 12`/`case 639` light arm — every `case 12` in
  `TileLightScanner.cs` (`:838`/`:1123`/`:1250`/`:1895`/`:2173`) is a
  NESTED style-row inside another kind's switch (torch 4 / chandelier 93 /
  lamp 34 colour tables), and 639 has NO scanner mention at all. Their
  visible glow is the SHINE effect (`Main.tileShine[12]=300`
  `Main.cs:7518`, `[639]=300` `:9652` + `tileShine2[639]` `:9653`) — the
  `Main.shine` light modulation in `GetFinalLight` (`:4384-4387`) and the
  sparkle dust the g85 `orb_shine_sparkle` already ports. Not an AddLight
  source either.
* **The g18 co-lane census (read, not duplicated).** W4-g18
  (`547154708`/`a99dfa964`) re-derived f9's five-site verdict: Main.cs has
  EXACTLY five `Lighting.AddLight` call sites, ALL in `DrawProj`'s
  hook-chain arms (the 332 Christmas hook `:29016-29028` four colour rows
  + the 488 Illuminant hook `:29186`), all claimed by e8's in-flight
  `tile_lights.rs`. Its NEGATIVE census: "Main.cs has NO update-side or
  ambient AddLight: tile ambience (torches/lava/glowing blocks) is
  scanner-seeded (TileLightScanner, ported in tr-core/tile_light), not
  entity-list fed." The orb family is exactly that scanner-seeded
  ambience — so there is NO orb AddLight site for g18/g19 to claim, and
  the g85 ambient/shine wiring (the dust) + the landed scanner arm (the
  light) are the complete picture.

**Nothing to wire for item 4** — the light emit was already landed
(scanner) and the g85 dust wiring completes the draw side. This section
is the verification record the contract asked for.

### Wiring-round-2 verification

Code-only: every ported line re-read against the cited C# windows
(`TileDrawing.cs:897-968`/`:4358-4402`/`:4435-4467`/`:6624-6686`,
`Main.cs:9420-9467`/`:9649-9653`/`:21778-21818`, `Item.cs:4914-4927`,
`Player.cs:9981-9984`, `NPC.cs:442-471`/`:1282-1453`/`:4165-4502`,
`WorldGen.cs:54169-54344`, `TileLightScanner.cs:2965-2983` + the nested
`case 12` rows, `TileID.cs:504`/`:551`/`:578-609`). Co-lane reads: d9
`667e1f0c2` (the tileSpelunker bitset), d60's module doc (the prologue
deferral), c36 `6c8eb75dd` (the unconditional notify), g18
`547154708`/`a99dfa964` (the AddLight census), d35 (the GoblinScout kind).
2 new spelunker witnesses + the extended four-case notify census WRITTEN
NOT RUN — no cargo/rustc/rustfmt/tests/dotnet/graphify invoked. 20
witnesses total on the branch (18 prior + 2 spelunker; the notify census
extends an existing witness).

## 8. The FINAL phase (2026-11-21, same branch, code-only) — the summon split, the meteor gates, the keep-one receipt

### 8.1 The c36 progression-7 keep-one — FINAL VERDICT: the g85 case-31 gate is CANONICAL

**Re-probe of the c36 lane at its LATEST pushed tip `4e124425c`** (the
RE-PROBE doc-reconciliation commit; the three RE-PROBE commits
`4a8d56d99`/`774902cfc`/`4e124425c` touch the real-estate roster, the
20-row gap list and the docs — NONE touches `check_orb`). The
`git grep` of the tip confirms:

* `crates/tr-core/src/edit/kill.rs` (c36 tip): `check_orb` still rolls
  the 5-way table for EVERY kind (its `crimson` branch feeds 12/639/696
  too), still raises `shadow_orb_smashed`/`shadow_orb_count`
  UNCONDITIONALLY, and still sets `world.orb_smash_notify_pending = true`
  UNCONDITIONALLY (its `:1168-1170` block, outside any kind gate).
* `crates/tr-app/src/world_sim/editing.rs` (c36 tip): the same
  `break_tile` drain (`mem::take` → `notify_progression_net(7)`) at the
  same slot as g85's — the drain halves are IDENTICAL.
* The c36 tip's own witness (`kill.rs:5017-5025`) asserts the notify on
  a generic orb smash — it does NOT pin the per-kind census.

**Verdict: c36 did NOT fix the unconditional set in flight.** The g85
round-2 reconciliation (§7.3) stands as the four-case-correct half:

1. **Keep ONE `World.orb_smash_notify_pending` field** — the two
   definitions are behaviourally identical (a `bool`, default false);
   keep g85's doc comment (it records the case-31 position).
2. **Keep ONE `break_tile` drain** — byte-identical logic; either copy.
3. **Keep the g85 `check_orb` WHOLE** — the four-case drop switch AND
   the `if kind == 31` flag block (the smash flags + the notify + the
   final-phase `orb_smash_crimson` bit). c36's `check_orb` must be
   DROPPED at merge, not merged line-by-line: its unconditional flag
   block would re-break the 12/639/696 census the g85 witnesses pin
   (`w4_g85_the_orb_drop_switch_is_four_cases_not_one_table` fails on
   c36's copy — the 12 break asserts `!orb_smash_notify_pending`).
4. The c36 lane's OTHER batch-5 sites (NOT_THE_BEES, GO_LAVA_FISHING,
   TALK_TO_NPC_AT_MAX_HAPPINESS) are disjoint — keep them all.

The merger's one-line rule: **g85's `check_orb` + either drain + either
field; c36's `check_orb` is the stale pre-four-case base.**

### 8.2 The SHADOW/CORRUPT smash differences — census + the summon split WIRED (`21e8eb1d7`)

**The C# census (`WorldGen.cs:54169-54344`, re-read whole).** The orb
family's smash behaviour splits on TWO axes:

* **The KIND axis** (the four-case switch, §6 seam 3): 696 the replica
  (5469/5470, no roll, no flags), 12 the Heart object (29), 639 the Mana
  Crystal (109), 31 the real orb/heart (the 5-way roll + ALL the world
  flags). Only 31 progresses the world.
* **The CRIMSON axis** (`flag` = `frameX >= 36`, `:54175-54180` — a
  per-SMASH bit, NOT the world evil): inside case 31 it selects the
  drop table (crimson: Undertaker/Crimson Rod/Rotted Fork/pet/Panic
  Necklace; corruption: Musket/Vilethorn/Ball O' Hurt/pet/Band of
  Starpower) AND the summon half (`:54312-54318`):
  - crimson → `NPC.SpawnOnPlayer(plr, 266)` — the **Brain of Cthulhu**;
  - corruption → `NPC.SpawnOnPlayer(plr, 13)` — the **Eater of Worlds**.
  The living-boss gate `:54296` `!(AnyNPCs(266) & flag) && (!AnyNPCs(13)
  || flag)` reduces PER HALF: crimson → `!AnyNPCs(266)` (a living EoW
  does NOT block the Brain), corruption → `!AnyNPCs(13)` (a living
  Brain does NOT block the EoW). A drunk world's chasm holds both
  halves; each summons its own boss.

**The smash→boss-spawn chain, verified end to end.** The chain is NOT
`world.downed*`-gated (the W-A2-3 T3c census already pinned the
living-boss scan): case-31 smash → `shadowOrbCount++` (`:54293`) → the
`>= 3` threshold (`:54294`, OR the FTW+DontStarve-not-remix override) →
the living-boss gate → `shadowOrbCount = 0` + the nearest-player scan
(`:54299-54311`, Manhattan distance over all 255 slots) →
`SpawnOnPlayer(plr, 13|266)` (`NPC.cs:80475` — the `netMode == 1` entry
refusal `:80477`, then the 500×50 spawn-area scan → `SpawnBoss`
`:80631`). The port's count-poll (`npcs.rs`, the W-A2-3 site) keeps the
threshold/gate/reset shape; the nearest-player scan folds to the port's
single player and the spawn-area scan to the standing at-the-player
deviation (`use_boss_summon_item`'s same reduction).

**What was missing (now WIRED, `21e8eb1d7`):** the pre-fix poll summoned
the EoW for BOTH halves ("the crimson-Brain arm rides the heart-
detection wave" — a deferral that never landed). The fix:

* `World.orb_smash_crimson` (new field): the core kill plan records the
  smash's own `crimson` bit inside the `if kind == 31` block, beside the
  smash flags — the poll reads the LAST completed smash's bit (the
  vanilla summons synchronously inside that smash; the count-poll
  deferral keeps the bit stable between the smash and the poll).
* The poll's per-half gate + spawn: crimson → the Brain (`Npc::new`,
  the `use_boss_summon_item` 1331 arm's shape — the Creepers' orbit is
  the recorded follow-up), corruption → the worm head (unchanged).
* The `:54294` FTW+DontStarve-not-remix override (every smash summons;
  the `count >= 1` poll floor is the smash-edge stand-in — an idle
  FTW world at count 0 never summons).
* The `:54322-54330` chat arm (the else of the THRESHOLD, not of the
  living-boss gate — a blocked third smash chats nothing): misc[10]
  "A horrible chill...", the second misc[11] "Screams echo...",
  `ChatColors.World`. The port broadcasts via a count watermark
  (`World.orb_smash_chat_count`, port-only bookkeeping, resets with the
  count on summon) — the poll cannot see per-smash edges.
* The `SpawnOnPlayer` `netMode == 1` client guard (`app.is_client()`).

**The f35 co-lane read (`b72cdfbfc`, the expert-drop-trees tip).** f35
owns the DEATH side (NPCLoot/DoDeathEvents drop trees); its `npcs.rs`
carries the SAME orb-boss death arm (`:5168-5186` at its tip — the
meteor + downed_eow block, identical to main's pre-g85 shape) and the
SAME pre-fix EoW-only summon poll (`:5699-5714`). f35 does NOT touch
`check_orb` or the summon's crimson axis — no overlap to reconcile
beyond the standard keep-one: at merge, g85's death-arm gates (the
skyblock unlock + the surface gate, §8.3) wrap f35's identical
meteor/downed block, and g85's summon poll replaces f35's stale copy.
f35's drop-tree content (the 23-kind boss table, the potion chain) is
disjoint — keep it all.

### 8.3 The meteorite-landing gates — census + the missing gates WIRED (`21e8eb1d7`)

**The contract question:** does the smash flag feed the meteor
eligibility? **Verdict: NO — and that is the C# behaviour.** The full
trigger census (three sites, all re-read):

1. **The orb-boss DEATH** (`NPC.cs:79443-79458`, the 13/14/15/266 case
   group): `this.boss` → the skyblock lowTiles `shadowOrbSmashed = true`
   unlock (`:79449-79452`) → `isThereAWorldSurface && (!downedBoss2 ||
   Next(2) == 0)` → `spawnMeteor = true` (`:79453-79456`) →
   `SetEventFlagCleared(ref downedBoss2, 14)` (`:79457`). The smash
   itself sets NO meteor flag — the summoned boss must DIE. LANDED
   pre-g85 (`npcs.rs` boss_death_roll + the shared `downed_eow`); g85
   final adds the TWO missing gates: the skyblock unlock (the seed-flag
   lowTiles stand-in — the `loot.rs` LootFlags.skyblock convention; a
   skyblock world past the 10% tile threshold keeps the flag down, the
   recorded stand-in deviation) and the `worldSurface > 50.0` gate
   (`Main.cs:2788` — the remix/FTW inversion never meteors).
2. **The nightly bookend** (`Main.cs:65230`, `UpdateTime_StartNight`):
   `Next(50) == 0 && netMode != 1 && downedBoss2` — the DRAW is the
   leftmost operand (consumed every bookend regardless), the boss flag
   is the DEATH flag (`downed_eow`), NOT `shadowOrbSmashed`. LANDED
   pre-g85 (`weather.rs` night_roll); g85 final extracts
   `meteor_night_bookend` with the client gate on the RAISE only and
   the `:65234-65237` no-surface cancel (boundary-only — a mid-day
   death flag survives to the next morning's `HandleMeteorFall`).
   The crossing arm itself stays the recorded DEAD arm (ANALYSIS
   S1-A-10's follow-up row: the clock advances in `time_phase` before
   `update_world_tick`, so `dusk_pre && time >= 32400` never fires
   there) — the helper keeps the gate faithful and directly
   witnessable; re-homing the crossing is the clock lane's call.
3. **The resolution** (`Main.cs:65469-65502`, `HandleMeteorFall`):
   `time > 16200 && spawnMeteor` → clear + `dropMeteor` (the skyblock
   `StartMeteorShower` + the `Next(3)` retry are the recorded
   deviations). LANDED (`meteor.rs::resolve`, the 16200 gate, DW-2.2's
   stream witness). LIVE in the port — the boss-death flag resolves on
   the next morning tick.

**The chain, end to end:** 3-orb smash → summon (g85 §8.2) → boss dies
→ `downed_eow = true` + the guaranteed first `spawn_meteor` (surface-
gated) → the next morning `resolve` lands the ore; afterwards the
nightly 1/50 keeps the world meteor-eligible. The goblin unlock rides
the SAME death flag's sibling (`shadowOrbSmashed`, set by the smash
itself OR the skyblock death) → the dawn invasion roll
(`world.rs::roll_morning_invasion`, landed) + the scout gate (§7.2's
contract).

### 8.4 FINAL verification + the g85-family-complete declaration

Code-only: every ported line re-read against the cited C# windows
(`WorldGen.cs:54169-54344` whole, `NPC.cs:79443-79458`/`:80475-80633`,
`Main.cs:2788`/`:65135-65194`/`:65210-65246`/`:65469-65502`,
`WorldGen.cs:3205-3234` the lowTiles census). Co-lane reads: c36
`4e124425c` (the tip re-probe — the unconditional set UNFIXED, §8.1),
f35 `b72cdfbfc` (the death-side drop trees — the identical death arm +
the stale summon poll, §8.2). 4 new witnesses + the four-case witness
extension WRITTEN NOT RUN — no cargo/rustc/rustfmt/tests/dotnet/graphify
invoked. 24 witnesses total on the branch (20 prior + 4 final; the
crimson-bit census extends an existing witness).

**G85-FAMILY-COMPLETE.** The shadow-orb/crimson-heart family's full
behaviour surface is now ported or contract-published on this branch:

| Half | State |
| --- | --- |
| The draw slice (ambient/shine/entry-gate/spelunker) | WIRED (§6 seam 1, §7.1) |
| The hammer fail-tick dust + the 3-hit progression | WIRED (§6 seam 2) |
| The four-case loot switch + the gun-pair | WIRED (§6 seam 3) |
| The progression-7 notify (case-31-gated) | WIRED + the c36 keep-one receipt (§7.3, §8.1) |
| The light emit | VERIFIED scanner-seeded, nothing to wire (§7.4) |
| The Goblin Scout spawn gate | CONTRACT published (§7.2 — the ladder is the spawner lanes') |
| The SHADOW/CORRUPT summon split + the chat arm + the FTW override | WIRED (§8.2) |
| The orb-boss death gates (skyblock unlock, surface gate) | WIRED (§8.3) |
| The meteorite trigger census (death/bookend/resolution) | VERIFIED + the missing gates WIRED (§8.3) |

Remaining cross-lane deferrals (NOT g85 scope, all cited): the 10-dust
break burst (d54's `tile_break_burst`), the fail sound (c14), the full
tileSpelunker set (d9), the day-surface ladder (the spawner lanes), the
meteor-crossing re-home (the clock lane), the Brain's Creeper orbit
(the Boss AI wave), the `SpawnOnPlayer` spawn-area scan (the standing
at-the-player deviation).

## 9. The RETIRE phase (2026-11-21, same branch, doc-only) — the exact merger receipts

### 9.1 The c36 check_orb DROP receipt — exact line ranges at tip `3a64a1b96`

**Re-probe at the NEW latest pushed tip `3a64a1b96`** (NOTIFY-SYNC (5);
the §8.1 read was at `4e124425c`). The `4e124425c..3a64a1b96` diff
touches `achievement_hooks.rs`, `npcs.rs` (a comment-only hunk at
`:4945`, the Truffle arrival note), and the docs — **`edit/kill.rs`,
`world.rs` and the `editing.rs` drain are UNTOUCHED**, so the §8.1
verdict stands verbatim: c36's `check_orb` is STILL the unconditional
pre-four-case base. The exact ranges the merger needs (all at c36 tip
`3a64a1b96`, verified by direct read):

| Site | c36 tip `3a64a1b96` | g85 tip `dd237e331` (KEEP) |
| --- | --- | --- |
| `kill.rs` dispatch arm | `:650-652` — `12 \| 31 \| 639 => check_orb` (NO 696) | `:650-654` — `12 \| 31 \| 639 \| 696` |
| `kill.rs` fn doc-comment | `:1087-1095` | `:1089-1099` |
| **`kill.rs` `fn check_orb` WHOLE** | **`:1096-1184` — DROP THIS ENTIRE FN** | `:1100-1208` — keep whole |
| the stale drop table | `:1129-1159` — ONE 5-way table; the `crimson` branch feeds 12/639/696 too; the roll is SKIPPED on the forced first smash (`:1136-1140` — the `:54240` draw-then-override order lost) | `:1133-1172` — the four-case switch, the roll drawn even when forced (`:1148-1149`) |
| the unconditional flag block | `:1164-1170` — `shadow_orb_smashed`/`shadow_orb_count`/`orb_smash_notify_pending` set for EVERY kind, outside any gate; NO `orb_smash_crimson` | `:1177-1194` — the `if kind == 31` block (flags + `orb_smash_crimson` `:1186` + notify `:1193`) |
| `world.rs` field | `:380-384` (`orb_smash_notify_pending` only; the doc says "every completed orb/heart smash") + default `:734` | `:398-409` (the case-31 doc) + `orb_smash_crimson` `:380-390` + `orb_smash_chat_count` `:391-397`; defaults `:759-761` |
| `editing.rs` `break_tile` drain | `:5661-5665` (`mem::take` → `notify_progression_net(7)`) | `:5642-5650` — behaviourally IDENTICAL; either copy |
| witnesses | `the_orb_break_sets_the_world_flag_and_count` `:4984-5028` (asserts the notify on a generic smash — would PASS on c36's copy, FAIL to pin the census) + `the_first_smash_forced_gun_and_the_roll_table` `:5030-5074` | the same two at `:5008-5048`/`:5050-5094` PLUS `w4_g85_the_orb_drop_switch_is_four_cases_not_one_table` `:5103-5174` (the 12/639/696 no-notify pins + the crimson-bit census) |

**The merger's cut list (c36 side):** delete `kill.rs:1087-1184`
(doc-comment + fn), keep g85's `:1089-1208` in its place; take g85's
dispatch arm (`696` included); keep ONE field (g85's doc) + ONE drain
(either); DROP c36's two orb witnesses ONLY IF g85's supersets are
present (they are — same names, extended bodies); keep every other
c36 batch-5/NOTIFY-SYNC site (NOT_THE_BEES, GO_LAVA_FISHING,
TALK_TO_NPC_AT_MAX_HAPPINESS, the 40/43 retirements, the TO_INFINITY/18
published hunks — all disjoint from the orb lane).

**The 5043-drop-hunk item (c36's Torch God's Favor) — READ, NOT g85's.**
At `3a64a1b96` the three-way keep-one is recorded in c36's
`research/30:330-339` + `ANALYSIS.md:14614-14618`: **c36** owns the USE
half (`editing.rs:2089-2092` the locked-use gate + `:16325-16349`
`use_torch_gods_favor`, the achievement/bestiary credit); **f33b**
(`333479e0`) owns the EVENT machine + the survival **5043 DROP**
(`RequestNewItem`, `torch_god.rs`) + the flag READS
(`Player.cs:18066-18076`); **g90b** (`1f1b88f0`) the neighbouring 5326
loaf. The DROP hunk is f33b's — c36 carries no drop code, and the orb
lane touches no 5043 site: nothing for g85 to keep or drop there; the
merge keeps both halves of the favor flag per c36's own note.

### 9.2 The f35 stale summon-poll DROP hunk — exact ranges at tip `1abae9ade`

**Re-probe at the NEW latest pushed tip `1abae9ade`** (the DOC-SWEEP
bag-open census; the §8.2 read was at `b72cdfbfc`). The
`b72cdfbfc..1abae9ade` `npcs.rs` diff is TWO hunks — `:5342` (+6, the
coin/loot side) and `:6969` (+98, the bag-open helpers) — **neither
touches the summon poll or the death arm**: the stale copy survives
verbatim. The exact merger ranges (f35 tip `1abae9ade`, direct read):

| Site | f35 tip `1abae9ade` | g85 tip `dd237e331` (KEEP) |
| --- | --- | --- |
| **the stale summon poll — DROP WHOLE** | **`npcs.rs:5693-5725`** — the comment block `:5693-5704` (still says "the crimson-Brain arm rides the heart-detection wave") + the code `:5705-5725`: `count >= 3 && !AnyNPCs(EoW-head)` only, EoW-only spawn for BOTH halves, NO crimson bit, NO FTW override, NO chat arm, NO client guard | `npcs.rs:5714-5805` — the full replacement: the per-half living-boss gate (`:5742-5750`), the `orb_smash_crimson` read (`:5741`), the FTW+DontStarve-not-remix override + the count-1 smash-edge floor (`:5736-5755`), the client guard (`:5757`), the Brain/EoW split spawn (`:5771-5781`), the chat watermark arm (`:5783-5804`) |
| the orb-boss death arm | `npcs.rs:5168-5186` — the pre-g85 shape (meteor + `downed_eow`, NO skyblock unlock, NO surface gate) | `npcs.rs:5168-5213` — g85's gates WRAP f35's identical inner block (the skyblock unlock `:5182-5188` + the `world_surface > 50.0` gate `:5194-5208` around the byte-identical `was_down`/`boss_death_roll` core) |
| the resummon witness | `npcs.rs:14369-14406` `the_orbs_resummon_the_eow_after_the_first_kill` | `npcs.rs:14357-14394` — **byte-identical** (verified by direct compare); keep ONE copy, either |
| the new summon witnesses | none | `w4_g85_the_crimson_half_summons_the_brain_not_the_eow` `:14397-14464`, `w4_g85_the_ftw_starve_pair_summons_on_every_smash_and_the_chat_arm` `:14467-14523`, `w4_g85_the_orb_boss_death_arms_the_shared_flag_and_the_skyblock_unlock` `:14526-14572` |

**The merger's cut list (f35 side):** delete `npcs.rs:5693-5725`
(comment + poll) and put g85's `:5714-5805` at that slot; take g85's
death arm `:5168-5213` (it strictly wraps f35's core — no f35 content
lost); keep ONE resummon witness; keep ALL of f35's drop-tree content
(the 23-kind boss table, the bag-open census's `:5342`/`:6969` hunks,
the crate/present/misc/heal chains) — disjoint from the summon lane.
The g85 poll's `World.orb_smash_crimson`/`orb_smash_chat_count` fields
(`world.rs:380-397`, defaults `:759-760`) come with g85's `check_orb`
per §9.1 — f35 carries neither field, so no field conflict.

### 9.3 The Goblin Scout contract — re-verified UNCLAIMED, final task-seed

**The claim census re-run at the CURRENT tips** (the §7.2 read was the
first wiring round). `git grep -il "scout"` over
`crates/tr-app/src/spawner.rs` (NOTE: the file lives at
`crates/tr-app/src/spawner.rs`, NOT `world_sim/`) across **all 356
pushed `task/W4-*` branches: ZERO hits.** The named lanes at their
latest tips: g61c `b5ff1803d` (DONE — the spawn/teleport dust visuals,
no ladder content), f42b `75f055fcf` (ACTIVE — phase 3 ports the
sandstorm/underground-desert arm `NPC.cs:3949-4018` +
`Spawning_SandstoneCheck`, a DIFFERENT zone arm; its rung-9
sandstone-check neighbour does NOT reach the day-surface scout rung),
c38 `4d8644738`, c40 `29af285f9`, c41 `72a93e062`, c6, f32 `c05ef86f7`,
g17 `a790e88d7` — all re-grepped, none carries scout/4478 content.
**The scout is STILL UNCLAIMED** → the §7.2 contract stands as
published (rung 11, `Next(15)` always after the static gate, the
conditional `Next(7)` only while `!downedGoblins && shadowOrbSmashed`,
`SpawnNPC(...,73)`) and this lane seeds the FINAL task.

**The kind dependency is IN FLIGHT, not missing:** d35's tip
`e5558b0a9` carries `NpcKind::GoblinScout` (`npc.rs:1089-1092`, the
`NPC.cs:9732-9746` data row) + the bestiary/netglue/lang rows — the
spawner-lane owner gates the rung's wiring on d35's merge (or writes
against the kind with a merge-order note, the c36 NOTIFY-SYNC
convention).

**FINAL TASK-SEED (registered as R-24-11 in research/90 §2(d)):** port
the day-surface ladder `NPC.cs:4202-4501` (16 rungs, the §7.2 draw
table VERBATIM — the per-rung draw positions are the contract; rungs
1/2/4/5/6/7/9 draw BEFORE their right-terms) into
`crates/tr-app/src/spawner.rs:1155-1160` (the day fallthrough to
`spawn_basic_slime`), the scout rung reading
`ctx.world.shadow_orb_smashed` (the g85 case-31-gated flag) +
`ctx.world.downed_goblins` + `SeedFlags::REMIX`; enclosing gates
surface/day/!graveyard/!invaders per §7.2. Size M (the ladder is ~15
draw-bearing rungs, NOT a one-arm port); dep d35 (the 73 kind) + the
g85 merge (the flag's four-case census); source
`W4-g85-orb-draw.md` §7.2 + §9.3. The g85 lane wires NOTHING here —
the stream-fidelity rule (every preceding rung's draw) makes a
scout-only insertion a desync; the contract is the deliverable.
