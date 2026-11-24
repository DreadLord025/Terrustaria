# 352 — W4-d1 falling-block FINAL census + the falling-block-family declaration (RETIRE)

**RENAME CHAIN 246 → 352** (registry-150 §9 UNIFIED FINAL, d54 tip
`f5c5adf50`, row 352): this doc's first-add take of 246 lost to **e23's
published 18b-fork reservation — e23 KEEPS 246** (§7.4, publish
`72226e500` 14:47:50 predates d1's add `47482bdeb`); §9 assigns this doc
**352**. Renamed per §9.3 (chain-banners cite §9); the original
numbering paragraph below is kept as the historical first-add record.

Lane: `task/W4-d1-proj-ai-1-10` (worktree TR-W4-d1). Date: **2026-11-21**.
Code-only mode — witnesses WRITTEN NOT RUN; no cargo/rustc/rustfmt/tests/
dotnet/graphify invoked. Verified by re-reading the cited C# blocks whole
and hand arithmetic.

**Number per registry-150** (`150-research-number-registry.md`, the W4-d54
collision registry): the free renumber block 151–236 is fully assigned;
237+ stays free for future lanes. The 237–245 claims at the pushed tips
(this lane's scan, `git ls-tree docs/research/` per `task/W4-*` tip +
the INDEX rows): 237 e23/g48/g61 (three-way), 238 c8/d42/g61 (three-way),
239 d42/f21/g56 (three-way), 240 c42/d53/g58 (three-way), 241/242/243 g58,
244/245 g48 (INDEX-claimed, files at tip). **246 is the first number no
tip carries** (file or INDEX row) — this doc takes it. The 237–240
multi-claims are the registry's own problem (the lanes resolve them per
§3 first-pushed-wins); this lane does not touch them.

Companion commits on this branch: the sync merge `f491afc11` (origin/main
`051600da8`, the anchors keep-both) + the retire code `dd7b6f32d`.

## 1. The open follow-ups — census verdicts + dispositions

The three follow-ups the wave-3 ANALYSIS section documented (the
frozen-tail carve-out's wiring note, the forced-PlaceTile micro-deviation,
the tile-cut skip). All three CENSUSED WHOLE and PORTED this round — none
needed a task-seed owner-lane.

### 1a. The natural-fall spawner move (`spawn_falling` → the data path) — DONE

C# read whole: `WorldGen.SpawnFallingBlockProjectile`
(`WorldGen.cs:86039-86093`): the `netMode == 1` refusal (`:86041-86044`),
the `BlockBelowMakesSandFall` + `AllowsSandfall` gates (`:86051-86058` —
the port's `check_falling` already carries them), `GetSandfallProjData`
(`:86059`), `tileCache.ClearTile()` (`:86060` — the caller clears), the
`netMode == 0` spawn (`:86061-86068`): `NewProjectile(GetProjectileSource_
TileBreak(i, j), i*16+8, j*16+8, 0f, 0.41f, projType, dmg, 0f,
Main.myPlayer)` + `Main.projectile[num].ai[0] = 1f` + `SquareTileFrame` +
the two `ExploitDestroyQueue` enqueues; the `netMode == 2` dedupe variant
(`:86069-86091`, the 1000-slot `timeLeft 3600±60` + 4-px distance scan,
the 2.5-then-0.5 velocity + the +2 Y shift — MP-only, the port's
single-player fold).

Port: `spawn_falling` now delegates to `spawn_with_data_ai(proj_kind, x,
y, 0.0, 0.41, damage, 0.0, 1.0, 0.0)` — the rowed `SetDefaults` stats
(the 31's `:1131-1140` shape), the `ai0 = 1` argument landing the C#'s
post-spawn `ai[0] = 1f` write BEFORE the first tick (the arm's
`ai[0] == 0` owner latch never fires for a natural fall — the channel
aim-steer stays the weapon spawns' own). The `tile_kind` parameter stays
for the call-site shape (the place map derives the tile from the
projectile kind — `GetSandfallProjData`'s inverse).

**The frozen tail is REMOVED** (the retire's core): the pool's
per-sub-step `falling_tile` gravity block + the post-loop
`landed`/`can_place_back` bottom-edge placement are gone; the
dispatch/death-hook/player-damage gates drop their
`falling_tile.is_none()` carve-out terms; the `falling_tile` field stays
`None` on every slot (the save/net shape). The `tr-core/falling.rs`
flight helpers (`TILE_FALL_GRAVITY`, `TIME_LEFT_DEFAULT`, `landed`,
`can_place_back`) are removed with it — zero remaining callers.

**The behaviour deltas over the frozen tail** (each now the C#'s own
shape, recorded in ANALYSIS):
- pen −1 (the rows) vs the frozen pen 1 — the natural falls no longer
  die on the first NPC contact (the C# `penetrate = -1`).
- The blocked-move kill + the centre-cell forced place vs the frozen
  bottom-edge probe — the landing cell is the same in the common case
  (the collision resolves the blocked velocity to rest ON the solid,
  the centre cell is the last free row), but the C# way now carries the
  half-brick up-shift, the 1081 side-slide, the below-tile refusals and
  the item-drop-on-refusal.
- The player-damage pass runs (`FALLING_HOSTILE_KINDS` — the nine
  friendly+hostile natural falls hurt the player, the C# `Damage_EVP`
  `:14775`); the frozen tail never did.
- The dust-gate draws stay on the `UpdateProjectiles` stream (the
  per-kind `Next(2)`/`Next(3)`/`Next(20)` gates — the frozen tail drew
  nothing); the tile-cut chain runs (1c below).
- The `ai[0] = 1` gravity ladder (0.41, the 10-cap) + the +0.1 spin —
  the frozen tail's gravity was the same 0.41/10 but had no spin, no
  tile walk, no blocked-arm distinction.
- The `SquareTileFrame` + `ExploitDestroyQueue` tails (`:86065-86067`):
  the port's break hook already re-frames through `break_tile`'s
  `after_edit`; the exploit-destroy queue is the MP anti-dupe layer
  (unported, the `break_tile` precedent).

### 1b. The forced-PlaceTile bypass set — CENSUSED WHOLE + PORTED

C# read whole: `WorldGen.PlaceTile` (`WorldGen.cs:59342-60295`). The
`forced` flag is the FIRST disjunct of the `:59373` gate — it bypasses
`Collision.EmptyTile` (`Collision.cs:1359-1381`: the active occupant AND
the 255-player + maxNPCs rectangle-overlap scan) and the `!tileSolid`
/ grass-on-dirt / moss disjuncts. The port's `place_tile` wall-350
refusal is a `CanKillTile`-derived gate the C# `PlaceTile` does NOT
carry — the forced route bypasses it too.

The full gate census for the served kinds (the place map's tiles:
0/53/57/59/112/116/123/224/234/330-333/495):
- `:59345-59348` the gen-time 488 refusal — `isGeneratingOrLoadingWorld`
  folds false at runtime.
- `:59349-59352` `num >= TileID.Count` (754) — the port's `TILE_COUNT`
  gate, ported.
- `:59364-59371` the 23/199-on-59 → 661/662 rewrites — no served kind.
- `:59375-59430` the grass/moss inner refusals + the 38-moss remap — no
  served kind is 2/23/60/70/109/199/633/661/662 or a `Main.tileMoss`
  member ({179-183, 381, 534, 536, 539, 625, 627}, `Main.cs:7007-7047`).
- `:59431-59449` the 81 pot support gate — no served kind.
- `:59450-59453` the `IsADripTile` gate — no served kind.
- `:59454-59472` the liquid refusals (the torch family + the
  3/20/24/27/32/51/69/72/201/352/529/624/637/656/697/700/701 list) — no
  served kind; a wet cell places.
- `:59473-59476` the free cell's `Clear(Tile | TilePaint | Slope)` —
  ported (the wall + liquid survive, the stale frames/paint/slope wipe).
- `:59477-59482` `ResetsHalfBrickPlacementAttempt` — no served kind.
- The `:59483-60176` per-kind placement arms (the flowers, the furniture,
  the chests, the torches, the doors…) — no served kind matches; the
  legacy default arm (`:60234-60250`) writes `active = true, type = num`.
- `:60237-60247` the 53/396/397 paint arms — the rainbow paint needs
  `SpecialSeedFeatures.RainbowSandAndBlackSandWalls` (the shimmer-ocean
  mode the port does not generate — folds off, the `convert_sized`
  precedent); the anniversary `color(7)` needs `tenthAnniversaryWorld &&
  !remixWorld && !notTheBeesWorld && !dontStarveWorld` — PORTED over
  `world.seeds` (the 53 is the only served member).
- `:60255-60262` the `TruncatesWalls` `SquareWallFrame` (no served kind)
  + the `SquareTileFrame` re-frame (the port's `after_edit`) + `result =
  true`.
- `:60263-60290` the `mute: false` sound tail: 127 → Item30, 314 →
  Item52, 330-333 → the legacy Coins 18, default → the Dig 0 — the
  port's `place_tile_sound` table already carries it; the falling arm
  now queues the cue on the pool's sound intent (the drain routes id-0
  and id-18 → `play_place_sound`, the id-2 styles → `play_proj_sound`).
  The 22/140 dust (3× dust 14) — no served kind.

Port: `tr_core::edit::place_tile_forced` (the new free-cell route; the
occupied-cell paint-keep is a recorded micro-deviation for a path no
caller reaches — `death_place` gates on `!active` at `:77516`).
`death_place` rides it + returns the place-cue intent beside the drop
intent; the pool queues both.

### 1c. The tile-cut bodies — CENSUSED + PORTED (the falling-block slice)

C# read whole: `HandleMovement`'s `flag` block
(`Projectile.cs:17904-17985`). Two blocks, in order:
1. `:17910-17947` — the **484 rolling-cactus** block (the wave-3 doc's
   "484 vine block" was a misnomer — `TileID.RollingCactus = 484`,
   `TileID.cs:1597`; `Main.tileCut[484] = true`, `Main.cs:7466`). Runs on
   EVERY blocked move, NO call-site rand.
2. `:17948-17985` — the cracked-brick block (`TileID.Sets.CrackedBricks`
   = {481, 482, 483}, the generated `SET_CRACKED_BRICKS`). The
   `Main.rand.Next(3) == 0` is the FIRST `&&` operand — drawn on every
   blocked move regardless of the trailing gates (the ai026_c precedent,
   the wave-3 draw already stayed).

The shared gate: `owner == Main.myPlayer && CanCutTiles() && (friendly ||
hostile) && damage > 0`. `CanCutTiles` (`:15323-15334`) passes every
served kind (aiStyle 10 is not on the 45/137/92/105/106 style refusal
list; no served kind is on the type list — the 10/11 powders are, but
they are aiStyle 6, never here). `friendly || hostile` holds for every
served kind EXCEPT the coin piles (stock both-false). `damage > 0`
excludes the snow variants' zero-damage falls (`GetSandfallProjData`'s
330-333 dmg 0 — the coin piles again). `GetTileCutIgnorance(false, trap)`
(`Player.cs:44632-44644`): `trap` false for every served row,
`dontHurtNature` has no ported granter (the `bug_net` precedent) — the
ignore set is `TileCutIgnore.None` (all-false, `TileID.cs:111`).

The box (`:17912-17931`): `(int)(pos + vel − 1)/16` ..
`(int)((pos + size + vel + 1)/16) + 1`, clamped to `[0, maxTiles]` — the
position + velocity box BEFORE the integration (HandleMovement runs
ahead of `UpdatePosition` `:20445`). Each matching cell through
`WorldGen.CanCutTile` (`:86828-86839` — the context does not reach the
body; the port's `tileobject::can_cut_tile`) and `WorldGen.KillTile`
(the port's full `kill_tile`, the tile-walk precedent; the `netMode`
sends are MP-only).

Port: `projectile_ai_falling::cut_tiles_in_flight_box` (one walk, the
kind test is the parameter — the second-copy helper rule), called in the
C# order at the blocked-move site: the 484 walk, THEN the `Next(3)`,
THEN the cracked walk on `== 0`. The cutting layer's OTHER slices (the
light-items/flare families, the melee `ItemCheck_CutTiles`) keep their
own ports/skips — this is the falling-block slice only.

## 2. The f25-spray compose + the d18 DUP keep-one

### 2a. The seven solution-spray rows — the keep-one verdict

Two branches rowed the same seven kinds (146 Hallowed / 147 Corrupt /
148 Crimson / 149 Mushroom, `Projectile.cs:2350-2397` — four
line-identical copies of the 145 arm `:2338-2349`; 1015 Blue / 1016 Red
/ 1017 Purple, `:9633-9636` → `DefaultToSpray` `:10691-10702`):
- **f25** `5e83d3ecb` (closeout 2/5, tip `d587b128d`): the rows as the
  145 shape with `RESET_STATE` — **alpha 0 / ignoreWater false** (the
  145 row's pre-existing M3-ITEM-6 dropped pair, propagated).
- **d18** `78ffe8e11` (tip `8248a54be`): the rows VERBATIM — **alpha 255
  + ignoreWater true** (the `:2356`/`:2360` etc. lines the f25 copy
  dropped), the ignoreWater overlay + the census-flags extension + the
  row witness.

**KEEP-ONE: the d18 shape wins** — it is the verbatim `SetDefaults`
(the C# blocks set `alpha = 255` and `ignoreWater = true` explicitly;
the f25 copy's dropped pair is the 145's own documented deviation, not
the solutions'). This branch rows the d18 shape (`dd7b6f32d`): the
`146..=149 | 1015..=1017` arm with `(false, -2, false, -1, 255, 1.0)` +
the ignoreWater overlay + the flags-census extension + the
`w4_d1_retire_the_solution_spray_rows_are_the_145_shape` witness.

**Merger instruction:** when f25 and d18 merge, keep THIS branch's rows
(the d18 shape); drop the f25 `RESET_STATE` copy and the d18 duplicate
arm (the same `146..=149 | 1015..=1017` pattern — a duplicate match arm
is a hard error under the dispatch's `#[deny(unreachable_patterns)]`
convention; the data table's match is not deny-guarded but the duplicate
is still wrong). The f25 ammo witness
(`w4_f25_closeout_all_eight_solutions_resolve_on_both_clentaminators`)
and the d18 row witness both stay — they pin the same rows from two
angles (the ammo resolution + the row shape). The 145's own dropped
alpha/ignoreWater stays its M3-ITEM-6 deviation (unchanged).

### 2b. The ai_spray conversion-cell activation — the compose table

The `ai_spray` per-kind ladder (`:26789-26825`) maps the kinds to the
`WorldGen.Convert` types; the owner-gated conversion (`:26826-26835`)
calls `Convert(Center.ToTileCoordinates(), type, size 2/3)`. The cell
ownership (the f25 12g-ledger's Convert-switch map + this lane's census):

| type | kind(s) | cell owner | cell port | ai_spray arm status |
| --- | --- | --- | --- | --- |
| 0 | 145 (+ default) | base (M3-ITEM-6) | `spread::convert_cleanse` | LIVE (base) |
| 1 | 147 Corrupt | **d1** (the powder port `78dc52720`) | `spread::convert_sized` type 1 | **ACTIVATED this round** |
| 2 | 146 Hallowed | f25 (C3) | `spread::convert_sized_hallow` | f25's arm at its merge |
| 3 | 148 Mushroom | g42 (G7) | `spread::convert_sized_mushroom` | g42's arm at its merge |
| 4 | 149 Red | **d1** (the powder port) | `spread::convert_sized` type 4 | **ACTIVATED this round** |
| 5/6/7 | 1015/1016/1017 | f25 (C4) | `spread::convert_sized_spray` | f25's arm at its merge |

This branch's `ai_spray` now dispatches
`match conversion { 0 => convert_cleanse, 1 | 4 => convert_sized(..,
conversion, size), _ => {} }` — the types 1/4 ride d1's OWN powder cells
(the same `Convert(i, j, type, size)` entry the powders call at size 1;
the spray calls it at size 2/3). The 133 cap / `ai[0]++` / spin run for
every kind regardless (the f25/d18 convention).

**Merger instruction (the compose):** f25's tip dispatches
`{ 0 => cleanse, 2 => hallow, 5|6|7 => spray, _ => {} }`; g42's published
contract adds `3 => convert_sized_mushroom`. The merged match is
`{ 0 => cleanse, 1 | 4 => convert_sized, 2 => hallow, 3 => mushroom,
5|6|7 => spray, _ => {} }` — this branch's `1 | 4` arm + f25's `2`/`5|6|7`
arms + g42's `3` arm, ONE match, no duplicates. The `CONV_THORN`
constant dedupe (the f25 12g-ledger's d1 note) still applies at the
spread.rs merge.

**Activation confirmation (this branch):** the 147/149 sprays now
convert (the witness
`w4_d1_retire_the_corrupt_and_red_sprays_convert_their_cells`: the grass
2 → 23/199, the stone 1 → 25/203, the sand 53 → 112/234, the size-3
solution disc, the owner gate, the f25/g42-owned kinds stay no-ops with
the machine running). The full eight-kind activation lands at the
f25/g42 merges — the rows are here, the cells compose per the table.

## 3. The 502 Meowmere row — the d75 owed gate, censused

The prompt's question: does d1's band 1-10 own the 502? **Census verdict:
NO — and the row is already landed by d75 itself.**

- The 502 is **aiStyle 8** (`Projectile.cs:5732-5742`: 16×16, friendly,
  melee, `penetrate = 5`, `usesLocalNPCImmunity` + `localNPCHitCooldown
  10`) — the ball-of-fire family, NOT the aiStyle-1..10 band d1 owns
  (the band's styles are 1/2/3/4/5/6/7/8/9/10, but the 502's row was
  never in d1's rowed set — d1's style-8 census is verify-only, the
  `ai_ball_of_fire` arm already serves the rowed kinds).
- **d75 part 5 (`d328fad8c`, 2026-11-21T13:02:36+02:00) rowed the 502
  itself** — the commit message records the re-probe ("unrowed on d1
  e271b5bc8 / d77 d61e6362d / f1 bcdfcd62e") and the row + the
  `:24346-24352` velocity rotation in `ai_ball_of_fire` + the melee
  census extension + the rowless-set witness migration. The part-4 owed
  five-bounce chime gate (`:18206-18225`, the `(37, 6..10)` intents) goes
  LIVE through `spawn_with_data`'s row guard on d75's own tip.
- This lane's scan confirms: the ONLY 502-row carrier across the pushed
  `task/W4-*` tips is `task/W4-d75-proj-sounds-1`
  (`MEOWMERE => (16.0, 16.0, 8, 5, TIME_LEFT_DEFAULT, 0, true, false,
  false, (true, 10, false, -1, 0, 1.0))`). The row matches the C# arm
  verbatim (re-read `:5732-5742` this round: 16×16, aiStyle 8, pen 5,
  local-10, the stock 3600 life / tileCollide true / alpha 0 / scale 1).
- **No d1 action needed** — the row is d75's, the gate is armed on d75's
  tip, the merge lands it. The d75 chime-gate dependency is RESOLVED
  (the row it needed exists). This lane does NOT duplicate the row (the
  keep-one convention — a second 502 arm would be a duplicate match).

The 502's remaining skipped effects (d75's own record, confirmed by this
lane's re-read of `:18226-18253`): the hue dust ring (the
`10 + ai[0]*4` particle loop, the `RotatedByRandom(π)` + the per-particle
`NextFloat` scale + the `hslToRgb` colour), the owner-gated Damage
resize (the `40 + 8*ai[0]` box, the pen −1 flash), the disco AddLight,
the Kill dust trail, the Shimmer lift, the GetAlpha tint — the dust/
effect layer's census, d75's lane.

## 4. The falling-block-family declaration (the family bible)

**FALLING-BLOCK FAMILY COMPLETE** (the aiStyle-10 lane, W4-d1):

- **Rows:** all 19 served kinds rowed — the 15 wave-3 rows
  (17/31/39/40/56/67/71/179/241/812/411-414/1081) + the 4 sandgun balls
  (42/65/68/354, the S1-D-5 "42-row gap" closed wave 3). The 109 hostile
  snowball + the 736-738 debris trio keep their own arms (kind-gated,
  the dispatch's `FALLING_BLOCK_KINDS` set).
- **Arm:** `projectile_ai_falling::step` — the whole `AI_010`
  (`:57826-58100`) + `HandleMovement`'s style-10 block (`:17592-17610`)
  + the blocked arms (`:19829-19853`) + the tile-cut chain
  (`:17910-17985`, this round) + the `Kill_ExplodeTiles` death placement
  (`:77387-77570`, the forced route this round).
- **Spawner:** the natural-fall trigger (`check_falling` →
  `spawn_falling`) rides the data path (this round) — ONE flight path
  for every served kind, the frozen-tail carve-out retired.
- **Damage:** the PVE refusal (the coin piles' stock friendly false) +
  the `FALLING_HOSTILE_KINDS` player-damage pass (the nine natural
  falls) + the sandgun balls' friendly+ranged (no player damage).
- **Witnesses:** the wave-1/2/3 witnesses + this round's 12 new
  (the forced-place 3, the spray-row 1, the spray-activation 1, the
  data-path-spawn 1, the tile-cut 3, the falling_tests re-derivations)
  — all WRITTEN NOT RUN (code-only).

**Remaining (NOT this lane's — the owner-lanes' merges):**
- The f25/g42 spray-cell compose (§2b's table) — the types 2/3/5/6/7
  arms land at those merges.
- The d18/f25 spray-row dedupe (§2a) — the merger keeps this branch's
  d18-shape rows.
- The 502 row is d75's (§3) — no d1 action.
- The cutting layer's other slices (the light-items/flare/melee families)
  — their own lanes; the falling-block slice is done. W4-d1 SEAL (§6)
  added the 491 FlyingKnife's blocked move as the walk's second caller
  (`pub(crate)` — the second-copy helper rule, no duplicate walk).
- The MP tails (the `netMode == 2` spawn variant, the `netUpdate` writes,
  the `ExploitDestroyQueue`) — the net lane's.

## 5. Verification record (code-only)

- C# re-read WHOLE this round: `WorldGen.SpawnFallingBlockProjectile`
  (`:86039-86093`), `GetSandfallProjData` (`:86514-86560`),
  `WorldGen.PlaceTile` (`:59342-60295`), `Collision.EmptyTile`
  (`:1359-1381`), `HandleMovement`'s cut blocks (`:17904-17985`),
  `CanCutTiles` (`:15323-15334`), `GetTileCutIgnorance`
  (`Player.cs:44632-44644`), `CanCutTile` (`:86828-86839`), the 502 arm
  (`:5732-5742`) + the bounce/rotation (`:18206-18253`/`:24346-24357`),
  the spray rows (`:2338-2397`/`:9633-9636`/`:10691-10702`) + the
  ai_spray ladder (`:26786-26835`).
- The generated sets decoded for the census: `TILE_MOSS` {179-183, 381,
  534, 536, 539, 625, 627}, `SET_CONVERSION_STONE` {1, 25, 117, 203},
  `SET_CONVERSION_SAND` {53, 112, 116, 234}, `SET_CRACKED_BRICKS`
  {481, 482, 483} — the served kinds match none of the PlaceTile
  grass/moss/liquid gates; the spray witness's conversion targets are
  the sets' own members.
- The research-number scan: `git ls-tree docs/research/` per pushed
  `task/W4-*` tip + the INDEX rows — 246 first-free (§0, the historical
  first-add scan; SUPERSEDED by registry §9 — 246 = e23's published
  reservation, this doc renumbered 352, see the chain-banner).
- The 502-row carrier scan: `git grep` per tip — d75 only (§3).
- No cargo/tests/rustfmt/graphify run (the task's code-only mandate).
  The witnesses are written, not run; every expected value hand-derived
  from the cited C# lines.

## 6. The SEAL round (W4-d1, 2026-11-21) — the 491 FlyingKnife + the 981 SilverBullet

The lane's last two e3 task seeds (research/125's RETIRE phase: the
`_Old` LIVE/DEAD caller census + the 345-branch unrowed scan) landed in
this round. **No new research number is taken** — registry-150 §9.3
("New docs require a registry check + Lead assignment; NO self-picks")
folds this round into THIS doc; the falling-block-family bible stays §4
(the declaration section), the SEAL receipts ride here. The two owed
code-comment cites from the 246→352 rename (`projectile/data.rs` +
`projectile_ai_misc.rs` ×2, §9 row 352's note) are PAID in this round's
commit — `git grep 'research/246|246-d1-falling'` over crates/docs/
anchors is empty (stale-zero re-verified).

### 6a. The 491 FlyingKnife — row + the LIVE `_Old` machine (the e3 seed)

The e3 census verdict (research/125 §RETIRE (2)): `AI_009_MagicMissiles_Old`
(`Projectile.cs:43427-43653`) is LIVE — ONE caller, `AI_009_MagicMissiles`'s
type-491 head (`:43197-43201`, delegate + return); the 491 row was UNROWED
on every pushed tip; owner lane d1 (the aiStyle-6/9 band remainder). The
`AI_019_Spears_Old` sibling is DEAD (zero callers — cite-drop, not d1's).

- **Row** (`projectile/data.rs`, `FLYING_KNIFE => (26.0, 26.0, 9, -1,
  TIME_LEFT_DEFAULT, 0, true, false, false, RESET_STATE)`): the
  `SetDefaults` arm `:5404-5412` verbatim — 26×26, aiStyle 9, friendly,
  `melee = true` (the melee-flag census extension), `penetrate = -1`;
  the stock 3600 life / tileCollide true / extraUpdates 0 / alpha 0 /
  scale 1 / no ignoreWater stand (the arm writes no override). The item
  wire already exists: item 3030 Flying Knife (`itemdata.rs`, channel,
  melee, shoot 491, shootSpeed 17).
- **Machine** (`projectile.rs::ai_magic_missile_old`, the style-9
  dispatch's `kind == FLYING_KNIFE` head branch — the C#'s own head
  shape): the owner block `:43429-43538` (channel speed 20 — the 16's
  15 / the default 12 are dead branches; the close case `num4 < 100f`
  ×1.1 boost to the 20 cap + the `ai[0] = -10` latch at 0
  `:43452-43467`; the far case `> 20` `(v*4 + aim)/5` blend `:43484`;
  the mid case velocity outright `:43502-43503`; the release skips the
  `type != 491` re-aim block `:43509-43535` and latches `ai[0] = 1`
  `:43536`); the 491 block `:43540-43583` (`localAI[0]++`; from tick 16
  while released the fly-home — tileCollide drops, the owner-distance
  < 20 kill, the `(v*5 + toOwner·25)/6` blend; the rotation ladder
  `:43556-43567`; the 2000-px kill `:43568-43571`; the dust gate
  `Next(2) == 0` → the `Next(3)` type pick 15/57/58 — BOTH draws stay on
  the pass stream, the dust body skips); the +16 fall clamp
  `:43649-43652` (the `!flag` tail `:43645-43648` never runs — the 491
  sets flag true). The body's 16/34/79 branches are DEAD for the only
  reachable kind — kept as citations (the census-F2 convention the e3
  seed pinned).
- **Movement**: the `GetCollisionParams` 10×10 centre resize
  (`:19957-19960` — the 491 is on the list, the resizeAnchor 0.5
  default); a blocked move runs the tile-cut chain FIRST (the shared
  `cut_tiles_in_flight_box`, now `pub(crate)` — the 484 walk, THEN the
  `Next(3)`, THEN the cracked walk on `== 0`; `CanCutTiles` passes —
  aiStyle 9 is not on the `:15323-15334` refusal list, the 491 not on
  the type list), THEN the 491 bounce arm (`:18464-18478`): `ai[0] <= 0
  → -10`, each blocked axis with a non-zero pre-collision component
  reflects at FULL strength (`lastVelocity * -1f`). The arm sits in the
  OUTER per-type chain — the style-9 kill (`:19325-19348`, nested under
  `:18635`) NEVER reaches the 491: **it bounces, it does not die on
  tiles**. `UpdatePosition` (`:20445`) integrates the post-bounce
  velocity (the 491 is not on the skip list `:20435`).
- **Skips** (each noted in the arm's doc): the `netUpdate` writes
  (`:43479`/`:43498`/`:43508`), the dust bodies (the e3 emitter pattern
  activates them in the dust follow-up — the 5 sites `:43574-43635`),
  the wet branch (the liquid flags fold dry), the
  `LimitPointToPlayerReachableArea` clamp rides the `channel_target`
  seam (S1-P-i23, the use path pre-clamps). A `None` player folds the
  owner centre to the knife's own centre (the falling-arm convention).
- **Witnesses** (`w4_d1_flying_knife_tests`, 5, WRITTEN NOT RUN): the
  row + item-3030 wire; the release latch (`ai[0] = 1`, NOT the modern
  arm's −1; rot +0.3·direction; pos (164,160); draws `[(0,2)]`); the
  channel close boost (10→11, the −10 latch); the fly-home kill at tick
  16 (the zero-velocity fold — the first 15 ticks live, tick 16 the
  distance-0 < 20 kill + tileCollide dropped); the bounce (the solid
  cell (11,10) blocks the 10×10 box's +x probe, `ai[0] = -10`, vel.x
  −4.4 full-strength, draws `[(0,2),(0,3)]` — the dust gate BEFORE the
  cut chain's draw, the C# order).

### 6b. The 981 SilverBullet — the special-bullets module's twelfth kind

The e3/d75-flagged unrowed sibling (research/125: "981 SilverBullet —
STILL unrowed", the full 345-branch remote scan; the
`ai001_silver_bullet_drip_981` emitter STAGED at the fade-chain slot
`:56746` between the 265/355 arms `:56728` and the 357 trail `:56753`).
Routed through `projectile_special_bullets` (the S1-P-i18 module — the
kind joins `SPECIAL_BULLET_KINDS` 11→12), NOT a new module:

- **Row** (`SILVER_BULLET => (4.0, 4.0, 1, 1, 600, 1, true, false,
  false, (false, -2, false, -1, 255, 1.2))`): `:9340-9353` verbatim —
  4×4, aiStyle 1, friendly, penetrate 1, light 0.5 (the
  `set_defaults_light` row), alpha 255, scale 1.2, timeLeft 600,
  ranged (the census extension), extraUpdates 1; the stock tileCollide
  true stands — the shot dies on tile contact.
- **Arm** (`projectile_special_bullets::step`, widened with the pass's
  rng): the 981 rests (`flag3 = false`, `:56030` — no gravity, the
  generic `ai[0] >= 15` arm never fires); the drip gate `Next(6)`
  (`:56746`) is an EXPLICIT call-site draw — it STAYS on the
  `UpdateProjectiles` stream, one per sub-step (the W4-d1
  falling/powder explicit-draw convention; the module's other eleven
  kinds keep their zero-draw census, the parameter unread for them —
  the test wrapper passes a fresh stream so every pre-existing witness
  stays byte-identical); the dust-279 body (noLightEmittance +
  noGravity ×0.2) SKIPS with the dust layer. The rotation tail
  `atan2 + 1.57` (`:57748-57751`) + the +16 clamp + the probe stand-in
  movement are the module's shared shape.
- **Damage**: `damage_pve`'s Werewolf-bane ×3 (`:13385-13388` — `type
  == 981 && targetNPC.type == 104`), at the C# site AFTER the
  banner/expert arms (unported layers) and BEFORE the yoyo doublings
  (`:13389`).
- **Light**: `proj_lights.rs`'s own 981 arm (`:16343-16347` —
  `num2 *= 0.7f; num3 *= 0.7f;` — the 14/110/180/242/302 sibling below
  scales the blue by 0.1, the silver keeps 0.7 on both).
- **Skips**: the Kill arm's `HitTiles` + Item10 (`:76040-76043`, the
  common bullet arm `:76035-76039` precedent — the module's death-hook
  sound convention) and the SilverBulletSparkle particle request
  (`:76044-76048`, the particle layer).
- **Witnesses** (3, WRITTEN NOT RUN): the row + light + covers; the
  rest + the one `Next(6)` draw + rot 1.57 + the free step; the
  tile-contact kill (the blocked-move default `:19849-19853`, the
  module's probe stand-in).

### 6c. The SEAL receipts

- **e3 research/125 (the RETIRE phase)**: the 491 seed's row+machine
  wave DONE (§6a — the `_Old` body's dead branches kept verbatim per
  the seed's own instruction); the 981 row DONE (§6b — the
  `ai001_silver_bullet_drip_981` emitter's staged slot is now cited by
  the live arm; its dust body activates in the dust follow-up).
- **d75's owed gate**: the 502 row census stands (§3 — d75 part 5
  `d328fad8c` owns it, no d1 action).
- **The 246→352 rename's owed cites**: PAID (`projectile/data.rs`'s
  spray-compose comment + `projectile_ai_misc.rs`'s module doc + spray
  witness doc, all now cite research/352 with the §9 chain note).
- **Hand verification (code-only)**: the new witnesses' arithmetic
  re-derived against the cited C# — the release test's pos (164,160)
  (the free integrate of vel (4,0)); the close-case 10×1.1 = 11 < 20
  (no cap); the fly-home kill's tick-16 gate (`localAI[0] = 16 > 15`);
  the bounce test's collision cell (11,10) = x 176..192 vs the 10×10
  box's +x probe 172.4..182.4 (overlap), the `lastVelocity.Y == 0`
  guard keeping vel.y 0, vel.x = −(4×1.1) = −4.4; the draw orders
  (`Next(2)` at `:43572` before the cut chain's `Next(3)` at `:17948`;
  the 981's one `Next(6)` per sub-step). No mismatch found.
- No cargo/tests/rustfmt/graphify run (the code-only mandate stands).