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
