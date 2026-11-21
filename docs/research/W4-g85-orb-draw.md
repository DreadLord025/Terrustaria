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
