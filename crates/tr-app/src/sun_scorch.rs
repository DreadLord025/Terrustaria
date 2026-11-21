//! The vampire seed's sun-scorch system (W4-f30, progression 33 FEAR_THE_SUN).
//!
//! `Player.UpdateSunScorch` (`Player.cs:28626-28670`), its exposure scan
//! `VampireSeedSunlightExposure` (`:28672-28719`) and the counter ramp
//! `UpdateSunScorchValues` (`:28580-28604`). The C# call sits between
//! `UpdateEquips` (`:25633`) and `UpdateArmorSets` (`:25685`) — the port hosts
//! it at the `update_equips` tail in `world_sim/player.rs`, right before the
//! armor-set bonus fold (the Molten `buffImmune[24]` gate reads the flag the
//! scan arms here, `ArmorSetBonuses.cs:285-288`).
//!
//! Deferred halves (cited, docs/ANALYSIS.md "The vampire sun-scorch (W4-f30)"):
//! the sizzle looped sound (`SoundID.VampireSizzle`, `:28590-28603`), the
//! `ParticleOrchestraType.VampireOnFire` request (`:28654-28660`) and the
//! `PlayerFrame()` side effects (`:28644` — the visible-accessory / robe /
//! cape recompute). The progression-33 check reads the effective-armor fold
//! the port's `PlayerFrame` head uses (`:35765-35767`); the `SetMatch` robe
//! rewrites (`:35774-35812`) never touch the vampire set's 124/85/72 ids, so
//! the check's outcome is identical without them.

use tr_core::World;

use crate::npc::Npc;
use crate::player::Player;

/// `SceneMetrics.MoonLordSkyIntensity` (`SceneMetrics.cs:299-310`):
/// `1 - Utils.SmoothStep(3000, 6000, DistanceToMoonLord)` while a Moon Lord
/// (net id 398, the port's `MoonLordCore`) is scanned — `None` otherwise.
/// `DistanceToMoonLord` (`:283-296`) is the Euclidean distance from the scan
/// centre to `ClosestNPCPosition[398]`; the port reads the live NPC list at
/// the player centre (the metrics scan's perspective player).
fn moon_lord_sky_intensity(npcs: &[Npc], cx: f32, cy: f32) -> Option<f32> {
    let mut best: Option<f32> = None;
    for n in npcs {
        if n.hp <= 0 || n.kind != crate::npc::NpcKind::MoonLordCore {
            continue;
        }
        let (nx, ny) = n.center();
        let d = ((nx - cx).powi(2) + (ny - cy).powi(2)).sqrt();
        best = Some(best.map_or(d, |b: f32| b.min(d)));
    }
    // `Utils.SmoothStep` (`Utils.cs:126-129`): the clamped inverse lerp.
    best.map(|d| 1.0 - ((d - 3000.0) / (6000.0 - 3000.0)).clamp(0.0, 1.0))
}

/// `WorldGen.SolidTile3` (`WorldGen.cs:70465-70483`): `nactive() &&
/// tileSolid[type] && !tileSolidTop[type]` — the loosest solid test (the
/// `stray.rs` precedent; the null-tile arm maps to out-of-bounds false).
fn solid_tile3(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && !t.inactive()
        && tr_core::tiledata::TILE_SOLID.has(t.kind)
        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
}

/// `Player.VampireSeedSunlightExposure` (`Player.cs:28672-28719`): the
/// 15-tile sky scan above the player's feet. Sets
/// `vampireBurningInSunlight` when the column is open to the sun.
///
/// `show_invisible` is `Main.ShouldShowInvisibleBlocksAndWalls()` — the port's
/// `TR_SHOW_INVISIBLE` pin plus the Spectre Goggles / biomesight family
/// (`Player.CanSeeInvisibleBlocks`); the caller folds them.
#[allow(clippy::too_many_arguments)]
fn vampire_seed_sunlight_exposure(
    p: &mut Player,
    world: &World,
    npcs: &[Npc],
    vampire_seed: bool,
    day_time: bool,
    raining: bool,
    eclipse: bool,
    zone_graveyard: bool,
    zone_glowshroom: bool,
    show_invisible: bool,
) {
    // `:28674-28675` — the tile column at the centre X, the feet row minus 1.
    let (cx, cy) = p.center();
    let (_, by) = p.bottom();
    let num = (cx as i32) / 16;
    let mut num2 = ((by - 1.0) as i32) / 16;
    // `:28676-28679` — `!Main.vampireSeed` returns.
    if !vampire_seed {
        return;
    }
    // `:28680-28685` — the surface/day/weather/zone gate, then the Moon Lord
    // sky's shade (`> 0.5` intensity cancels the burn).
    let mut flag = (num2 as f64) < world.world_surface
        && day_time
        && !raining
        && !eclipse
        && !zone_graveyard
        && !zone_glowshroom;
    if flag && moon_lord_sky_intensity(npcs, cx, cy).is_some_and(|v| v > 0.5) {
        flag = false;
    }
    // `:28686-28689` — wet players never burn.
    if !flag || p.wet {
        return;
    }
    // `:28690-28698` — the Umbrella (946) / Tragic Umbrella (4707) shade the
    // holder; the Bat mount (56) cancels the shade (it flies above it).
    let mut flag2 = p.held_item_kind == 946 || p.held_item_kind == 4707;
    if p.mount.is_active() && p.mount.kind() == 56 {
        flag2 = false;
    }
    if flag2 {
        return;
    }
    // `:28699-28718` — the 15-step climb. The wall test reads the CURRENT
    // cell; the solid test reads the cell AFTER the decrement (the C# keeps
    // the stale `tile` reference for its type/invisible terms — the port
    // re-reads both cells to mirror it).
    let mut num4 = 15;
    while num4 > 0 {
        let Some(tile) = world.tile(num, num2) else {
            break; // `:28703-28706` — the null tile breaks the climb.
        };
        let wall = tile.wall;
        let kind = tile.kind;
        let invis_block = world.invisible_block(num, num2);
        let invis_wall = world.invisible_wall(num, num2);
        // `:28707-28711` — no wall, glass (21) / echo-coated (318) walls, or
        // an invisible wall while invisibility is hidden: open sky.
        if wall == 0 || wall == 21 || wall == 318 || (!show_invisible && invis_wall) {
            p.vampire_burning_in_sunlight = true;
            break;
        }
        num4 -= 1;
        num2 -= 1;
        // `:28714-28717` — a solid non-platform block above stops the climb,
        // EXCEPT glass (54), invisible blocks while hidden, and the bubble
        // (541) while hidden. The type/invisible terms read the PRE-decrement
        // cell (the C#'s stale `tile`); the SolidTile3 test reads the new one.
        if solid_tile3(world, num, num2)
            && kind != 54
            && (!invis_block || show_invisible)
            && (kind != 541 || show_invisible)
        {
            break;
        }
    }
}

/// `Player.UpdateSunScorchValues` (`Player.cs:28580-28604`): the counter
/// ramps +1 while burning, −6 (−2 dead) otherwise, clamped to [0, 300].
/// The sizzle sound's volume lerp (`:28589-28603`) is the deferred audio half.
pub fn update_sun_scorch_values(p: &mut Player) {
    let mut num = 6;
    if p.dead {
        p.vampire_burning_in_sunlight = false;
        num = 2;
    }
    p.sun_scorch_counter = (p.sun_scorch_counter + if p.vampire_burning_in_sunlight { 1 } else { -num })
        .clamp(0, 300);
}

/// The effective-armor slot fold `PlayerFrame`'s head uses
/// (`Player.cs:35765-35767`): the vanity slot's id when it carries one
/// (`>= 0`), else the functional slot's. The port has no loadouts, so
/// `equipped[slot]` IS the effective kind (the `update_player` armor-tail
/// precedent).
fn effective_slot(p: &Player, slot: usize, pick: fn(&crate::registry::ItemDef) -> i16) -> i32 {
    crate::registry::get(p.equipped[slot]).map_or(-1, |d| pick(d) as i32)
}

/// `Player.UpdateSunScorch` (`Player.cs:28626-28670`) — the local-player
/// per-tick pass. Returns `true` on the tick the counter crosses 120 while
/// the full vampire vanity set (head 124 / body 85 / legs 72) is worn — the
/// caller fires `NotifyProgressionEvent(33)` (`:28647`, FEAR_THE_SUN).
///
/// The C# runs `PlayerFrame()` inside the crossing branch and restores the
/// head/body/legs afterwards (`:28641-28651`); the port reads the effective
/// slots directly (the `SetMatch` rewrites never touch the vampire ids).
#[allow(clippy::too_many_arguments)]
pub fn update_sun_scorch(
    p: &mut Player,
    world: &World,
    npcs: &[Npc],
    vampire_seed: bool,
    day_time: bool,
    raining: bool,
    eclipse: bool,
    zone_graveyard: bool,
    zone_glowshroom: bool,
    show_invisible: bool,
) -> bool {
    // `:28632-28634` — the exposure scan, then the counter ramp (the old
    // counter is captured first, `:28633`).
    vampire_seed_sunlight_exposure(
        p,
        world,
        npcs,
        vampire_seed,
        day_time,
        raining,
        eclipse,
        zone_graveyard,
        zone_glowshroom,
        show_invisible,
    );
    let num = p.sun_scorch_counter;
    update_sun_scorch_values(p);
    // `:28635-28638` — no burn, or below the 120 threshold: nothing fires.
    if !p.vampire_burning_in_sunlight || p.sun_scorch_counter < 120 {
        return false;
    }
    let mut notify33 = false;
    // `:28639-28652` — the crossing tick (old < 120, new >= 120): the
    // vampire-set check fires 33 once.
    if num < 120 {
        let head_v = effective_slot(p, 10, |d| d.head_slot);
        let head = if head_v >= 0 { head_v } else { effective_slot(p, 0, |d| d.head_slot) };
        let body_v = effective_slot(p, 11, |d| d.body_slot);
        let body = if body_v >= 0 { body_v } else { effective_slot(p, 1, |d| d.body_slot) };
        let legs_v = effective_slot(p, 12, |d| d.leg_slot);
        let legs = if legs_v >= 0 { legs_v } else { effective_slot(p, 2, |d| d.leg_slot) };
        if head == 124 && body == 85 && legs == 72 {
            notify33 = true;
        }
    }
    // `:28653` — `Array.Clear(buffImmune, 0, BuffID.Count)`: the burn strips
    // every immunity (the Molten set's fire immunity included).
    p.buffs.reset_buff_immune();
    // `:28654-28660` — the VampireOnFire particle request: cited skip (the
    // particle orchestrator is unported). The `!onFire` gate reads the
    // OnFire! ! [24] buff slot.
    // `:28661-28663` — the three 3-tick debuffs: OnFire 24, Cursed 23, Slow 32.
    p.buffs.add_buff(24, 3);
    p.buffs.add_buff(23, 3);
    p.buffs.add_buff(32, 3);
    // `:28664-28667` — the mount dismounts (the space check applies).
    if p.mount.is_active() {
        p.try_dismount(world);
    }
    // `:28668-28669` — the flight aids die.
    p.wings_logic = 0;
    p.rocket_boots = 0;
    notify33
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    /// A vampire-seed rig: the player standing above the surface in open
    /// daylight (the `World::new` column is air, `world_surface = h/4`).
    fn rig() -> (Player, World) {
        let mut w = World::new("scorch".into(), 64, 200);
        w.micro_seeds
            .insert(tr_core::seeds::MicroSeedFlags::VAMPIRISM);
        let mut p = Player::spawn(&w);
        p.x = 512.0; // centre tile 32
        p.y = 100.0; // feet row 8, above world_surface (50)
        p.wet = false;
        (p, w)
    }

    fn scorch(p: &mut Player, w: &World) -> bool {
        update_sun_scorch(p, w, &[], true, true, false, false, false, false, false)
    }

    /// `:28699-28718` — the open sky arms the burn; the counter ramps +1
    /// per tick and the 120 crossing fires the debuff triple exactly once.
    #[test]
    fn the_open_sky_burns_and_the_120_crossing_arms_the_debuffs() {
        let (mut p, w) = rig();
        for _ in 0..119 {
            assert!(!scorch(&mut p, &w), "no notify before the crossing");
        }
        assert!(p.vampire_burning_in_sunlight);
        assert_eq!(p.sun_scorch_counter, 119);
        assert!(p.buffs.find_buff_index(24).is_none(), "no debuffs yet");
        // The 120th tick crosses: the three 3-tick debuffs land, the
        // immunities clear, the flight aids die.
        p.wings_logic = 5;
        p.rocket_boots = 2;
        p.buffs.buff_immune[24] = true;
        assert!(!scorch(&mut p, &w), "no vampire set worn — no 33");
        assert_eq!(p.sun_scorch_counter, 120);
        assert!(p.buffs.find_buff_index(24).is_some(), "OnFire 24");
        assert!(p.buffs.find_buff_index(23).is_some(), "Cursed 23");
        assert!(p.buffs.find_buff_index(32).is_some(), "Slow 32");
        assert!(!p.buffs.buff_immune[24], "the Array.Clear strips it");
        assert_eq!(p.wings_logic, 0);
        assert_eq!(p.rocket_boots, 0);
        // The next tick is past the crossing — no second notify.
        assert!(!scorch(&mut p, &w));
        assert_eq!(p.sun_scorch_counter, 121);
    }

    /// `:28641-28651` — the crossing tick with the vampire vanity set
    /// (1763/1764/1765 → head 124 / body 85 / legs 72) fires 33.
    #[test]
    fn the_vampire_set_on_the_crossing_tick_fires_33() {
        let (mut p, w) = rig();
        p.equipped[0] = 1763; // Vampire Mask, head_slot 124
        p.equipped[1] = 1764; // Vampire Shirt, body_slot 85
        p.equipped[2] = 1765; // Vampire Pants, leg_slot 72
        p.sun_scorch_counter = 119;
        assert!(scorch(&mut p, &w), "the crossing + the set fires 33");
        // A missing piece refuses.
        let (mut q, w2) = rig();
        q.equipped[0] = 1763;
        q.equipped[1] = 1764;
        q.sun_scorch_counter = 119;
        assert!(!scorch(&mut q, &w2), "two of three refuses");
    }

    /// `:28681` — night, rain, eclipse, graveyard, glowshroom and the
    /// below-surface row each refuse the burn; `:28686` wet refuses.
    #[test]
    fn the_weather_and_zone_gates_refuse_the_burn() {
        let (mut p, w) = rig();
        // Night.
        update_sun_scorch(&mut p, &w, &[], true, false, false, false, false, false, false);
        assert!(!p.vampire_burning_in_sunlight);
        // Rain.
        update_sun_scorch(&mut p, &w, &[], true, true, true, false, false, false, false);
        assert!(!p.vampire_burning_in_sunlight);
        // Eclipse.
        update_sun_scorch(&mut p, &w, &[], true, true, false, true, false, false, false);
        assert!(!p.vampire_burning_in_sunlight);
        // Graveyard / glowshroom.
        update_sun_scorch(&mut p, &w, &[], true, true, false, false, true, false, false);
        assert!(!p.vampire_burning_in_sunlight);
        update_sun_scorch(&mut p, &w, &[], true, true, false, false, false, true, false);
        assert!(!p.vampire_burning_in_sunlight);
        // Wet.
        p.wet = true;
        assert!(!scorch(&mut p, &w));
        assert!(!p.vampire_burning_in_sunlight);
        // Below the surface.
        let (mut q, w2) = rig();
        q.y = 1000.0; // feet row 65 > world_surface 50
        assert!(!scorch(&mut q, &w2));
        assert!(!q.vampire_burning_in_sunlight);
        // The non-vampire world never burns.
        let (mut r, w3) = rig();
        update_sun_scorch(&mut r, &w3, &[], false, true, false, false, false, false, false);
        assert!(!r.vampire_burning_in_sunlight);
    }

    /// `:28690-28698` — the Umbrella (946) / Tragic Umbrella (4707) shade
    /// the holder; the Bat mount (56) cancels the shade. The witness drives
    /// the shade predicate directly (the mount's private fields are the
    /// `mount.rs` animator's; `set_mount`'s box growth would move the scan
    /// column out from under the rig).
    #[test]
    fn the_umbrella_shades_and_the_bat_mount_cancels_it() {
        let (mut p, w) = rig();
        p.held_item_kind = 946;
        assert!(!scorch(&mut p, &w));
        assert!(!p.vampire_burning_in_sunlight);
        p.held_item_kind = 4707;
        assert!(!scorch(&mut p, &w));
        assert!(!p.vampire_burning_in_sunlight);
        // The shade predicate (`:28690-28694`): umbrella && !bat-mount.
        let shade = |held: u16, mount_active: bool, mount_kind: i32| {
            let mut f = held == 946 || held == 4707;
            if mount_active && mount_kind == 56 {
                f = false;
            }
            f
        };
        assert!(shade(946, false, -1), "the umbrella shades on foot");
        assert!(!shade(946, true, 56), "the bat cancels the shade");
        assert!(shade(946, true, 23), "other mounts keep the shade");
        assert!(!shade(0, false, -1), "no umbrella, no shade");
    }

    /// `:28707-28717` — a stone wall blocks the sky; glass (21) / echo (318)
    /// walls and a hidden invisible wall stay open; a solid block above
    /// stops the climb, glass (54) does not.
    #[test]
    fn the_wall_and_block_scan_follows_the_source() {
        // A stone wall at the feet row blocks.
        let (mut p, mut w) = rig();
        let (_, by) = p.bottom();
        let row = ((by - 1.0) as i32) / 16;
        w.tile_mut(32, row).unwrap().wall = 1;
        assert!(!scorch(&mut p, &w));
        assert!(!p.vampire_burning_in_sunlight);
        // Glass (21) and echo (318) walls stay open.
        for wall in [21u16, 318] {
            let (mut q, mut wq) = rig();
            wq.tile_mut(32, row).unwrap().wall = wall;
            assert!(!scorch(&mut q, &wq));
            assert!(q.vampire_burning_in_sunlight, "wall {wall} is transparent");
        }
        // A hidden invisible wall stays open; showing it blocks.
        let (mut r, mut wr) = rig();
        wr.set_invisible_wall(32, row, true);
        wr.tile_mut(32, row).unwrap().wall = 1;
        assert!(!scorch(&mut r, &wr));
        assert!(r.vampire_burning_in_sunlight, "hidden invisible wall is open");
        update_sun_scorch(&mut r, &wr, &[], true, true, false, false, false, false, true);
        assert!(!r.vampire_burning_in_sunlight, "shown invisible wall blocks");
        // A solid stone block two rows up stops the climb (no burn).
        let (mut s, mut ws) = rig();
        let t = ws.tile_mut(32, row - 2).unwrap();
        t.kind = 1; // Stone
        t.flags = FLAG_ACTIVE;
        assert!(!scorch(&mut s, &ws));
        assert!(!s.vampire_burning_in_sunlight);
        // A glass block (54) does not stop it.
        let (mut u, mut wu) = rig();
        let t = wu.tile_mut(32, row - 2).unwrap();
        t.kind = 54;
        t.flags = FLAG_ACTIVE;
        assert!(!scorch(&mut u, &wu));
        assert!(u.vampire_burning_in_sunlight, "glass is transparent");
    }

    /// `:28580-28604` — the counter ramps −6 alive / −2 dead, clamped at 0;
    /// the dead tick clears the burn flag. The cap is 300.
    #[test]
    fn the_counter_ramp_and_cap_follow_the_source() {
        let (mut p, _w) = rig();
        p.sun_scorch_counter = 10;
        update_sun_scorch_values(&mut p);
        assert_eq!(p.sun_scorch_counter, 4, "−6 alive");
        p.dead = true;
        p.vampire_burning_in_sunlight = true;
        update_sun_scorch_values(&mut p);
        assert!(!p.vampire_burning_in_sunlight, "the dead tick clears it");
        assert_eq!(p.sun_scorch_counter, 2, "−2 dead");
        update_sun_scorch_values(&mut p);
        assert_eq!(p.sun_scorch_counter, 0, "clamped at 0");
        // The burning ramp caps at 300.
        let (mut q, _w2) = rig();
        q.sun_scorch_counter = 299;
        q.vampire_burning_in_sunlight = true;
        update_sun_scorch_values(&mut q);
        assert_eq!(q.sun_scorch_counter, 300);
        update_sun_scorch_values(&mut q);
        assert_eq!(q.sun_scorch_counter, 300, "clamped at 300");
    }

    /// `SceneMetrics.cs:299-310` — a Moon Lord within 4500 px shades the
    /// player (intensity > 0.5); beyond 6000 px the shade lifts.
    #[test]
    fn the_moon_lord_sky_shades_the_burn() {
        let (mut p, w) = rig();
        let (cx, cy) = p.center();
        let mut moon = Npc::new(crate::npc::NpcKind::MoonLordCore, cx + 4000.0, cy, 1, 0.0);
        moon.hp = moon.max_hp;
        // intensity = 1 - (4000-3000)/3000 = 0.667 > 0.5 → shaded.
        update_sun_scorch(&mut p, &w, &[moon], true, true, false, false, false, false, false);
        assert!(!p.vampire_burning_in_sunlight, "the close Moon Lord shades");
        // Far away: intensity 0 → the burn arms.
        let mut far = Npc::new(crate::npc::NpcKind::MoonLordCore, cx + 7000.0, cy, 1, 0.0);
        far.hp = far.max_hp;
        update_sun_scorch(&mut p, &w, &[far], true, true, false, false, false, false, false);
        assert!(p.vampire_burning_in_sunlight, "the far Moon Lord does not shade");
    }
}
