//! The Torch God event + the Torch God's Favor grant (W4-f33).
//!
//! The C# has no `TorchGod.cs` / `TorchGodPunishment` files — the event is
//! the `Player.happyFunTorchTime` machine inline in `Player.cs`:
//! - the trigger tail of `UpdateTorchLuck_ConsumeCountersAndCalculate`
//!   (`Player.cs:18066-18086`): cooldown 0, biome torches locked, >100 lit
//!   torches nearby, below the surface, no favor (5043) anywhere in the
//!   inventory → the event arms (`happyFunTorchTime = true`, attacks 0);
//! - `TryRecalculatingTorchLuck`'s head (`:17731-17742`): while the event
//!   runs the scan cursor zeroes and `TorchAttack` runs INSTEAD of the scan;
//!   otherwise `torchGodCooldown` decrements;
//! - `TorchAttack` (`:17813-17910`): the 21-tick cadence (`torchFunTimer`
//!   counts 0→21), the ±100-tile lit-torch scan into the static 400-slot
//!   candidate arrays, the ONE `Main.rand.Next(num)` pick per volley, the
//!   torch snuff (`frameX += 66`) + the 949 projectile launch, the survival
//!   grant (`numberOfTorchAttacksMade >= 95` → `RequestNewItem(5043)`) and
//!   the 200-attack cap (`maxTorchAttacks`, `:57128`);
//! - `RelightTorches` (`:17797-17811`): the 3600-tick cooldown + the
//!   unlit-list relight (`frameX -= 66`);
//! - `EndOngoingTorchGodEvent` (`:17718-17729`): the death/surface abort.
//!
//! The projectile side is `AI_184_BadTorchLuck` (`Projectile.cs:42787-42822`)
//! + `SelfHurtPlayers` (`:15025-15062`) + the 949 `Kill()` dust census
//! (`:70976-70995`) — see `projectile_ai_torch_god.rs` / `projectile.rs`.
//!
//! Ownership (W4 coordination): this task owns the EVENT + the favor ITEM
//! grant; the favor's power-up USE arm (`ItemCheck_UseTorchGodsFavor`,
//! `Player.cs:45226-45243`) is a6's; the torch-luck scan itself is e27's —
//! the seam here only ADDS the C# event head/trigger tail around the
//! existing `luck.rs` scan (`torch_scan_step_event`), no scan behaviour
//! changes (the plain `torch_scan_step` delegates with no event context).
//!
//! RNG contract: every draw is `Main.rand` under the `UpdatePlayers` swap
//! (`Main.cs:17754` — `TorchAttack` runs inside `Player.Update`): exactly
//! ONE `Next(num)` per volley (`:17881`), the grant's `Item.NewItem`
//! velocity pair `Next(-30,31)` / `Next(-40,-15)` (`Item.cs:49749-49750`,
//! `Prefix(0)` draws NOTHING, `:511-514`). ALL draws happen IN-TICK inside
//! `update_luck` (the C# call position); only the tile writes / the pool
//! spawn / the item-vec push defer to the `world_sim` drain (the port's
//! `pending_*` convention — the player tick holds `&World`, not `&mut`).
//! The dust censuses' `Next(3)` rolls ride the `UpdateProjectiles` swap
//! (`Main.cs:17775`).

use tr_core::loot::DropRng;
use tr_core::world::World;

use crate::luck::LuckState;

/// `Player.maxTorchAttacks` (`Player.cs:3074`, set 200 at `:57128`).
pub const MAX_TORCH_ATTACKS: usize = 200;
/// `Player._torchAttackPosX`'s length (`Player.cs:57129`: `new int[400]`).
pub const TORCH_ATTACK_CANDIDATES: usize = 400;
/// The survival threshold (`Player.cs:17870`/`:17904`: `>= 95`).
pub const FAVOR_SURVIVAL_ATTACKS: i32 = 95;
/// `RelightTorches`' cooldown (`Player.cs:17799`: 3600 ticks = 1 minute).
pub const TORCH_GOD_COOLDOWN: i32 = 3600;
/// `ItemID.TorchGodsFavor` (`ItemID.cs:11769`).
pub const TORCH_GODS_FAVOR: u16 = 5043;
/// `ProjectileID.TorchGod` (`ProjectileID.cs:2452`).
pub const TORCH_GOD_PROJECTILE: u16 = 949;

/// The Torch God event state — the `Player.cs:3066-3080` field block.
/// Lives on `Player` beside `luck_state` (the scan state stays e27's).
#[derive(Debug, Clone, PartialEq)]
pub struct TorchGodState {
    /// `Player.happyFunTorchTime` (`:3066`) — the event is running.
    pub happy_fun_torch_time: bool,
    /// `Player.torchFunTimer` (`:3068`) — the volley cadence counter
    /// (fires when it passes 20; the finale arms −180).
    pub torch_fun_timer: i32,
    /// `Player.torchGodCooldown` (`:3070`) — decrements once per
    /// `TryRecalculatingTorchLuck` call while the event is NOT running.
    pub torch_god_cooldown: i32,
    /// `Player.numberOfTorchAttacksMade` (`:3072`).
    pub number_of_torch_attacks_made: i32,
    /// `Player.unlitTorchX` (`:3076`) — the snuffed torches' columns.
    pub unlit_torch_x: [i32; MAX_TORCH_ATTACKS],
    /// `Player.unlitTorchY` (`:3078`).
    pub unlit_torch_y: [i32; MAX_TORCH_ATTACKS],
}

impl Default for TorchGodState {
    fn default() -> Self {
        Self {
            happy_fun_torch_time: false,
            torch_fun_timer: 0,
            torch_god_cooldown: 0,
            number_of_torch_attacks_made: 0,
            unlit_torch_x: [0; MAX_TORCH_ATTACKS],
            unlit_torch_y: [0; MAX_TORCH_ATTACKS],
        }
    }
}

/// One volley's launch request — the `Projectile.NewProjectile` call at
/// `Player.cs:17900-17903` (the world_sim seam owns the pool). The unlit
/// list write already landed inside [`torch_attack_step`]; the tile snuff
/// rides [`TorchGodPending::snuff`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TorchLaunch {
    /// The spawn point `(torchX * 16 + 8, torchY * 16)` (`:17895`).
    pub x: f32,
    pub y: f32,
    /// The unit aim (player center − spawn) × 8 (`:17896-17899`).
    pub vx: f32,
    pub vy: f32,
    /// `(int)(40f * HostileProjectileDamageMultiplier.Sample(difficulty))`
    /// (`:17887`) — Journey 20 / Classic 40 / Expert 80 / Master 120.
    pub damage: i32,
    /// `ai[0]` = `TorchID.Dust[clamp(frameY / 22, 0, 23)]` (`:17888-17889`).
    pub dust_id: i32,
    /// `ai[1]` = the pre-normalize distance (`:17897`, `num9`).
    pub distance: f32,
}

/// The tick's deferred Torch God writes — the `pending_*` convention: the
/// decision + ALL RNG run in-tick inside `update_luck` (the C# call
/// position, `Player.cs:25645`); the tile mutations / the pool spawn / the
/// item push land at the `world_sim` drain right after the player tick
/// (the same fixed step — the next tick's scans see them, like the C#).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TorchGodPending {
    /// The volley's torch snuff (`Player.cs:17890`: `frameX += 66`).
    pub snuff: Option<(i32, i32)>,
    /// `RelightTorches`' tile loop is due (`:17800-17810`) — the cooldown
    /// already armed in-tick; the drain walks the unlit list.
    pub relight: bool,
    /// The 949 launch (`:17900-17903`).
    pub launch: Option<TorchLaunch>,
    /// The survival grant (`:17872`): (drop center, velocity X, velocity
    /// Y) — the two `Item.NewItem` draws already consumed in-tick.
    pub favor_drop: Option<((f32, f32), f32, f32)>,
}

/// `GameDifficultyData.HostileProjectileDamageMultiplier`
/// (`GameDifficultyData.cs:67`/`:85`): `LinearCurve((Journey 0.5, 0.5),
/// (Master 3, 3))` — a straight line through the origin, so
/// `Sample(d) == d` for every difficulty in 0.5..=3 (and the Legendary 4
/// extrapolation keeps the identity). `LinearCurve.Sample`
/// (`GameDifficultyData.cs:35-55`) clamps below the first key (0.5) and
/// extrapolates past the last with the same segment slope (1).
pub fn hostile_projectile_damage_multiplier(difficulty: f32) -> f32 {
    if difficulty <= 0.5 {
        return 0.5;
    }
    // (value − 0.5) * (3 − 0.5) / (3 − 0.5) + 0.5 — the C#'s f32 arithmetic.
    (difficulty - 0.5) * 2.5 / 2.5 + 0.5
}

/// The volley damage (`Player.cs:17887`):
/// `(int)(40f * HostileProjectileDamageMultiplier.Sample(Main.Difficulty))`.
/// The port is single-difficulty Classic (`Main.Difficulty = 1`,
/// `GameDifficultyLevel.cs:18` — the `World::MOON_POINT_SCALAR` fold), so
/// play reads 40; the parameter keeps the curve witnessable.
pub fn torch_attack_damage(difficulty: f32) -> i32 {
    (40.0 * hostile_projectile_damage_multiplier(difficulty)) as i32
}

/// `Player.EndOngoingTorchGodEvent` (`Player.cs:17718-17729`): while the
/// event runs, relight every snuffed torch, drop the flag. The
/// `NetMessage.SendData(4)` tail is net-only. Callers: `TorchAttack`'s
/// surface abort (`:17821`) and `KillMe` (`:38857`). The tile loop defers
/// (the return value feeds [`TorchGodPending::relight`]); the cooldown
/// arms here, in C# order (`RelightTorches` sets it FIRST, `:17799`).
pub fn end_ongoing_torch_god_event(state: &mut TorchGodState) -> bool {
    if state.happy_fun_torch_time {
        state.torch_god_cooldown = TORCH_GOD_COOLDOWN;
        state.happy_fun_torch_time = false;
        return true;
    }
    false
}

/// `Player.RelightTorches`' tile loop (`Player.cs:17800-17810`): walk the
/// unlit list in attack order — an active torch tile
/// (`TileID.Sets.Torches` = {4}, `TileID.cs:615`) whose `frameX >= 66`
/// gets `frameX -= 66`. The `SendTileSquare` per relight is net-only.
/// The drain calls this AFTER the in-tick cooldown write.
pub fn apply_relight(state: &TorchGodState, world: &mut World) {
    for i in 0..state.number_of_torch_attacks_made as usize {
        let (x, y) = (state.unlit_torch_x[i], state.unlit_torch_y[i]);
        if let Some(t) = world.tile_mut(x, y) {
            if t.active() && tr_core::tiledata::SET_TORCHES.has(t.kind) && t.frame_x >= 66 {
                t.frame_x -= 66;
            }
        }
    }
}

/// `Utils.Clamp` (`Utils.cs:223-234`) — the max test FIRST, so a world
/// narrower than the margin folds onto the max (the `luck.rs` scan's
/// recorded fold, kept identical here).
fn utils_clamp(value: i32, min: i32, max: i32) -> i32 {
    if value > max {
        max
    } else if value < min {
        min
    } else {
        value
    }
}

/// `Player.TorchAttack` (`Player.cs:17813-17910`) — one call per
/// `TryRecalculatingTorchLuck` tick while the event runs. The
/// `whoAmI != Main.myPlayer` head (`:17815`) folds away (single player).
/// `center` is `base.Center`; `pos_y` is `base.position.Y` (the surface
/// gate reads the FEET, not the center). `rng` is the `UpdatePlayers`
/// stream — exactly one `next_range(0, num)` per volley (`:17881`) plus
/// the grant's velocity pair when the survival arm fires.
///
/// The state writes (the unlit list, the counters, the cooldown) land
/// here; the tile snuff / relight, the projectile spawn and the favor
/// drop ride the pending struct (the player tick holds `&World`).
pub fn torch_attack_step(
    state: &mut TorchGodState,
    buffs: &mut crate::buffs::BuffSlots,
    world: &World,
    pos_y: f32,
    center: (f32, f32),
    difficulty: f32,
    rng: &mut dyn DropRng,
) -> TorchGodPending {
    let mut out = TorchGodPending::default();
    // `:17819-17823` — surfacing ends the event (relight + cooldown).
    if (pos_y as f64) < world.world_surface * 16.0 {
        out.relight = end_ongoing_torch_god_event(state);
        return out;
    }
    // `:17824` — `AddBuff(80, 2)`: the Blackout indicator (`BuffID.cs:331`),
    // re-granted every event tick.
    buffs.add_buff(80, 2);
    // `:17825-17830` — the 21-tick cadence: ++ then `<= 20` returns, so the
    // first volley lands on tick 21 (the finale's −180 arms a 201-tick gap).
    state.torch_fun_timer += 1;
    if state.torch_fun_timer <= 20 {
        return out;
    }
    state.torch_fun_timer = 0;
    // `:17831-17861` — the candidate scan: x OUTER, y inner, the ±100-tile
    // box clamped 10..max−10 (`Utils.Clamp`'s max-first fold), lit torches
    // only (`active & type == 4 & frameX < 66`), capped at the 400-slot
    // static arrays (the inner break AND the outer break, `:17851-17860`).
    let mut cand_x = [0i32; TORCH_ATTACK_CANDIDATES];
    let mut cand_y = [0i32; TORCH_ATTACK_CANDIDATES];
    let mut num = 0usize;
    let x_lo = utils_clamp(center.0 as i32 / 16 - 100, 10, world.width - 10);
    let x_hi = utils_clamp(center.0 as i32 / 16 + 100, 10, world.width - 10);
    let y_lo = utils_clamp(center.1 as i32 / 16 - 100, 10, world.height - 10);
    let y_hi = utils_clamp(center.1 as i32 / 16 + 100, 10, world.height - 10);
    'outer: for i in x_lo..=x_hi {
        for j in y_lo..=y_hi {
            if let Some(t) = world.tile(i, j) {
                if t.active() && t.kind == 4 && t.frame_x < 66 {
                    cand_x[num] = i;
                    cand_y[num] = j;
                    num += 1;
                    if num >= TORCH_ATTACK_CANDIDATES {
                        break 'outer;
                    }
                }
            }
        }
    }
    // `:17862-17874` — no candidates OR the 200-attack cap: the event ends
    // (relight + flag clear; NO SendData in single-player), and surviving
    // ≥ 95 attacks grants the favor at the player center.
    if num == 0 || state.number_of_torch_attacks_made >= MAX_TORCH_ATTACKS as i32 {
        state.torch_god_cooldown = TORCH_GOD_COOLDOWN;
        out.relight = true;
        state.happy_fun_torch_time = false;
        if state.number_of_torch_attacks_made >= FAVOR_SURVIVAL_ATTACKS {
            // `Item.RequestNewItem(..., base.Center, 5043, 1, 0,
            // ReserveForLocalPlayer)` (`:17872` → `Item.cs:49639-49646` →
            // `NewItem` `:49654`): `Prefix(0)` draws NOTHING
            // (`Item.cs:511-514`), then the velocity pair
            // `Next(-30, 31) * 0.1` / `Next(-40, -15) * 0.1`
            // (`:49749-49750` — 5043 is not in `ItemID.Sets.ItemNoGravity`,
            // `ItemID.cs:1320`). The draws run HERE (the C# call position);
            // the drop push defers. The port's generic `spawn_item_drop_px`
            // pins a DIFFERENT order (the −1 natural roll + the phase
            // triple, ANALYSIS "Item drop RNG") — this grant keeps the C#
            // call's own two draws.
            let vx = rng.next_range(-30, 31) as f32 * 0.1;
            let vy = rng.next_range(-40, -15) as f32 * 0.1;
            out.favor_drop = Some((center, vx, vy));
        }
        return out;
    }
    // `:17881` — THE volley draw: one `Main.rand.Next(num)` on the
    // UpdatePlayers stream, before the tile re-validation.
    let pick = rng.next_range(0, num as i32) as usize;
    let (tx, ty) = (cand_x[pick], cand_y[pick]);
    // `:17884` — re-validate the picked cell (it cannot have changed since
    // the scan in the same call, kept 1:1 anyway).
    let Some(tile) = world.tile(tx, ty) else {
        return out;
    };
    if tile.kind != 4 || tile.frame_x >= 66 {
        return out;
    }
    let style = utils_clamp(tile.frame_y as i32 / 22, 0, crate::luck::TORCH_ID_COUNT as i32 - 1);
    let dust_id = crate::projectile_ai_torch_god::TORCH_DUST[style as usize];
    // `:17890-17893` — snuff the torch (the write defers), record it for
    // the relight.
    out.snuff = Some((tx, ty));
    let made = state.number_of_torch_attacks_made as usize;
    state.unlit_torch_x[made] = tx;
    state.unlit_torch_y[made] = ty;
    state.number_of_torch_attacks_made += 1;
    // `:17895-17899` — the spawn point is the torch cell's TOP-center
    // `(x*16+8, y*16)`; the aim is (player center − spawn), its length kept
    // as `ai[1]`, then normalized × 8.
    let sx = tx as f32 * 16.0 + 8.0;
    let sy = ty as f32 * 16.0;
    let (aim, distance) = crate::projectile_ai_torch_god::launch_aim(
        glam::Vec2::new(sx, sy),
        glam::Vec2::new(center.0, center.1),
    );
    out.launch = Some(TorchLaunch {
        x: sx,
        y: sy,
        vx: aim.x,
        vy: aim.y,
        damage: torch_attack_damage(difficulty),
        dust_id,
        distance,
    });
    // `:17904-17907` — the finale: the LAST candidate consumed at ≥ 95
    // attacks, or the 200-cap reached → −180 (the next volley waits 201
    // ticks, then the cap/no-candidate arm ends the event).
    if (num == 1 && state.number_of_torch_attacks_made >= FAVOR_SURVIVAL_ATTACKS)
        || state.number_of_torch_attacks_made >= MAX_TORCH_ATTACKS as i32
    {
        state.torch_fun_timer = -180;
    }
    out
}

/// The event context `luck.rs`'s scan seam threads through — the player
/// fields the C# head/trigger read (`TryRecalculatingTorchLuck` /
/// `UpdateTorchLuck_ConsumeCountersAndCalculate` are `Player` methods).
pub struct TorchGodEventCtx<'a> {
    pub state: &'a mut TorchGodState,
    pub buffs: &'a mut crate::buffs::BuffSlots,
    /// `Player.unlockedBiomeTorches` (`:18066`).
    pub unlocked_biome_torches: bool,
    /// `base.position.Y` — the trigger/abort gates read the feet.
    pub pos_y: f32,
    /// The C#'s inventory scan (`:18069-18076`): ANY of the 59
    /// `inventory` slots holding type 5043 (the port scans the 50-grid +
    /// coins + ammo — the same 59 slots, `Player.cs:1576`).
    pub has_favor: bool,
    /// `Main.Difficulty` — the port's Classic 1.0 fold.
    pub difficulty: f32,
}

/// `Player.TryRecalculatingTorchLuck`'s event head (`Player.cs:17733-17742`)
/// — the seam `luck.rs`'s `torch_scan_step_event` calls FIRST. Returns
/// `Some` when the event consumed the tick (the caller must NOT run the
/// scan): the cursor zeroes and `TorchAttack` runs. Otherwise the cooldown
/// decrements and the scan proceeds. The scan body itself stays e27's.
pub fn try_recalculating_torch_luck_head(
    ctx: &mut TorchGodEventCtx<'_>,
    luck: &mut LuckState,
    world: &World,
    center: (f32, f32),
    rng: &mut dyn DropRng,
) -> Option<TorchGodPending> {
    if ctx.state.happy_fun_torch_time {
        luck.lucky_torch_counter = 0;
        return Some(torch_attack_step(
            ctx.state,
            ctx.buffs,
            world,
            ctx.pos_y,
            center,
            ctx.difficulty,
            rng,
        ));
    }
    if ctx.state.torch_god_cooldown > 0 {
        ctx.state.torch_god_cooldown -= 1;
    }
    None
}

/// `UpdateTorchLuck_ConsumeCountersAndCalculate`'s trigger tail
/// (`Player.cs:18066-18086`) — the seam `luck.rs`'s consume calls AFTER the
/// luck clamp, BEFORE `nearbyTorches = 0`.
pub fn torch_god_trigger_check(ctx: &mut TorchGodEventCtx<'_>, luck: &LuckState, world_surface: f64) {
    if ctx.state.torch_god_cooldown <= 0
        && !ctx.unlocked_biome_torches
        && luck.nearby_torches > 100
        && !ctx.state.happy_fun_torch_time
        && (ctx.pos_y as f64) > world_surface * 16.0
        && !ctx.has_favor
    {
        ctx.state.happy_fun_torch_time = true;
        ctx.state.number_of_torch_attacks_made = 0;
        // The `netMode == 1` SendData(4) tail (`:18081-18084`) is net-only.
    }
}

/// The favor drop's push half — the drain calls it with the in-tick draws
/// ([`TorchGodPending::favor_drop`]). The drop's box is 18×18
/// (`Item.cs:40440-40441`), centered on the player center
/// (`WorldItem.Center = center`, `:49736` → top-left = center − 9).
/// `ReserveForLocalPlayer` in single-player (`netMode == 0`) is a no-op
/// (`WorldItem.ApplySpawnOwnership`, `WorldItem.cs:295-305` — the reserve
/// only runs on a server), so no grab delay lands.
pub fn push_favor_drop(
    items: &mut Vec<crate::item::ItemDrop>,
    center: (f32, f32),
    vx: f32,
    vy: f32,
) {
    let drop = crate::item::ItemDrop::new(
        TORCH_GODS_FAVOR,
        1,
        0,
        center.0 - 9.0,
        center.1 - 9.0,
        vx,
        vy,
        0.0,
    );
    items.push(drop);
    // The port's 300-drop overflow guard (the `spawn_item_drop_px` tail).
    if items.len() > 300 {
        items.remove(0);
    }
}

/// The world_sim drain — called right after `p.update_with_grapple` inside
/// the fixed-step closure (`world_sim::update_player`), and once on the
/// dead branch (the `KillMe` abort queues its relight there). Applies the
/// tick's deferred writes in the C#'s body order: the snuff (`:17890`) /
/// the relight loop (`:17800-17810`), then the 949 spawn (`:17900-17903`),
/// then the favor drop (`:17872`). ZERO RNG here — every draw already ran
/// in-tick on the `UpdatePlayers` stream. The field-wise signature keeps
/// the closure's `p` borrow apart from the app fields (the
/// `spawn_item_drop_fields` convention).
pub fn drain_pending(
    p: &mut crate::player::Player,
    world: &mut World,
    projectiles: &mut crate::projectile::ProjectilePool,
    items: &mut Vec<crate::item::ItemDrop>,
    local_owner: u8,
) {
    let pending = std::mem::take(&mut p.pending_torch_god);
    if pending == TorchGodPending::default() {
        return;
    }
    // The tile writes: `Main.tile[x,y].frameX += 66` (`Player.cs:17890`) /
    // the relight's `-= 66` (`:17807`). The `SendTileSquare` calls are
    // net-only.
    if let Some((x, y)) = pending.snuff {
        if let Some(t) = world.tile_mut(x, y) {
            if t.active() && t.kind == 4 && t.frame_x < 66 {
                t.frame_x += 66;
            }
        }
    }
    if pending.relight {
        apply_relight(&p.torch_god, world);
    }
    // `Projectile.NewProjectile(GetProjectileSource_Misc(10), vector,
    // vector2, 949, damage, 1f, whoAmI, num8, num9)` (`:17900`) + the
    // `ai[0]`/`ai[1]` re-writes (`:17901-17902` — the same values the
    // spawn arguments carry) + `netUpdate = true` (`:17903`).
    if let Some(launch) = pending.launch {
        if let Some(slot) = projectiles.spawn_with_data_ai_owned(
            TORCH_GOD_PROJECTILE,
            local_owner,
            launch.x,
            launch.y,
            launch.vx,
            launch.vy,
            launch.damage,
            1.0,
            launch.dust_id as f32,
            launch.distance,
        ) {
            // The stock `friendly = false` (`Projectile.cs:788` — the 949
            // arm never sets it): the pool's PVE walk must refuse the shot
            // (`Damage_PVE_Inner` `:12611`); the hurt rides the AI's own
            // SelfHurtPlayers. The slot default is friendly = true.
            projectiles.slots_mut()[slot].friendly = false;
            projectiles.slots_mut()[slot].net_update = true;
        }
    }
    // `Item.RequestNewItem(..., base.Center, 5043, ...)` (`:17872`).
    if let Some((center, vx, vy)) = pending.favor_drop {
        push_favor_drop(items, center, vx, vy);
    }
}

/// W4-f33 witnesses — WRITTEN NOT RUN (code-only mode, no compiler
/// invoked). The event machine's cadence, the RNG contract (one
/// `Next(num)` per volley + the grant's velocity pair, nothing else), the
/// trigger gates, the relight loop and the difficulty curve — each pinned
/// to the C# lines in the comments above.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile_ai_torch_god::TORCH_DUST;
    use tr_core::tile::FLAG_ACTIVE;
    use tr_core::UnifiedRandom;

    /// A 220×800 world → surface row 200 (3200 px); the deep test center
    /// (1760, 6400) sits at tile (110, 400), well below.
    fn world() -> World {
        World::new("t".into(), 220, 800)
    }

    fn place_torch(w: &mut World, x: i32, y: i32, frame_x: i16, style: i16) {
        if let Some(t) = w.tile_mut(x, y) {
            t.kind = 4;
            t.frame_x = frame_x;
            t.frame_y = style * 22;
            t.flags |= FLAG_ACTIVE;
        }
    }

    /// The deep center every volley test uses: tile (110, 400).
    const CENTER: (f32, f32) = (1760.0, 6400.0);
    const POS_Y: f32 = 6379.0; // the feet, below the 3200 px surface

    fn step(
        s: &mut TorchGodState,
        b: &mut crate::buffs::BuffSlots,
        w: &World,
        rng: &mut UnifiedRandom,
    ) -> TorchGodPending {
        torch_attack_step(s, b, w, POS_Y, CENTER, 1.0, rng)
    }

    #[test]
    fn w4_f33_the_difficulty_curve_is_the_identity_line() {
        // `GameDifficultyData.cs:85` — (Journey 0.5, 0.5) → (Master 3, 3):
        // Sample(d) == d on the segment, clamped below, extrapolated above.
        assert_eq!(hostile_projectile_damage_multiplier(0.5), 0.5);
        assert_eq!(hostile_projectile_damage_multiplier(1.0), 1.0);
        assert_eq!(hostile_projectile_damage_multiplier(2.0), 2.0);
        assert_eq!(hostile_projectile_damage_multiplier(3.0), 3.0);
        assert_eq!(hostile_projectile_damage_multiplier(4.0), 4.0);
        assert_eq!(hostile_projectile_damage_multiplier(0.0), 0.5);
        // `Player.cs:17887` — (int)(40f * Sample): the four difficulties.
        assert_eq!(torch_attack_damage(0.5), 20);
        assert_eq!(torch_attack_damage(1.0), 40);
        assert_eq!(torch_attack_damage(2.0), 80);
        assert_eq!(torch_attack_damage(3.0), 120);
    }

    #[test]
    fn w4_f33_the_volley_cadence_is_21_ticks() {
        // `Player.cs:17825-17830` — ++ then `<= 20` returns: ticks 1..20
        // do nothing (but re-grant Blackout), tick 21 fires the volley.
        let mut w = world();
        place_torch(&mut w, 110, 400, 0, 0);
        let mut s = TorchGodState { happy_fun_torch_time: true, ..Default::default() };
        let mut b = crate::buffs::BuffSlots::default();
        let mut rng = UnifiedRandom::new(1);
        for tick in 1..=20 {
            let out = step(&mut s, &mut b, &w, &mut rng);
            assert_eq!(out, TorchGodPending::default(), "tick {tick} is a wait");
            assert_eq!(s.torch_fun_timer, tick);
        }
        // Blackout 80 re-granted every event tick (`:17824`).
        let idx = b.find_buff_index(80).expect("Blackout granted");
        assert!(b.buff_time[idx] > 0);
        let out = step(&mut s, &mut b, &w, &mut rng);
        assert_eq!(s.torch_fun_timer, 0, "the volley resets the timer");
        assert_eq!(out.snuff, Some((110, 400)));
        assert_eq!(s.number_of_torch_attacks_made, 1);
        let launch = out.launch.expect("the 949 launch");
        // `:17895` — the torch cell's TOP-center.
        assert_eq!((launch.x, launch.y), (110.0 * 16.0 + 8.0, 400.0 * 16.0));
        assert_eq!(launch.damage, 40);
        assert_eq!(launch.dust_id, TORCH_DUST[0]); // style 0 → dust 6
        // `:17896-17899` — the aim points at the player center, speed 8.
        // The torch (110,400) spawns at (1768, 6400); the center (1760,
        // 6400) is 8 px to the LEFT → the aim is (−8, 0).
        let speed = (launch.vx * launch.vx + launch.vy * launch.vy).sqrt();
        assert!((speed - 8.0).abs() < 1e-4, "speed {speed}");
        assert!(launch.vx < 0.0 && launch.vy == 0.0, "aim left");
        // `ai[1]` = the pre-normalize distance (`num9`).
        let dx = CENTER.0 - launch.x;
        let dy = CENTER.1 - launch.y;
        assert!((launch.distance - (dx * dx + dy * dy).sqrt()).abs() < 1e-3);
    }

    #[test]
    fn w4_f33_one_rand_draw_per_volley_on_the_update_players_stream() {
        // `Player.cs:17881` — exactly ONE `Main.rand.Next(num)` per volley;
        // the pick is the draw's value (the candidate list is scan-ordered).
        let mut w = world();
        // Three lit torches, x-outer/y-inner scan order: (100,400),
        // (101,400), (102,400) — all inside the ±100 box.
        place_torch(&mut w, 100, 400, 0, 0);
        place_torch(&mut w, 101, 400, 0, 1);
        place_torch(&mut w, 102, 400, 0, 2);
        let mut s = TorchGodState { happy_fun_torch_time: true, ..Default::default() };
        let mut b = crate::buffs::BuffSlots::default();
        let seed = 777;
        let mut rng = UnifiedRandom::new(seed);
        let mut expect = UnifiedRandom::new(seed);
        let pick = expect.next_range(0, 3);
        for _ in 0..20 {
            step(&mut s, &mut b, &w, &mut rng);
        }
        let out = step(&mut s, &mut b, &w, &mut rng);
        assert_eq!(out.snuff, Some((100 + pick, 400)), "the draw picks the cell");
        assert_eq!(
            out.launch.unwrap().dust_id,
            TORCH_DUST[pick as usize],
            "the style rides frameY/22"
        );
        // The stream consumed exactly the one draw — byte-equal states.
        assert_eq!(rng, expect, "one Next(num), nothing else");
    }

    #[test]
    fn w4_f33_the_candidate_scan_caps_at_400_and_orders_x_outer() {
        // `Player.cs:17841-17861` — x OUTER, y inner, the 400-slot cap with
        // BOTH breaks. The ±100 box around tile (110,400) scans y 300..500
        // (201 rows): column 100's 201 + column 101's first 199 fill the
        // 400 slots; the rest of the 500 placed torches never enter.
        let mut w = world();
        for x in 100..102 {
            for y in 300..550 {
                place_torch(&mut w, x, y, 0, 0);
            }
        }
        let mut s = TorchGodState { happy_fun_torch_time: true, ..Default::default() };
        let mut b = crate::buffs::BuffSlots::default();
        // Find a seed whose Next(400) lands in column 101 (pick >= 201) so
        // the witness pins BOTH the x-outer order AND the cap truncation.
        let mut seed = 0;
        let pick = loop {
            let p = UnifiedRandom::new(seed).next_range(0, 400);
            if p >= 201 {
                break p;
            }
            seed += 1;
        };
        let mut rng = UnifiedRandom::new(seed);
        for _ in 0..20 {
            step(&mut s, &mut b, &w, &mut rng);
        }
        let out = step(&mut s, &mut b, &w, &mut rng);
        let (ex, ey) = (101, 300 + pick - 201);
        assert_eq!(out.snuff, Some((ex, ey)), "the cap keeps the first 400, x-outer");
    }

    #[test]
    fn w4_f33_no_candidates_ends_the_event_without_a_grant() {
        // `Player.cs:17862-17874` — num == 0: relight + flag clear +
        // cooldown; attacks < 95 → NO favor.
        let w = world(); // no torches
        let mut s = TorchGodState {
            happy_fun_torch_time: true,
            number_of_torch_attacks_made: 10,
            ..Default::default()
        };
        let mut b = crate::buffs::BuffSlots::default();
        let mut rng = UnifiedRandom::new(9);
        for _ in 0..20 {
            step(&mut s, &mut b, &w, &mut rng);
        }
        let out = step(&mut s, &mut b, &w, &mut rng);
        assert!(out.relight);
        assert!(!s.happy_fun_torch_time);
        assert_eq!(s.torch_god_cooldown, TORCH_GOD_COOLDOWN);
        assert!(out.favor_drop.is_none(), "10 attacks < 95");
        assert_eq!(rng, UnifiedRandom::new(9), "the end arm draws nothing");
    }

    #[test]
    fn w4_f33_survival_grants_the_favor_with_the_new_item_velocity_pair() {
        // `Player.cs:17870-17873` — attacks >= 95 at the end → the 5043
        // drop; `Item.cs:49749-49750` — Next(-30,31)*0.1 / Next(-40,-15)*0.1,
        // Prefix(0) draws nothing (`:511-514`).
        let w = world();
        let mut s = TorchGodState {
            happy_fun_torch_time: true,
            number_of_torch_attacks_made: 95,
            ..Default::default()
        };
        let mut b = crate::buffs::BuffSlots::default();
        let mut rng = UnifiedRandom::new(555);
        let mut expect = UnifiedRandom::new(555);
        let vx = expect.next_range(-30, 31) as f32 * 0.1;
        let vy = expect.next_range(-40, -15) as f32 * 0.1;
        for _ in 0..20 {
            step(&mut s, &mut b, &w, &mut rng);
        }
        let out = step(&mut s, &mut b, &w, &mut rng);
        assert_eq!(out.favor_drop, Some((CENTER, vx, vy)));
        assert_eq!(rng, expect, "exactly the two NewItem draws");
        // The push half: an 18×18 box centered on the player center.
        let mut items = Vec::new();
        push_favor_drop(&mut items, CENTER, vx, vy);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, TORCH_GODS_FAVOR);
        assert_eq!(items[0].count, 1);
        assert_eq!(items[0].prefix, 0);
        assert_eq!(items[0].pos, glam::Vec2::new(CENTER.0 - 9.0, CENTER.1 - 9.0));
        // ReserveForLocalPlayer in single-player is a no-op
        // (`WorldItem.cs:295-305`) — no grab delay.
        assert_eq!(items[0].grab_delay_time, 0);
    }

    #[test]
    fn w4_f33_the_last_candidate_at_95_arms_the_finale_delay() {
        // `Player.cs:17904-17907` — `(num == 1 && attacks >= 95) || attacks
        // >= 200` → torchFunTimer = −180 (the next volley waits 201 ticks).
        let mut w = world();
        place_torch(&mut w, 110, 400, 0, 0);
        let mut s = TorchGodState {
            happy_fun_torch_time: true,
            number_of_torch_attacks_made: 94,
            ..Default::default()
        };
        let mut b = crate::buffs::BuffSlots::default();
        let mut rng = UnifiedRandom::new(3);
        for _ in 0..20 {
            step(&mut s, &mut b, &w, &mut rng);
        }
        let out = step(&mut s, &mut b, &w, &mut rng);
        assert!(out.launch.is_some(), "the 95th attack fires");
        assert_eq!(s.number_of_torch_attacks_made, 95);
        assert_eq!(s.torch_fun_timer, -180, "the finale delay");
        // The snuffed torch is no longer a candidate: 201 ticks later the
        // no-candidate arm ends the event WITH the grant.
        // (The snuff write is deferred — apply it like the drain does.)
        if let Some((x, y)) = out.snuff {
            w.tile_mut(x, y).unwrap().frame_x += 66;
        }
        for _ in 0..200 {
            let out = step(&mut s, &mut b, &w, &mut rng);
            assert_eq!(out, TorchGodPending::default());
        }
        let out = step(&mut s, &mut b, &w, &mut rng);
        assert!(out.relight);
        assert!(!s.happy_fun_torch_time);
        assert!(out.favor_drop.is_some(), "95 survived → the favor");
    }

    #[test]
    fn w4_f33_the_200_cap_ends_the_event() {
        // `Player.cs:17862` — attacks >= maxTorchAttacks (200) ends the
        // event even with candidates left; >= 95 → the grant fires.
        let mut w = world();
        place_torch(&mut w, 110, 400, 0, 0);
        let mut s = TorchGodState {
            happy_fun_torch_time: true,
            number_of_torch_attacks_made: 200,
            ..Default::default()
        };
        let mut b = crate::buffs::BuffSlots::default();
        let mut rng = UnifiedRandom::new(11);
        for _ in 0..20 {
            step(&mut s, &mut b, &w, &mut rng);
        }
        let out = step(&mut s, &mut b, &w, &mut rng);
        assert!(out.relight);
        assert!(!s.happy_fun_torch_time);
        assert!(out.launch.is_none(), "the cap arm never launches");
        assert!(out.favor_drop.is_some());
    }

    #[test]
    fn w4_f33_surfacing_aborts_the_event() {
        // `Player.cs:17819-17823` — position.Y above the surface ends the
        // event BEFORE the buff/cadence (no Blackout grant on the abort).
        let w = world();
        let mut s = TorchGodState { happy_fun_torch_time: true, ..Default::default() };
        let mut b = crate::buffs::BuffSlots::default();
        let mut rng = UnifiedRandom::new(1);
        let out = torch_attack_step(&mut s, &mut b, &w, 100.0, CENTER, 1.0, &mut rng);
        assert!(out.relight);
        assert!(!s.happy_fun_torch_time);
        assert_eq!(s.torch_god_cooldown, TORCH_GOD_COOLDOWN);
        assert!(b.find_buff_index(80).is_none(), "no buff on the abort tick");
        assert_eq!(rng, UnifiedRandom::new(1), "the abort draws nothing");
    }

    #[test]
    fn w4_f33_the_relight_loop_restores_only_snuffed_torches() {
        // `Player.cs:17800-17810` — the unlit list in attack order; an
        // active torch with frameX >= 66 gets −66; a broken/changed tile
        // is skipped.
        let mut w = world();
        place_torch(&mut w, 100, 400, 66, 0); // snuffed → relights
        place_torch(&mut w, 101, 400, 0, 0); // lit → untouched
        place_torch(&mut w, 102, 400, 66, 0);
        w.tile_mut(102, 400).unwrap().kind = 1; // no longer a torch → skip
        let s = TorchGodState {
            number_of_torch_attacks_made: 3,
            unlit_torch_x: {
                let mut a = [0; MAX_TORCH_ATTACKS];
                a[..3].copy_from_slice(&[100, 101, 102]);
                a
            },
            unlit_torch_y: {
                let mut a = [0; MAX_TORCH_ATTACKS];
                a[..3].copy_from_slice(&[400, 400, 400]);
                a
            },
            ..Default::default()
        };
        apply_relight(&s, &mut w);
        assert_eq!(w.tile(100, 400).unwrap().frame_x, 0);
        assert_eq!(w.tile(101, 400).unwrap().frame_x, 0);
        assert_eq!(w.tile(102, 400).unwrap().frame_x, 66, "the dirt tile keeps its frame");
    }

    #[test]
    fn w4_f33_the_trigger_gates_match_player_cs_18066() {
        // cooldown 0, !unlocked, >100 torches, !running, below surface,
        // no favor — every gate must hold.
        let mut luck = LuckState { nearby_torches: 101, ..Default::default() };
        let mut state = TorchGodState::default();
        let mut buffs = crate::buffs::BuffSlots::default();
        let mut ctx = TorchGodEventCtx {
            state: &mut state,
            buffs: &mut buffs,
            unlocked_biome_torches: false,
            pos_y: POS_Y,
            has_favor: false,
            difficulty: 1.0,
        };
        torch_god_trigger_check(&mut ctx, &luck, 200.0);
        assert!(state.happy_fun_torch_time);
        assert_eq!(state.number_of_torch_attacks_made, 0);
        // Each gate blocks: exactly 100 torches...
        let cases: [(bool, bool, i32, i32, f32); 5] = [
            (true, false, 101, 0, POS_Y),  // unlocked
            (false, true, 101, 0, POS_Y),  // has favor
            (false, false, 100, 0, POS_Y), // not > 100
            (false, false, 101, 1, POS_Y), // cooldown
            (false, false, 101, 0, 100.0), // above surface
        ];
        for (unlocked, favor, torches, cooldown, y) in cases {
            let mut state = TorchGodState { torch_god_cooldown: cooldown, ..Default::default() };
            let mut buffs = crate::buffs::BuffSlots::default();
            luck.nearby_torches = torches;
            let mut ctx = TorchGodEventCtx {
                state: &mut state,
                buffs: &mut buffs,
                unlocked_biome_torches: unlocked,
                pos_y: y,
                has_favor: favor,
                difficulty: 1.0,
            };
            torch_god_trigger_check(&mut ctx, &luck, 200.0);
            assert!(!state.happy_fun_torch_time, "gate ({unlocked},{favor},{torches},{cooldown},{y})");
        }
    }

    #[test]
    fn w4_f33_the_event_head_replaces_the_scan_and_zeroes_the_cursor() {
        // `Player.cs:17733-17742` — running: cursor 0 + TorchAttack; idle:
        // the cooldown decrements once per call.
        let w = world();
        let mut luck = LuckState { lucky_torch_counter: 37, ..Default::default() };
        let mut state = TorchGodState {
            happy_fun_torch_time: true,
            torch_god_cooldown: 50,
            ..Default::default()
        };
        let mut buffs = crate::buffs::BuffSlots::default();
        let mut rng = UnifiedRandom::new(1);
        let mut ctx = TorchGodEventCtx {
            state: &mut state,
            buffs: &mut buffs,
            unlocked_biome_torches: false,
            pos_y: POS_Y,
            has_favor: false,
            difficulty: 1.0,
        };
        let out = try_recalculating_torch_luck_head(&mut ctx, &mut luck, &w, CENTER, &mut rng);
        assert!(out.is_some(), "the event consumes the tick");
        assert_eq!(luck.lucky_torch_counter, 0);
        assert_eq!(ctx.state.torch_god_cooldown, 50, "no decrement while running");
        // Idle: the cooldown ticks down, the scan proceeds.
        ctx.state.happy_fun_torch_time = false;
        let out = try_recalculating_torch_luck_head(&mut ctx, &mut luck, &w, CENTER, &mut rng);
        assert!(out.is_none());
        assert_eq!(ctx.state.torch_god_cooldown, 49);
    }

    #[test]
    fn w4_f33_the_scan_seam_triggers_the_event_at_the_consume() {
        // The full luck.rs seam: >100 lit torches in the 81-row window,
        // the consume's trigger tail arms the event BEFORE nearbyTorches
        // resets; the next tick's head runs TorchAttack instead.
        let mut w = world();
        // 101 torches around tile (110, 400) — the window is x 70..150,
        // y 360..440 centered on (1760, 6400).
        for i in 0..101 {
            place_torch(&mut w, 70 + i % 80, 360 + i / 80, 0, 0);
        }
        let mut luck = LuckState::default();
        let mut state = TorchGodState::default();
        let mut buffs = crate::buffs::BuffSlots::default();
        let mut rng = UnifiedRandom::new(2024);
        // The first call consumes immediately (the (0,0) anchor is above
        // the surface) and anchors the window at the center.
        {
            let mut ctx = TorchGodEventCtx {
                state: &mut state,
                buffs: &mut buffs,
                unlocked_biome_torches: false,
                pos_y: POS_Y,
                has_favor: false,
                difficulty: 1.0,
            };
            let out = crate::luck::torch_scan_step_event(
                &mut luck, &w, CENTER, false, None, Default::default(), &mut ctx, &mut rng,
            );
            assert_eq!(out, TorchGodPending::default());
        }
        assert_eq!(luck.next_torch_luck_check_center, CENTER);
        // 81 rows walk the window; the last row's consume triggers.
        for _ in 0..81 {
            let mut ctx = TorchGodEventCtx {
                state: &mut state,
                buffs: &mut buffs,
                unlocked_biome_torches: false,
                pos_y: POS_Y,
                has_favor: false,
                difficulty: 1.0,
            };
            let _ = crate::luck::torch_scan_step_event(
                &mut luck, &w, CENTER, false, None, Default::default(), &mut ctx, &mut rng,
            );
        }
        assert!(state.happy_fun_torch_time, "101 torches > 100 → the event");
        assert_eq!(luck.nearby_torches, 0, "the reset runs after the trigger");
        // The next tick: the head consumes it — a wait tick (cadence 1).
        let mut ctx = TorchGodEventCtx {
            state: &mut state,
            buffs: &mut buffs,
            unlocked_biome_torches: false,
            pos_y: POS_Y,
            has_favor: false,
            difficulty: 1.0,
        };
        let out = crate::luck::torch_scan_step_event(
            &mut luck, &w, CENTER, false, None, Default::default(), &mut ctx, &mut rng,
        );
        assert_eq!(out, TorchGodPending::default());
        assert_eq!(state.torch_fun_timer, 1, "TorchAttack ran, not the scan");
        assert!(buffs.find_buff_index(80).is_some(), "Blackout granted");
    }

    #[test]
    fn w4_f33_death_arms_the_relight_through_end_ongoing() {
        // `Player.cs:38855-38858` → `:17718-17729`: the abort relights +
        // clears; a second call is a no-op (the flag is down).
        let mut s = TorchGodState { happy_fun_torch_time: true, ..Default::default() };
        assert!(end_ongoing_torch_god_event(&mut s));
        assert!(!s.happy_fun_torch_time);
        assert_eq!(s.torch_god_cooldown, TORCH_GOD_COOLDOWN);
        assert!(!end_ongoing_torch_god_event(&mut s), "idempotent");
    }
}