//! The Rod of Discord 1326 / Rod of Harmony 5335 use path —
//! `Player.ItemCheck_UseTeleportRod` (`Player.cs:45476-45534`, called from
//! `ItemCheck_OwnerOnlyCode` `:43594`). W4-g89f.
//!
//! ## The machine (one function, two kinds)
//!
//! The gate (`:45478`): `myPlayer == whoAmI && (type == 1326 || 5335) &&
//! itemAnimation > 0 && ItemTimeIsZero` — the mid-swing action tick, the
//! boss-spawner/recall convention (the use START ran on the press edge;
//! `StartActualUse` leaves `itemTime` at 0, so the first action tick IS the
//! press tick). Then:
//!
//! 1. `ApplyItemTime(sItem)` (`:45482`) — the 20-tick cadence re-arm
//!    (`Item.cs:42318-42324` — both rows useTime 20 == useAnimation 20).
//! 2. The aim point (`:45483-45493`): `(mouseX + screenPosition.X,
//!    mouseY + screenPosition.Y - height)` (the `gravDir == 1` arm — the
//!    port keeps gravDir 1, `player.rs:4802`), then `X -= width / 2`.
//! 3. `LimitPointToPlayerReachableArea` (`:45494` → `:45447-45473`) — the
//!    shared `editing.rs` clamp (the 1920x1200 view box).
//! 4. The bounds flag (`:45495`): the point inside the 50-px world inset.
//! 5. `Collision.AnyWallOfTypeOnLine(playerCell, aimCell + (w/2, h/2),
//!    350)` (`:45496-45501`) — the Lihzahrd temple wall blocks the line.
//! 6. The refusal (`:45502-45511`): out of bounds, OR the aim cell is
//!    wall 87 (temple brick) with `!downedPlantBoss && (remixWorld ||
//!    below worldSurface)`, OR `SolidCollision(point, width, height)`.
//!    A refusal keeps the cadence armed (step 1 already ran) — the C#
//!    returns without teleporting, the swing continues.
//! 7. `Teleport(point, 1)` (`:45512`) — the style-1 teleport: the port's
//!    `Player::teleport_style` (hooks/vanity/shimmer strip + position +
//!    fallStart + oldPosition, `Player.cs:37414-37496` subset) plus the
//!    style-1 camera lerp (`:45459-45463` — `lerpSpeedupDelay = 10`).
//!    The `SendData(65)` sync (`:45513`) has no single-player port.
//! 8. **The 5335 difference** (`:45514-45517`): `if (sItem.type != 1326)
//!    return;` — the Rod of Harmony (the shimmered rod, shimmer table
//!    1326→5335) SKIPS the whole chaos-state block. No self-damage, no
//!    Chaos State buff. That is the item's entire reason to exist.
//! 9. The 1326 chaos-state block (`:45518-45533`): while buff 88 is up,
//!    `statLife -= statLifeMax2 / 7` (INTEGER division); the death reason
//!    is `ByOther(13)` or — on `Main.rand.Next(2) == 0` — `ByOther(14/15)`
//!    (the Male/Female rows; the port has no player Male flag, the render
//!    draws the base male frame, so the roll's 1-arm books the gendered
//!    row and the 0-arm the plain one — the en-US text is identical,
//!    ANALYSIS W4-g89f). At `statLife <= 0` → `KillMe(reason, 1.0, 0)`
//!    (the port's `dot_death` booking + the npcs-pass death drain, the
//!    DoT convention). Then `lifeRegenCount = 0; lifeRegenTime = 0`
//!    (`:45530-45531` — runs even when the damage did not kill).
//! 10. `AddBuff(88, 360)` (`:45533`) — the 6-second Chaos State debuff,
//!     EVERY 1326 teleport (the first one arms it; the next one inside
//!     the window pays the life tax).
//!
//! ## Scope / deviations (ANALYSIS "W4-g89f")
//!
//! * The `Main.TeleportEffect` dust/sound bursts (`:37444`/`:37489`) are
//!   the unported `TeleportEffect` system (the recall module's §Scope
//!   precedent — `recall_teleport.rs:88`); the camera lerp IS ported.
//!   W4-g89h: this citation is SUPERSEDED for the rod once the g11
//!   TeleportEffect union lands — the `teleport_style(…, 1)` call queues
//!   the Entry/Exit pair and the union's style-1 arm (`teleport_dust.rs`,
//!   the `0 | 1` shape: Item8 cue + type-164 dust) fires through the
//!   replay, no rod-side dust code. TWO merger actions ride research/145
//!   (first published as 139, renamed — g52c owns 139; §2a carries the
//!   EXACT drain-move hunks):
//!   the editing-phase drain (the rod fires in `update_editing`, AFTER the
//!   player tick — the queue must drain at the editing tail on
//!   `UpdatePlayers` or the chaos `Next(2)` roll swaps ahead of the dust
//!   draws, a determinism break) + the inline-camera dedup (the replay
//!   owns the style-1 camera, gap 5 — retire this module's inline call at
//!   the union). Neither is applied here (the union is not on this base).
//! * `Teleport`'s unported tails (`teleport_style`'s own doc): the
//!   funkytown/environment-immunity timers, the pressure-plate scans,
//!   the spectating reset, the unbreakable-wall scan, the shadows.
//! * The chaos-state read is the buff-88 slot scan (`buffs.find_buff_index`)
//!   — the port's `DebuffEffects.chaos_state` table row exists
//!   (`buff_effects.rs:46`) but has no live wiring; the slot scan is the
//!   canonical `Player.chaosState` equivalent (`Player.cs:10090-10093`
//!   sets it from the buff every tick).
//! * W4-g89g chaos-state CONSUMER census (C# `chaosState` has exactly four
//!   sites: the field `Player.cs:1986`, the buff-88 latch `:10090-10093`,
//!   the `ResetEffects` clear `:18442`, and this rod block `:45518` — no
//!   other reader). There is NO chaos-specific UI/draw: buff 88 rides the
//!   generic buff-icon loop (`Main.cs:43215+`; the port's
//!   `ui/resources.rs::draw_buffs` + `Buff_88.xnb` + the
//!   `(88, "ChaosState")` lang row), `Main.debuff[88] = true`
//!   (`Main.cs:8895`) makes the right-click cancel a no-op
//!   (`TryRemovingBuff`'s `!Main.debuff[b]` gate — the port's
//!   `Player::try_removing_buff` + `buffs::DEBUFF_IDS`), and 88 is NOT in
//!   `BuffID.Sets.NurseCannotRemoveDebuff` (`BuffID.cs` CreateBoolSet —
//!   the port's `buffs::NURSE_CANNOT_REMOVE_DEBUFF`), so the Nurse heals
//!   it away and it counts in `GetNurseHealCost` (the port's
//!   `remountable_debuff_count` / `clear_nurse_removable_debuffs`).
//!   The `ResetEffects` clear is observation-free in the port: the latch
//!   re-derives from the buff slots every tick, so a per-tick bool would
//!   read the same value the slot scan reads here.
//! * RNG: the ONE `Main.rand.Next(2)` draw (`:45522`) rides the
//!   `UpdatePlayers` stream (ItemCheck runs under the `Main.cs:17754`
//!   swap) and fires ONLY when the chaos damage lands (buff 88 up).

use glam::Vec2;
use tr_core::TILE_PX;

use crate::app::App;
use crate::player::Player;
use crate::registry;
use crate::regen::DotDeath;

/// `ItemID.RodofDiscord` 1326 (`ItemID.cs:3410`) / `RodOfHarmony` 5335
/// (`ItemID.cs:12353`) — the `Player.cs:45478` pair.
pub(crate) const ROD_OF_DISCORD: u16 = 1326;
pub(crate) const ROD_OF_HARMONY: u16 = 5335;

/// One 60 Hz tick in seconds — the fixed step the item timers pace against.
const ONE_TICK_SECS: f32 = 1.0 / 60.0;

/// The C# INTEGER tick comparisons out of the port's per-frame seconds
/// timers — the `editing.rs::ticks_of` convention.
fn ticks_of(secs: f32) -> i32 {
    (secs * 60.0).round() as i32
}

/// Whether the held kind rides the teleport-rod arm.
pub(crate) fn is_teleport_rod(kind: u16) -> bool {
    kind == ROD_OF_DISCORD || kind == ROD_OF_HARMONY
}

/// The held item's PREFIXED `useTime` in seconds (`ApplyItemTime` →
/// `SetItemTime(sItem.useTime)`, `Player.cs:4416-4419`) — the
/// `recall_teleport::use_time_secs` convention.
fn use_time_secs(held: crate::item::ItemSlot) -> f32 {
    crate::prefix::prefixed_stats(held.kind, held.prefix)
        .map(|s| s.use_time as f32 * ONE_TICK_SECS)
        .filter(|t| *t > 0.0)
        .unwrap_or_else(|| {
            registry::get(held.kind)
                .map(|d| d.use_time as f32 * ONE_TICK_SECS)
                .filter(|t| *t > 0.0)
                .unwrap_or_else(|| crate::item::use_animation_secs_of(held.kind, held.prefix))
        })
}

/// The style-1 teleport leg (`Player.Teleport(point, 1)`,
/// `Player.cs:37414-37496`): the port's `teleport_style` subset + the
/// myPlayer camera transition (`:45456-45463` — style 1's
/// `lerpSpeedupDelay = 10`). The `TeleportEffect` bursts are unported
/// (the module doc).
fn rod_teleport(app: &mut App, point: Vec2) {
    let Some(p) = app.player.as_mut() else {
        return;
    };
    let old = Vec2::new(p.x, p.y);
    p.teleport_style(point.x, point.y, 1);
    // `:45463` StartCameraTransitionForPlayerTeleport(vector, 0.1f, 10).
    let mut state = std::mem::take(&mut app.camera_state);
    crate::world_sim::camera::start_camera_transition_for_player_teleport(
        &mut state, old, 0.1, 10,
    );
    app.camera_state = state;
}

/// The 1326 chaos-state block (`Player.cs:45518-45533`): the life tax,
/// the death-reason roll, the regen reset. Runs AFTER the teleport, only
/// for the Rod of Discord (the 5335 arm returned at `:45514-45517`).
fn chaos_state_tax(app: &mut App) {
    let Some(p) = app.player.as_mut() else {
        return;
    };
    // `this.chaosState` — the buff-88 latch (`Player.cs:10090-10093`).
    if p.buffs.find_buff_index(88).is_some() {
        // `:45520`: `statLife -= statLifeMax2 / 7` — INTEGER division; the
        // port's single `max_hp` is the effective max (the statLifeMax2
        // standing-in, `player.rs:3612`). The C# statLife can go negative;
        // the port's collapsed hp lane floors at 0 (the `hurt_inner`
        // convention) — the `<= 0` death test below reads the same outcome.
        p.hp = (p.hp - p.max_hp / 7).max(0);
        // `:45521-45525`: the reason is ByOther(13), or on Next(2)==0 the
        // gendered ByOther(14/15) — the draw runs UNCONDITIONALLY inside
        // the chaos block (before the death test). It rides UpdatePlayers
        // (the module doc).
        let gendered = {
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            rng.next_max(2) == 0
        };
        let reason = if gendered { DotDeath::TeleportGendered } else { DotDeath::Teleport };
        // `:45526-45529`: `statLife <= 0` → KillMe. The port books the
        // reason; the npcs-pass drain (`world_sim/npcs.rs:6464`) runs the
        // drops + start_death when hp <= 0 (the DoT convention).
        if p.hp <= 0 {
            p.dot_death = Some(reason);
        }
        // `:45530-45531`: the regen reset runs even when the tax did not
        // kill (and even when it did — the C# writes both unconditionally).
        p.regen.life_regen_count = 0;
        p.regen.life_regen_time = 0.0;
    }
    // `:45533`: AddBuff(88, 360) — EVERY 1326 teleport arms the window.
    p.buffs.add_buff(88, 360);
}

/// The per-tick arm body (`Player.cs:45478-45533`), run while the swing is
/// live. The gate is `itemAnimation > 0 && ItemTimeIsZero` — the action
/// tick. A REFUSED aim (bounds/temple/solid) keeps the cadence armed and
/// the swing running (the C# returns after ApplyItemTime, no teleport).
fn run_rod_arm(app: &mut App, held: crate::item::ItemSlot) {
    if ticks_of(app.swing) <= 0 || ticks_of(app.item_time) != 0 {
        return;
    }
    // `:45482` ApplyItemTime — runs BEFORE the aim gates (a refusal still
    // pays the cadence).
    app.item_time = use_time_secs(held);
    let Some(p) = app.player.as_ref() else {
        return;
    };
    // `:45483-45493` — the aim point. The port's `world_pos` is the
    // cursor's world px (`Main.mouseX + screenPosition.X`); the gravDir-1
    // arm subtracts the player height, then X -= width/2.
    let (sw, sh) = app.screen_size();
    let mouse_world = Vec2::new(
        app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
        app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
    );
    let mut point = Vec2::new(mouse_world.x - p.width / 2.0, mouse_world.y - p.height);
    // `:45494` LimitPointToPlayerReachableArea — the shared clamp.
    let center = Vec2::from(p.center());
    point = crate::world_sim::editing::limit_point_to_player_reachable_area(
        center, point, &app.world,
    );
    // `:45495` — the 50-px world-inset bounds flag.
    let mut flag = point.x > 50.0
        && point.x < (app.world.width as f32 * TILE_PX - 50.0)
        && point.y > 50.0
        && point.y < (app.world.height as f32 * TILE_PX - 50.0);
    // `:45496-45501` — the temple-wall line scan (wall 350).
    let (pcx, pcy) = p.center();
    let from = ((pcx / TILE_PX) as i32, (pcy / TILE_PX) as i32);
    let to = (
        ((point.x + p.width / 2.0) / TILE_PX) as i32,
        ((point.y + p.height / 2.0) / TILE_PX) as i32,
    );
    if crate::collision::any_wall_of_type_on_line(&app.world, from.0, from.1, to.0, to.1, 350) {
        flag = false;
    }
    if !flag {
        return;
    }
    // `:45506-45511` — the aim-cell temple gate + the solid refusal.
    let num = (point.x / TILE_PX) as i32;
    let num2 = (point.y / TILE_PX) as i32;
    let remix = app.world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
    let temple_wall = app
        .world
        .tile(num, num2)
        .is_some_and(|t| t.wall == 87);
    if (temple_wall
        && !app.world.downed_plant_boss
        && (remix || num2 as f64 > app.world.world_surface))
        || Player::solid_collision(&app.world, point.x, point.y, p.width, p.height)
    {
        return;
    }
    // `:45512` Teleport(point, 1). (`:45513` SendData(65) — no SP port.)
    rod_teleport(app, point);
    // `:45514-45517` — the Rod of Harmony skips the chaos block entirely.
    if held.kind != ROD_OF_DISCORD {
        return;
    }
    chaos_state_tax(app);
}

/// The rod `ItemCheck` machine, called once per tick from `update_editing`
/// (the recall/boss-spawner position — after the swing/item-time decay,
/// before the cursor gates; the C# arm runs in `ItemCheck_OwnerOnlyCode`
/// regardless of the cursor). `press_edge` is the caller's true
/// previous-frame pair. Returns true when it STARTED a use this tick.
///
/// The start gate is the C#'s `controlUseItem && releaseUseItem &&
/// itemAnimation == 0` press edge (`Player.cs:42733`); both rods are
/// `autoReuse: false` (`Item.cs:42318-42324`). The per-tick arm runs
/// whenever the swing is live, even with the button released (the C#
/// gate is `itemAnimation > 0 && ItemTimeIsZero` only).
pub(crate) fn rod_item_check(app: &mut App, press_edge: bool) -> bool {
    let held = app.inventory[app.selected];
    if held.count == 0 || !is_teleport_rod(held.kind) {
        return false;
    }
    // The dead player runs UpdateDead instead of ItemCheck; CCed zeroes
    // the animation (`:42598-42603`) — the recall machine's gate.
    if app.player.as_ref().is_some_and(|p| p.dead || p.cced()) {
        return false;
    }
    if ticks_of(app.swing) > 0 {
        run_rod_arm(app, held);
        return false;
    }
    if app.mine_down && !app.inventory_open && press_edge {
        // `ItemCheck_StartActualUse` → ApplyItemAnimation (`:51680`): the
        // swing lasts useAnimation (20 ticks, no melee/summon scaling —
        // the rows are class None); the Item8 UseSound plays at the start.
        app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
        app.swing = app.swing_max;
        crate::world_sim::editing::start_swing_extras(app, held.kind);
        app.attack_cd = app.swing_max;
        // StartActualUse leaves itemTime at 0 → the arm's ItemTimeIsZero
        // leg fires THIS tick (the recall precedent).
        app.item_time = 0.0;
        run_rod_arm(app, held);
        app.mine_target = None;
        app.mine_timer = 0.0;
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::World;

    /// A 512×512 open-air world (a dirt floor at y=120 far below the aim),
    /// the player parked at (60, 100) tiles, the camera on the player.
    /// The size matters: the reachable-area clamp's 1920x1200 box needs a
    /// play area larger than the view (the i23 small-world hang-off).
    fn rig(kind: u16) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = World::new("rod".into(), 512, 512);
        for x in 0..512 {
            let i = w.index(x, 120);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        app.world = w;
        let mut p = Player::spawn(&app.world);
        p.x = 60.0 * TILE_PX;
        p.y = 100.0 * TILE_PX;
        app.player = Some(p);
        app.camera = Vec2::new(60.0 * TILE_PX, 100.0 * TILE_PX);
        // The cursor at the screen centre → the aim is the player's own
        // cell column, one height above (open air — no solid refusal).
        let (sw, sh) = app.screen_size();
        app.mouse_pos = Vec2::new(sw * 0.5, sh * 0.5);
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind,
            name: registry::name(kind),
            count: 1,
            prefix: 0,
        };
        app
    }

    fn press(app: &mut App) {
        app.mine_prev_down = false;
        app.mine_released = true;
        app.mine_down = true;
    }

    /// W4-g89f — the 5335 Rod of Harmony teleports to the aim and NEVER
    /// arms Chaos State (`Player.cs:45514-45517` — the `type != 1326`
    /// early return). The stack is NOT consumed (both rows
    /// `consumable: false`, `Item.cs:42318-42324`).
    #[test]
    fn w4_g89f_the_harmony_rod_teleports_without_chaos_state() {
        let mut app = rig(ROD_OF_HARMONY);
        let (old_x, old_y) = {
            let p = app.player.as_ref().unwrap();
            (p.x, p.y)
        };
        press(&mut app);
        assert!(rod_item_check(&mut app, true), "the press started the use");
        let p = app.player.as_ref().unwrap();
        // The aim: cursor at the player centre, minus (width/2, height).
        assert_eq!(p.x, old_x - p.width / 2.0, "the teleport landed (`:45512`)");
        assert_eq!(p.y, old_y - p.height, "the gravDir-1 aim (`:45487`)");
        assert!(
            p.buffs.find_buff_index(88).is_none(),
            "the harmony rod NEVER arms Chaos State (`:45514-45517`)"
        );
        assert_eq!(app.inventory[app.selected].count, 1, "the rod is not consumable");
        assert_eq!(
            ticks_of(app.item_time),
            20,
            "ApplyItemTime re-armed the 20-tick cadence (`:45482`)"
        );
        // A second harmony teleport inside the (absent) window: still no
        // buff, no tax — the arm is pure mobility.
        app.swing = 0.0;
        app.item_time = 0.0;
        press(&mut app);
        assert!(rod_item_check(&mut app, true));
        let p = app.player.as_ref().unwrap();
        assert!(p.buffs.find_buff_index(88).is_none());
        assert_eq!(p.hp, p.max_hp, "no life tax ever");
    }

    /// W4-g89f — the 1326 Rod of Discord: the FIRST teleport arms buff 88
    /// for 360 ticks (`:45533`) with no tax (the buff was not up at the
    /// read); the SECOND inside the window pays `statLifeMax2 / 7`
    /// (`:45520`, INTEGER division) and resets the regen ramp
    /// (`:45530-45531`).
    #[test]
    fn w4_g89f_the_discord_rod_arms_chaos_state_then_taxes_life() {
        let mut app = rig(ROD_OF_DISCORD);
        press(&mut app);
        assert!(rod_item_check(&mut app, true));
        let p = app.player.as_ref().unwrap();
        let slot = p.buffs.find_buff_index(88).expect("the first teleport arms 88");
        assert_eq!(p.buffs.buff_time[slot], 360, "the 6-second window (`:45533`)");
        assert_eq!(p.hp, p.max_hp, "the first teleport pays no tax");
        // The second teleport inside the window: the life tax.
        app.swing = 0.0;
        app.item_time = 0.0;
        let max_hp = app.player.as_ref().unwrap().max_hp;
        press(&mut app);
        assert!(rod_item_check(&mut app, true));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.hp, max_hp - max_hp / 7, "the INTEGER-division tax (`:45520`)");
        assert_eq!(p.regen.life_regen_count, 0, "the regen reset (`:45530`)");
        assert_eq!(p.regen.life_regen_time, 0.0, "(`:45531`)");
        assert!(p.buffs.find_buff_index(88).is_some(), "the window re-arms");
    }

    /// W4-g89f — the chaos tax KILLS at low life (`:45526-45529`): the
    /// reason books into `dot_death` (the npcs-pass drain runs KillMe),
    /// the regen reset still lands (`:45530-45531`), and the buff re-arms.
    /// The `Next(2)` draw rides UpdatePlayers — the witness pins the draw
    /// count (ONE per taxed teleport) on a cloned stream.
    #[test]
    fn w4_g89f_the_chaos_tax_books_the_teleport_death_at_low_life() {
        let mut app = rig(ROD_OF_DISCORD);
        // Arm the window, then drop life below one tax.
        press(&mut app);
        assert!(rod_item_check(&mut app, true));
        let max_hp = app.player.as_ref().unwrap().max_hp;
        app.player.as_mut().unwrap().hp = max_hp / 7; // one tax kills
        app.swing = 0.0;
        app.item_time = 0.0;
        let mut witness = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        press(&mut app);
        assert!(rod_item_check(&mut app, true));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.hp, 0, "the tax zeroed the life (`:45520`)");
        assert!(p.dot_death.is_some(), "the death reason booked (`:45526-45529`)");
        assert!(matches!(
            p.dot_death,
            Some(DotDeath::Teleport) | Some(DotDeath::TeleportGendered)
        ));
        assert_eq!(p.regen.life_regen_count, 0, "the reset still lands");
        // Exactly ONE Next(2) draw on the stream (the reason roll).
        let _ = witness.next_max(2);
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            witness,
            "one Next(2) draw, no others"
        );
    }

    /// W4-g89f — the temple-wall line scan refuses the teleport
    /// (`:45498-45501` — wall 350 on the line), but the cadence stays
    /// armed (`ApplyItemTime` ran at `:45482` BEFORE the gates). The
    /// player does not move.
    #[test]
    fn w4_g89f_the_temple_wall_line_refuses_but_pays_the_cadence() {
        let mut app = rig(ROD_OF_HARMONY);
        // A wall-350 column between the player and the aim cell.
        let i = app.world.index(60, 99);
        app.world.tiles[i].wall = 350;
        let (old_x, old_y) = {
            let p = app.player.as_ref().unwrap();
            (p.x, p.y)
        };
        press(&mut app);
        assert!(rod_item_check(&mut app, true), "the use started");
        let p = app.player.as_ref().unwrap();
        assert_eq!((p.x, p.y), (old_x, old_y), "the wall refused the teleport");
        assert_eq!(ticks_of(app.item_time), 20, "the cadence still armed (`:45482`)");
    }

    /// W4-g89f — the solid-collision refusal (`:45508`): an aim inside a
    /// solid block refuses; the aim-cell temple gate (`:45508` wall 87 +
    /// !downedPlantBoss + below surface) refuses too.
    #[test]
    fn w4_g89f_solid_and_temple_aims_refuse() {
        let mut app = rig(ROD_OF_HARMONY);
        // Fill the aim cell (60, 99) solid — the aim box overlaps it.
        let i = app.world.index(60, 99);
        app.world.tiles[i] = Tile { kind: 1, ..Default::default() };
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        let (old_x, old_y) = {
            let p = app.player.as_ref().unwrap();
            (p.x, p.y)
        };
        press(&mut app);
        assert!(rod_item_check(&mut app, true));
        let p = app.player.as_ref().unwrap();
        assert_eq!((p.x, p.y), (old_x, old_y), "SolidCollision refused (`:45508`)");
        // Clear the solid, set the temple wall 87 instead — below the
        // surface (world_surface = 50), PlantBoss not downed → refused.
        app.world.tiles[i] = Tile { wall: 87, ..Default::default() };
        app.swing = 0.0;
        app.item_time = 0.0;
        press(&mut app);
        assert!(rod_item_check(&mut app, true));
        let p = app.player.as_ref().unwrap();
        assert_eq!((p.x, p.y), (old_x, old_y), "the temple gate refused (`:45508`)");
        // DownedPlantBoss opens the temple.
        app.world.downed_plant_boss = true;
        app.swing = 0.0;
        app.item_time = 0.0;
        press(&mut app);
        assert!(rod_item_check(&mut app, true));
        let p = app.player.as_ref().unwrap();
        assert_ne!((p.x, p.y), (old_x, old_y), "the downed boss opens the temple");
    }

    /// W4-g89g — the chaos-state CONSUMER census, pinned: buff 88 is a
    /// debuff (`Main.debuff[88] = true`, `Main.cs:8895`), so the buff-icon
    /// right-click cancel is a NO-OP (`TryRemovingBuff`'s `!Main.debuff[b]`
    /// gate, `Main.cs:43371` — the port's `Player::try_removing_buff`), and
    /// 88 is NOT in `NurseCannotRemoveDebuff` (`BuffID.cs`'s CreateBoolSet),
    /// so it counts in `GetNurseHealCost` (`Main.cs:40059`) and clears at
    /// the Nurse heal (`:40133`). The C# `chaosState` bool has no other
    /// consumer (`Player.cs:1986`/`:10092`/`:18442`/`:45518` — the rod
    /// block is the ONLY reader); there is no chaos-specific draw.
    #[test]
    fn w4_g89g_the_chaos_state_buff_is_nurse_removable_but_not_cancel_clickable() {
        let mut app = rig(ROD_OF_DISCORD);
        press(&mut app);
        assert!(rod_item_check(&mut app, true));
        let p = app.player.as_mut().unwrap();
        let slot = p.buffs.find_buff_index(88).expect("the rod armed 88");
        // The debuff table refuses the right-click cancel (`Main.cs:8895`
        // + `:43371` — the port's `try_removing_buff` returns false and
        // leaves the slot alone).
        assert!(crate::buffs::is_debuff(88), "`Main.debuff[88] = true`");
        let world = app.world.clone();
        let cancelled = app.player.as_mut().unwrap().try_removing_buff(&world, slot, 88, [0, 0]);
        assert!(!cancelled, "the cancel click is a no-op on a debuff");
        let p = app.player.as_mut().unwrap();
        assert!(p.buffs.find_buff_index(88).is_some(), "the buff stayed");
        // The Nurse CAN remove it (88 is absent from the protected set) —
        // it counts while buffTime > 60 and clears at the heal.
        assert!(!crate::buffs::nurse_cannot_remove(88), "88 is nurse-removable");
        assert_eq!(p.remountable_debuff_count(), 1, "the 360-tick window counts");
        p.clear_nurse_removable_debuffs();
        assert!(p.buffs.find_buff_index(88).is_none(), "the heal cleared it");
        // With the window gone the next discord teleport pays NO tax
        // (the slot scan reads the same state the C# chaosState latch
        // would — `Player.cs:10090-10093`).
        let max_hp = p.max_hp;
        app.swing = 0.0;
        app.item_time = 0.0;
        press(&mut app);
        assert!(rod_item_check(&mut app, true));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.hp, max_hp, "no window, no tax — the buff re-arms fresh");
        assert!(p.buffs.find_buff_index(88).is_some());
    }
}
