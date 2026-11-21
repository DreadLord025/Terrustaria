# 77 — W4-g78 wiring-phase attach contracts (the contested seams)

> W4 lane g78 · wiring phase · 2026-11-21 · base `3e7bdba0` (the DOC-SWEEP
> lane 4 census, research/76) · code-only mode (no cargo/tests/graphify).
> This document publishes the EXACT hunks for the seams g78 could not wire
> on its own branch because they are contested by in-flight lanes. The
> contest evidence is the per-branch `git diff origin/main...<branch>`
> hunk census run 2026-11-21 (every `origin/task/W4-*` branch scanned for
> the target files/regions). **The merger (c42d / the research/95 union
> lane) applies these hunks; g78 does not.**

## What g78 WIRED on its own branch (the uncontested seams)

| Seam | Where | Evidence it is uncontested |
| --- | --- | --- |
| The 14-arm NPC debuff tint chain (`Main.cs:22939`) | `render/entities.rs` colour site (`let c = style.shade(...)` :2180-region) | ZERO lanes touch the :2176-2181 window (the npc-kind lanes' hunks all land at :3170+ / the signature tail :2156) |
| The behind-tiles black-out scan (`Main.cs:22922-22938`) | the same site, BEFORE the chain | same window |
| The G6 `npc_key` tint-widen | `npc.rs::render_key` free fn (:18631) | ZERO lanes touch :18631-18637 (every npc.rs hunk stops at the kind-arm insertions ~:18601) |
| The spelunker bitset keep-one prep + `spot_in_range` | `spelunker_helper.rs` (g78's own file) | — |
| The attach-ready 473/1008 AI arms (`SpelunkerSpot`) | `projectile_ai_light_items.rs` | only d76 touches the file (the fairy-hover :425 region + the doc line; the doc-line edit unions trivially — both remove their landed skip from the same list) |

The Hunter floor's `hunter_active`/`hunter_friendly` inputs stay `false`
in the live fold (`tint_flags_from_buffs`) until §2's threading lands —
the debuff arms are independent of them, so the wired family is complete
without the potion.

## §1 — the SpelunkerProjectileHelper dispatch attach (the DropRng contract)

The C# has THREE `AddSpotToCheck` callers (research/76 censused two; the
third is new here):

| # | C# site | Cadence | Gate | Stream |
| --- | --- | --- | --- | --- |
| 1 | the 473 glowstick head, `Projectile.cs:24836-24847` | `localAI[0]` 10-tick counter wrap | `Main.netMode != 2` wraps the WHOLE block (the counter included); `(Center - player.Center).Length() < screenWidth + 480` | `Main.rand` = **UpdateProjectiles** (`Main.cs:17775-17777` swap wraps `UpdateWorld_Projectiles`) |
| 2 | the 1008 Spelunker Flare's STUCK arm, `Projectile.cs:27124-27127` | EVERY stuck tick (no counter); runs AFTER the un-stick check and REGARDLESS of it (a sibling `if`, not an else) | same netMode + distance gates | same UpdateProjectiles |
| 3 | the HELD Spelunker Glowstick 3002, `Player.cs:50094-50099` (`ItemCheck_EmitHeldItemLight`, called from `ItemCheck` `:42849` behind `!JustDroppedAnItem && IsAllowedToHoldItems` `:42847`) | `spelunkerTimer` (a `byte` field, `Player.cs:810`) 10-tick counter; `!pulley`-gated (`:50081`); NO distance gate, NO netMode gate | — | `Main.rand` = **UpdatePlayers** (`Main.cs:17754-17757` swap wraps `UpdatePlayers`) |

The helper's `Next(4)` draws ride the CALLER's stage stream — one stream
per call site, never a new one. The `_positionsChecked`/`_tilesChecked`
sets are ONE shared helper instance (`Main.instance.SpelunkerProjectileHelper`,
`Main.cs:2233`) — all three callers dedupe against the SAME sets, and the
10-tick clear cadence (§1.1) resets all of them.

### §1.1 — the pool fields + the pass head/tail (`projectile.rs`, `world_sim/projectiles.rs`)

CONTESTED: the `ProjectilePool` struct tail (`golf_effects` anchor) is
touched by d9/d13/d16/g4/g16/g65/g82; the pass head (`set_disco` anchor)
by d13/d16/g4/g64; the pass tail by d13/d16/f49/f50/g11/g16/g63/g64/g65/g82.
The hunks below are written against the CURRENT main line numbers; the
merger re-anchors on the union state.

**(a) the pool struct** — after `golf_effects` (the last field):

```rust
    /// W4-g78 — `Main.instance.SpelunkerProjectileHelper` (`Main.cs:2233`):
    /// the ONE shared ore-spot helper (the 473 head, the 1008 stuck arm and
    /// the held 3002 all dedupe against its sets). The pass head ticks its
    /// 10-tick clear cadence; the AI arms attach it per slot.
    pub(crate) spelunker: crate::spelunker_helper::SpelunkerProjectileHelper,
    /// W4-g78 — the per-tick spot context (`Main.player[myPlayer].Center`
    /// + `Main.screenWidth` + `Main.netMode != 2`); the pass head fills it
    /// (the `set_disco` pattern). `None` player = the distance gate fails
    /// closed.
    spelunker_player: Option<Vec2>,
    spelunker_screen_width: f32,
    spelunker_net_ok: bool,
    /// W4-g78 — the arms' queued 204-dust requests this tick, drained by
    /// the pass tail into `app.dust` (the `pending_proj_sounds` shape).
    pending_spelunker_dust: Vec<crate::spelunker_helper::SpelunkerDust>,
```

`new()`: `spelunker: Default::default(), spelunker_player: None,
spelunker_screen_width: 1280.0, spelunker_net_ok: true,
pending_spelunker_dust: Vec::new(),`.

**(b) the pass head** (`world_sim/projectiles.rs::update_projectiles`,
after `app.projectiles.set_disco(disco);` — the d13/d16 setter cluster):

```rust
    // W4-g78 — `Main.PreUpdateAllProjectiles` (`Main.cs:17535-17539`,
    // called from `UpdateWorld_Projectiles` `:18001` BEFORE the 1000-slot
    // walk): the helper's 10-tick dedupe-clear cadence, once per pass.
    app.projectiles.spelunker.on_pre_update_all_projectiles();
    app.projectiles.spelunker_player = app.player.as_ref().map(|p| {
        let (x, y) = p.center();
        Vec2::new(x, y)
    });
    app.projectiles.spelunker_screen_width = sw;
    app.projectiles.spelunker_net_ok = app.net_mode() != 2;
```

(`sw` is the `app.screen_size()` width already bound at :144; `net_mode()`
is the `app/netglue.rs:2105` role enum — 2 = host/server.)

**(c) the aiStyle-14 dispatch arm** (`projectile.rs` :17091-17098, the
`GLOWSTICK_KINDS` branch — CONTESTED by f5/g63/g64's dust rewrites of the
same arm; the swap keeps their dust shape):

```rust
                            if crate::projectile_ai_light_items::GLOWSTICK_KINDS
                                .contains(&self.slots[i].kind)
                            {
                                // W4-g78 — the 473 head's AddSpotToCheck
                                // (`Projectile.cs:24844`): the helper +
                                // the per-tick context attach here; the
                                // dust requests queue for the pass tail.
                                let mut out = Vec::new();
                                let spot = crate::projectile_ai_light_items::SpelunkerSpot {
                                    helper: &mut self.spelunker,
                                    player_center: self.spelunker_player,
                                    screen_width: self.spelunker_screen_width,
                                    net_mode_ok: self.spelunker_net_ok,
                                    out: &mut out,
                                };
                                crate::projectile_ai_light_items::glowstick_step_with_spot(
                                    &mut self.slots[i],
                                    world,
                                    rng,
                                    Some(spot),
                                );
                                self.pending_spelunker_dust.extend(out);
                            } else {
```

**(d) the aiStyle-33 dispatch arm** (:17153-17159) — the same
`flare_step_with_spot(…, Some(spot))` swap + the `out` extend.

**(e) the pass tail** (`world_sim/projectiles.rs`, after the
`drain_proj_sounds` loop — the S1-E-21h cluster):

```rust
    // W4-g78 — the ore-spot 204 dust requests (`SpelunkerProjectileHelper.
    // CheckSpot` `:58-61`): `Dust.NewDustDirect(position, 16, 16, 204,
    // 0f, 0f, 150, default, 0.3f)` then fadeIn 0.75 / velocity ×0.1 /
    // noLight. The helper's Next(4) gate already drew on the
    // UpdateProjectiles stream mid-walk; the SPAWN's internal
    // `Dust.NewDust` rolls (the velocity jitter) draw here on the SAME
    // stream at the pass tail — the S1-E-21c queue→drain deviation
    // (ANALYSIS), the C# spawns inline mid-walk.
    let ctx = app.spawn_ctx();
    for d in app.projectiles.pending_spelunker_dust.drain(..) {
        let idx = app.dust.new_dust_direct(
            app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
            &ctx,
            &app.world,
            glam::Vec2::new(d.x, d.y),
            16,
            16,
            d.dust_type,
            0.0,
            0.0,
            d.alpha,
            tr_core::particles::Color::default(),
            d.scale,
        );
        let slot = app.dust.slot_mut(idx);
        slot.fade_in = d.fade_in;
        slot.velocity *= d.velocity_scale;
        slot.no_light = d.no_light;
    }
```

NOTE the borrow: `pending_spelunker_dust` lives on the pool, `app.dust`
is a sibling App field — the drain-into-local-Vec-first pattern (the
`take_cart_sparks` precedent) if the borrow checker refuses the direct
`app.projectiles.…drain` + `app.dust` pair.

### §1.2 — the RNG-stream verdict (the determinism contract)

* The helper's `Next(4)` per newly-checked spelunkable tile draws on
  **UpdateProjectiles** for callers 1+2 (the C# `Main.rand` under the
  `UpdateWorld_Projectiles` swap) and **UpdatePlayers** for caller 3.
  A repeat tile rolls NOTHING (`_tilesChecked.Add && Next(4)`).
* The dust SPAWN's internal rolls (`Dust.NewDust`'s velocity jitter,
  `Dust.cs:77-98`) are the port's deferred-to-tail draws (the S1-E-21c
  convention) — they land AFTER the walk's other UpdateProjectiles draws
  instead of mid-walk. This is the SAME recorded deviation every dust
  queue carries; the merger must NOT "fix" it by moving the spawn inline
  (the pool cannot borrow `app.dust` mid-walk).
* The 10-tick clear cadence is ONE counter shared by all callers — a
  parked glowstick + a held 3002 re-roll their windows on the SAME tick.

### §1.3 — the held-3002 attach (`player.rs` / `player_lights.rs`)

CONTESTED: `player.rs` is touched by 53 branches (the f33 torch-god lane
rewrites `player_lights.rs`'s held-light family). The C# counter is a
`byte spelunkerTimer` field (`Player.cs:810`) — the port adds it to
`Player` (reset in `ResetEffects`? NO — the C# never resets it there; it
is a plain field that persists, `:50094-50099` is its ONLY writer). The
attach point is the `player_lights.rs` 3002 arm (:434-443, the comment
"the spelunkerTimer half is the ore-spot helper, not a light — unported"):

```rust
    if kind == 3002 {
        // ... the existing light emit ...
        // W4-g78 — `Player.cs:50094-50099`: the held-glowstick ore-spot
        // counter (NO distance gate, NO netMode gate; `!pulley` wraps the
        // whole 3002 block `:50081`). The helper is the SAME pool instance
        // (§1.1) — the player tick runs BEFORE the projectile pass
        // (`Main.cs:17754` vs `:17775`), so the held scan's Next(4) draws
        // land on UpdatePlayers BEFORE the pass's UpdateProjectiles draws.
        if !p.pulley {
            p.spelunker_timer += 1;
            if p.spelunker_timer >= 10 {
                p.spelunker_timer = 0;
                // add_spot_to_check(p.center(), UpdatePlayers stream, out)
                // → the player tick's dust drain (the take_sticky_dust
                // precedent, world_sim/player.rs:1366).
            }
        }
    }
```

The `player_lights` fn is pure (returns light tuples) — the counter +
helper call must ride the CALLER (`world_sim/player.rs`'s tick, where
`app` is borrowable), not the pure fn. The merger threads the helper
through the player tick's existing dust-drain cluster.

## §2 — the Hunter-floor threading (the contested `draw_npcs` tail)

CONTESTED: the `draw_npcs` signature tail (`behind_tiles: bool,` anchor)
is touched by c39 (the `party` param) and d70 (the
`latest_filter_strength` param); the call sites (`render/tiles.rs` :586
/:1237) by g83/g85/g12/d70. The G6 key struct (`WorldFrameKey`) by
g12/g80/f14/d70.

**(a) the signature** — after the c39/d70 params land, add:

```rust
    // W4-g78 — `CanApplyHunterPotionEffects` (`NPC.cs:91019-91022`): the
    // perspective player's `detectCreature` (buff 17). The Hunter floor
    // (`:91110-91139`) needs it per NPC draw.
    hunter_active: bool,
```

**(b) the call sites** (`render/tiles.rs` :586/:1237 + `draw_tiles`'s own
threading from `app/frame/draw.rs`): pass
`player.is_some_and(|p| p.detect_creature)` (the `render_player` binding
already threaded for the player draw).

**(c) the fold** — `tint_flags_from_buffs` gains the `hunter_active`
parameter (or the call site overrides the field after the fold):

```rust
        let mut f = crate::npc_hunter_tint::tint_flags_from_buffs(n);
        f.hunter_active = hunter_active;
        // KEEP-ONE: f47's shimmer predicate (`NPC.cs:91008-91017`, the
        // 689 override `NPCID.cs:10128-10132`) — NOT re-derived here.
        f.hunter_friendly = tr_core::shimmer::appears_friendly_to_hunter_potion(
            n.kind.net_id() as u16, n.friendly,
            crate::bug_net::catch_item(n.kind.net_id(), n.ai[2]),
            n.damage, n.max_hp,
        );
```

(f47's `appears_friendly_to_hunter_potion(npc_type: u16, friendly,
catch_item, damage, life_max)` — the signature on
`origin/task/W4-f47-liquid-shimmer-2` `shimmer.rs:189`; the `catch_item`
rows are `bug_net.rs:124`'s port, the butterfly's live `ai[2]` style
included.)

**(d) the G6 key** — the `hunter_active` flag is a PLAYER input, so it
rides the key as a new field (the d70 `moon_lord_strength` precedent —
gated on `hunter_active && any lifeMax>1 NPC present` to keep potion-free
worlds bit-identical), OR folds into `player_key` (the player's buff bar
already rides it? — NO, `player_key` is position/animation only; the
merger adds the field). The `tint_key` buff hash g78 landed covers the
NPC side; the potion side needs this one bit.

**(e) the draw-order swap** (`Main.cs:63290-63299`): `detectCreature`
moves `DoDraw_DrawNPCsBehindTiles` AFTER `DoDraw_Tiles_Solid`. The port's
`draw_tiles` runs the behind-tiles NPC pass at :586 (before the solid
tile pass) — the swap is a conditional reorder of that ONE call (the
`hunter_active` flag again). CONTESTED by every tiles.rs lane; the merger
applies it with the §2(a-d) threading as ONE unit (the floor without the
swap draws hunter-lit NPCs BEHIND solid tiles — wrong).

## §3 — the 657 SwitchMonolith use-arm + the Check2xX break-drop (for c42d)

CONTESTED: `edit.rs::interact_at` (c42/f11/e10/f34/f41/g24/e28/c39…),
`world_sim/editing/doors.rs` (c42/c36/e10/f34/f41/g24/d79/d8/e17/e28/f29/
f32/f22…), `edit/kill.rs::check_object` (c31/f41/g17/g47/c14/c29/g42/g45/
g46/g85/f25/c36…), `wiring.rs` (c42/e20/e21/f11/d6/g47/g73/d106/g11).
e21's kind-match fix (`fdbf37d9`) is the `switch_monolith` the use-arm
calls — READ `origin/task/W4-e21-wiring-device-arms-3` latest before
applying (the base port flipped ANY kind; e21 gates to the nine).

### §3.1 — the right-click use arm (`Player.cs:32271-32277`)

The C# chain arm (AFTER the 207 fountain arm `:32265-32270`, BEFORE the
455 party arm `:32278-32284`):

```csharp
else if (Main.tile[myX, myY].type == 410 || ... || type == 657 || ... || type == 733)
{
    flag2 = true;
    SoundEngine.PlaySound(28, myX * 16, myY * 16, 0);
    this.GamepadEnableGrappleCooldown();
    WorldGen.SwitchMonolith(myX, myY);
}
```

**(a) `edit.rs::interact_at`** — a new kind-literal arm BEFORE the
registry-def lookup (:320, the dials/banks/chairs precedent — the
monoliths have registry defs but the C# tests the kind literal here):

```rust
    // W4-g78 — the monolith family (`Player.cs:32271-32277`): the
    // right-click flips the sprite via `WorldGen.SwitchMonolith`. The
    // nine kinds are the C# literals (410/480/509/657/658/720/721/725/733).
    if matches!(t.kind, 410 | 480 | 509 | 657 | 658 | 720 | 721 | 725 | 733) {
        return TileInteraction::MonolithSwitch { x: i, y: j };
    }
```

+ the `TileInteraction::MonolithSwitch { x: i32, y: i32 }` variant (the
`HitSwitch` shape).

**(b) `doors.rs::interact_tile`** — the match arm (the `HitSwitch` arm's
shape, :216-234):

```rust
        edit::TileInteraction::MonolithSwitch { x, y } => {
            // `Player.cs:32274` — the Mech-28 click at the tile's top-left
            // (the SAME cue the lever uses; `play_switch_click` is the
            // port's case-28 façade, the UpdatePlayers stream — the tile
            // use runs in `Player.ItemCheck`, `Main.cs:17754`).
            app.audio.play_switch_click(
                (*x * 16) as f32,
                (*y * 16) as f32,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
            // `:32275` — GamepadEnableGrappleCooldown: the port has no
            // gamepad layer (recorded deviation, the f11 precedent).
            // `:32276` — the flip. e21's kind-gated switch_monolith is a
            // Wiring method; the use path runs it with `running = false`
            // (the SkipWire tail is wiring-only) — the merger exposes a
            // `switch_monolith_use` wrapper OR calls the existing method
            // on a fresh non-running Wiring (the C# `WorldGen.SwitchMonolith`
            // is a STATIC fn — the `Wiring.running` gate reads the global
            // flag, false outside a wire flood).
            app.wiring.switch_monolith_use(&mut app.world, *x, *y);
            app.world_key = None;
            return true;
        }
```

The `switch_monolith_use` wrapper (in `wiring.rs`, e21's lane): decode
the clicked cell's kind/frameX/frameY, call the existing private
`switch_monolith(world, i, j, kind, fx, fy)` — the `running` flag is
already false outside `TripWire`, so the SkipWire tail no-ops (the C#
`Wiring.running` static is false during a player use). NO new flip logic
— e21's kind-match is the single source.

### §3.2 — the Check2xX break check (`WorldGen.cs:37993-38287`, drop 5345)

The C# dispatch (`:85738`) lists 657 in the Check2xX group; the routine
is the SAME machine `check_clock` already ports (the 2×X integrity scan +
the bottom-support test), with `num4 = 3` (the 657 is 2×3; the clock's
`num4 = 5` is the `type == 104` override `:38014-38017`). The drop
(`:38221-38224`): `Item.NewItem(source, num*16, j*16, 32, 32, 5345)` —
at the ORIGIN cell `(num, j)` (the LEFT column, the TRIGGER row — NOT the
sprite's top-left `num6`), a 32×32 box.

**(a) `edit/kill.rs::check_object`** — the dispatch arm (the `104 =>
check_clock` sibling, :669):

```rust
        // W4-g78 — the monolith family rides Check2xX (`WorldGen.cs:85738`
        // lists 410/480/509/657/658/720/721/725; the 733 is 3×3, NOT in
        // the group — it has no Check2xX arm in the C# dispatch).
        410 | 480 | 509 | 657 | 658 | 720 | 721 | 725 => check_monolith(world, x, y),
```

**(b) the `check_monolith` fn** — `check_clock`'s shape with `height = 3`
and the per-kind drop table (`WorldGen.cs:38209-38240`):

```rust
/// `WorldGen.Check2xX` (`WorldGen.cs:37993-38287`) for the monolith family
/// (the 2×3 sprite; the clock's 2×5 is the same machine with height 5).
/// The drop is the family's item at the ORIGIN cell `(num, j)` — the LEFT
/// column, the TRIGGER row (`:38221-38224`'s `num*16, j*16`, NOT the
/// sprite top-left `num6`), a 32×32 box.
fn check_monolith(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let t = world.tile(x, y)?;
    let frame_x = t.frame_x;
    let mut ox = x;
    if frame_x % 36 == 18 {
        ox -= 1;
    }
    let mut num3 = world.tile(ox, y)?.frame_y as i32 / 18;
    let height = 3;
    let mut style_offset = 0i32;
    while num3 >= height {
        num3 -= height;
        style_offset += height * 18;
    }
    // `:38029-38032` — the 410's lit-row offset quirk: `if (type == 410
    // && num5 != 0) num5 += 2;` (the 410's lit frames are 56, not 54 —
    // the +2 keeps the scan aligned; the OTHER kinds' 54 needs no quirk).
    if kind == 410 && style_offset != 0 {
        style_offset += 2;
    }
    let oy = y - num3;
    let top_frame_x = world.tile(ox, y)?.frame_x;
    let mut broken = false;
    let mut region = Vec::with_capacity(6);
    for l in oy..oy + height {
        for (dx, fx) in [(0i32, top_frame_x), (1, top_frame_x + 18)] {
            let k = ox + dx;
            region.push((k, l));
            let broken_cell = world.tile(k, l).is_none_or(|c| {
                !c.active()
                    || c.kind != kind
                    || c.frame_y != ((l - oy) * 18 + style_offset) as i16
                    || c.frame_x != fx
            });
            if broken_cell {
                broken = true;
            }
        }
    }
    if !solid_tile_allow_bottom_slope(world, ox, oy + height)
        || !solid_tile_allow_bottom_slope(world, ox + 1, oy + height)
    {
        broken = true;
    }
    if !broken {
        return None;
    }
    // The per-kind drop (`WorldGen.cs:38209-38240`): 410 → frameX/36+3536,
    // 480 → 4054, 509 → 4318, 657 → 5345, 658 → 5347, 720 → 5598,
    // 721 → 5599, 725 → 5655. The 410's style decode reads the ORIGIN
    // cell's frameX (`frameX` local `:38037` = `Main.tile[num, j].frameX`).
    let item = match kind {
        410 => Some(top_frame_x as i32 / 36 + 3536),
        480 => Some(4054),
        509 => Some(4318),
        657 => Some(5345),
        658 => Some(5347),
        720 => Some(5598),
        721 => Some(5599),
        725 => Some(5655),
        _ => None,
    };
    Some(CheckPlan {
        drop: Some(ObjectDrop { tile_kind: kind, x: ox, y, item }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None, // Check2xX has no TileFrame tail (the CheckPlan doc).
    })
}
```

NOTE the drop's `x: ox, y` — the C# spawns at `(num, j)` where `num` is
the LEFT-column origin and `j` the TRIGGER row (the clicked cell's row,
NOT `oy`). The 32×32 box enters `Item.NewItem` ONLY through the centre
conversion (`Item.cs:49651`: `center = (X + Width/2, Y + Height/2)` →
`(num*16+16, j*16+16)`); the spawn VELOCITY is the box-independent
`Next(-30,31)*0.1` / `Next(-40,-15)*0.1` jitter (`:49749-49750`). The
port's `spawn_item_drop` feeds the tile TOP-LEFT (`items.rs:745`) and
`ItemDrop::new` adds its own +1 px — so the port's object drops land ~15 px
left/up of the C# centre. This is a PRE-EXISTING gap shared by EVERY
`ObjectDrop` consumer (the clock's 32×32 `:38154`, the statue's `:38158`,
the dial's `:38162` all carry it) — NOT monolith-specific; the merger
fixes it once at the `ObjectDrop` → spawn seam (pass the centre, not the
top-left) rather than per-check.

NOTE the 733 (the 3×3 Moon Lord monolith) is NOT in the Check2xX dispatch
group — its break check is unported in C# too (the C# has no 733 arm in
the `:85718-85744` group); the port matches by omission.

### §3.3 — the witness contract (for the merger)

* The use arm: a right-click on a 657 cell flips the whole 2×3 body's
  frameY ±54 (the e21 kind-gate: a non-monolith cell inside the rect keeps
  its frames), plays ONE Mech-28 on UpdatePlayers, consumes the click.
* The break check: kill ONE cell of a placed 657 → the whole 2×3 dies,
  ONE 5345 drops at `(ox*16, j*16)` (the origin column, the trigger row);
  an intact body with solid support returns None.
* The 410 lit-row quirk: a lit 410 (frameY 56+) breaks correctly (the
  +2 style offset); the other kinds' 54-row needs no quirk.

## §4 — the IsTileSpelunkable keep-one (the dedupe hunks)

FOUR copies exist today (research/76 censused three; the d9 bitset is the
canonical form):

| Copy | Where | Form |
| --- | --- | --- |
| d9 | `projectile_ai_style90.rs:69` `TILE_SPELUNKER: TileSet` + `is_tile_spelunkable` | the 12-word bitset + the 185/186 windows — **CANONICAL** |
| g78 | `spelunker_helper.rs` `TILE_SPELUNKER: TileSet` + `is_tile_spelunkable` | the d9 bitset VERBATIM (this wiring phase) + the windows |
| d13 | `projectile_ai_pets124.rs:57` `is_tile_spelunkable` | an inline `&[u16]` 49-id table + the windows |
| d56 | `tr_render::animated_tiles::is_tile_spelunkable` | the tr-render copy (no tr-core dep) |

**The keep-one target: d9's `projectile_ai_style90` copy** (the bitset is
the efficient form; the tr-app crate can reach it; the tr-render crate
CANNOT depend on tr-app, so d56's copy needs the bitset moved to tr-core
OR duplicated — the merger decides: the cleanest is a
`tr_core::tiledata`-adjacent hand-transcribed set, but tiledata is
GENERATED (never hand-edit) — so the bitset stays in tr-app and d56's
tr-render copy is the recorded exception, the `SWAYS_IN_WIND_BASIC`
precedent).

**The dedupe hunks (at the wiring merges):**

1. **g78's copy deletes** — `spelunker_helper.rs`'s `TILE_SPELUNKER` static
   + `TILE_SPELUNKER_IDS` const + `is_tile_spelunkable` fn replace with
   `pub use crate::projectile_ai_style90::{TILE_SPELUNKER, is_tile_spelunkable};`
   (the census witness moves to d9's module OR keeps a re-export smoke
   test). The `check_spot` call site (`is_tile_spelunkable(tile.kind, …)`)
   is unchanged.
2. **d13's inline table deletes** — `projectile_ai_pets124.rs:57`'s fn
   replaces with `use crate::projectile_ai_style90::is_tile_spelunkable;`
   (the call site :578 is unchanged).
3. **d56's tr-render copy** — stays (the crate-boundary exception) OR the
   bitset words move to a shared `tr_core` hand-transcribed module (NOT
   tiledata/ — the generated-files rule); the merger's call. The census
   witness (g78's `w4_g78_the_spelunker_table_census`, now proving
   bitset==49-id-list for EVERY id in 0..TILE_COUNT) is the equivalence
   proof whichever form wins.

The 185/186 frame windows are IDENTICAL in all four copies (the
`frameY == 18 && frameX in 576..=882` / `frameX in 864..=1170` inclusive
ranges, `Main.cs:21789-21796`) — the dedupe is lossless.

## §5 — the residual co-lane list (unchanged from research/76)

The d56 vision floors, the f12 danger_sense live field, the f47 shimmer
predicate consumer, the npc-kind/style lanes' unported tint flags
(midas/ichor/betsysCurse/stinky/brokenArmor/chlorophyteSpore/potentAcid/
drippingSparkleSlime — NOW FOLDED by g78's `tint_flags_from_buffs`, so the
"unported-flag remainder" is CLOSED for the tint consumer; the flags'
OTHER consumers (the DoT tally etc.) stay the buff-wave lanes').

## Witnesses (WRITTEN NOT RUN — code-only)

This wiring phase adds 9 witnesses on top of research/76's 14:
* `spelunker_helper.rs` (1 new): `w4_g78_the_caller_distance_gate_is_strict`
  (+ the census witness widened to the bitset==list equivalence).
* `npc_hunter_tint.rs` (4 new): `w4_g78_the_buff_fold_maps_the_csharp_ids`,
  `w4_g78_the_golem_parts_refuse_the_display`,
  `w4_g78_the_tint_key_tracks_the_buff_bar`,
  `w4_g78_the_wired_chain_tints_a_poisoned_npc`.
* `projectile_ai_light_items.rs` (4 new):
  `w4_g78_the_473_head_attaches_on_the_counter_wrap`,
  `w4_g78_the_server_skips_the_whole_473_block`,
  `w4_g78_the_1008_stuck_arm_checks_every_tick_deduped`,
  `w4_g78_the_attach_distance_gate_refuses_far_players`.

The §1-§3 contract hunks carry their witness contracts inline (§3.3) —
the merger writes them when the hunks land.