//! W4-d4 × W4-g82 union — the rope-coil lane's EXTRAS (research/169's
//! keep-one verdict: d4's `projectile.rs::ai_rope_coil` + the four
//! `projectile/data.rs` rows + wind-list 35 stay the single aiStyle-35
//! machine; g82's `coil_step`/`coil_proj_row`/`coil_rope_kind`/`is_rope_cell`
//! are the duplicate half and are NOT landed here). This module keeps the
//! pieces d4 never claimed, ported from g82's `projectile_rope_coil.rs`
//! (`67c9a740`):
//!
//! * [`coil_death_drop`] — `Kill()`'s refund ladder
//!   (`Projectile.cs:77572-77615`). WIRED: the pool's once-per-death hook
//!   (`projectile.rs`, the bobber-arm seam) queues `(item, stack, centre,
//!   zero_velocity)` into `pending_death_drops` (the ladder passes no
//!   velocity — `false`); the world sim drains it at the pass tail
//!   (`world_sim/projectiles.rs`) as an `Item.NewItem` world drop on the
//!   `UpdateItems` stream (the two-integer scatter pair; the beach ball's
//!   `Vector2.Zero` sibling arm draws nothing). d4's coil killed with NO
//!   refund — the one behavioural gap the 26e comparison found, closed
//!   here.
//! * [`coil_chain_sheets`]/[`coil_chain_segments`]/[`draw_coil_chain`] —
//!   the trailing-chain placement preview (`Main.DrawProjDirect`,
//!   `Main.cs:28499-28669`). The coil items have NO `createTile`
//!   (`Item.cs:12697-12711`/`:27730-27743`/`:28530-28558`), so the chain of
//!   rope links trailing the thrown coil IS the placement preview.
//!   UNWIRED draw half (the g82b draw lane's): `draw_with_hand`
//!   (`projectile.rs`) must call [`draw_coil_chain`] gated on [`is_coil`]
//!   INSTEAD of the generic sprite light tint (each link carries its own
//!   `Lighting.GetColor`), and the `Chain16`/`Chain38`/`Chain39`/
//!   `Chains_4..7` sheets must join the `assets.rs` static list (the
//!   `FishingLine` precedent). The contract is documented in
//!   docs/ANALYSIS.md "The rope-coil lane extras (W4-d4 union)".
//!
//! The usability walk (`Player.PlaceThing_Tiles_CheckRopeUsability`,
//! `Player.cs:41506-41545`) landed directly in `world_sim/editing.rs`
//! (`check_rope_usability`, wired into `use_tile_item`) — it reads the
//! app's tile-target seam, so it lives with its call site, not here.
//!
//! Deviations (docs/ANALYSIS.md): the dust walks sharing the chain
//! geometry (`Projectile.cs:74615-74731`) skip with the dust layer; the
//! `ai[1] == 0` sound cue (`:74617-74620`/`:74681-74684`) is WIRED at the
//! pool's death hook (retire phase): the int overload's LEGACY id 0 = Dig
//! (NOT Item1 — the g82 `3258189cb` correction), queued `(0, 0)` into
//! `pending_proj_sounds` and drained through the `play_place_sound`
//! facade's id-0 arm, UNGATED by owner/noDropItem (it sits in `Kill()`'s
//! dust chain); the refund ladder + the beach-ball return sit INSIDE
//! `Kill_ExplodeTiles` behind the `:77267` owner refusal + the `:77367`
//! `!noDropItem` head (the retire fix — the earlier no-gate reading
//! confused the grab reservation with a spawn gate); the refund drop's
//! `ReserveForLocalPlayer`
//! ownership sets only `enemyGrabDelayTime` in netMode 0
//! (`Item.cs:49757-49760`, `WorldItem.cs:295-305`) — the port's enemy-grab
//! clock is unported, so the drop is player-grabbable immediately (the
//! lucky-coin precedent, `world_sim/editing.rs`); the C# spawn draws ride
//! `Main.rand` inline mid-`Kill()` (the `UpdateProjectiles` swap), the
//! port defers them to the pass tail's `UpdateItems` stream (the bobber
//! overflow's recorded stream deviation).

use glam::Vec2;
use tr_core::TILE_PX;

use crate::projectile::{ROPE_COIL, SILK_ROPE_COIL, VINE_ROPE_COIL, WEB_ROPE_COIL};
use crate::projectile::Projectile;

/// The `DrawProjDirect` chain-draw gate (`Main.cs:28499`/`:28549`/`:28610`).
/// The death hook's ai1==0 Dig cue gates on it (wired); the g82b draw
/// wiring gates `draw_with_hand` on it too (module doc).
//! W4-g82 — the rope-coil lane: the four coil projectiles' aiStyle-35
//! machine (`Projectile.AI()`'s `aiStyle == 35` block,
//! `Projectile.cs:27189-27291`), their trailing-chain placement preview
//! (`Main.DrawProjDirect`, `Main.cs:28499-28669` — the coil's "placement
//! preview" IS this chain: the coil items have NO `createTile`
//! (`Item.cs:12697-12711`/`:27730-27743`/`:28530-28558`), so they never
//! enter the `TileObject.objectPreview` path (`TileDrawing.cs:737-740`);
//! the chain of rope links trailing the thrown coil previews where the
//! rope will land), the `Kill()` refund (`Projectile.cs:77572-77615`) and
//! the rope-target usability walk (`Player.PlaceThing_Tiles_CheckRopeUsability`,
//! `Player.cs:41506-41545`).
//!
//! The four kinds (`Projectile.SetDefaults`):
//! * **171** Rope Coil (`:2570-2578`, shared with 505/506): 14x14,
//!   aiStyle 35, `penetrate = -1`, `tileCollide = false`, `timeLeft = 400`.
//! * **475** Vine Rope Coil (`:5496-5504`): the same row.
//! * **505** Silk Rope Coil / **506** Web Rope Coil (`:2570-2578`).
//! The placed rope kind per coil (`:27227-27239`): 171→213 Rope,
//! 475→353 VineRope, 505→365 SilkRope, 506→366 WebRope. The chain item
//! (214) is NOT a coil — it places by hand (`Item.cs` case 85).
//!
//! **WIRED (2026-11-21, the wiring phase).** The call sites, all live:
//! * [`coil_proj_row`] — the four rows landed verbatim in
//!   `projectile/data.rs` (`171 | 505 | 506` + `475`, appended before the
//!   None arm); this fn stays the witness-pinned reference. The item
//!   rows already existed (`itemdata.rs` 985/3005/3079/3080 → projectile
//!   171/475/505/506).
//! * [`coil_step`] — the `35 =>` arm of `ProjectilePool::update`'s
//!   `match data.ai_style` (`projectile.rs`, ahead of the 36 arm — the
//!   C# body order), `world: &mut World` via the per-arm mutable
//!   reborrow (the `ai_ice_block`/`ai_dungeon_debris` precedent). The
//!   returned `TileKill` records are DISCARDED at the dispatch (the
//!   `let _ = kill_tile` precedent — the cascade runs, the per-cell
//!   drops stay unspawned, recorded in ANALYSIS). The arm also pins
//!   `friendly = false` (the `:788` reset) and the PVP pass exempts the
//!   coils (the debris/slime list); the generic
//!   `AutomaticallyChangesDirection` write covers style 35.
//! * [`coil_chain_segments`] + [`draw_coil_chain`] — `Projectile::draw_with_hand`
//!   (`projectile.rs`), ahead of the sprite emit, gated on [`is_coil`]
//!   (the c6 pumpking-chain precedent — the C# chain arms run before the
//!   generic emit, each link carries its own `Lighting.GetColor`). The
//!   `Chain16`/`Chain38`/`Chain39`/`Chains_4..7` sheets ride
//!   `assets.rs`'s `STATIC_SHEET_NAMES` + `Lookups::coil_chains` /
//!   `coil_chain_uv` (the `glow_sheets` keyed-name precedent).
//! * [`coil_death_drop`] — `ProjectilePool::coil_kill` at the `!active`
//!   death hook beside `thrown_consumable_death` (the d77 kill_tail_cue
//!   pattern — the refund AND the ai1==0 cue both live inside `Kill()`);
//!   the drops drain through `world_sim/projectiles.rs` on the
//!   `UpdateProjectiles` stream (`RequestNewItem` = a world drop).
//! * [`check_rope_usability`] — `use_tile_item` (`world_sim/editing.rs`),
//!   right after the registry resolve (the `Player.cs:39726` gate-chain
//!   position); the walked target shadows `ty` for every downstream
//!   gate (the C#'s `Player.tileTargetY` static write).
//!
//! Deviations (recorded in docs/ANALYSIS.md "The rope-coil lane (W4-g82)"):
//! * **FINAL keep-one (2026-11-21, `docs/research/26e-f-rope-union-final.md`)**:
//!   this module's [`coil_step`] is the CANONICAL style-35 machine — the
//!   d4 branch's `ai_rope_coil` + its four `data.rs` const rows are the
//!   drop half (the re-adjudication reversed 26e's KEEP-d4 verdict after
//!   g82 wired the full lane: refund + cue + usability + chain draw +
//!   the friendly pin + PVP exemption + the style-35 direction write,
//!   none of which d4 carries; the two faithfulness deltas 26e found —
//!   the null-abort integrate and the dead-tick integrate — are FIXED
//!   here, so the machine is now a superset of d4's).
//! * The dust walks that share the chain geometry (`Projectile.cs:74615-
//!   74677` for 475/505/506, `:74679-74731` for 171 — 4 dust per segment
//!   past `ai[1]`, types 3/30/239/129) are SKIPPED with the dust layer;
//!   they draw ZERO state (the `ai_style2` precedent). The `ai[1] == 0`
//!   sound cue (`:74617-74620`/`:74681-74684`, the int overload's legacy
//!   id 0 = Dig — the port's `(0, 0)` cue, the d77 map's thud shape) is
//!   WIRED at `coil_kill` (the death hook — both sites live inside
//!   `Kill()`, ahead of `active = false`).
//! * `NetMessage.SendData(17, …)` after each KillTile/PlaceTile
//!   (`:27269`/`:27277`) is the net layer's seam — skipped (single-player).
//! * `WorldGen.IsTileLoaded` (`Player.cs:41530-41537`) is always true in
//!   the port's single-player world; the SendData(159) request arm is
//!   skipped with it.
//! * The C# `TileCollision` probe (`:27206`) rides the port's
//!   `collision::tile_collision` (gravDir 1, hoik true — the coil's
//!   `fallThrough: true, fall2: true` pair maps to the port's
//!   `fall_through`/`fall2` args).

use glam::Vec2;
use tr_core::edit::TileKill;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World};

use crate::projectile::Projectile;

/// The four coil kinds (`ProjectileID.cs`: RopeCoil 171, VineRopeCoil 475,
/// SilkRopeCoil 505, WebRopeCoil 506).
pub(crate) const ROPE_COIL: u16 = 171;
pub(crate) const VINE_ROPE_COIL: u16 = 475;
pub(crate) const SILK_ROPE_COIL: u16 = 505;
pub(crate) const WEB_ROPE_COIL: u16 = 506;

/// The `DrawProjDirect` chain-draw gate (`Main.cs:28499`/`:28549`/`:28610`).
#[inline]
pub(crate) fn is_coil(kind: u16) -> bool {
    matches!(kind, ROPE_COIL | VINE_ROPE_COIL | SILK_ROPE_COIL | WEB_ROPE_COIL)
}

/// `Kill()`'s coil refund (`Projectile.cs:77572-77615`): an unplaced coil
/// (`ai[1] == 0`) drops ONE coil item back; a partially placed one
/// (`ai[1] < 10`) drops `10 - ai[1]` ROPE items (the placed links stay in
/// the world). At `ai[1] >= 10` nothing drops. Returns `(item, count)`.
/// Ported verbatim from g82's `coil_death_drop` (`67c9a740`).
pub(crate) fn coil_death_drop(kind: u16, placed: f32) -> Option<(u16, i32)> {
    let (coil_item, rope_item) = match kind {
        ROPE_COIL => (985u16, 965u16),   // :77576/:77580
        VINE_ROPE_COIL => (3005, 2996), // :77587/:77591
        SILK_ROPE_COIL => (3079, 3077), // :77598/:77602
        WEB_ROPE_COIL => (3080, 3078),  // :77609/:77613
        _ => return None,
    };
    if placed == 0.0 {
        Some((coil_item, 1))
    } else if placed < 10.0 {
        Some((rope_item, (10.0 - placed) as i32))
/// The `SetDefaults` row shared by the four coils
/// (`Projectile.cs:2570-2578` for 171/505/506, `:5496-5504` for 475):
/// `(w, h, ai_style, penetrate, time_left, tile_collide)`. The stock
/// `extraUpdates = 0`, `alpha = 0`, `scale = 1` stand (`:779`/`:785`).
pub(crate) fn coil_proj_row(kind: u16) -> Option<(f32, f32, u16, i32, i32, bool)> {
    if is_coil(kind) {
        Some((14.0, 14.0, 35, -1, 400, false))
    } else {
        None
    }
}

/// The rope kind the coil places (`Projectile.cs:27227-27239`): the
/// default 213, then the per-type overrides.
#[inline]
pub(crate) fn coil_rope_kind(kind: u16) -> u16 {
    match kind {
        VINE_ROPE_COIL => 353,
        WEB_ROPE_COIL => 366,
        SILK_ROPE_COIL => 365,
        _ => 213,
    }
}

/// `WorldGen.IsRope(x, y)` — the 2-arg overload (`WorldGen.cs:70074-70079`):
/// a rope cell is true; a platform/track/planter needs rope on both ends.
/// The render-side twin lives in `render/ropes.rs::is_rope`; this copy
/// keeps the module free of the render dependency (the d56 inlined-table
/// convention).
fn is_rope_cell(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !t.active() {
        return false;
    }
    if td::TILE_ROPE.has(t.kind) {
        return true;
    }
    if t.kind != 314 && t.kind != 380 && !td::SET_PLATFORMS.has(t.kind) {
        return false;
    }
    // GetRopeEnds (WorldGen.cs:70026-70072), rangeToCheck 5,
    // treatEmptyAsTheRopeEnd false — both ends must be found.
    let mut top = -1;
    let mut i = 1;
    while i <= 5 {
        let Some(u) = world.tile(x, y - i) else { break };
        if !u.active() {
            break;
        }
        if td::TILE_ROPE.has(u.kind) {
            top = y - i;
            break;
        }
        i += 1;
    }
    if top == -1 {
        return false;
    }
    let mut j = 1 + (y - top);
    while j <= 6 {
        let Some(d) = world.tile(x, top + j) else { break };
        if !d.active() {
            break;
        }
        if td::TILE_ROPE.has(d.kind) {
            return true;
        }
        j += 1;
    }
    false
}

/// `AI()`'s aiStyle-35 arm (`Projectile.cs:27189-27291`), verbatim. The
/// arm owns its integration: `tileCollide = false` means `HandleMovement`
/// free-moves (`position += velocity`, the `kill_step` no-collide path);
/// on contact the C# adds the COLLISION-ADJUSTED velocity (`:27242`) and
/// runs the placement walk, then `Kill()` (`:27290`). Returns the
/// `KillTile` records the cut-tile clears produced (`:27268`) — the
/// wired dispatch DISCARDS them (the `ai_ice_block`/`ai_dungeon_debris`
/// `let _ = kill_tile` precedent: the cascade runs, the per-cell drops
/// stay unspawned — recorded in ANALYSIS).
///
/// RNG: the arm draws ZERO `Main.rand` — the dust loops that share the
/// geometry live in `Kill()` (`:74615+`/`:74679+`), not the AI.
pub(crate) fn coil_step(p: &mut Projectile, world: &mut World, local_owner: u8) -> Vec<TileKill> {
    let mut kills = Vec::new();
    // The `SetDefaults` reset's `friendly = false` (`:788` — neither coil
    // arm sets it): the port's Projectile Default carries true, so the
    // wired dispatch pins it here every tick (the `ai_dungeon_debris`
    // precedent). `hostile` stays false (`:764`) — neither damage pass
    // claims a coil.
    p.friendly = false;
    // :27191-27200 — the age counter, then the 30-tick gravity gate.
    p.ai[0] += 1.0;
    if p.ai[0] > 30.0 {
        p.vel.y += 0.2;
        p.vel.x *= 0.985;
        if p.vel.y > 14.0 {
            p.vel.y = 14.0;
        }
    }
    // :27201 — the spin rides |vx|+|vy| and the facing.
    p.rot += (p.vel.x.abs() + p.vel.y.abs()) * p.direction as f32 * 0.02;
    // :27202-27205 — only the owner's machine places rope. The port's
    // free-move stands in for HandleMovement on this early return too
    // (the C# returns from AI only; HandleMovement still runs, :17015).
    if p.owner != local_owner {
        p.pos += p.vel;
        return kills;
    }
    // :27206-27222 — the collision probe; a bottom-sloped tile at the
    // destination centre snaps the coil onto the slope's underside.
    let tc = crate::collision::tile_collision(
        world,
        [p.pos.x, p.pos.y],
        [p.vel.x, p.vel.y],
        p.w,
        p.h,
        true,
        true,
    );
    let adjusted = Vec2::new(tc.vel[0], tc.vel[1]);
    let mut contacted = adjusted != p.vel;
    if !contacted {
        let cx = (p.pos.x + p.w * 0.5 + p.vel.x) as i32 / TILE_PX as i32;
        let cy = (p.pos.y + p.h * 0.5 + p.vel.y) as i32 / TILE_PX as i32;
        if let Some(t) = world.tile(cx, cy) {
            if t.active() && t.bottom_slope() {
                contacted = true;
                p.pos.y = (cy * 16 + 16 + 8) as f32;
                p.pos.x = (cx * 16 + 8) as f32;
            }
        }
    }
    if !contacted {
        // :27223-27226 — the free move (HandleMovement, tileCollide false).
        p.pos += p.vel;
        return kills;
    }
    // :27227-27242 — the rope kind, the placement column, the adjusted move.
    let rope_kind = coil_rope_kind(p.kind);
    // `base.width / 2` is INTEGER division in the C# (14/2 = 7).
    let tx = (p.pos.x + (p.w as i32 / 2) as f32) as i32 / TILE_PX as i32;
    let mut ty = (p.pos.y + (p.h as i32 / 2) as f32) as i32 / TILE_PX as i32;
    p.pos += adjusted;
    let mut budget = 10;
    // :27244-27247 — a null tile aborts WITHOUT Kill (the coil keeps flying).
    // The C# returns from AI only — HandleMovement still runs (`:17015` →
    // `UpdatePosition` `:20445` integrates the dead/alive slot alike; 35 is
    // not on the `:20435` skip list), so the free integrate runs on this
    // abort too (the final-phase fix — d4's arm carried it, the port's
    // first landing missed it; the 26e-f final union doc, delta 5).
    if world.tile(tx, ty).is_none() {
        p.pos += p.vel;
        return kills;
    }
    // :27248-27250 — skip down through the existing rope column.
    while is_rope_cell(world, tx, ty) {
        ty += 1;
    }
    // :27251-27289 — the 10-rope placement walk.
    let mut alternator = false;
    while budget > 0 {
        let mut placed_here = false;
        let Some(t) = world.tile(tx, ty).copied() else {
            break;
        };
        if t.active() {
            if t.kind == 314 || td::SET_PLATFORMS.has(t.kind) {
                // :27261-27264 — platforms/track flip the alternator; they
                // do NOT consume budget (the rope passes through them).
                alternator = !alternator;
            } else if tr_core::tileobject::tile_cut(t.kind) || t.kind == 165 {
                // :27265-27270 — cut tiles (and the guide dummy 165) are
                // cleared; the re-read below places rope in their cell.
                alternator = false;
                if let Some(kill) = tr_core::edit::kill_tile(world, tx, ty, false) {
                    kills.push(kill);
                }
            }
        }
        let active_now = world.tile(tx, ty).is_some_and(|t| t.active());
        if !active_now {
            // :27272-27279 — place one rope link, bump the placed counter.
            alternator = false;
            placed_here = true;
            tr_core::edit::place_tile(world, tx, ty, rope_kind);
            p.ai[1] += 1.0;
        } else if !alternator {
            // :27280-27283 — a solid non-cut cell ends the walk.
            budget = 0;
        }
        if placed_here {
            budget -= 1;
        }
        ty += 1;
    }
    // :27290 — Kill(). The refund rides coil_death_drop at the hook drain.
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
    // The dead-tick integrate: C# `Update` calls HandleMovement →
    // `UpdatePosition` (`:20445` `position += velocity`) BEFORE the
    // `!active` return (`:17027`), so the placement tick moves the dead
    // slot too — on top of the collision-adjusted step above, faithful
    // (the ai_trap_spear precedent; the 26e-f final union doc, delta 8).
    p.pos += p.vel;
    kills
}

/// One link of the trailing chain: world position + rotation + which
/// sheet to draw. `sheet` indexes [`coil_chain_sheets`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CoilLink {
    pub pos: [f32; 2],
    pub rot: f32,
    pub sheet: usize,
}

/// The chain sheets per coil, in draw-index order (`Main.cs:28546` /
/// `:28601-28605` / `:28660-28666`):
/// * 171 → `Chain16` (one sheet, index 0).
/// * 475 → `Chain38` (even draw index) / `Chain39` (odd) — indices 0/1.
/// * 505 → `Chains[4 + i%2]` = `Chains_4`/`Chains_5` — indices 0/1.
/// * 506 → `Chains[6 + i%2]` = `Chains_6`/`Chains_7` — indices 0/1.
/// The asset names the wiring step adds to the `assets.rs` static list.
#[allow(dead_code)] // the draw wiring (g82b) resolves the names
pub(crate) fn coil_chain_sheets(kind: u16) -> &'static [&'static str] {
    match kind {
        VINE_ROPE_COIL => &["Chain38", "Chain39"],
        SILK_ROPE_COIL => &["Chains_4", "Chains_5"],
        WEB_ROPE_COIL => &["Chains_6", "Chains_7"],
        _ => &["Chain16"],
    }
}

/// `Main.DrawProjDirect`'s coil chain walk (`Main.cs:28499-28547` for 171,
/// `:28549-28608` for 475, `:28610-28668` for 505/506 — the three bodies
/// are term-for-term identical bar the sheet pick), verbatim:
/// * the link count `(int)(30f * min(ai[0]/17, 1))` (`:28505-28509`);
/// * the sag step `0.4f * min(ai[0]/30, 1)` accumulating per link
///   (`:28510-28517`/`:28539-28540`);
/// * the 5.6-px normalize with the manhattan-<1 shrink quirk
///   (`:28522-28527` — `num52 *= |dx| + |dy| / 1f`, the `/1f` a no-op);
///   at a fully stopped coil (`vel == 0`, `ai[0] == 0`) the step and the
///   length are both 0 — `0/0` NaN propagates through every link, the
///   C#'s own quirk, kept (g82's NaN pin);
/// * the rotation `atan2(dy, dx) - 1.57f` (`:28531`);
/// * the REVERSE draw order (`:28542` — `for (num54 = --num47; num54 >= 0;
///   num54--)`), so the far end draws first and the coil-adjacent link
///   last; the sheet alternation counts the DRAW index from 0 at the far
///   end (`:28592`/`:28653`), not the walk index.
/// The geometry is shared with the skipped dust walks (`Projectile.cs:
/// 74615+`/`:74679+`) — same constants, same order.
pub(crate) fn coil_chain_segments(p: &Projectile) -> Vec<CoilLink> {
    let sheets = coil_chain_sheets(p.kind);
    let mut center = [p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5];
    let mut dx = 0.0 - p.vel.x;
    let mut dy = 0.0 - p.vel.y;
    // :28504-28509 — the count ramp over the first 17 ticks.
    let ramp = if p.ai[0] <= 17.0 { p.ai[0] / 17.0 } else { 1.0 };
    let count = (30.0f32 * ramp) as usize;
    // :28510-28517 — the sag ramp over the first 30 ticks.
    let sag_ramp = if p.ai[0] <= 30.0 { p.ai[0] / 30.0 } else { 1.0 };
    let sag_step = 0.4f32 * sag_ramp;
    let mut sag = sag_step;
    dy += sag;
    let mut walk = Vec::with_capacity(count);
    for _ in 0..count {
        // :28522-28530 — the normalize + the manhattan shrink quirk.
        let len = (dx * dx + dy * dy).sqrt();
        let mut step = 5.6f32;
        if dx.abs() + dy.abs() < 1.0 {
            step *= dx.abs() + dy.abs();
        }
        let k = step / len;
        dx *= k;
        dy *= k;
        // :28531-28534 — record BEFORE the advance.
        let rot = dy.atan2(dx) - 1.57;
        walk.push((center, rot));
        // :28535-28540 — advance, reset, accumulate the sag.
        center = [center[0] + dx, center[1] + dy];
        dx = 0.0 - p.vel.x;
        dy = 0.0 - p.vel.y;
        sag += sag_step;
        dy += sag;
    }
    // :28542-28547 / :28593-28607 / :28654-28667 — the reverse emit.
    let mut links = Vec::with_capacity(count);
    for (draw_i, &(pos, rot)) in walk.iter().rev().enumerate() {
        let sheet = if sheets.len() == 1 { 0 } else { draw_i % 2 };
        links.push(CoilLink { pos, rot, sheet });
    }
    links
}

/// The chain draw hook (`Main.cs:28546`/`:28601-28605`/`:28666`): one
/// rotated quad per link, scale 0.8, origin = half the sheet, tinted by
/// `Lighting.GetColor((int)x/16, (int)(y/16f))` at the LINK's position
/// (not the coil's). `sheet_uv` resolves the packed sheet by name (the
/// wiring step's `assets.rs` addition); returns the quad count.
/// UNWIRED — the g82b draw lane calls it from `draw_with_hand` (module
/// doc); the witnesses exercise the geometry, not the emit.
#[allow(dead_code)] // the draw wiring (g82b) consumes the emit half
pub(crate) fn draw_coil_chain(
    p: &Projectile,
    r: &mut tr_render::SpriteRenderer,
    light: &tr_core::lighting::LightMap,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    sheet_uv: impl Fn(&str) -> Option<([f32; 4], f32, f32)>,
) -> u32 {
    let sheets = coil_chain_sheets(p.kind);
    let mut quads = 0;
    for link in coil_chain_segments(p) {
        let Some((uv, w, h)) = sheet_uv(sheets[link.sheet]) else {
            continue;
        };
        let tx = link.pos[0] as i32 / TILE_PX as i32;
        let ty = link.pos[1] as i32 / TILE_PX as i32;
        let c = if light.w > 0 && light.h > 0 {
            light.get(
                tx.clamp(light.x0, light.x0 + light.w - 1),
                ty.clamp(light.y0, light.y0 + light.h - 1),
            )
        } else {
            [1.0; 3]
        };
        let snapped = crate::render::entities::snap_to_screen_pub(link.pos, camera, zoom, screen);
        // EntitySpriteDraw(scale: 0.8f, origin: (W*0.5, H*0.5)).
        r.push_sprite_origin_rot(
            snapped,
            [w * 0.8 * zoom, h * 0.8 * zoom],
            uv,
            [c[0], c[1], c[2], 1.0],
            link.rot,
            [w * 0.5 * zoom, h * 0.5 * zoom],
        );
        quads += 1;
    }
    quads
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proj(kind: u16) -> Projectile {
        Projectile {
            active: true,
            kind,
            owner: 0,
            w: 14.0,
            h: 14.0,
            ..Default::default()
        }
    }

    // Projectile.cs:77572-77615 — the refund ladder (g82's witness,
    // re-homed at the union). WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_d4_union_the_death_refund_ladder() {
        assert_eq!(coil_death_drop(171, 0.0), Some((985, 1)));
        assert_eq!(coil_death_drop(171, 3.0), Some((965, 7)));
        assert_eq!(coil_death_drop(171, 9.0), Some((965, 1)));
        assert_eq!(coil_death_drop(171, 10.0), None);
        assert_eq!(coil_death_drop(475, 0.0), Some((3005, 1)));
        assert_eq!(coil_death_drop(475, 4.0), Some((2996, 6)));
        assert_eq!(coil_death_drop(505, 0.0), Some((3079, 1)));
        assert_eq!(coil_death_drop(505, 2.0), Some((3077, 8)));
        assert_eq!(coil_death_drop(506, 0.0), Some((3080, 1)));
        assert_eq!(coil_death_drop(506, 5.0), Some((3078, 5)));
        assert_eq!(coil_death_drop(172, 0.0), None);
        assert!(is_coil(171) && is_coil(475) && is_coil(505) && is_coil(506));
        assert!(!is_coil(214) && !is_coil(155));
    }

    // Main.cs:28501-28547 — the reverse draw order + the ramps.
    #[test]
    fn w4_d4_union_the_chain_walks_backward_in_reverse_draw_order() {
/// `Kill()`'s coil refund (`Projectile.cs:77572-77615`): an unplaced coil
/// (`ai[1] == 0`) drops ONE coil item back; a partially placed one
/// (`ai[1] < 10`) drops `10 - ai[1]` ROPE items (the placed links stay in
/// the world). At `ai[1] >= 10` nothing drops. Returns `(item, count)`.
pub(crate) fn coil_death_drop(kind: u16, placed: f32) -> Option<(u16, i32)> {
    let (coil_item, rope_item) = match kind {
        ROPE_COIL => (985u16, 965u16),   // :77576/:77580
        VINE_ROPE_COIL => (3005, 2996), // :77587/:77591
        SILK_ROPE_COIL => (3079, 3077), // :77598/:77602
        WEB_ROPE_COIL => (3080, 3078),  // :77609/:77613
        _ => return None,
    };
    if placed == 0.0 {
        Some((coil_item, 1))
    } else if placed < 10.0 {
        Some((rope_item, (10.0 - placed) as i32))
    } else {
        None
    }
}

/// `Player.PlaceThing_Tiles_CheckRopeUsability` (`Player.cs:41506-41545`),
/// verbatim: when the held item places a rope AND the target is already a
/// rope, walk DOWN the column (through ropes, track 314, platforms and
/// planters 380) to find the first free cell — at most 5 consecutive
/// non-rope cells (the `num2 > num` break, `:41520-41523`), reset by every
/// rope cell. The chain (214) ignores lava (`flag`, `:41514`); every other
/// rope stops above a lava cell (`:41515`). The target moves only when the
/// landing cell is free or cut/breakable (`:41540`). Returns
/// `(can_use, new_target_y)`; `IsTileLoaded` is always true in the port
/// (the documented deviation).
pub(crate) fn check_rope_usability(
    world: &World,
    create_tile: u16,
    can_use: bool,
    target_x: i32,
    target_y: i32,
) -> (bool, i32) {
    let mut y = target_y;
    let gate = can_use
        && td::TILE_ROPE.has(create_tile)
        && world
            .tile(target_x, target_y)
            .is_some_and(|t| t.active() && td::TILE_ROPE.has(t.kind));
    if !gate {
        return (can_use, y);
    }
    let max_non_rope = 5;
    let mut non_rope = 0;
    let ignores_lava = create_tile == 214;
    loop {
        let Some(t) = world.tile(target_x, y) else { break };
        let passable = t.active()
            && (td::TILE_ROPE.has(t.kind)
                || t.kind == 314
                || td::SET_PLATFORMS.has(t.kind)
                || t.kind == 380);
        if !passable || y >= world.height - 5 {
            break;
        }
        // :41515 — the lava gate reads the cell BELOW. `Tile.lava()` is
        // the header-bit test alone (`Tile.cs`, `(bTileHeader & 0x60) ==
        // 32`) — NO liquid-amount check.
        if !ignores_lava
            && world
                .tile(target_x, y + 1)
                .is_some_and(|b| b.liquid_kind() == tr_core::tile::LiquidKind::Lava)
        {
            break;
        }
        if t.kind == 314 || td::SET_PLATFORMS.has(t.kind) || t.kind == 380 {
            non_rope += 1;
            if non_rope > max_non_rope {
                break;
            }
        } else {
            non_rope = 0;
        }
        y += 1;
    }
    // :41539-41543 — the landing cell must be free or cut/breakable.
    let landing_ok = world.tile(target_x, y).is_some_and(|t| {
        !t.active()
            || tr_core::tileobject::tile_cut(t.kind)
            || tr_core::tileobject::breakable_when_placing(t.kind)
    });
    if landing_ok {
        (can_use, y)
    } else {
        (can_use, target_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{LiquidKind, FLAG_ACTIVE};

    fn proj(kind: u16) -> Projectile {
        let mut p = Projectile { kind, ..Default::default() };
        let (w, h, _ai, pen, life, tc) = coil_proj_row(kind).unwrap();
        p.active = true;
        p.w = w;
        p.h = h;
        p.penetrate = pen;
        p.max_penetrate = pen;
        p.time_left = life;
        p.tile_collide = tc;
        p.owner = 0;
        p.direction = 1;
        p
    }

    fn world() -> World {
        World::new("coil".into(), 64, 200)
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let t = w.tile_mut(x, y).unwrap();
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
    }

    // The SetDefaults rows + the rope-kind map (Projectile.cs:2570-2578,
    // :5496-5504, :27227-27239). WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_g82_the_four_rows_and_rope_kinds() {
        for k in [171u16, 475, 505, 506] {
            assert_eq!(coil_proj_row(k), Some((14.0, 14.0, 35, -1, 400, false)));
        }
        assert_eq!(coil_proj_row(172), None);
        assert_eq!(coil_rope_kind(171), 213);
        assert_eq!(coil_rope_kind(475), 353);
        assert_eq!(coil_rope_kind(505), 365);
        assert_eq!(coil_rope_kind(506), 366);
    }

    // :27191-27201 — the counter, the 30-tick gravity gate, the spin.
    #[test]
    fn w4_g82_the_flight_counter_gravity_and_spin() {
        let mut w = world();
        let mut p = proj(171);
        p.pos = Vec2::new(300.0, 300.0);
        p.vel = Vec2::new(10.0, 0.0);
        for _ in 0..30 {
            coil_step(&mut p, &mut w, 0);
        }
        assert_eq!(p.ai[0], 30.0);
        assert_eq!(p.vel.y, 0.0, "no gravity through tick 30");
        assert_eq!(p.vel.x, 10.0, "no decay through tick 30");
        coil_step(&mut p, &mut w, 0);
        assert_eq!(p.ai[0], 31.0);
        assert!((p.vel.y - 0.2).abs() < 1e-6, "tick 31 adds 0.2");
        assert!((p.vel.x - 9.85).abs() < 1e-6, "tick 31 decays by 0.985");
        // The vy cap 14 (:27196-27199).
        p.vel.y = 13.95;
        coil_step(&mut p, &mut w, 0);
        assert_eq!(p.vel.y, 14.0);
        // The spin: (|vx|+|vy|) * direction * 0.02 per tick (:27201).
        let mut q = proj(171);
        q.pos = Vec2::new(300.0, 300.0);
        q.vel = Vec2::new(3.0, 4.0);
        q.direction = -1;
        coil_step(&mut q, &mut w, 0);
        assert!((q.rot - (-0.14)).abs() < 1e-6, "(3+4)*-1*0.02");
    }

    // :27202-27205 — a non-owner coil flies but never places.
    #[test]
    fn w4_g82_the_owner_gate() {
        let mut w = world();
        put(&mut w, 20, 21, 1); // a solid floor under the flight path
        let mut p = proj(171);
        p.owner = 1;
        p.pos = Vec2::new(320.0 - 7.0, 320.0 - 7.0);
        p.vel = Vec2::new(0.0, 10.0);
        coil_step(&mut p, &mut w, 0);
        assert!(p.active, "the non-owner never runs the placement walk");
        assert_eq!(p.ai[1], 0.0);
    }

    // :27240-27290 — the coil lands on a solid floor and places one rope
    // link in the free cell just above it, then dies. Contact geometry:
    // pos.y 460 → box bottom 474, +vel 10 → 484 overlaps the floor top 480,
    // so TileCollision clamps vy to 6 (≠10 → contact). Centre (320,467) →
    // tile (20,29); the walk places there, row 30 is solid → budget 0.
    #[test]
    fn w4_g82_the_placement_walk_stops_at_solid_and_kills() {
        let mut w = world();
        for x in 18..24 {
            put(&mut w, x, 30, 1);
        }
        let mut p = proj(171);
        p.pos = Vec2::new(320.0 - 7.0, 460.0);
        p.vel = Vec2::new(0.0, 10.0);
        let kills = coil_step(&mut p, &mut w, 0);
        assert!(kills.is_empty());
        assert!(!p.active, "the Kill at :27290");
        assert_eq!(p.ai[1], 1.0, "one link placed above the floor");
        assert!(w.tile(20, 29).unwrap().active());
        assert_eq!(w.tile(20, 29).unwrap().kind, 213);
    }

    // :27261-27264 — platforms flip the alternator and pass through
    // without consuming budget (fall-through collision). Contact geometry:
    // pos.y 520.5 → box bottom 534.5, +vel 10 → 544.5 strictly overlaps the
    // row-34 floor top 544, so vy clamps to 9.5 (≠10 → contact); the centre
    // (320,527.5) lands on the row-32 PLATFORM, so the walk flips the
    // alternator there, places one vine link in the free row 33, and stops
    // at the row-34 floor.
    #[test]
    fn w4_g82_platforms_pass_through_without_budget() {
        let mut w = world();
        put(&mut w, 20, 31, 19); // platform
        put(&mut w, 20, 32, 19); // platform
        for x in 18..24 {
            put(&mut w, x, 34, 1);
        }
        let mut p = proj(475);
        p.pos = Vec2::new(320.0 - 7.0, 520.5);
        p.vel = Vec2::new(0.0, 10.0);
        coil_step(&mut p, &mut w, 0);
        assert!(!p.active);
        assert_eq!(p.ai[1], 1.0);
        assert_eq!(w.tile(20, 33).unwrap().kind, 353, "the vine kind");
        assert_eq!(w.tile(20, 32).unwrap().kind, 19, "platforms untouched");
    }

    // :27243/:27252/:27286 — the 10-link budget cap. The coil contacts a
    // wall on its right (x-only clamp) with a free shaft below its centre
    // column, so the walk places the full 10 links then stops on budget.
    #[test]
    fn w4_g82_the_walk_places_at_most_ten_links() {
        let mut w = world();
        // A wall at column 20 beside the flight path (rows 18..20).
        for y in 18..=20 {
            put(&mut w, 20, y, 1);
        }
        // The centre column 19 is a free shaft down to a floor at row 59.
        put(&mut w, 19, 59, 1);
        let mut p = proj(171);
        // Box 300..314, +vx 10 → 324 overlaps the wall's left face 320;
        // TileCollision clamps vx to 6 (and vy to 6 against the wall's
        // row-20 bottom edge — both ≠ the raw vel → contact). Centre
        // (307,307) → tile (19,19), the free shaft head.
        p.pos = Vec2::new(300.0, 300.0);
        p.vel = Vec2::new(10.0, 0.0);
        coil_step(&mut p, &mut w, 0);
        assert!(!p.active);
        assert_eq!(p.ai[1], 10.0, "the budget caps at 10 (:27243)");
        for y in 19..29 {
            assert_eq!(w.tile(19, y).unwrap().kind, 213, "link at row {y}");
        }
        assert!(!w.tile(19, 29).unwrap().active(), "the 11th cell stays free");
    }

    // The final-phase faithfulness fixes (26e-f deltas 5+8): the C# `AI()`
    // returns leave HandleMovement running (`:17015` → `UpdatePosition`
    // `:20445`, 35 not on the `:20435` skip list; the `!active` return sits
    // AFTER, `:17027`) — so BOTH the null-tile abort AND the placement kill
    // tick integrate the velocity.
    #[test]
    fn w4_g82_the_null_abort_and_the_kill_tick_still_integrate() {
        // The kill-tick integrate: the floor-contact rig — after the
        // adjusted step (+0, +6) the dead slot ALSO adds vel (0, 10).
        let mut w = world();
        for x in 18..24 {
            put(&mut w, x, 30, 1);
        }
        let mut p = proj(171);
        p.pos = Vec2::new(320.0 - 7.0, 460.0);
        p.vel = Vec2::new(0.0, 10.0);
        coil_step(&mut p, &mut w, 0);
        assert!(!p.active);
        assert!(
            (p.pos.y - (460.0 + 6.0 + 10.0)).abs() < 1e-4,
            "the adjusted +6 THEN the dead-tick +10 (`:20445`)"
        );
        // The null-tile abort: the ONLY reachable null-centre contact for
        // a 14px box is the bottomSlope snap arm (`:27216-27221`) — it
        // SETS pos.y = ty*16+24, pushing the centre a full cell BELOW the
        // sloped tile. Rig: a 32-row world (rows 0..31), a slope-4
        // (bottom-sloped) dirt cell at (20, 31) — the LAST row. The coil
        // at (313, 486), box (313..327, 486..500), vel (0, 5): the target
        // box overlaps the cell, but the slope-4 pass-through needs
        // pos.y ≥ ry (486 < 496 — false) and every clamp arm's rest-side
        // test fails (bottom 500 > ry 496, right 327 > rx 320, left 313
        // < rx+16 336, top 486 < ry+16 512) → adjusted == vel → the else
        // probe: centre+vel = (320, 498) → tile (20, 31), active +
        // bottom_slope → the snap: pos = (20*16+8, 31*16+16+8) =
        // (328, 520). The walk anchor reads the POST-snap centre
        // (335, 527) → tile (20, 32) — NULL (32 rows). The abort returns
        // WITHOUT Kill; the fix integrates the raw vel ON TOP of the
        // adjusted step the anchor block already applied.
        let mut w2 = World::new("t".into(), 64, 32);
        {
            let t = w2.tile_mut(20, 31).unwrap();
            t.kind = 1;
            t.flags |= FLAG_ACTIVE;
            t.set_slope(4); // bottomSlope (slope() 3|4)
        }
        let mut q = proj(171);
        q.pos = Vec2::new(313.0, 486.0);
        q.vel = Vec2::new(0.0, 5.0);
        let kills = coil_step(&mut q, &mut w2, 0);
        assert!(kills.is_empty());
        assert!(q.active, "the null abort never kills (`:27244-27247`)");
        assert_eq!(q.ai[1], 0.0);
        assert!(
            (q.pos.y - (520.0 + 5.0 + 5.0)).abs() < 1e-4 && (q.pos.x - 328.0).abs() < 1e-4,
            "snap 520 + adjusted 5 + the abort's raw-vel 5 (HandleMovement, `:17015`)"
        );
    }

    // Main.cs:28504-28547 — the chain geometry: count ramp, sag, reverse
    // order, the 5.6 normalize. Hand-derived for vel (0, 14), ai0 = 30:
    // count 30, sag_step 0.4; link k's dy = -14 + 0.4*(k+1) stays negative
    // through k=29 (-2), so every link normalizes to 5.6 UP the -vel axis
    // (the manhattan-<1 shrink quirk cancels for axis-aligned steps:
    // step = 5.6*m, len = m → k = 5.6). rot = atan2(-5.6, 0) - 1.57.
    // NOTE: vel (0, 10) would hit dy == 0 at k=24 → 0/0 NaN in BOTH the
    // C# and the port (the quirk's `num52 *= 0` then `0/0`); the witness
    // avoids the degenerate axis on purpose.
    #[test]
    fn w4_g82_the_chain_walks_backward_in_reverse_draw_order() {
        let mut p = proj(171);
        p.pos = Vec2::new(100.0, 100.0);
        p.vel = Vec2::new(0.0, 14.0);
        p.ai[0] = 30.0;
        let links = coil_chain_segments(&p);
        assert_eq!(links.len(), 30, "(int)(30 * 30/17 clamped 1)");
        // The FIRST drawn link is the FAR end (walk index 29): the coil
        // centre minus 29 * 5.6 on Y (every step normalizes to 5.6 up).
        // centre minus 30 * 5.6 on Y (every step normalizes to 5.6 up).
        let far = links[0];
        assert!((far.pos[0] - 107.0).abs() < 1e-3, "centre x = 100+7");
        assert!((far.pos[1] - (107.0 - 29.0 * 5.6)).abs() < 1e-3);
        // The LAST drawn link is the coil centre itself (walk index 0).
        let near = links[29];
        assert!((near.pos[0] - 107.0).abs() < 1e-3);
        assert!((near.pos[1] - 107.0).abs() < 1e-3);
        // Every link rotates atan2(-5.6, 0) - 1.57 = -PI/2 - 1.57.
        let want = (-5.6f32).atan2(0.0) - 1.57;
        for l in &links {
            assert!((l.rot - want).abs() < 1e-5);
            assert_eq!(l.sheet, 0, "171 draws Chain16 only");
        }
        // The count ramp: ai0 = 0 → no links; ai0 = 8 → (int)(30 * 8/17)
        // = (int)14.117 = 14.
        // The count ramp: ai0 = 0 → no links; ai0 = 17 → 30; ai0 = 8 →
        // (int)(30 * 8/17) = (int)14.117 = 14.
        p.ai[0] = 0.0;
        assert_eq!(coil_chain_segments(&p).len(), 0);
        p.ai[0] = 8.0;
        assert_eq!(coil_chain_segments(&p).len(), 14);
    }

    // Main.cs:28592-28607/:28653-28667 — the sheet alternation counts the
    // DRAW index (0 at the far end), not the walk index.
    #[test]
    fn w4_d4_union_the_sheet_alternation_counts_the_draw_index() {
    fn w4_g82_the_sheet_alternation_counts_the_draw_index() {
        let mut p = proj(475);
        p.pos = Vec2::new(100.0, 100.0);
        p.vel = Vec2::new(0.0, 14.0);
        p.ai[0] = 30.0;
        let links = coil_chain_segments(&p);
        assert_eq!(links[0].sheet, 0, "draw index 0 → Chain38");
        assert_eq!(links[1].sheet, 1, "draw index 1 → Chain39");
        assert_eq!(links[29].sheet, 1, "draw index 29 odd → Chain39");
        assert_eq!(coil_chain_sheets(505), &["Chains_4", "Chains_5"]);
        assert_eq!(coil_chain_sheets(506), &["Chains_6", "Chains_7"]);
        // 506's num79 = 6 + i%2 (Main.cs:28660-28665) — the same parity.
        let mut q = proj(506);
        q.pos = Vec2::new(100.0, 100.0);
        q.vel = Vec2::new(0.0, 14.0);
        q.ai[0] = 30.0;
        let lq = coil_chain_segments(&q);
        assert_eq!((lq[0].sheet, lq[1].sheet), (0, 1));
    }

    // Main.cs:28522-28528 — the manhattan-<1 shrink quirk incl. the kept
    // 0/0 NaN pin: when -velocity exactly cancels the first sag step
    // (vel = (0, 0.4) at ai[0] >= 30), dx = dy = 0 → step 5.6*0 = 0,
    // len 0 → k = 0/0 NaN propagates through every later link (the C#
    // `num52 / num51` divides the same zero pair), never panics.
    #[test]
    fn w4_d4_union_the_cancelled_sag_keeps_the_nan_quirk() {
        let mut p = proj(171);
        p.pos = Vec2::new(100.0, 100.0);
        p.vel = Vec2::new(0.0, 0.4);
        p.ai[0] = 30.0;
        let links = coil_chain_segments(&p);
        assert_eq!(links.len(), 30);
        // The near link (walk index 0) records the pre-advance centre, but
        // its rotation is computed AFTER the 0/0 normalize (`:28528-28531`)
        // — atan2(NaN, NaN) - 1.57 is already NaN on the first link.
        let near = links[29];
        assert!((near.pos[0] - 107.0).abs() < 1e-3);
        assert!((near.pos[1] - 107.0).abs() < 1e-3);
        assert!(near.rot.is_nan(), "the rot reads the post-normalize pair");
        // Every later link walked through 0/0 — positions and rotations
        // NaN, the C# array carries the same.
        assert!(links[0].pos[0].is_nan(), "the far end walked through 0/0");
        assert!(links[0].rot.is_nan(), "atan2(NaN, NaN) - 1.57 stays NaN");
    // Projectile.cs:77572-77615 — the refund ladder.
    #[test]
    fn w4_g82_the_death_refund_ladder() {
        assert_eq!(coil_death_drop(171, 0.0), Some((985, 1)));
        assert_eq!(coil_death_drop(171, 3.0), Some((965, 7)));
        assert_eq!(coil_death_drop(171, 9.0), Some((965, 1)));
        assert_eq!(coil_death_drop(171, 10.0), None);
        assert_eq!(coil_death_drop(475, 0.0), Some((3005, 1)));
        assert_eq!(coil_death_drop(475, 4.0), Some((2996, 6)));
        assert_eq!(coil_death_drop(505, 0.0), Some((3079, 1)));
        assert_eq!(coil_death_drop(505, 2.0), Some((3077, 8)));
        assert_eq!(coil_death_drop(506, 0.0), Some((3080, 1)));
        assert_eq!(coil_death_drop(506, 5.0), Some((3078, 5)));
        assert_eq!(coil_death_drop(172, 0.0), None);
    }

    // Player.cs:41506-41545 — the usability walk.
    #[test]
    fn w4_g82_the_usability_walk_descends_the_rope_column() {
        let mut w = world();
        put(&mut w, 20, 20, 213);
        put(&mut w, 20, 21, 213);
        // Free cell at 22 → the target moves there.
        let (ok, y) = check_rope_usability(&w, 213, true, 20, 20);
        assert!(ok);
        assert_eq!(y, 22);
        // A non-rope held item never walks.
        assert_eq!(check_rope_usability(&w, 1, true, 20, 20), (true, 20));
        // A non-rope target never walks.
        let mut w2 = world();
        put(&mut w2, 20, 20, 1);
        assert_eq!(check_rope_usability(&w2, 213, true, 20, 20), (true, 20));
    }

    // :41515-41523 — the 5-platform cap and the lava gate (chain exempt).
    #[test]
    fn w4_g82_the_platform_cap_and_the_lava_gate() {
        let mut w = world();
        put(&mut w, 20, 20, 213);
        for y in 21..=26 {
            put(&mut w, 20, y, 19); // six platforms
        }
        // The 6th platform trips num2 > 5 → the walk stops AT it; the
        // landing cell (a platform) is not free/cut → target unchanged.
        let (_, y) = check_rope_usability(&w, 213, true, 20, 20);
        assert_eq!(y, 20, "the capped walk never retargets onto a platform");
        // Lava below the next cell stops a rope walk…
        let mut w = world();
        put(&mut w, 20, 20, 213);
        {
            let t = w.tile_mut(20, 21).unwrap();
            t.liquid = 255;
            t.set_liquid_kind(LiquidKind::Lava);
        }
        let (_, y) = check_rope_usability(&w, 213, true, 20, 20);
        assert_eq!(y, 20, "the lava gate holds the target");
        // …but the chain (214) ignores lava and walks through to the free
        // cell at 21 (the lava cell itself is inactive → landing ok).
        let (_, y) = check_rope_usability(&w, 214, true, 20, 20);
        assert_eq!(y, 21);
    }

    // W4-f3 compose — the end-to-end place→attach→ride→coil-refund chain
    // (origin/task/W4-f3-dust-player-3 0b11548f's pulley machine attaches
    // to the SAME `td::TILE_ROPE` set this module's placement writes).
    // This branch carries the placement + refund legs LIVE; the attach +
    // ride legs are pinned here via the EXACT predicates f3's machine
    // reads (`find_pulley`'s `is_rope` gate :19652 + the lone-cell
    // `tile_rope_kind` neighbours :19665, `can_move_forward_on_rope`
    // :24250) — the f3 player fns themselves live on the f3 branch; the
    // merged tree runs the full chain (the 26e-f union doc's compose
    // section). WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_g82_f3_compose_the_placed_column_attaches_and_refunds() {
        // LEG 1 — PLACE: the coil lands on the row-30 floor and writes a
        // 213 link at (20, 29) (the placement-walk rig).
        let mut w = world();
        for x in 18..24 {
            put(&mut w, x, 30, 1);
        }
        let mut p = proj(171);
        p.pos = Vec2::new(320.0 - 7.0, 460.0);
        p.vel = Vec2::new(0.0, 10.0);
        coil_step(&mut p, &mut w, 0);
        assert!(!p.active);
        assert_eq!(p.ai[1], 1.0);
        assert_eq!(w.tile(20, 29).unwrap().kind, 213);
        // LEG 2 — ATTACH predicates: a player hanging at the link's row.
        // f3 `find_pulley` probes ((x + width/2)/16, (y - 8)/16): seat the
        // player centre on column 20, y = 29*16 + 8 → probe row 29 — the
        // placed cell. The `is_rope` gate answers true (a TILE_ROPE kind,
        // the early return, no column probe needed).
        assert!(td::TILE_ROPE.has(w.tile(20, 29).unwrap().kind));
        assert!(crate::render::ropes::is_rope(&w, 20, 29).0, "the :19652 gate");
        // The lone-cell arm (:19656-19668): one link, no rope-kind
        // neighbour above/below → the row*16+22 hang snap applies.
        fn rope_nbr(w: &World, x: i32, y: i32) -> bool {
            w.tile(x, y)
                .is_some_and(|t| t.active() && td::TILE_ROPE.has(t.kind))
        }
        assert!(!rope_nbr(&w, 20, 28) && !rope_nbr(&w, 20, 30), "a lone cell");
        // LEG 3 — RIDE predicate: the placed kind is non-solid (the
        // attach's solid_collision probe passes; f3 0b11548f verified the
        // rope kinds non-solid) and a second link below extends the
        // climbable column — place one by hand the way the usability walk
        // would, then the descent probe reads rope at the next row.
        assert!(!td::TILE_SOLID.has(213), "the hang box never collides");
        put(&mut w, 20, 30, 213); // over the floor cell — a rope column
        assert!(rope_nbr(&w, 20, 30), "the ride's next-row probe (:24250 shape)");
        // LEG 4 — COIL-REFUND: the same coil's death ladder pays back
        // 10-ai1 = 9 rope items (the world drop the pool hook queues).
        assert_eq!(coil_death_drop(p.kind, p.ai[1]), Some((965, 9)));
        // And an UNPLACED coil (ai1 == 0) refunds the coil item itself —
        // the throw that never lands costs nothing.
        assert_eq!(coil_death_drop(171, 0.0), Some((985, 1)));
    }
}
