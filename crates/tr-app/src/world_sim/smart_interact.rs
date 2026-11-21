//! The smart-interact tile-targeting lookup — `Player.SmartInteractLookup`
//! (`Player.cs:17374-17398`) + `SmartInteractSystem.RunQuery`
//! (`GameContent/ObjectInteractions/SmartInteractSystem.cs:31-61`) + the
//! `TileSmartInteractCandidateProvider`
//! (`TileSmartInteractCandidateProvider.cs:63-480`), the feed of
//! `Main.SmartInteractTileCoords`/`…Selected` (`Main.cs:2035-2037`) that
//! `render/tile_outlines.rs`'s highlight-area gate reads.
//!
//! W4-g86 wiring seam 1. The landed smart-cursor lane
//! (`tr_core::smart_cursor`, `SmartCursorHelper.SmartCursorLookup`) is the
//! TOOL lookup (`Main.SmartCursorX/Y` re-point); this module is the OTHER
//! half of the `Player.cs:25446-25447` pair — the interactable-object scan
//! that fills the outline lists. It runs right after the smart-cursor
//! re-point in `world_sim/editing.rs` (the port's `Player.Update` seam).
//!
//! ## Census (what feeds vs what outlines)
//!
//! `FillPotentialTargetTiles` (`:359-480`) admits: campfires (215),
//! `Recipe.TileUsedInRecipes` (the recipe stations incl. the altar 26 and
//! the `TileCountsAs`-inherited 77/133/134/302/304/355/699/17/96),
//! `CountsAsWaterForCrafting` (172/207), the 57-id literal switch
//! (`:385-444`) and the six conditional ids (chairs 15 / toilets 497 within
//! 40 px of the player's feet, LihzahrdAltar 237 with item 1293,
//! SnowballLauncher 212 with item 949, Sundial 356 / Moondial 663 off
//! cooldown). `ProvideCandidate` (`:76-357`) then sizes each target through
//! the big type switch (`:115-289`), anchors it at its top-left cell and
//! fills the point lists. Every one of the 95 `HasOutlines` ids
//! (`TileID.cs:551`) has BOTH a feed arm and a size arm — the census found
//! no orphan outline tile; the five sized-but-not-outlined ids (14 Tables,
//! 469 Tables2, 491 VoidVault, 573 TatteredWoodSign, 597 TeleportationPylon)
//! feed the interact machinery but draw no outline (491/597 have their own
//! selection glow in `render/pylons.rs`).
//!
//! ## The provider chain + the vote (re-verified with all providers live)
//!
//! `SmartInteractSystem.RunQuery` (`SmartInteractSystem.cs:31-61`) runs the
//! providers in PRIORITY order — Potion-of-Return, Projectile, NPC, Tile
//! (`:15-18`) — collects every candidate, BREAKS the loop on a
//! `DistanceFromCursor == 0f` candidate (`:46-49`), then votes with a
//! STRICT `>` (`:55`): the FIRST (highest-priority) candidate wins ties.
//! The earlier "the tile candidate always wins" note held only while the
//! other three providers were unported. With them live:
//! * a zero-distance potion/proj/NPC win short-circuits the chain BEFORE
//!   the tile provider runs — the outline lists stay EMPTY that tick;
//! * a non-zero NPC/proj/potion win still lets the tile provider fill the
//!   lists, but the winner's `WinCandidacy` never sets the interaction
//!   rect (it stays −1 from `ClearSelfAndPrepareForCheck`), so
//!   `InSmartCursorHighlightArea`'s rect gate refuses every cell — the
//!   outlines are suppressed exactly as in the C#.
//!
//! ## Deliberate state (documented in ANALYSIS "The smart-interact feed")
//!
//! * The Potion-of-Return provider ports the gate hitbox + the vote; the
//!   RETURN-leg interaction (`PotionOfReturnGateInteractionChecker` →
//!   `DoPotionOfReturnReturnToOriginalUsePosition`, `Teleport` style 8)
//!   rides the unported `Player.Teleport` system — the recall_teleport.rs
//!   follow-up. The projectile provider's interactable kinds (the flying
//!   piggy bank 525, the void lens 734, Chester 960, the Palworld/ClayPot
//!   minions) have no ported spawners yet, so its list is empty in
//!   practice — the scan is live and faithful either way. Grappling hooks
//!   are NOT interactable (`Projectile.IsInteractable`'s 10-id switch,
//!   `Projectile.cs:78828-78846`, has no hook id).
//! * Gamepad arms PARKED (no gamepad UI in the port):
//!   `UILinkPointNavigator.InUse` / `PlayerInput.UsingGamepad &&
//!   Main.HoveringOverAnNPC` (`Player.cs:17377`'s selected-clear),
//!   `ShowGamepadHints` (`:17381`'s gate + `:17389`'s
//!   `DemandOnlyZeroDistanceTargets`), the tile provider's strict
//!   zero-distance arm (`TileSmartInteractCandidateProvider.cs:339-347`,
//!   `num3/num4` fed by `flag3` `:303-307`) and its
//!   `SmartInteractShowingFake` win (`:44-47`), the NPC/proj providers'
//!   `DemandOnlyZeroDistanceTargets && !flag` refusals
//!   (`NPCSmartInteractCandidateProvider.cs:67-70`,
//!   `ProjectileSmartInteractCandidateProvider.cs:68-71`). With the
//!   gamepad state false the lookup gates on `Main.SmartCursorIsUsed`
//!   alone (the runtime `smart_cursor_used`) and every provider runs its
//!   full-interaction arm.
//! * `Main.hideUI` has no port state (the F11 toggle is unported) — the
//!   outline gate's `hide_ui` reads false.
//! * `Main.DrawSmartInteract` (`Main.cs:46660-46691`, the aimed-cell
//!   MagicPixel marker) has NO call site anywhere in the 1.4.5.8
//!   decompile — dead code, deliberately not ported.
//! * RNG census: ZERO draws — the whole scan is integer/float maths.

use crate::app::App;
use tr_core::smart_cursor::in_tile_bounds;
use tr_core::tiledata as td;
use tr_core::World;

/// `Main.SmartInteractTileCoords`/`…Selected` + the interaction rect + the
/// genuine/fake flags (`Main.cs:2035-2043`, `ClearSmartInteract`
/// `:48054-48062`). The lists keep insertion order (the C# `List<Point>`);
/// membership is order-irrelevant, duplicates never enter (`:321`).
#[derive(Default)]
pub(crate) struct SmartInteractState {
    /// `Main.SmartInteractTileCoords` (`Main.cs:2035`).
    pub coords: Vec<(i32, i32)>,
    /// `Main.SmartInteractTileCoordsSelected` (`Main.cs:2037`).
    pub selected: Vec<(i32, i32)>,
    /// `Main.TileInteractionLX/LY/HX/HY` (`Main.cs:2039-2045`), −1 when
    /// nothing won (`ClearSelfAndPrepareForCheck` `:67-70`).
    pub lx: i32,
    pub ly: i32,
    pub hx: i32,
    pub hy: i32,
    /// `Main.SmartInteractX/Y` (`WinCandidacy` `:42-43`), −1 idle.
    pub aimed_x: i32,
    pub aimed_y: i32,
    /// `Main.SmartInteractShowingGenuine/Fake` (`:44-51`).
    pub showing_genuine: bool,
    pub showing_fake: bool,
    /// `Main.SmartInteractNPC` (`Main.cs:2029`, the NPC provider's
    /// `WinCandidacy`, `NPCSmartInteractCandidateProvider.cs:16`): the
    /// winning town NPC's index (the port's `Vec<Npc>` index — the C#
    /// `Main.npc` slot), −1 idle.
    pub npc: i32,
    /// `Main.SmartInteractProj` (`Main.cs:2031`,
    /// `ProjectileSmartInteractCandidateProvider.cs:16`): the winning
    /// interactable projectile's pool slot, −1 idle.
    pub proj: i32,
    /// `Main.SmartInteractPotionOfReturn` (`Main.cs:2033`,
    /// `PotionOfReturnSmartInteractCandidateProvider.cs:13`).
    pub potion_of_return: bool,
}

impl SmartInteractState {
    /// `Main.ClearSmartInteract` (`Main.cs:48054-48062`) — the flags, the
    /// NPC/proj picks and both lists. The interaction rect is NOT cleared
    /// here (the C# clears it in `ClearSelfAndPrepareForCheck` /
    /// `DrawCursor` only); `SmartInteractPotionOfReturn` is NOT cleared
    /// here either (only the potion provider's clear touches it, `:25-28`).
    pub fn clear_smart_interact(&mut self) {
        self.showing_genuine = false;
        self.showing_fake = false;
        self.npc = -1;
        self.proj = -1;
        self.coords.clear();
        self.selected.clear();
    }

    /// `TileSmartInteractCandidateProvider.ClearSelfAndPrepareForCheck`
    /// (`:63-74`) + the other providers' clears folded into the one
    /// `RunQuery.Clear` call (`SmartInteractSystem.cs:22-29`): the aimed
    /// cell + the rect to −1, both lists empty, the potion flag false
    /// (`PotionOfReturnSmartInteractCandidateProvider.cs:25-28`). The
    /// NPC/proj picks are already −1 from `ClearSmartInteract`.
    fn clear_self_and_prepare(&mut self) {
        self.aimed_x = -1;
        self.aimed_y = -1;
        self.lx = -1;
        self.hx = -1;
        self.ly = -1;
        self.hy = -1;
        self.potion_of_return = false;
        self.coords.clear();
        self.selected.clear();
    }
}

/// `ProvideCandidate`'s size switch (`TileSmartInteractCandidateProvider.cs:115-289`):
/// `(width, height, frameY-stride padding)` in tiles; the padding `num11`
/// defaults to 2 and the stride is `18 * height + padding` (`:300`).
/// `None` = the type has no arm (`num7 == 0 || num8 == 0` skip, `:295`).
/// The campfire override (`:290-294`) forces (3, 2) with the default 2.
pub(crate) fn interact_tile_size(kind: u16) -> Option<(i32, i32, i32)> {
    // `:290-294` — the campfire set override (215, `TileID.cs:616`) runs
    // BEFORE the zero-check (`:295`): 215 has no switch arm, the override
    // is what sizes it.
    if td::SET_CAMPFIRES.has(kind) {
        return Some((3, 2, 2));
    }
    let (w, h, pad) = match kind {
        136 | 144 | 494 => (1, 1, 0),
        216 | 338 => (1, 2, 2),
        15 | 497 => (1, 2, 4),
        10 => (1, 3, 0),
        388 | 389 => (1, 5, 2),
        16 | 18 | 29 | 134 | 387 => (2, 1, 2),
        21 | 55 | 85 | 94 | 96 | 97 | 125 | 132 | 172 | 287 | 335 | 386 | 411 | 425 | 441
        | 467 | 468 | 573 | 621 | 622 => (2, 2, 2),
        79 | 139 | 510 | 511 => (2, 2, 0),
        11 | 356 | 410 | 470 | 480 | 509 | 657 | 658 | 663 | 720 | 721 | 725 => (2, 3, 0),
        207 => (2, 4, 0),
        104 => (2, 5, 2),
        88 => (3, 1, 0),
        14 | 17 | 26 | 77 | 86 | 89 | 114 | 133 | 217 | 218 | 237 | 377 | 469 => (3, 2, 2),
        106 | 220 | 228 | 243 | 247 | 283 | 300 | 301 | 302 | 303 | 304 | 305 | 306 | 307
        | 308 | 355 | 412 | 499 => (3, 3, 2),
        354 | 455 | 491 | 733 => (3, 3, 0),
        487 => (4, 2, 0),
        212 => (4, 3, 2),
        209 => (4, 3, 0),
        101 | 102 | 463 | 475 | 597 => (3, 4, 2),
        699 => (4, 4, 2),
        464 => (5, 4, 2),
        _ => return None,
    };
    Some((w, h, pad))
}

/// `Utils.ClosestPointInRect` (`Utils.cs:1581-1601`): the point clamped
/// into the rect `[x, x+w] × [y, y+h]` (the Right/Bottom edges are
/// INCLUSIVE — `Rectangle.Right = X + Width`).
fn closest_point_in_rect(x: f32, y: f32, w: f32, h: f32, px: f32, py: f32) -> (f32, f32) {
    let cx = if px < x {
        x
    } else if px > x + w {
        x + w
    } else {
        px
    };
    let cy = if py < y {
        y
    } else if py > y + h {
        y + h
    } else {
        py
    };
    (cx, cy)
}

/// `Rectangle.Distance(Vector2)` (`Utils.cs:1507-1542`): 0 when the point
/// is inside (`FloatIntersect`, `:982-989`), the axis gap when it shares
/// the rect's X or Y span, else the corner `Vector2.Distance`. The rect
/// is `(x, y, w, h)` with the inclusive Right/Bottom edges.
fn rect_distance(x: f32, y: f32, w: f32, h: f32, px: f32, py: f32) -> f32 {
    let (right, bottom) = (x + w, y + h);
    // `FloatIntersect` — the point is a 0×0 rect.
    if !(x > px || y > py || right < px || bottom < py) {
        return 0.0;
    }
    if px >= x && px <= right {
        return if py < y { y - py } else { py - bottom };
    }
    if py >= y && py <= bottom {
        return if px < x { x - px } else { px - right };
    }
    let (cx, cy) = if px < x {
        if py < y { (x, y) } else { (x, bottom) }
    } else if py < y {
        (right, y)
    } else {
        (right, bottom)
    };
    ((cx - px).powi(2) + (cy - py).powi(2)).sqrt()
}

/// `TileReachCheckSettings.Simple.GetTileRegion`
/// (`TileReachCheckSettings.cs:45-54`, TB = 0): the CURRENT
/// `Player.tileRangeX/Y` capped at the 20-tile `TileReachLimit`
/// (`:20-43`), the truncation/ceil tile box. Returns `(lx, ly, hx, hy)`.
fn simple_tile_region(p: &crate::player::Player) -> (i32, i32, i32, i32) {
    let rx = (p.tile_range_x() as i32).min(tr_core::smart_cursor::SIMPLE_TILE_REACH_LIMIT);
    let ry = (p.tile_range_y() as i32).min(tr_core::smart_cursor::SIMPLE_TILE_REACH_LIMIT);
    let lx = (p.x / 16.0) as i32 - rx;
    let hx = ((p.x + crate::player::WIDTH) / 16.0).ceil() as i32 - 1 + rx;
    let ly = (p.y / 16.0) as i32 - ry;
    let hy = ((p.y + p.height) / 16.0).ceil() as i32 - 1 + ry;
    (lx, ly, hx, hy)
}

/// One provider's candidate (`ISmartInteractCandidate`): the distance
/// that ranks it in `RunQuery`'s vote + the `WinCandidacy` payload. The
/// four C# `ReusableCandidate` classes fold into this enum.
enum Candidate {
    /// `PotionOfReturnSmartInteractCandidateProvider.ReusableCandidate`.
    PotionOfReturn { dist: f32 },
    /// `ProjectileSmartInteractCandidateProvider.ReusableCandidate`.
    Projectile { dist: f32, slot: i32 },
    /// `NPCSmartInteractCandidateProvider.ReusableCandidate`.
    Npc { dist: f32, index: i32 },
    /// `TileSmartInteractCandidateProvider.ReusableCandidate` — the rect
    /// is ALREADY padded ±10 by `Reuse` (`:352` passes `settings.LX-10`
    /// …), `WinCandidacy` (`:52-55`) subtracts/adds 10 AGAIN (the C#
    /// double-pad, kept verbatim).
    Tile { dist: f32, ax: i32, ay: i32, lx: i32, ly: i32, hx: i32, hy: i32 },
}

impl Candidate {
    /// `ISmartInteractCandidate.DistanceFromCursor`.
    fn distance(&self) -> f32 {
        match *self {
            Candidate::PotionOfReturn { dist }
            | Candidate::Projectile { dist, .. }
            | Candidate::Npc { dist, .. }
            | Candidate::Tile { dist, .. } => dist,
        }
    }

    /// `ISmartInteractCandidate.WinCandidacy` — the per-provider writes.
    fn win(self, state: &mut SmartInteractState) {
        match self {
            // `PotionOfReturnSmartInteractCandidateProvider.cs:11-15`.
            Candidate::PotionOfReturn { .. } => {
                state.potion_of_return = true;
                state.showing_genuine = true;
            }
            // `ProjectileSmartInteractCandidateProvider.cs:14-18`.
            Candidate::Projectile { slot, .. } => {
                state.proj = slot;
                state.showing_genuine = true;
            }
            // `NPCSmartInteractCandidateProvider.cs:14-18`.
            Candidate::Npc { index, .. } => {
                state.npc = index;
                state.showing_genuine = true;
            }
            // `TileSmartInteractCandidateProvider.cs:40-56` — the
            // strict/fake arm is the parked gamepad zero-distance win.
            Candidate::Tile { ax, ay, lx, ly, hx, hy, .. } => {
                state.aimed_x = ax;
                state.aimed_y = ay;
                state.showing_genuine = true;
                state.lx = lx - 10;
                state.ly = ly - 10;
                state.hx = hx + 10;
                state.hy = hy + 10;
            }
        }
    }
}

/// `PotionOfReturnHelper.TryGetGateHitbox` (`PotionOfReturnHelper.cs:7-17`):
/// the home position + (0, −21) centred 24×40 rect
/// (`Utils.CenteredRectangle`, `Utils.cs:947-950` — the `(int)` truncation).
/// `None` when no comeback point is recorded.
fn potion_gate_hitbox(p: &crate::player::Player) -> Option<(f32, f32, f32, f32)> {
    let (hx, hy) = p.potion_of_return_home_position?;
    Some(((hx - 12.0) as i32 as f32, (hy - 21.0 - 20.0) as i32 as f32, 24.0, 40.0))
}

/// `PotionOfReturnSmartInteractCandidateProvider.ProvideCandidate`
/// (`:30-47`): the gate hitbox's closest point to the mouse, its distance
/// and its tile — refused when the tile leaves the scan region. Runs
/// regardless of `FullInteraction` (the C# has no such gate here).
fn provide_potion_candidate(
    p: &crate::player::Player,
    mouse: (f32, f32),
    lx: i32,
    ly: i32,
    hx: i32,
    hy: i32,
) -> Option<Candidate> {
    let (x, y, w, h) = potion_gate_hitbox(p)?;
    let (cx, cy) = closest_point_in_rect(x, y, w, h, mouse.0, mouse.1);
    let dist = ((cx - mouse.0).powi(2) + (cy - mouse.1).powi(2)).sqrt();
    // `:39-42` — `vector.ToTileCoordinates()` (`Utils.cs:1807-1810`).
    let (tx, ty) = ((cx as i32) >> 4, (cy as i32) >> 4);
    if tx < lx || tx > hx || ty < ly || ty > hy {
        return None;
    }
    Some(Candidate::PotionOfReturn { dist })
}

/// `Player.UpdateNearbyInteractableProjectilesList` (`Player.cs:23135-23151`)
/// + `IsProjectileInteractableAndInInteractionRange` (`:23153-23169`): the
/// per-tick interactable-projectile list the projectile provider reads
/// (`GetListOfProjectilesToInteractWithHack`, `:31908-31911`).
/// `CurrentFrameFlags.HadAnActiveInteractableProjectile` is raised by
/// exactly three AI arms — aiStyle 97's flying piggy bank 525
/// (`Projectile.cs:33070`, the only `aiStyle = 97` kind, `:5953-5961`),
/// `AI_154_VoidLens` 734 (`:52156`) and `AI_026`'s Chester 960 arm
/// (`:58182-58186`) — and reset every frame (`Main.cs:18000`); the port
/// derives it from the live pool (the three kinds' AIs run every tick
/// while active, so the flag ≡ one of them is alive). The Palworld/ClayPot
/// minions ARE `IsInteractable` (`:78828-78846`) but raise no flag — the
/// C# quirk the derivation keeps.
fn update_interactable_projectiles(app: &App) -> Vec<i32> {
    let mut out = Vec::new();
    let had_flag = app
        .projectiles
        .slots()
        .iter()
        .any(|q| q.active && matches!(q.kind, 525 | 734 | 960));
    if !had_flag {
        return out;
    }
    let Some(p) = app.player.as_ref() else { return out };
    let (pcx, pcy) = p.center();
    let (lx, ly, hx, hy) = simple_tile_region(p);
    for (i, q) in app.projectiles.slots().iter().enumerate() {
        if !q.active || !projectile_is_interactable(q.kind) {
            continue;
        }
        // `:23163-23164` — the hitbox's closest point to the player
        // centre, tiled, gated by the Simple reach region (UNCLAMPED —
        // `IsInTileInteractionRange` `:32025-32029` reads GetTileRegion
        // directly, not the lookup's 10-tile-clamped settings).
        let (cx, cy) = closest_point_in_rect(q.pos.x, q.pos.y, q.w, q.h, pcx, pcy);
        let (tx, ty) = ((cx as i32) >> 4, (cy as i32) >> 4);
        if in_tile_bounds(tx, ty, lx, ly, hx, hy) {
            out.push(i as i32);
        }
    }
    out
}

/// `Projectile.IsInteractable` (`Projectile.cs:78828-78846`): the 10-id
/// switch — FlyingPiggyBank 525, VoidLens 734, ChesterPet 960, the
/// Palworld minions 1093/1094/1098/1112/1113/1123 and ClayPotMinion 1118
/// (`ProjectileID.cs:1604`/`:2022`/`:2474`/`:2740-2800`). NO grappling
/// hook id — hooks are not smart-interact candidates.
fn projectile_is_interactable(kind: u16) -> bool {
    matches!(kind, 525 | 734 | 960 | 1093 | 1094 | 1098 | 1112 | 1113 | 1118 | 1123)
}

/// `ProjectileSmartInteractCandidateProvider.ProvideCandidate` (`:34-79`):
/// the closest interactable projectile to the mouse over the prebuilt
/// list; a zero-distance hit breaks the scan (`:59-65`). The
/// `DemandOnlyZeroDistanceTargets && !flag` refusal (`:68-71`) is the
/// parked gamepad arm. `FullInteraction` is true (mouse mode).
fn provide_projectile_candidate(app: &App, mouse: (f32, f32), list: &[i32]) -> Option<Candidate> {
    let mut best: Option<(i32, f32)> = None;
    for &slot in list {
        let Some(q) = app.projectiles.slots().get(slot as usize) else { continue };
        if !q.active {
            continue;
        }
        let d = rect_distance(q.pos.x, q.pos.y, q.w, q.h, mouse.0, mouse.1);
        // `:54` — the strict `>` re-read of the stored best: FIRST wins ties.
        if best.is_none_or(|(_, bd)| bd > d) {
            best = Some((slot, d));
        }
        if d == 0.0 {
            best = Some((slot, 0.0));
            break;
        }
    }
    best.map(|(slot, dist)| Candidate::Projectile { dist, slot })
}

/// `NPCSmartInteractCandidateProvider.ProvideCandidate` (`:34-78`): the
/// closest town NPC whose hitbox intersects the Simple WORLD region
/// (`TileReachCheckSettings.cs:62-77` — the tile region ×16 with the
/// +15 Right/Bottom corrections, UNCLAMPED); a zero-distance hit breaks
/// the scan (`:58-64`). The `!flag` gamepad refusal (`:67-70`) is parked.
/// `FullInteraction` is true (mouse mode, `Player.cs:17390`).
fn provide_npc_candidate(app: &App, mouse: (f32, f32)) -> Option<Candidate> {
    let p = app.player.as_ref()?;
    let (tlx, tly, thx, thy) = simple_tile_region(p);
    // `GetWorldRegion` (`:62-71`): `Rectangle(LX*16, LY*16, HX*16+15 -
    // LX*16, HY*16+15 - LY*16)`.
    let (rx, ry) = (tlx as f32 * 16.0, tly as f32 * 16.0);
    let (rw, rh) = ((thx - tlx) as f32 * 16.0 + 15.0, (thy - tly) as f32 * 16.0 + 15.0);
    let mut best: Option<(i32, f32)> = None;
    for (i, n) in app.npcs.iter().enumerate() {
        // `:50` — `active && townNPC && Hitbox.Intersects(worldRegion)`.
        // The port's `hp > 0` is `active`; `is_town_resident` is the
        // `npc.townNPC` set (`npc/data.rs:923-954`). The `!flag` term is
        // always true before the zero-distance break.
        if n.hp <= 0 || !n.kind.is_town_resident() {
            continue;
        }
        let (w, h) = n.size();
        // `Rectangle.Intersects` — the half-open overlap test.
        if !(n.x < rx + rw && n.x + w > rx && n.y < ry + rh && n.y + h > ry) {
            continue;
        }
        let d = rect_distance(n.x, n.y, w, h, mouse.0, mouse.1);
        // `:53` — strict `>`: FIRST wins ties.
        if best.is_none_or(|(_, bd)| bd > d) {
            best = Some((i as i32, d));
        }
        if d == 0.0 {
            best = Some((i as i32, 0.0));
            break;
        }
    }
    best.map(|(index, dist)| Candidate::Npc { dist, index })
}

/// `FillPotentialTargetTiles` (`:359-480`): the row-major scan of the reach
/// rect collecting every interactable target cell. `has_1293`/`has_949`
/// are the player's `HasItem` reads (`Player.cs:55894-55904`), `snapping`
/// the `IsWithinSnappngRangeToTile(i, j, 40)` predicate
/// (`Player.cs:17365-17372`).
fn fill_potential_target_tiles(
    world: &World,
    lx: i32,
    ly: i32,
    hx: i32,
    hy: i32,
    has_1293: bool,
    has_949: bool,
    snapping: impl Fn(i32, i32) -> bool,
    fast_forward_dawn: bool,
    fast_forward_dusk: bool,
    sundial_cooldown: i32,
    moondial_cooldown: i32,
    is_mp_client: bool,
    targets: &mut Vec<(i32, i32)>,
) {
    for i in lx..=hx {
        for j in ly..=hy {
            let Some(tile) = world.tile(i, j) else { continue };
            if !tile.active() {
                continue;
            }
            let kind = tile.kind;
            if td::SET_CAMPFIRES.has(kind) {
                targets.push((i, j));
                continue;
            }
            if crate::craft::tile_used_in_recipes(kind) {
                targets.push((i, j));
                continue;
            }
            if td::SET_COUNTS_AS_WATER_FOR_CRAFTING.has(kind) {
                targets.push((i, j));
                continue;
            }
            match kind {
                10 | 11 | 21 | 29 | 55 | 79 | 85 | 88 | 89 | 97 | 102 | 104 | 125 | 132
                | 136 | 139 | 144 | 207 | 209 | 216 | 287 | 335 | 338 | 354 | 377 | 386
                | 387 | 388 | 389 | 410 | 411 | 425 | 441 | 455 | 463 | 464 | 467 | 468
                | 470 | 475 | 480 | 487 | 491 | 494 | 509 | 510 | 511 | 573 | 597 | 621
                | 657 | 658 | 699 | 720 | 721 | 725 | 733 => targets.push((i, j)),
                15 | 497 => {
                    if snapping(i, j) {
                        targets.push((i, j));
                    }
                }
                237 => {
                    if has_1293 {
                        targets.push((i, j));
                    }
                }
                212 => {
                    if has_949 {
                        targets.push((i, j));
                    }
                }
                356 => {
                    if !fast_forward_dawn && (is_mp_client || sundial_cooldown == 0) {
                        targets.push((i, j));
                    }
                }
                663 => {
                    if !fast_forward_dusk && (is_mp_client || moondial_cooldown == 0) {
                        targets.push((i, j));
                    }
                }
                _ => {}
            }
        }
    }
}

/// `TileSmartInteractCandidateProvider.ProvideCandidate` (`:76-357`) with
/// `FullInteraction = true` and `DemandOnlyZeroDistanceTargets = false`
/// (the mouse-mode `SmartInteractLookup` settings, `Player.cs:17389-17390`).
/// Fills the lists on `state` and returns the winning candidate — the
/// genuine arm (`:349-355`): the winner's 16 px cell rect distance
/// (`Utils.ClosestPointInRect` + `Vector2.Distance`, `Utils.cs:1581-1601`)
/// and the ALREADY ±10-padded rect (`Reuse` `:352` passes `settings.LX-10`
/// …; `WinCandidacy` `:52-55` pads AGAIN — the C# double-pad, kept).
#[allow(clippy::too_many_arguments)]
fn provide_tile_candidate(
    world: &World,
    state: &mut SmartInteractState,
    mouse: (f32, f32),
    lx: i32,
    ly: i32,
    hx: i32,
    hy: i32,
    targets: &[(i32, i32)],
) -> Option<Candidate> {
    // `:79` — `settings.mousevec.ToTileCoordinates()` (`Utils.cs:1807-1810`:
    // `(int)v >> 4`, the truncation-then-arithmetic-shift).
    let point = ((mouse.0 as i32) >> 4, (mouse.1 as i32) >> 4);
    // `:81-103` — the closest-target vote: tile centre `*16 + 8`, the
    // `num5 == -1f || num6 <= num5` sentinel (LATER targets win ties), then
    // the reach-rect gate on the winner.
    let mut num = -1;
    let mut num2 = -1;
    if !targets.is_empty() {
        let mut best_d = -1.0f32;
        let mut best = targets[0];
        for &(tx, ty) in targets {
            let cx = tx as f32 * 16.0 + 8.0;
            let cy = ty as f32 * 16.0 + 8.0;
            let d = ((cx - mouse.0).powi(2) + (cy - mouse.1).powi(2)).sqrt();
            if best_d == -1.0 || d <= best_d {
                best_d = d;
                best = (tx, ty);
            }
        }
        if in_tile_bounds(best.0, best.1, lx, ly, hx, hy) {
            num = best.0;
            num2 = best.1;
        }
    }
    // `:104-338` — the per-target size/anchor walk filling the lists.
    let mut flag = false; // a selected object was already found (`:104`)
    for &(tx, ty) in targets {
        let Some(tile) = world.tile(tx, ty) else { continue };
        let Some((w, h, pad)) = interact_tile_size(tile.kind) else {
            continue; // `:295-298`
        };
        // `:299-300` — the top-left anchor: the frame modulo the object's
        // pixel stride, back to tiles.
        let ax = tx - (tile.frame_x as i32) % (18 * w) / 18;
        let ay = ty - (tile.frame_y as i32) % (18 * h + pad) / 18;
        // `:301-302` — the winner inside this object (`flag2`) and the
        // mouse tile inside it (`flag3`). `flag3` feeds only the
        // zero-distance candidate (`num3/num4`, `:303-307`) — the gamepad
        // `DemandOnlyZeroDistanceTargets` arm, unported (module docs).
        let mut flag2 = in_tile_bounds(num, num2, ax, ay, ax + w - 1, ay + h - 1);
        let _flag3 = in_tile_bounds(point.0, point.1, ax, ay, ax + w - 1, ay + h - 1);
        // `:308-311` — `FullInteraction` is true, so no `flag2 &= flag3`.
        // `:312-315` — only the FIRST selected object keeps its selection.
        if flag {
            flag2 = false;
        }
        // `:316-333` — the cell fill: dedup against the coords list; the
        // selected list only for the winning object, the coords list for
        // every object (FullInteraction).
        for k in ax..ax + w {
            for l in ay..ay + h {
                let p = (k, l);
                if !state.coords.contains(&p) {
                    if flag2 {
                        state.selected.push(p);
                    }
                    state.coords.push(p);
                }
            }
        }
        // `:334-337` — `if (!flag & flag2) flag = true`.
        if !flag && flag2 {
            flag = true;
        }
    }
    // `:339-348` — DemandOnlyZeroDistanceTargets is false (mouse mode).
    // `:349-356` — the genuine candidate: the winner's 16 px cell rect
    // distance + the ±10-padded rect (`Reuse` `:352`).
    if num != -1 && num2 != -1 {
        let dist = rect_distance(num as f32 * 16.0, num2 as f32 * 16.0, 16.0, 16.0, mouse.0, mouse.1);
        Some(Candidate::Tile {
            dist,
            ax: num,
            ay: num2,
            lx: lx - 10,
            ly: ly - 10,
            hx: hx + 10,
            hy: hy + 10,
        })
    } else {
        None
    }
}

/// `Player.SmartInteractLookup` (`Player.cs:17374-17398`) +
/// `SmartInteractLookup_PrepareCommonlyUsedInfo` (`:17400-17408`) +
/// `SmartInteractSystem.RunQuery` (`SmartInteractSystem.cs:31-61`) with the
/// full four-provider chain (Potion → Projectile → NPC → Tile). `mouse` is
/// `Main.MouseWorld` (no gravity flip in the port), `tile_target` the
/// CURRENT `Player.tileTargetX/Y` (post-smart-cursor re-point — the block
/// provider reads it, `BlockBecauseYouAreOverAnImportantTile.cs:9-10`).
pub(crate) fn smart_interact_lookup(
    app: &mut App,
    mouse: (f32, f32),
    tile_target: (i32, i32),
) {
    let state = &mut app.smart_interact;
    // `:17376` — ClearSmartInteract first, every tick.
    state.clear_smart_interact();
    // `:17377-17380` — the gamepad/navigator selected-clear: no port state.
    // `:17381-17385` — `(!ShowGamepadHints && !SmartCursorIsUsed) ||
    // !mouseItem.IsAir`: the port has no gamepad hints, so the gate is the
    // runtime smart-cursor flag + an empty cursor stack.
    let mouse_item_air = app.mouse_item.kind == 0 || app.mouse_item.count == 0;
    if !crate::world_sim::smart_cursor::enabled(app) || !mouse_item_air {
        state.clear_self_and_prepare();
        return;
    }
    let Some(player) = app.player.as_ref() else {
        state.clear_self_and_prepare();
        return;
    };
    // `:17402-17407` — the Simple reach region + the 10-tile world clamps.
    // The region reads the CURRENT `Player.tileRangeX/Y`
    // (`TileReachCheckSettings.GetRanges`, `TileReachCheckSettings.cs:22-23`
    // — the journey/accessory range the port's `tile_range_x/y()` carry,
    // the `craft::tile_reach` convention); the boost is 0 (no `TB` argument
    // at this call site, `Player.cs:17403`).
    let reach_x =
        (player.tile_range_x() as i32).min(tr_core::smart_cursor::SIMPLE_TILE_REACH_LIMIT);
    let reach_y =
        (player.tile_range_y() as i32).min(tr_core::smart_cursor::SIMPLE_TILE_REACH_LIMIT);
    let px = (player.x / 16.0) as i32;
    let py = (player.y / 16.0) as i32;
    let hx0 = ((player.x + crate::player::WIDTH) / 16.0).ceil() as i32 - 1;
    let hy0 = ((player.y + player.height) / 16.0).ceil() as i32 - 1;
    let (lx, ly, hx, hy) = (px - reach_x, py - reach_y, hx0 + reach_x, hy0 + reach_y);
    let (w, h) = (app.world.width, app.world.height);
    let lx = lx.clamp(10, w - 10);
    let hx = hx.clamp(10, w - 10);
    let ly = ly.clamp(10, h - 10);
    let hy = hy.clamp(10, h - 10);
    // `RunQuery`'s Clear (`SmartInteractSystem.cs:33` → the provider's
    // `ClearSelfAndPrepareForCheck`, `:63-74`).
    state.clear_self_and_prepare();
    // `BlockBecauseYouAreOverAnImportantTile` (`:7-25`): InWorld fluff 10
    // on the tile target + the DisableSmartInteract set.
    let (ttx, tty) = tile_target;
    if ttx < 10 || ttx >= w - 10 || tty < 10 || tty >= h - 10 {
        return;
    }
    if app
        .world
        .tile(ttx, tty)
        .is_some_and(|t| t.active() && td::SET_DISABLE_SMART_INTERACT.has(t.kind))
    {
        return;
    }
    // The provider's inputs: HasItem over the 58-slot inventory (the port
    // carries the 50 main grid — coins/ammo never hold 1293/949), the
    // snapping predicate and the dial cooldowns.
    let has = |kind: u16| app.inventory.iter().any(|s| s.kind == kind && s.count > 0);
    let has_1293 = has(1293);
    let has_949 = has(949);
    let (pcx, _) = player.center();
    let (_, pbottom) = player.bottom();
    let snapping = |i: i32, j: i32| {
        let dx = i as f32 * 16.0 + 8.0 - pcx;
        let dy = j as f32 * 16.0 + 8.0 - (pbottom - 16.0);
        (dx * dx + dy * dy).sqrt() <= 40.0
    };
    let mut targets = Vec::new();
    fill_potential_target_tiles(
        &app.world,
        lx,
        ly,
        hx,
        hy,
        has_1293,
        has_949,
        snapping,
        app.world.fast_forward_to_dawn,
        app.world.fast_forward_to_dusk,
        app.world.sundial_cooldown,
        app.world.moondial_cooldown,
        false, // `Main.netMode == 1` — the port is singleplayer.
        &mut targets,
    );
    // `RunQuery` (`SmartInteractSystem.cs:31-61`) — the provider chain in
    // priority order (Potion → Projectile → NPC → Tile, `:15-18`). A
    // provider returning NO candidate does not stop the chain (`:43`'s
    // `if`); only a ZERO-distance candidate breaks it (`:46-49`). The
    // vote is the strict `>` (`:52-59`, FIRST wins ties), then the
    // winner's `WinCandidacy` (`:60`).
    let interactable = update_interactable_projectiles(app);
    let player = app.player.as_ref().expect("checked above");
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut zero_break = false;
    // 1. `PotionOfReturnSmartInteractCandidateProvider`.
    if let Some(c) = provide_potion_candidate(player, mouse, lx, ly, hx, hy) {
        zero_break = c.distance() == 0.0;
        candidates.push(c);
    }
    // 2. `ProjectileSmartInteractCandidateProvider`.
    if !zero_break {
        if let Some(c) = provide_projectile_candidate(app, mouse, &interactable) {
            zero_break = c.distance() == 0.0;
            candidates.push(c);
        }
    }
    // 3. `NPCSmartInteractCandidateProvider`.
    if !zero_break {
        if let Some(c) = provide_npc_candidate(app, mouse) {
            zero_break = c.distance() == 0.0;
            candidates.push(c);
        }
    }
    // 4. `TileSmartInteractCandidateProvider` — fills the outline lists
    //    even when it loses the vote (the C# runs it before the vote).
    if !zero_break {
        let state = &mut app.smart_interact;
        if let Some(c) =
            provide_tile_candidate(&app.world, state, mouse, lx, ly, hx, hy, &targets)
        {
            candidates.push(c);
        }
    }
    // The vote: `smartInteractCandidate == null || current.Distance >
    // candidate.Distance` — the first candidate wins ties.
    let mut winner: Option<Candidate> = None;
    for c in candidates {
        if winner.as_ref().is_none_or(|w| w.distance() > c.distance()) {
            winner = Some(c);
        }
    }
    if let Some(c) = winner {
        c.win(&mut app.smart_interact);
    }
}

#[cfg(test)]
mod w4_g86_feed_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A 120x120 world, the player at tile (35, 40), a 3x2 altar (26) with
    /// its top-left at (40, 38) — every cell framed like `Framing` would.
    fn altar_app() -> App {
        let mut w = World::new("w".into(), 120, 120);
        w.frame_important = vec![false; td::TILE_COUNT];
        for dx in 0..3 {
            for dy in 0..2 {
                let i = w.index(40 + dx, 38 + dy);
                w.tiles[i] = Tile {
                    kind: 26,
                    frame_x: (dx * 18) as i16,
                    frame_y: (dy * 18) as i16,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 35.0 * 16.0;
        player.y = 40.0 * 16.0;
        app.player = Some(player);
        app.smart_cursor_used = true;
        app
    }

    /// The altar's six cells all enter the coords list; the mouse over the
    /// object selects all six; the interaction rect is the reach ± 10.
    #[test]
    fn w4_g86_the_altar_fills_six_cells_and_selects_under_the_mouse() {
        let mut app = altar_app();
        // The mouse at the altar's centre cell (41, 38).
        let mouse = (41.0 * 16.0 + 8.0, 38.0 * 16.0 + 8.0);
        smart_interact_lookup(&mut app, mouse, (41, 38));
        let s = &app.smart_interact;
        assert_eq!(s.coords.len(), 6, "the 3x2 altar's cells");
        for dx in 0..3 {
            for dy in 0..2 {
                assert!(s.coords.contains(&(40 + dx, 38 + dy)));
                assert!(s.selected.contains(&(40 + dx, 38 + dy)));
            }
        }
        assert!(s.showing_genuine);
        assert_eq!((s.aimed_x, s.aimed_y), (41, 38));
        // The reach rect around (35, 40): x 30..41 (width 20 → ceil col 36),
        // y 37..45 (height 42 → ceil((640+42)/16)-1 = 42, +3). The C# pads
        // TWICE: `Reuse` (`:352`) passes `settings.LX-10`… and
        // `WinCandidacy` (`:52-55`) subtracts/adds 10 AGAIN — so the final
        // rect is the reach ± 20 (the earlier single-provider port folded
        // the double-pad to one; the faithful chain restores it).
        assert_eq!((s.lx, s.ly, s.hx, s.hy), (10, 17, 61, 65));
    }

    /// The mouse OFF the altar: the NEAREST target still wins and selects
    /// (`flag2` tests the winning target's object, `:301`, not the mouse —
    /// `FullInteraction` skips the `flag2 &= flag3` gate, `:308-311`), so
    /// the altar's cells enter BOTH lists; the aimed cell is the nearest
    /// target cell, not the mouse tile.
    #[test]
    fn w4_g86_the_off_object_mouse_still_selects_the_nearest_object() {
        let mut app = altar_app();
        let mouse = (35.0 * 16.0 + 8.0, 39.0 * 16.0 + 8.0);
        smart_interact_lookup(&mut app, mouse, (35, 39));
        let s = &app.smart_interact;
        assert_eq!(s.coords.len(), 6);
        assert_eq!(s.selected.len(), 6, "the nearest object keeps its selection");
        assert!(s.showing_genuine);
        // The nearest target to (568, 632): (40, 39) centre (648, 632), d=80.
        assert_eq!((s.aimed_x, s.aimed_y), (40, 39));
    }

    /// The runtime smart-cursor flag off / a loaded cursor stack: the
    /// lists clear and stay empty (`Player.cs:17381-17385`).
    #[test]
    fn w4_g86_the_gates_clear_the_lists() {
        let mut app = altar_app();
        app.smart_cursor_used = false;
        smart_interact_lookup(&mut app, (656.0, 616.0), (41, 38));
        assert!(app.smart_interact.coords.is_empty());
        assert_eq!(app.smart_interact.lx, -1, "the rect resets to -1");
        let mut app = altar_app();
        app.mouse_item = crate::item::ItemSlot { kind: 1, name: "x", count: 1, prefix: 0 };
        smart_interact_lookup(&mut app, (656.0, 616.0), (41, 38));
        assert!(app.smart_interact.coords.is_empty(), "mouseItem not air");
    }

    /// The block provider: a DisableSmartInteract tile under the tile
    /// target refuses the whole query (`BlockBecauseYouAreOverAnImportantTile`).
    #[test]
    fn w4_g86_the_important_tile_block_refuses_the_query() {
        let mut app = altar_app();
        // A torch (4) at the target cell — in DisableSmartInteract
        // (`TileID.cs:510`).
        let i = app.world.index(35, 39);
        app.world.tiles[i] = Tile { kind: 4, ..Default::default() };
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        smart_interact_lookup(&mut app, (656.0, 616.0), (35, 39));
        assert!(app.smart_interact.coords.is_empty());
        assert!(!app.smart_interact.showing_genuine);
    }

    /// The size switch's spot rows (`:115-289`): the altar (3,2,2), the
    /// chest (2,2,2), the door (1,3,0), the campfire override (3,2,2), the
    /// work bench (2,1,2) and the unswitched kind's None.
    #[test]
    fn w4_g86_the_size_switch_rows_match_the_csharp() {
        assert_eq!(interact_tile_size(26), Some((3, 2, 2)));
        assert_eq!(interact_tile_size(21), Some((2, 2, 2)));
        assert_eq!(interact_tile_size(10), Some((1, 3, 0)));
        assert_eq!(interact_tile_size(215), Some((3, 2, 2)));
        assert_eq!(interact_tile_size(18), Some((2, 1, 2)));
        assert_eq!(interact_tile_size(15), Some((1, 2, 4)));
        assert_eq!(interact_tile_size(464), Some((5, 4, 2)));
        assert_eq!(interact_tile_size(0), None);
        assert_eq!(interact_tile_size(695), None, "the replica never feeds");
    }

    /// Every HasOutlines id has a size arm (the census: no orphan outline
    /// tile) and the five sized-but-not-outlined ids stay out of the set.
    #[test]
    fn w4_g86_every_has_outlines_tile_has_a_size_arm() {
        for kind in 0..td::TILE_COUNT as u16 {
            if td::SET_HAS_OUTLINES.has(kind) {
                assert!(
                    interact_tile_size(kind).is_some(),
                    "HasOutlines {kind} has no feed size arm"
                );
            }
        }
        for kind in [14u16, 469, 491, 573, 597] {
            assert!(interact_tile_size(kind).is_some());
            assert!(!td::SET_HAS_OUTLINES.has(kind));
        }
    }

    // ---- the provider chain (the vote re-verified with NPC/proj/potion) ----

    /// A Guide (18×40) at world pixels `(x, y)`, hp full.
    fn guide_at(app: &mut App, x: f32, y: f32) {
        let mut g = crate::npc::Npc::new(crate::npc::NpcKind::Guide, x, y, 1, 0.0);
        g.hp = g.max_hp;
        app.npcs.push(g);
    }

    /// `Rectangle.Distance` (`Utils.cs:1507-1542`): inside → 0, the axis
    /// gap when the point shares a span, the corner distance otherwise.
    #[test]
    fn w4_g86_rect_distance_matches_the_utils_math() {
        // Inside (inclusive edges).
        assert_eq!(rect_distance(100.0, 100.0, 20.0, 20.0, 110.0, 110.0), 0.0);
        assert_eq!(rect_distance(100.0, 100.0, 20.0, 20.0, 120.0, 120.0), 0.0);
        // Below the rect, within its X span: the Y gap.
        assert_eq!(rect_distance(100.0, 100.0, 20.0, 20.0, 110.0, 150.0), 30.0);
        // Right of the rect, within its Y span: the X gap.
        assert_eq!(rect_distance(100.0, 100.0, 20.0, 20.0, 150.0, 110.0), 30.0);
        // The bottom-right corner: sqrt(30² + 30²).
        let d = rect_distance(100.0, 100.0, 20.0, 20.0, 150.0, 150.0);
        assert!((d - (1800.0f32).sqrt()).abs() < 1e-4);
    }

    /// The NPC provider wins the vote when it is closer than the tile:
    /// `SmartInteractNPC` is set, the interaction rect stays −1 (the NPC
    /// `WinCandidacy` never sets it), so the outlines are suppressed even
    /// though the tile provider filled the lists.
    #[test]
    fn w4_g86_the_closer_npc_wins_and_suppresses_the_outline_rect() {
        let mut app = altar_app();
        // The Guide right under the mouse (altar centre 664, 616): its
        // hitbox contains the mouse → distance 0 → the zero-break runs
        // BEFORE the tile provider, so the lists stay EMPTY.
        guide_at(&mut app, 656.0, 600.0);
        let mouse = (664.0, 616.0);
        smart_interact_lookup(&mut app, mouse, (41, 38));
        let s = &app.smart_interact;
        assert_eq!(s.npc, 0, "the NPC provider's WinCandidacy");
        assert!(s.showing_genuine);
        assert!(s.coords.is_empty(), "the zero-distance NPC breaks before the tile provider");
        assert_eq!((s.lx, s.ly, s.hx, s.hy), (-1, -1, -1, -1), "the NPC win never sets the rect");
    }

    /// A NON-zero-distance NPC win still lets the tile provider fill the
    /// lists (the C# runs it before the vote), but the NPC wins the vote
    /// and the rect stays −1 — the outline gate refuses every cell.
    #[test]
    fn w4_g86_the_nonzero_npc_win_fills_lists_but_keeps_the_rect_minus_one() {
        let mut app = altar_app();
        // The Guide near the player but its box OFF the mouse: mouse at
        // the altar centre (664, 616), the Guide at (568, 600) — inside
        // the world region, distance > 0. The altar's nearest cell (41,38)
        // is distance 0 from the mouse, so the TILE is closer and wins.
        // To make the NPC win instead, put the mouse OFF the altar but
        // ON the Guide's side: mouse (575, 610) is inside the Guide box
        // (568..586, 600..640) → NPC distance 0 → zero-break, lists empty.
        // So use a mouse OUTSIDE the Guide box but closer to it than to
        // any altar cell: mouse (600, 610). Guide box right edge 586 →
        // NPC distance 14; nearest altar cell (40,39) centre (648,632) →
        // tile cell-rect distance ≈ sqrt((640-600)² + (624-610)²) ≈ 42.
        guide_at(&mut app, 568.0, 600.0);
        let mouse = (600.0, 610.0);
        smart_interact_lookup(&mut app, mouse, (37, 38));
        let s = &app.smart_interact;
        assert_eq!(s.npc, 0, "the NPC is closer than the altar");
        assert!(s.showing_genuine);
        assert_eq!(s.coords.len(), 6, "the tile provider still fills the lists");
        assert_eq!((s.lx, s.ly, s.hx, s.hy), (-1, -1, -1, -1), "the NPC win keeps the rect −1");
    }

    /// The tile wins when it is closer than the NPC: the aimed cell + the
    /// double-padded rect land, `SmartInteractNPC` stays −1.
    #[test]
    fn w4_g86_the_closer_tile_wins_over_the_npc() {
        let mut app = altar_app();
        // The Guide far from the mouse (which sits on the altar centre).
        guide_at(&mut app, 568.0, 600.0);
        let mouse = (664.0, 616.0); // on altar cell (41,38) → tile distance 0
        smart_interact_lookup(&mut app, mouse, (41, 38));
        let s = &app.smart_interact;
        assert_eq!(s.npc, -1, "the tile wins");
        assert_eq!((s.aimed_x, s.aimed_y), (41, 38));
        assert_eq!(s.coords.len(), 6);
        assert_eq!((s.lx, s.ly, s.hx, s.hy), (10, 17, 61, 65), "the double-padded rect");
    }

    /// The potion provider: a recorded comeback point puts a 24×40 gate
    /// at home + (0,−21); the mouse inside it → distance 0 → the potion
    /// wins the zero-break before the tile provider.
    #[test]
    fn w4_g86_the_potion_gate_wins_when_the_mouse_is_inside() {
        let mut app = altar_app();
        // Home at the altar centre (664, 616): gate centre (664, 595),
        // rect (652, 575, 24, 40). The mouse (664, 616) is BELOW the gate
        // (bottom 615) by 1 → distance 1, not zero. Put the mouse inside.
        if let Some(p) = app.player.as_mut() {
            p.potion_of_return_home_position = Some((664.0, 616.0));
        }
        let mouse = (664.0, 595.0); // gate centre → distance 0
        smart_interact_lookup(&mut app, mouse, (41, 37));
        let s = &app.smart_interact;
        assert!(s.potion_of_return, "the potion provider's WinCandidacy");
        assert!(s.showing_genuine);
        assert!(s.coords.is_empty(), "the zero-distance potion breaks before the tile provider");
        assert_eq!(s.npc, -1);
        assert_eq!(s.proj, -1);
    }

    /// `Projectile.IsInteractable`'s 10-id switch (`Projectile.cs:78828`):
    /// the piggy bank / void lens / Chester / Palworld / ClayPot minions
    /// are interactable; a grappling hook (aiStyle 7) and a plain arrow
    /// are NOT.
    #[test]
    fn w4_g86_the_interactable_projectile_set_matches_the_switch() {
        for kind in [525u16, 734, 960, 1093, 1094, 1098, 1112, 1113, 1118, 1123] {
            assert!(projectile_is_interactable(kind), "{kind} is interactable");
        }
        // 13/14 are the grappling-hook family, 1/2 the arrows — none is
        // in the 10-id switch.
        for kind in [1u16, 2, 13, 14, 526, 733, 961, 1119] {
            assert!(!projectile_is_interactable(kind), "{kind} is not interactable");
        }
    }
}
