# 65 — Main.cs dust-site batch 3 (W4-g13): census completion + witnesses

Status: DONE (W4-g13, batch 3 of 3, code-only mode — witnesses WRITTEN,
NOT RUN). Completes the census `docs/research/64-dust-main-sites-claims.md`
(W4-g12's claim doc) pins left open: sites 20-23's enclosing methods.

## Census completion (sites 17-23, the g13 third)

| # | Line | Family | Enclosing method | Port |
| - | ---- | ------ | ---------------- | ---- |
| 17 | 24756 | NewDust | `DrawNPCDirect` (`:22513`), the type-398 Moon Lord core arm (`:24701-24764`) | `main_dust::moon_lord_tentacle_tip_puff` + `tentacle_tip_position` |
| 18 | 27381 | NewDustDirect | `DrawNPCExtras` (`:26912`), the type-550 Tavernkeep `ai[0] == 18` ale arm (`:27353-27397`) | `main_dust::tavernkeep_ale_sparkle` |
| 19 | 27384 | NewDustDirect | same arm, the second speck of the pair | same |
| 20 | 38664 | NewDust | `DrawItem` (`:38602`), the coin/heart/mana sparkle gate (`:38637-38666`) | `main_dust::world_item_sparkle` |
| 21 | 57766 | NewDust | `oldDrawWater` (`:57553`), the lit-surface sparkle (`:57747-57769`) | `main_dust::liquid_surface_sparkle` |
| 22 | 57789 | NewDust | `oldDrawWater`, the deep-liquid lava bubble (`:57787-57790`) | `main_dust::lava_bubble_pair` |
| 23 | 57793 | NewDust | `oldDrawWater`, the surface lava bubble + post chain (`:57791-57802`) | same |

## Findings

- **Site 17 is a DEAD ARM.** The enclosing `if (flag8)` reads
  `bool flag8 = false` (`Main.cs:24703`) which the type-398 arm never sets
  (verified: the only `flag8` writes in the file are other methods' local
  latches). The whole four-site cluster `:24736/24748/24752/24756` never
  fires in the C#. g12 owns the first three by the line-order split; g13
  ports the fourth verbatim behind the same dead flag parameter, with the
  tip-position maths (`:24732-24756`: shoulder anchor `(220,-60)*side`,
  half-length link, `Math.Acos` double bend, `RotatedBy` tip) split into
  `tentacle_tip_position` so the geometry is witnessable even though the
  spawn is dead.
- **Sites 18/19 are the Tavernkeep's ale-toss sparkle** (NPC 550
  `DD2Bartender`, item 353 Ale): 30 iterations of two type-4
  `NewDustDirect` gold specks `(245,200,30,155)` at scales 0.7/0.6, gated
  on the talked-to player (`(int)ai[2]` in `[0,255)`) holding Ale, facing
  the NPC (`direction == Math.Sign(n.Center.X - player.Center.X)`) with
  `player.Hitbox.Distance(n.Center) < 46` — the C#'s `&` is
  non-short-circuiting, both reads run — and the `localAI[3]` 0→1 edge
  (the failed gate decays 1→2). The 0.7 speck gets `noGravity` +
  `velocity *= 1f` (the C# no-op, kept); the 0.6 speck's velocity doubles.
  360 draws per burst (12 per iteration: two six-draw default blocks, no
  argument draws, type 4 takes no pool tweak).
- **Site 20 is `DrawItem`'s world-item sparkle**: the
  `FocusHelper.AllowWorldItemsToEmitEffects` (`!DrawnThisUpdate`) +
  coin/Heart(58)/ManaCrystal(109) + `color.R > 60` pre-gates short-circuit
  before the single `Next(500)` draw; the compare's right side is INTEGER
  division `color.R / 50`. Coins override to `default(Color)`/alpha 0/
  scale 1 and the type switch 71/72/73/74 → 244/245/246/247; post-spawn
  `velocity *= 0f`.
- **Sites 21-23 are the RETRO liquid path** (`oldDrawWater`, reached only
  when `!Lighting.NotRetro`, `Main.cs:57519-57522`): the port's liquid
  draw is the `LiquidRenderer` record path (`render/liquids.rs`), so these
  three wait on the retro-lighting seam — ported verbatim, unwired.
  Site 21: the `(R>20||B>20||G>20) && value.Y < 4` pre-gate, then
  `max(R,G,B)/30` (integer) vs one `Next(20000)` draw (drawn even at
  budget 0); white `(255,255,255)` / honey `(255,255,50)` (alpha 255, the
  3-arg XNA ctor), type 43 alpha 254 scale 0.75 at
  `(j*16, vector2.Y - 2)`, `velocity *= 0f`. Sites 22/23: the
  `AllowMiscDustEffects && Dust.lavaBubbles < 200` budget gate (the pool's
  `lava_bubbles` static; its reset/increment `Dust.cs:427/475` is the d50
  phase-2 physics lane), then two independent gates — `liquid > 200 &&
  Next(700) == 0` (deep bubble, all defaults) and `value.Y == 0 &&
  Next(350) == 0` (surface bubble, alpha 50 scale 1.5, then
  `velocity *= 0.8`, `X *= 2`, `Y -= Next(1,7)*0.1`, 1-in-10
  `Y *= Next(2,5)`, `noGravity`). The type-35 pool tweak
  (`velocity *= 0.1`, `Y = -0.5`) rides the internals before the posts.

## RNG contract (one slot per C# pass)

All seven sites execute on the DRAW path, outside every `Main.SwapRandom`
block (`Main.cs:17732-17809` wraps only the update stages) — in the C#
they roll the BARE `Main.rand`. The port's bare-stream slot is
`NamedRngs::global()` (`app/rng.rs`; the world-seed deviation is already
recorded there): the wiring wave passes `app.rngs.global()`, the same
shape as the minecart dust tick's player-less arm
(`app/frame/world.rs::dust_phase`). No new stream; the 17 named
simulation streams are untouched — the draw path never advances them.
Per-site draw census (witnessed by lockstep mirrors):

| Site | Gate draws | Hit draws | Miss draws | Pre-gate-fail draws |
| ---- | ---------- | --------- | ---------- | ------------------- |
| 17 (dead) | 0 | (7: 6 default + type-6 tweak) | — | 0 |
| 18/19 burst | 0 | 360 (30 × 2 × 6) | — | 0 |
| 20 | 1 (`Next(500)`) | 6 default | 0 | 0 |
| 21 | 1 (`Next(20000)`) | 6 default | 0 | 0 |
| 22 | 1 (`Next(700)`, liquid-gated) | 6 default | 0 | 0 |
| 23 | 1 (`Next(350)`, rect-gated) | 6 + `Next(1,7)` + `Next(10)` + cond. `Next(2,5)` | 0 | 0 |

## Witnesses (14, WRITTEN NOT RUN)

`main_dust.rs::tests` — the d51 mirror pattern (an independent
`UnifiedRandom` walks the same draw sequence; the final stream states
must be equal): the tentacle geometry + dead-flag zero-draw + enabled
7-draw mirror; the ale 60-speck 360-draw mirror, the edge latch, the
failed-gate decay census; the item-sparkle gate/spawn mirror (pinned hit
seed 100011 + 4 misses × 3 kinds), the pre-gate zero-draw census, the
coin-row type switch; the surface-sparkle mirror (pinned hit seed 100702
+ 4 misses × honey), the pre-gate + zero-budget draw census; the lava
pair's gate/post-chain mirror across 6 pinned seeds (100323 deep-only,
100042 surface-only, 219697 double, 108460 surface-with-kick, 1/42
misses), the double-hit ordering pin, and the budget/focus/shallow/
non-surface skip census. The hitting seeds are derived OFFLINE from a
transcription of the decompiled `UnifiedRandom.cs` (sanity-checked
against tr-core's pinned GOLDEN_42 prefix — the same offline-vector
precedent `app/rng.rs` used), so the witnesses are deterministic, not
probabilistic scans.

## Deferred seams

- ~~The draw-call wiring (the type-398 arm, `DrawNPCExtras`' 550 arm,
  `DrawItem`, the retro `oldDrawWater` path) — the wiring wave.~~ LANDED:
  the feeds (`moon_lord_tentacle_feed`, `tavernkeep_ale_feed`,
  `item_sparkle_feed`, `retro_liquid_dust_feed` + `retro_surface_rect` +
  `light_color_bytes`) run in `app/frame/draw.rs::draw_region` outside the
  G6 cache on `rngs.global()` in C# relative order. The retro sites gate on
  `LightMode::Retro | Trippy` (`!Lighting.NotRetro`); the liquid sprite draw
  stays on the `LiquidRenderer` record path. The `DrawnThisUpdate` set was
  relocated to end-of-Draw (`Main.cs:62478`) so the focus gate reads clear
  through the draw.
- ~~`Dust.lavaBubbles` maintenance — d50 phase 2 (`tr_core::dust_physics`,
  in flight); this module only reads the gate.~~ LANDED: the increment
  (`Dust.cs:475`, every live type-35 slot) now rides `DustPool::update_dust`
  before the move (`:477`); the head reset (`:427`) was already there.
- g11/g12's thirds (sites 1-16) — their branches; the boundary sites are
  owned whole per the claim doc (24752 g12's, 24756 g13's).
