//! Player-side `Dust.NewDust*` sites, the ItemCheck use-visual DUST-ONLY
//! arms (`Player.cs:44754-45046`, `ItemCheck_EmitUseVisuals`), W4-g17
//! wiring wave 2. These are the arms the gore lane's census listed as
//! "dust-only skips" — they spawn NO gore, only dust. The co-lane census
//! (g66/e3/d51/e1/f3/g15) confirmed they are UNCLAIMED (only the
//! `bug_net.rs` zero-draw comment + `buff_effects_accessory.rs`'s "not
//! ported yet" note reference them), so this lane ports them.
//!
//! WIRED: `world_sim::editing::item_use_visuals_tick` runs these arms in
//! the C# arm order on the `UpdatePlayers` stream (the ItemCheck swap,
//! `Main.cs:17754`), interleaved with the gore lane's 65/3065/enchant-7/
//! 1304 arms (`player_gore_d`). Every gate ALWAYS draws (the C# `if`s run
//! unconditionally when the item-type gate matches).
//!
//! ## Scope — the dust-only arms (source order)
//!
//! | C# site | item | arm |
//! | --- | --- | --- |
//! | `:44756-44765` | 989 | Next(5) gate, type Next(3)→15/57/58, vel×0.2 |
//! | `:44766-44771` | 2880 | Next(2) gate, SelectRandom(226,229), noGravity |
//! | `:44772-44779` | 44/45/103/104, 46 | Next(15) gate, type 14 |
//! | `:44780-44790` | 273/675 | Next(5) type-14 gate + always type-27 |
//! | `:44791-44795` | 723 | Next(2) gate, type 64, noGravity |
//! | `:44818-44827` | 190 / 213/5295 | always type-40 / type-3, noGravity |
//! | `:44828-44835` | 121 | 2× swung-path type-6 perfect dusts |
//! | `:44836-44840` | 122/217 | always type-6, noGravity |
//! | `:44841-44846` | 155 | always type-172, noGravity, vel×0.1 |
//! | `:44847-44852` | 676 | Next(3) gate, type 67, noGravity, vel×0.2 |
//! | `:44853-44859` | 3063 | always type-66, hslToRgb colour, vel/2 |
//! | `:44860-44866` | 3823 | always type-6 direct, noGravity, vel×2, fadeIn 0.9 |
//! | `:44867-44872` | 724 | Next(5) gate, type 67, noGravity, vel×0.2 |
//! | `:44873-44890` | 795-802 | 2× swung-path type-5 + Next(20) type-115 |
//! | `:44891-44907` | 367 | Next(3) type-57 + Next(4) type-43 gates |
//! | `:44918-44924` | frostBurn melee | Next(2) gate, type 135 |
//! | `:44925-45015` | meleeEnchant 1-6/8 | the non-7 enchant rows |
//! | `:45017-45023` | magmaStone melee | Next(3)!=0 gate, type 6 |
//!
//! ## RNG contract (the player tick's `Main.rand` stream)
//!
//! Every `NewDust` draws the 6 default-block internals (position X/Y
//! `Next(w-4)`/`Next(h-4)`, velocity X/Y `Next(-20,21)`, frame row
//! `Next(3)`, scale `Next(-20,21)`), plus the per-type tweak (type 6 hits
//! the `Dust.cs:214` +1 `Next(-10,6)`; types 57/58/64/67/135/169/172 etc.
//! hit their own — the pool owns them). The argument draws (the type
//! pick, the swung-path `NextFloat`) ride BEFORE the internals (C#
//! left-to-right). The post-spawn writes are arithmetic (no draws) except
//! the 3063 `hslToRgb(NextFloat())` colour and the 795-802 scale
//! `NextFloat` + the 121/795-802 swung-path `NextFloat`.
//!
//! ## Documented drops / stand-ins
//!
//! * The `noUseGraphic`/`noMelee` item fields are unported (the itemdata
//!   stand-in: every row noMelee-false); the melee enchant/frostBurn/
//!   magmaStone gates fold to `sItem.melee` (the `player_gore_d`
//!   convention). The `:44913-44917` phase-color AddLight arm is the
//!   light lane's (skipped). The `:45028-45045` mount-62-65 AddLight +
//!   type-306 dust is the mount/light lane's (skipped — no port mount
//!   62-65 visual).
//! * The swung-path arms (121, 795-802) read `GetPointOnSwungItemPath`
//!   (`melee_swing::point_on_swung_item_path`, `Player.cs:44742-44752`)
//!   with the caller's `itemRotation`/`itemLocation`/`itemScale`; the
//!   `gravDir` is the port's normal 1f.
//! * `Main.hslToRgb` (`Main.cs:47890`) is `ui::color::hsl_to_rgb` (the
//!   byte-exact single banker's rounding); the 3063 colour writes the
//!   result as sRGB bytes (alpha 255 — the 3-arg C# overload defaults
//!   `a = byte.MaxValue`).

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The item-use snapshot the dust arms read — the same `itemRectangle` +
/// direction/velocity the gore lane's `ItemUseGoreState` carries, plus the
/// swung-path inputs (the 121/795-802 arms) and the melee enchant flags.
#[derive(Clone, Copy, Debug)]
pub struct ItemUseDustState {
    /// `itemRectangle` top-left + size.
    pub rect_x: f32,
    pub rect_y: f32,
    pub rect_w: i32,
    pub rect_h: i32,
    /// `base.direction` / `base.velocity`.
    pub direction: i32,
    pub vx: f32,
    pub vy: f32,
    /// The swung-path inputs (`GetPointOnSwungItemPath`): the held item's
    /// `itemRotation`, `itemLocation`, the adjusted scale, `gravDir`.
    pub item_rotation: f32,
    pub item_location: [f32; 2],
    pub item_scale: f32,
    pub grav_dir: f32,
    /// `Player.meleeEnchant` (the flask rows), `frostBurn`, `magmaStone`.
    pub melee_enchant: u8,
    pub frost_burn: bool,
    pub magma_stone: bool,
    /// `sItem.melee` (the enchant/frostBurn/magmaStone gates read it).
    pub melee: bool,
}

impl ItemUseDustState {
    fn top_left(&self) -> Vec2 {
        Vec2::new(self.rect_x, self.rect_y)
    }
    /// The `base.velocity.X * 0.2 + direction*3` / `base.velocity.Y * 0.2`
    /// speed the melee dust arms pass (`:44786` etc.).
    fn melee_speed(&self) -> (f32, f32) {
        (
            self.vx * 0.2 + (self.direction * 3) as f32,
            self.vy * 0.2,
        )
    }
}

/// `Player.ItemCheck_EmitUseVisuals`' dust-only item arms BEFORE the gore
/// lane's 65/3065 arms (`Player.cs:44756-44795`): the 989/2880/44-46/103/
/// 104/273/675/723 arms, in the C# arm order. Every gate ALWAYS draws.
pub fn item_use_dust_pre(
    kind: u16,
    state: &ItemUseDustState,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let tl = state.top_left();
    let (w, h) = (state.rect_w, state.rect_h);
    let dir2 = (state.direction * 2) as f32;
    // `:44756-44765` — item 989 (the Enchanted Sword): Next(5) gate, type
    // Next(3)→15/57/58, speedX direction*2, alpha 150, scale 1.3, vel×0.2.
    if kind == 989 && rng.next_max(5) == 0 {
        let ty = match rng.next_max(3) {
            0 => 15,
            1 => 57,
            _ => 58,
        };
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, ty, dir2, 0.0, 150, Color::default(), 1.3);
        dust.slot_mut(idx).velocity *= 0.2;
    }
    // `:44766-44771` — item 2880: Next(2) gate, SelectRandom(226,229),
    // noGravity, vel×0.2.
    if kind == 2880 && rng.next_max(2) == 0 {
        let ty = [226i32, 229][rng.next_max(2) as usize];
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, ty, dir2, 0.0, 150, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.velocity *= 0.2;
        d.no_gravity = true;
    }
    // `:44772-44775` — items 44/45/103/104: Next(15) gate, type 14, scale 1.3.
    if matches!(kind, 44 | 45 | 103 | 104) && rng.next_max(15) == 0 {
        dust.new_dust(rng, ctx, world, tl, w, h, 14, dir2, 0.0, 150, Color::default(), 1.3);
    }
    // `:44776-44779` — item 46: Next(15) gate, type 14, scale 1.3.
    if kind == 46 && rng.next_max(15) == 0 {
        dust.new_dust(rng, ctx, world, tl, w, h, 14, dir2, 0.0, 150, Color::default(), 1.3);
    }
    // `:44780-44790` — items 273/675 (the Night's Edge / True Night's Edge):
    // the Next(5) type-14 gate (scale 1.4) THEN the always type-27 dust
    // (melee speed, alpha 100, scale 1.2, noGravity, vel/2).
    if matches!(kind, 273 | 675) {
        if rng.next_max(5) == 0 {
            dust.new_dust(rng, ctx, world, tl, w, h, 14, dir2, 0.0, 150, Color::default(), 1.4);
        }
        let (sx, sy) = state.melee_speed();
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, 27, sx, sy, 100, Color::default(), 1.2);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity.x /= 2.0;
        d.velocity.y /= 2.0;
    }
    // `:44791-44795` — item 723 (the Terra Blade): Next(2) gate, type 64,
    // alpha 150, scale 1.2, noGravity.
    if kind == 723 && rng.next_max(2) == 0 {
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, 64, 0.0, 0.0, 150, Color::default(), 1.2);
        dust.slot_mut(idx).no_gravity = true;
    }
}

/// `Player.ItemCheck_EmitUseVisuals`' dust-only item arms AFTER the gore
/// lane's 65/3065 arms and BEFORE its enchant-7/1304 arms (`Player.cs:
/// 44818-44987`): the 190/213/5295/121/122/217/155/676/3063/3823/724/
/// 795-802/367 arms + frostBurn + the meleeEnchant 1-6 rows. Every gate
/// ALWAYS draws.
pub fn item_use_dust_post(
    kind: u16,
    state: &ItemUseDustState,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let tl = state.top_left();
    let (w, h) = (state.rect_w, state.rect_h);
    // `:44818-44827` — item 190 (the Blade of Grass) / 213/5295 (the Staff
    // of Regrowth / Acorn Axe): always type-40 / type-3, melee speed,
    // alpha 0, scale 1.2, noGravity. The else-if: 190 first, then 213/5295.
    if kind == 190 {
        let (sx, sy) = state.melee_speed();
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, 40, sx, sy, 0, Color::default(), 1.2);
        dust.slot_mut(idx).no_gravity = true;
    } else if matches!(kind, 213 | 5295) {
        let (sx, sy) = state.melee_speed();
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, 3, sx, sy, 0, Color::default(), 1.2);
        dust.slot_mut(idx).no_gravity = true;
    }
    // `:44828-44835` — item 121 (the Muramasa): 2× swung-path type-6 perfect
    // dusts, the outward direction rotated PI/2*direction*gravDir ×4,
    // alpha 100, scale 2.5, noGravity.
    if kind == 121 {
        for _ in 0..2 {
            let t = 0.2 + 0.8 * rng.next_float();
            let (loc, outward) = crate::melee_swing::point_on_swung_item_path(
                70.0, 70.0, t, state.item_scale, state.item_rotation,
                state.item_location, state.direction, state.grav_dir, 0.0,
            );
            let angle = std::f32::consts::FRAC_PI_2 * state.direction as f32 * state.grav_dir;
            let vel = rotate(Vec2::new(outward[0], outward[1]), angle) * 4.0;
            let idx = dust.new_dust_perfect(
                rng, ctx, world, Vec2::new(loc[0], loc[1]), 6, Some(vel), 100, Color::default(), 2.5,
            );
            dust.slot_mut(idx).no_gravity = true;
        }
    }
    // `:44836-44840` — items 122/217 (the Fiery Greatsword / Volcano): always
    // type-6, melee speed, alpha 100, scale 1.9, noGravity.
    if matches!(kind, 122 | 217) {
        let (sx, sy) = state.melee_speed();
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, 6, sx, sy, 100, Color::default(), 1.9);
        dust.slot_mut(idx).no_gravity = true;
    }
    // `:44841-44846` — item 155 (the Light's Bane): always type-172, melee
    // speed, alpha 100, scale 0.9, noGravity, vel×0.1.
    if kind == 155 {
        let (sx, sy) = state.melee_speed();
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, 172, sx, sy, 100, Color::default(), 0.9);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 0.1;
    }
    // `:44847-44852` — item 676 (the True Excalibur): Next(3) gate, type 67,
    // melee speed, alpha 90, scale 1.5, noGravity, vel×0.2.
    if kind == 676 && rng.next_max(3) == 0 {
        let (sx, sy) = state.melee_speed();
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, 67, sx, sy, 90, Color::default(), 1.5);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 0.2;
    }
    // `:44853-44859` — item 3063: always type-66, alpha 150,
    // Color.Transparent, scale 0.85, then the hslToRgb(NextFloat(),1,0.5)
    // colour, noGravity, vel/2.
    if kind == 3063 {
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, 66, 0.0, 0.0, 150, Color::TRANSPARENT, 0.85);
        let hue = rng.next_float();
        // `Main.hslToRgb(NextFloat(), 1, 0.5)` — the byte-exact UI helper
        // (single banker's rounding, `ui/color.rs:85`), alpha 255 (the
        // 3-arg C# overload defaults `a = byte.MaxValue`).
        let rgb = crate::ui::color::hsl_to_rgb(hue as f64, 1.0, 0.5, 255);
        let d = dust.slot_mut(idx);
        d.color = Color::new(rgb[0], rgb[1], rgb[2], rgb[3]);
        d.no_gravity = true;
        d.velocity /= 2.0;
    }
    // `:44860-44866` — item 3823: always type-6 DIRECT, melee speed, alpha
    // 100, Color.Transparent, scale 0.7, noGravity, vel×2, fadeIn 0.9.
    if kind == 3823 {
        let (sx, sy) = state.melee_speed();
        let idx = dust.new_dust_direct(rng, ctx, world, tl, w, h, 6, sx, sy, 100, Color::TRANSPARENT, 0.7);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 2.0;
        d.fade_in = 0.9;
    }
    // `:44867-44872` — item 724 (the Ice Blade): Next(5) gate, type 67,
    // melee speed, alpha 90, scale 1.5, noGravity, vel×0.2.
    if kind == 724 && rng.next_max(5) == 0 {
        let (sx, sy) = state.melee_speed();
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, 67, sx, sy, 90, Color::default(), 1.5);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 0.2;
    }
    // `:44873-44890` — items 795-802: 2× swung-path type-5 perfect dusts
    // (scale 0.7+NextFloat*0.6) + the Next(20) type-115 gate (melee speed,
    // alpha 140, scale 0.7, position overwritten to the path point, fadeIn
    // 1.2, noGravity, vel×0.25 + vector*5).
    if (795..=802).contains(&kind) {
        for _ in 0..2 {
            let t = 0.2 + 0.8 * rng.next_float();
            let (loc, outward) = crate::melee_swing::point_on_swung_item_path(
                60.0, 60.0, t, state.item_scale, state.item_rotation,
                state.item_location, state.direction, state.grav_dir, 0.0,
            );
            let angle = std::f32::consts::FRAC_PI_2 * state.direction as f32 * state.grav_dir;
            let vector = rotate(Vec2::new(outward[0], outward[1]), angle);
            let scale = 0.7 + rng.next_float() * 0.6;
            dust.new_dust_perfect(
                rng, ctx, world, Vec2::new(loc[0], loc[1]), 5, Some(vector * 2.0), 100, Color::default(), scale,
            );
            if rng.next_max(20) == 0 {
                let (sx, sy) = state.melee_speed();
                let idx = dust.new_dust(rng, ctx, world, tl, w, h, 115, sx, sy, 140, Color::default(), 0.7);
                let d = dust.slot_mut(idx);
                d.position = Vec2::new(loc[0], loc[1]);
                d.fade_in = 1.2;
                d.no_gravity = true;
                d.velocity *= 0.25;
                d.velocity += vector * 5.0;
            }
        }
    }
    // `:44891-44907` — item 367: the Next(3) type-57 gate (melee speed,
    // alpha 100, scale 1.1, noGravity, vel/2, +direction*2 X) THEN the
    // Next(4) type-43 gate (alpha 254, scale 0.3, vel×0). The `else if
    // (type != 368)` is a no-op (the decompiler's dead `_ = sItem.type`).
    if kind == 367 {
        if rng.next_max(3) == 0 {
            let (sx, sy) = state.melee_speed();
            let idx = dust.new_dust(rng, ctx, world, tl, w, h, 57, sx, sy, 100, Color::default(), 1.1);
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity.x /= 2.0;
            d.velocity.y /= 2.0;
            d.velocity.x += (state.direction * 2) as f32;
        }
        if rng.next_max(4) == 0 {
            let idx = dust.new_dust(rng, ctx, world, tl, w, h, 43, 0.0, 0.0, 254, Color::default(), 0.3);
            dust.slot_mut(idx).velocity *= 0.0;
        }
    }
    // (The `:44913-44917` phase-color AddLight arm is the light lane's.)
    // `:44918-44924` — frostBurn melee: Next(2) gate, type 135, melee speed,
    // alpha 100, scale 2.5, noGravity, vel×0.7, vel.Y -= 0.5.
    if state.frost_burn && state.melee && rng.next_max(2) == 0 {
        let (sx, sy) = state.melee_speed();
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, 135, sx, sy, 100, Color::default(), 2.5);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 0.7;
        d.velocity.y -= 0.5;
    }
    // `:44925-44987` — the meleeEnchant ladder's 1-6 rows (the gore lane
    // owns 7, which runs after this in the caller; 8 runs in the tail).
    if state.melee && state.melee_enchant > 0 && state.melee_enchant <= 6 {
        enchant_dust_rows(state, dust, ctx, rng, world);
    }
}

/// `Player.ItemCheck_EmitUseVisuals`' dust-only TAIL after the gore lane's
/// enchant-7 arm and BEFORE its 1304 arm (`Player.cs:45009-45023`): the
/// meleeEnchant-8 row + the magmaStone arm. Every gate ALWAYS draws.
pub fn item_use_dust_tail(
    state: &ItemUseDustState,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let tl = state.top_left();
    let (w, h) = (state.rect_w, state.rect_h);
    // `:45009-45015` — enchant 8 (the type-46 row): Next(4) gate, type 46,
    // alpha 100, noGravity, fadeIn 1.5, vel×0.25.
    if state.melee && state.melee_enchant == 8 && rng.next_max(4) == 0 {
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, 46, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.fade_in = 1.5;
        d.velocity *= 0.25;
    }
    // `:45017-45023` — magmaStone melee: Next(3)!=0 gate, type 6, melee
    // speed, alpha 100, scale 2.5, noGravity, vel.X×2, vel.Y×2.
    if state.magma_stone && state.melee && rng.next_max(3) != 0 {
        let (sx, sy) = state.melee_speed();
        let idx = dust.new_dust(rng, ctx, world, tl, w, h, 6, sx, sy, 100, Color::default(), 2.5);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity.x *= 2.0;
        d.velocity.y *= 2.0;
    }
    // (The `:45028-45045` mount-62-65 AddLight + type-306 dust is the
    // mount/light lane's — no port mount 62-65 visual.)
}

/// The meleeEnchant ladder's dust-only rows (`Player.cs:44927-45015`),
/// excluding 7 (the gore lane's `enchant7_use_gore`). Each row's gate
/// ALWAYS draws when the enchant value matches.
fn enchant_dust_rows(
    state: &ItemUseDustState,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let tl = state.top_left();
    let (w, h) = (state.rect_w, state.rect_h);
    let (sx, sy) = state.melee_speed();
    match state.melee_enchant {
        // `:44927-44936` — enchant 1 (Venom): Next(3) gate, type 171, alpha
        // 100, noGravity, fadeIn 1.5, vel×0.25.
        1 => {
            if rng.next_max(3) == 0 {
                let idx = dust.new_dust(rng, ctx, world, tl, w, h, 171, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.fade_in = 1.5;
                d.velocity *= 0.25;
            }
        }
        // `:44937-44946` — enchant 2 (Cursed Flames): Next(2) gate, type 75,
        // melee speed, alpha 100, scale 2.5, noGravity, vel×0.7, vel.Y-=0.5.
        2 => {
            if rng.next_max(2) == 0 {
                let idx = dust.new_dust(rng, ctx, world, tl, w, h, 75, sx, sy, 100, Color::default(), 2.5);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.7;
                d.velocity.y -= 0.5;
            }
        }
        // `:44947-44956` — enchant 3 (Fire): Next(2) gate, type 6, melee
        // speed, alpha 100, scale 2.5, noGravity, vel×0.7, vel.Y-=0.5.
        3 => {
            if rng.next_max(2) == 0 {
                let idx = dust.new_dust(rng, ctx, world, tl, w, h, 6, sx, sy, 100, Color::default(), 2.5);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.7;
                d.velocity.y -= 0.5;
            }
        }
        // `:44957-44967` — enchant 4 (Frost): Next(2) gate, type 57, melee
        // speed, alpha 100, scale 1.1, noGravity, vel/2.
        4 => {
            if rng.next_max(2) == 0 {
                let idx = dust.new_dust(rng, ctx, world, tl, w, h, 57, sx, sy, 100, Color::default(), 1.1);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity.x /= 2.0;
                d.velocity.y /= 2.0;
            }
        }
        // `:44968-44977` — enchant 5 (Confetti? no — the type-169 row):
        // Next(2) gate, type 169, alpha 100, vel.X+=direction, vel.Y+=0.2,
        // noGravity.
        5 => {
            if rng.next_max(2) == 0 {
                let idx = dust.new_dust(rng, ctx, world, tl, w, h, 169, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity.x += state.direction as f32;
                d.velocity.y += 0.2;
                d.no_gravity = true;
            }
        }
        // `:44978-44987` — enchant 6 (the type-135 row): Next(2) gate, type
        // 135, alpha 100, vel.X+=direction, vel.Y+=0.2, noGravity.
        6 => {
            if rng.next_max(2) == 0 {
                let idx = dust.new_dust(rng, ctx, world, tl, w, h, 135, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity.x += state.direction as f32;
                d.velocity.y += 0.2;
                d.no_gravity = true;
            }
        }
        // 7 is the gore lane's (enchant7_use_gore) — skipped here.
        // 8 runs in the tail (item_use_dust_tail), after the gore lane's
        // enchant-7 arm and before its 1304 arm — the C# order.
        _ => {}
    }
}

/// XNA `Vector2.RotatedBy` (`Utils.cs:1697-1700`): the rotation matrix
/// (cos/sin in double of the widened float angle). The swung-path arms
/// rotate the outward direction by `PI/2 * direction * gravDir`.
fn rotate(v: Vec2, angle: f32) -> Vec2 {
    let (s, c) = (angle as f64).sin_cos();
    Vec2::new(
        (v.x as f64 * c - v.y as f64 * s) as f32,
        (v.x as f64 * s + v.y as f64 * c) as f32,
    )
}