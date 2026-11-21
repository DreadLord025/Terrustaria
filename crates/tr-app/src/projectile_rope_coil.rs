//! W4-d4 × W4-g82 union — the rope-coil lane's EXTRAS (research/26e's
//! keep-one verdict: d4's `projectile.rs::ai_rope_coil` + the four
//! `projectile/data.rs` rows + wind-list 35 stay the single aiStyle-35
//! machine; g82's `coil_step`/`coil_proj_row`/`coil_rope_kind`/`is_rope_cell`
//! are the duplicate half and are NOT landed here). This module keeps the
//! pieces d4 never claimed, ported from g82's `projectile_rope_coil.rs`
//! (`67c9a740`):
//!
//! * [`coil_death_drop`] — `Kill()`'s refund ladder
//!   (`Projectile.cs:77572-77615`). WIRED: the pool's once-per-death hook
//!   (`projectile.rs`, the bobber-arm seam) queues `(item, stack, centre)`
//!   into `pending_death_drops`; the world sim drains it at the pass tail
//!   (`world_sim/projectiles.rs`) as an `Item.NewItem` world drop on the
//!   `UpdateItems` stream. d4's coil killed with NO refund — the one
//!   behavioural gap the 26e comparison found, closed here.
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
//! `ai[1] == 0` sound cue (`:74617-74620`/`:74681-74684`) rides the draw
//! wiring's `wants_sound`; the refund drop's `ReserveForLocalPlayer`
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
/// The g82b draw wiring gates `draw_with_hand` on it (module doc).
#[allow(dead_code)] // the draw wiring (g82b) consumes the gate
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
    } else {
        None
    }
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
        let mut p = proj(171);
        p.pos = Vec2::new(100.0, 100.0);
        p.vel = Vec2::new(0.0, 14.0);
        p.ai[0] = 30.0;
        let links = coil_chain_segments(&p);
        assert_eq!(links.len(), 30, "(int)(30 * 30/17 clamped 1)");
        // The FIRST drawn link is the FAR end (walk index 29): the coil
        // centre minus 29 * 5.6 on Y (every step normalizes to 5.6 up).
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
        p.ai[0] = 0.0;
        assert_eq!(coil_chain_segments(&p).len(), 0);
        p.ai[0] = 8.0;
        assert_eq!(coil_chain_segments(&p).len(), 14);
    }

    // Main.cs:28592-28607/:28653-28667 — the sheet alternation counts the
    // DRAW index (0 at the far end), not the walk index.
    #[test]
    fn w4_d4_union_the_sheet_alternation_counts_the_draw_index() {
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
    }
}
