//! S1-C-7 wave B — `Player.Hurt`'s retaliate block (`Player.cs:38401-38517`):
//! the brain of confusion's confusion aura + dart, the star cloak's falling
//! stars and the honeycomb's bee swarm. `hurt_inner` has no NPC list /
//! projectile pool / world, so an accepted hit queues the final post-floor
//! damage (`pending_retaliate`) and the `world_sim` seams drain it here on
//! the hit lane's RNG stream — the C# rolls `Main.rand` inline inside `Hurt`,
//! which runs under the hitting stage's `SwapRandom` (contact:
//! `UpdateNPCs`, `Main.cs:17767`; hostile shots: `UpdateProjectiles`,
//! `Main.cs:17775`; hazards/fall: `UpdatePlayers`, `Main.cs:17754`).
//!
//! The C# gate is `(whoAmI == Main.myPlayer && ImmunityCooldownID.Sets.
//! Retaliate[cooldownCounter]) & flag` (`:38401`) — `flag` is the PRE-hit
//! per-counter state (`:38208`). `hurt_inner` queues only when the flag
//! held AND the counter is a Retaliate member (M4-HAZARD-1: the membership
//! reads `id_tables::immunity_retaliate(cooldown_counter)` — the S1-C-7
//! wave B `retaliate_lane` bool is retired): the contact/projectile/
//! fall/petrify lanes ride General or BossNoCheese (members,
//! `ImmunityCooldownID.cs:49`), the lava (`Lava`), tile-contact
//! (`TileContactDamage`) and bug-net (`WrongBugNet`) lanes are not —
//! vanilla lava/spike hits never retaliate. A cooldown-active
//! PaladinsShield hit (the `:38210` exemption) reaches the tail with flag
//! false and queues nothing.
//!
//! Deviations (ANALYSIS "The hurt modifiers (S1-C-7 wave B)"): the retaliate
//! block's `Main.rand` draws drain at the hitting pass's tail rather than
//! mid-pass (the S1-E-21c batch convention); master mode is unported
//! (classic + expert only); the `GetProjectileSource_Accessory` entity
//! sources have no port model (the Item references fold to armed bools).

use glam::Vec2;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::player::Player;
use crate::projectile::ProjectilePool;

/// The accepted-hurt snapshot the queue carries (`hurt_inner`'s tail): the
/// C# rolls the retaliate block inline inside `Hurt`, so the dart's
/// `velocity * 0.3`, the brain's distance, the stars' aim and the bees' spawn
/// all read the player's position/velocity AT THE HIT — BEFORE `Hurt`'s own
/// knockback (`Player.cs:38519-38524`) overwrites the velocity. The port
/// drains at the pass tail, so the snapshot freezes those reads at queue
/// time; only the flags and the Honey buff read the live player.
#[derive(Clone, Copy, Debug)]
pub struct RetaliateRequest {
    /// The final post-floor `num2` (`Player.cs:38378-38383`).
    pub damage: i32,
    /// `base.position` at the hit (the stars/bees spawn top-left).
    pub x: f32,
    pub y: f32,
    /// `base.Center` at the hit (the live hitbox — mount growth included).
    pub center: (f32, f32),
    /// `base.velocity` at the hit, pre-knockback.
    pub vx: f32,
    pub vy: f32,
}

/// `Player.beeType` (`Player.cs:12417-12426`): the strong-bee coin flip —
/// `strongBees && Main.rand.Next(2) == 0` arms `makeStrongBee` and picks the
/// giant bee 566, else the bee 181. The C# `makeStrongBee` field's only
/// readers are `beeDamage`/`beeKB` in the same spawn expression, so the port
/// returns the (kind, strong) pair instead of carrying the scratch bool.
/// Shared with the Bee Gun's shoot arm (`Player.cs:48328`, S1-P-i20).
pub(crate) fn bee_type(p: &Player, rng: &mut dyn DropRng) -> (u16, bool) {
    if p.strong_bees && rng.next_range(0, 2) == 0 {
        return (566, true);
    }
    (181, false)
}

/// `Player.beeDamage` (`Player.cs:12428-12435`): the strong bee adds
/// `Next(1, 4)`, the plain bee `Next(2)`.
pub(crate) fn bee_damage(strong: bool, dmg: i32, rng: &mut dyn DropRng) -> i32 {
    if strong {
        dmg + rng.next_range(1, 4)
    } else {
        dmg + rng.next_range(0, 2)
    }
}

/// `Player.beeKB` (`Player.cs:12437-12444`): the strong bee's knockback is
/// `0.5 + KB * 1.1`, the plain bee keeps `KB`.
pub(crate) fn bee_kb(strong: bool, kb: f32) -> f32 {
    if strong { 0.5 + kb * 1.1 } else { kb }
}

/// The drain: one accepted hurt's retaliate block, in the C#'s exact order
/// (brain → star cloak → honeycomb) and RNG draw order. `req` is the hit-moment
/// snapshot (damage = the final post-floor `num2`, `Player.cs:38378-38383`);
/// `owner` is the local player id (`Main.myPlayer`); `expert` mirrors
/// `Main.expertMode` (the port's `buffs.expert_mode` — master mode is
/// unported, its ×3/×2 arms are cited but unreachable).
pub fn drain_retaliate(
    p: &mut Player,
    pool: &mut ProjectilePool,
    npcs: &mut [Npc],
    req: RetaliateRequest,
    owner: u8,
    expert: bool,
    rng: &mut dyn DropRng,
) {
    let damage = req.damage;
    // The brain of confusion (`Player.cs:38403-38431`).
    if p.brain_of_confusion {
        for n in npcs.iter_mut() {
            // `Main.npc[n].active && !Main.npc[n].friendly && Main.rand.
            // Next(500) < 300 + (int)num2 * 2` (`:38407`) — the port's live
            // list stands in for `active` (the `hp > 0` scan convention).
            // The `&&` draws only for live hostile NPCs.
            if n.hp <= 0 || n.friendly {
                continue;
            }
            if rng.next_range(0, 500) >= 300 + damage * 2 {
                continue;
            }
            // `num8 = (npc.Center - Center).Length()` (`:38409`).
            let (ncx, ncy) = n.center();
            let (pcx, pcy) = req.center;
            let dist = ((ncx - pcx).powi(2) + (ncy - pcy).powi(2)).sqrt();
            // `num9 = Next(200 + num2/2, 301 + num2*2)` (`:38410`), then the
            // three compression steps in order (`:38411-38422`): >500 ×0.75
            // of the excess, >700 ×0.5, >900 ×0.25 — each reads the value
            // the previous step produced.
            let mut radius = rng.next_range(200 + damage / 2, 301 + damage * 2) as f32;
            if radius > 500.0 {
                radius = 500.0 + (radius - 500.0) * 0.75;
            }
            if radius > 700.0 {
                radius = 700.0 + (radius - 700.0) * 0.5;
            }
            if radius > 900.0 {
                radius = 900.0 + (radius - 900.0) * 0.25;
            }
            if dist < radius {
                // `AddBuff(31, Next(90 + num2/3, 300 + num2/2))`
                // (`:38425-38426`) — Confused.
                let time = rng.next_range(90 + damage / 3, 300 + damage / 2);
                n.buffs.add_buff(31, time);
            }
        }
        // The dart 565 (`:38430`): `Center.X + Next(-40, 40)`, `Center.Y -
        // Next(20, 60)`, `velocity * 0.3`, damage 0, kb 0 — all the hit-moment
        // reads (the block precedes `Hurt`'s own knockback `:38519`).
        let (cx, cy) = req.center;
        let x = cx + rng.next_range(-40, 40) as f32;
        let y = cy - rng.next_range(20, 60) as f32;
        pool.spawn(
            565,
            owner,
            Vec2::new(x, y),
            Vec2::new(req.vx * 0.3, req.vy * 0.3),
            0,
            0.0,
            false,
        );
    }
    // The star cloak (`Player.cs:38432-38481`).
    if p.star_cloak {
        // `:38434-38456`: 3 stars, type 726 default; the override checks
        // run in order starVeil 725 → beeCloak 724 → manaCloak 723 →
        // snakeCloak 1133, LAST wins.
        let mut kind: u16 = 726;
        if p.star_veil_override {
            kind = 725;
        }
        if p.bee_cloak_override {
            kind = 724;
        }
        if p.mana_cloak_override {
            kind = 723;
        }
        if p.snake_cloak_override {
            kind = 1133;
        }
        // `:38457-38465`: damage 75, ×3 master / ×2 expert. Master mode is
        // unported (classic + expert only).
        let mut damage_out = 75;
        if expert {
            damage_out *= 2;
        }
        for _ in 0..3 {
            // `:38471-38472`: `position.X + Next(-400, 400)`, `position.Y -
            // Next(500, 800)` — the TOP-LEFT position, not the center.
            let x = req.x + rng.next_range(-400, 400) as f32;
            let y = req.y - rng.next_range(500, 800) as f32;
            // `:38474-38477`: aim at the player's center (`position + width/
            // 2`, `position + height/2` — the live hitbox, mount growth
            // included), the x aim jittered by `Next(-100, 101)`, speed
            // `23 / len`.
            let (pcx, pcy) = req.center;
            let mut dx = pcx - x;
            let dy = pcy - y;
            dx += rng.next_range(-100, 101) as f32;
            let scale = 23.0 / (dx * dx + dy * dy).sqrt();
            // `:38478`: `NewProjectile(.., type, num12, 5f, whoAmI, 0f,
            // position.Y)` — kb 5, ai1 = the player's position.Y. W4-f48:
            // the five kinds now carry `proj_data` rows (`Projectile.cs:
            // 7646-7658` — the 24x24 box, penetrate 2, alpha 50, the local
            // immunity 10), so the spawn routes through the data seam (the
            // C# `NewProjectile` runs `SetDefaults`); `armorPenetration =
            // 25` (`:7657`) rides the typed setter (the gun_launch
            // precedent — no `ProjData` field).
            if let Some(slot) = pool.spawn_with_data_ai_owned(
                kind,
                owner,
                x,
                y,
                dx * scale,
                dy * scale,
                damage_out,
                5.0,
                0.0,
                req.y,
            ) {
                pool.set_armor_penetration(slot, 25);
            }
        }
    }
    // The honeycomb (`Player.cs:38482-38516`).
    if p.honey_comb {
        // `:38484-38496`: 1 bee, +1 per `Next(3) == 0` (two rolls), a third
        // roll only while `strongBees` (the `&&` draws nothing otherwise).
        let mut count = 1;
        if rng.next_range(0, 3) == 0 {
            count += 1;
        }
        if rng.next_range(0, 3) == 0 {
            count += 1;
        }
        if p.strong_bees && rng.next_range(0, 3) == 0 {
            count += 1;
        }
        // `:38497-38509`: base 13 (18 with strongBees), ×2 master / ×1.5
        // expert, truncated to int at the `beeDamage((int)num18)` call.
        let mut base = if p.strong_bees { 18.0 } else { 13.0 };
        if expert {
            base *= 1.5;
        }
        let base = base as i32;
        for _ in 0..count {
            // `:38513`: the named arguments evaluate left-to-right in the
            // written order — SpeedX, SpeedY, then Type (`beeType()`),
            // Damage (`beeDamage`), KnockBack (`beeKB(0f)`). The velocity
            // rolls precede the bee-type coin flip.
            let vx = rng.next_range(-35, 36) as f32 * 0.02;
            let vy = rng.next_range(-35, 36) as f32 * 0.02;
            let (kind, strong) = bee_type(p, rng);
            let dmg = bee_damage(strong, base, rng);
            let kb = bee_kb(strong, 0.0);
            pool.spawn(
                kind,
                owner,
                Vec2::new(req.x, req.y),
                Vec2::new(vx, vy),
                dmg,
                kb,
                false,
            );
        }
        // `:38515`: `AddBuff(48, 300)` — Honey.
        p.buffs.add_buff(48, 300);
    }
}

/// S1-C-7 wave B: the dismount-on-hit drain (`Player.cs:38396-38400`) —
/// `hurt_inner` queued the request when the source projectile was a
/// DismountsPlayersOnHit member and a mount was active; the caller owns
/// `&World` for `Mount.TryDismount` (`player.rs::try_dismount`).
pub fn drain_dismount(p: &mut Player, world: &tr_core::world::World) {
    if p.take_dismount_request() {
        p.try_dismount(world);
    }
}

/// W4-f50 — the hit-dust loop drain (`Player.cs:38527-38553`): the surviving
/// hit queued `(hitDirection, num2)`; the caller owns the dust pool + the
/// hit lane's stream. `num2 / statLifeMax2 * 100` particles burst from the
/// player's box, keyed to the armor set (stoned 1 / frost 135 / bone 26 /
/// default 5 — deviation 4 wires the frost/bone texture-id keys). Each
/// particle's velocity is `2 * hitDirection + hitDirection * num22 *
/// NextFloat()` on X, `-2` on Y (the C# `num21 + hitDirection * num22 *
/// NextFloat()`, `num22` stays 0). The `shader` post-arms are the dye
/// layer's (unported).
pub fn drain_hurt_dust(
    p: &mut Player,
    dust: &mut tr_core::particles::DustPool,
    ctx: &tr_core::particles::SpawnCtx,
    world: &tr_core::world::World,
    rng: &mut tr_core::UnifiedRandom,
) {
    let Some((hit_dir, num2)) = p.take_hurt_dust() else {
        return;
    };
    // `:38529`: `num20 = num2 / statLifeMax2 * 100.0` — the loop bound.
    let count = num2 as f64 / p.max_hp as f64 * 100.0;
    // The armor-set key (`:38534-38551`): stoned 1, frost 135, bone 26,
    // default 5. W4-f50 deviation 4: the `frost_armor`/`bone_armor` flags
    // now derive from the `ApplyArmorSoundAndDustChanges` texture-id scan
    // (`:38167-38186`) at the `UpdateArmorSets` seam; the stoned buff (156)
    // takes priority, then frost, then bone, then the default.
    let stoned = p.buffs.find_buff_index(156).is_some();
    let dust_type = if stoned {
        1
    } else if p.frost_armor {
        135
    } else if p.bone_armor {
        26
    } else {
        5
    };
    let num21 = 2.0 * hit_dir as f32;
    // `:38531`: `num22 = 0f` — NEVER incremented in the loop, so the X
    // velocity is just `num21`; the `NextFloat` draw is still consumed each
    // iteration (the C# `num21 + hitDirection * num22 * NextFloat()`).
    let num22 = 0.0f32;
    let mut num23 = 0.0f64;
    while num23 < count {
        let vx = num21 + hit_dir as f32 * num22 * rng.next_float();
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            glam::Vec2::new(p.x, p.y),
            p.width as i32,
            p.height as i32,
            dust_type,
            vx,
            -2.0,
            0,
            tr_core::particles::Color::default(),
            1.0,
        );
        let _ = idx;
        num23 += 1.0;
    }
}

/// W4-f50 — the football hurt drain (`Player.cs:38255-38269`): the accepted
/// hit queued the hit direction; the caller owns the projectile pool + the
/// inventory. The drain scans the 58 inventory slots for the Football (4743)
/// and spawns the 861 return at the player's center with `(-hitDirection *
/// 4, -6)`, damage 0, ai1 = 1, ai2 = stack, then clears the slot (the mouse
/// slot 58 clears `app.mouse_item`). The `NewProjectile` ai2 argument rides
/// the `set_ai2` setter (the `set_ai1` convention).
pub fn drain_football(
    p: &mut Player,
    pool: &mut ProjectilePool,
    inventory: &mut [crate::item::ItemSlot],
    mouse_item: &mut crate::item::ItemSlot,
    owner: u8,
) {
    let Some(hit_dir) = p.take_football() else {
        return;
    };
    let (cx, cy) = p.center();
    // The C# scans `inventory[0..59]` (`:38257`); the port's inventory is the
    // 50-slot array + the mouse slot. Every matching slot spawns + clears.
    for slot in inventory.iter_mut() {
        if slot.count > 0 && slot.kind == 4743 {
            let stack = slot.count;
            if let Some(idx) = pool.spawn(
                861,
                owner,
                glam::Vec2::new(cx, cy),
                glam::Vec2::new(-hit_dir as f32 * 4.0, -6.0),
                0,
                0.0,
                false,
            ) {
                pool.set_ai1(idx, 1.0);
                pool.set_ai2(idx, stack as f32);
            }
            *slot = crate::item::EMPTY_SLOT;
        }
    }
    if mouse_item.count > 0 && mouse_item.kind == 4743 {
        let stack = mouse_item.count;
        if let Some(idx) = pool.spawn(
            861,
            owner,
            glam::Vec2::new(cx, cy),
            glam::Vec2::new(-hit_dir as f32 * 4.0, -6.0),
            0,
            0.0,
            false,
        ) {
            pool.set_ai1(idx, 1.0);
            pool.set_ai2(idx, stack as f32);
        }
        *mouse_item = crate::item::EMPTY_SLOT;
    }
}

/// W4-f50 — the golf-club hurt drain (`Player.cs:38271-38280`): the accepted
/// hit queued the flag; the caller owns the projectile pool + the held item
/// kind. When the held item is a jousting lance (4788/4789/4790), every
/// owned 877/878/879 projectile deactivates.
pub fn drain_golf_club_kill(
    p: &mut Player,
    pool: &mut ProjectilePool,
    held_kind: u16,
    owner: u8,
) {
    if !p.take_golf_club_kill() {
        return;
    }
    if !matches!(held_kind, 4788 | 4789 | 4790) {
        return;
    }
    for proj in pool.slots_mut() {
        if proj.active && proj.owner == owner && matches!(proj.kind, 877 | 878 | 879) {
            proj.active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::world::World;

    fn player() -> Player {
        Player::spawn(&World::new("t".into(), 64, 800))
    }

    /// A scripted `DropRng` that records every `(min, max)` draw — the
    /// draw-order contract mirror (the `buff_effects_accessory` pattern).
    struct ScriptRng {
        rolls: Vec<i32>,
        draws: Vec<(i32, i32)>,
    }
    impl ScriptRng {
        fn with_rolls(rolls: &[i32]) -> Self {
            Self {
                rolls: rolls.to_vec(),
                draws: Vec::new(),
            }
        }
    }
    impl DropRng for ScriptRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            self.rolls.remove(0)
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    fn hostile_npc(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 0, 0.0);
        n.hp = n.max_hp;
        n
    }

    /// The hit-moment snapshot from the player's current state — the tests
    /// drain immediately after the "hit", so live == snapshot.
    fn req(p: &Player, damage: i32) -> RetaliateRequest {
        RetaliateRequest {
            damage,
            x: p.x,
            y: p.y,
            center: p.center(),
            vx: p.vx,
            vy: p.vy,
        }
    }

    #[test]
    fn s1_c_7b_no_flags_drain_nothing() {
        let mut p = player();
        let mut pool = ProjectilePool::new();
        let mut npcs = [hostile_npc(p.x + 10.0, p.y)];
        let mut rng = ScriptRng::with_rolls(&[]);
        let r = req(&p, 10);
        drain_retaliate(&mut p, &mut pool, &mut npcs, r, 0, false, &mut rng);
        assert!(rng.draws.is_empty(), "no source armed draws nothing");
        assert_eq!(pool.active_count(), 0);
        assert!(p.buffs.find_buff_index(48).is_none());
    }

    #[test]
    fn s1_c_7b_the_brain_confuses_in_range_and_spawns_the_dart() {
        // `Player.cs:38403-38431`: the per-NPC `Next(500)` gate, the radius
        // `Next(200 + num2/2, 301 + num2*2)`, the buff-time
        // `Next(90 + num2/3, 300 + num2/2)`, then the dart's two position
        // rolls. num2 = 10: gate `< 320`, radius `Next(205, 321)`, time
        // `Next(93, 305)`.
        let mut p = player();
        p.brain_of_confusion = true;
        p.vx = 2.0;
        p.vy = -4.0;
        let (pcx, pcy) = p.center();
        let mut pool = ProjectilePool::new();
        let mut npcs = [hostile_npc(pcx + 5.0, pcy)];
        // gate 0 (<320 passes), radius 210 (dist ~5 < 210), time 120, dart
        // x-roll 7, dart y-roll 30.
        let mut rng = ScriptRng::with_rolls(&[0, 210, 120, 7, 30]);
        let r = req(&p, 10);
        drain_retaliate(&mut p, &mut pool, &mut npcs, r, 0, false, &mut rng);
        assert_eq!(
            rng.draws,
            vec![(0, 500), (205, 321), (93, 305), (-40, 40), (20, 60)],
            "the C# draw order: gate → radius → time → dart x → dart y"
        );
        assert_eq!(npcs[0].buffs.find_buff_index(31).map(|i| npcs[0].buffs.buff_time[i]), Some(120));
        assert_eq!(pool.active_count(), 1, "the dart 565 spawns");
        let dart = &pool.slots()[0];
        assert_eq!(dart.kind, 565);
        assert_eq!(dart.damage, 0);
        assert_eq!(dart.pos, Vec2::new(pcx + 7.0, pcy - 30.0));
        assert_eq!(dart.vel, Vec2::new(0.6, -1.2), "velocity × 0.3");
    }

    #[test]
    fn s1_c_7b_the_brain_gate_and_range_skip_the_buff_but_not_the_dart() {
        // The `Next(500)` gate fails → no radius draw; a second NPC passes
        // the gate but sits outside the radius → no time draw. The dart
        // spawns regardless (`:38430` is outside the NPC loop).
        let mut p = player();
        p.brain_of_confusion = true;
        let (pcx, pcy) = p.center();
        let mut pool = ProjectilePool::new();
        let mut npcs = [
            hostile_npc(pcx + 5.0, pcy),
            hostile_npc(pcx + 400.0, pcy),
        ];
        // NPC 0: gate 499 (>= 320, skip). NPC 1: gate 0, radius 205 (< 400
        // dist, no buff). Dart rolls 0, 20.
        let mut rng = ScriptRng::with_rolls(&[499, 0, 205, 0, 20]);
        let r = req(&p, 10);
        drain_retaliate(&mut p, &mut pool, &mut npcs, r, 0, false, &mut rng);
        assert_eq!(
            rng.draws,
            vec![(0, 500), (0, 500), (205, 321), (-40, 40), (20, 60)]
        );
        assert!(npcs[0].buffs.find_buff_index(31).is_none());
        assert!(npcs[1].buffs.find_buff_index(31).is_none());
        assert_eq!(pool.active_count(), 1);
    }

    #[test]
    fn s1_c_7b_the_brain_skips_friendly_and_dead_npcs_without_drawing() {
        let mut p = player();
        p.brain_of_confusion = true;
        let mut pool = ProjectilePool::new();
        let mut friendly = hostile_npc(p.x, p.y);
        friendly.friendly = true;
        let mut dead = hostile_npc(p.x, p.y);
        dead.hp = 0;
        let mut npcs = [friendly, dead];
        let mut rng = ScriptRng::with_rolls(&[0, 20]);
        let r = req(&p, 10);
        drain_retaliate(&mut p, &mut pool, &mut npcs, r, 0, false, &mut rng);
        assert_eq!(rng.draws, vec![(-40, 40), (20, 60)], "only the dart rolls");
    }

    #[test]
    fn s1_c_7b_the_radius_compression_steps_run_in_order() {
        // num2 = 400: radius `Next(400, 1101)`; a raw 1000 compresses
        // >500 → 875, >700 → 787.5 (the >900 step reads 787.5, no-op). An
        // NPC at distance 800 is OUTSIDE 787.5 → no buff (the uncompressed
        // 1000 would have confused it).
        let mut p = player();
        p.brain_of_confusion = true;
        let (pcx, pcy) = p.center();
        let mut pool = ProjectilePool::new();
        let mut npcs = [hostile_npc(pcx + 800.0, pcy)];
        let mut rng = ScriptRng::with_rolls(&[0, 1000, 0, 20]);
        let r = req(&p, 400);
        drain_retaliate(&mut p, &mut pool, &mut npcs, r, 0, false, &mut rng);
        assert_eq!(rng.draws[1], (400, 1101), "200 + 400/2 .. 301 + 400*2");
        assert!(npcs[0].buffs.find_buff_index(31).is_none(), "800 > 787.5");
    }

    #[test]
    fn s1_c_7b_the_star_cloak_spawns_three_stars_with_the_c_aim() {
        // `Player.cs:38469-38479`: per star x-roll, y-roll, aim jitter;
        // speed 23/len; kb 5; ai1 = position.Y; damage 75 classic.
        let mut p = player();
        p.star_cloak = true;
        let mut pool = ProjectilePool::new();
        let mut npcs: [Npc; 0] = [];
        // star 0: x +0, y -500, jitter 0 → straight-down aim.
        // star 1: x +100, y -600, jitter -50.
        // star 2: x -200, y -700, jitter 100.
        let mut rng = ScriptRng::with_rolls(&[0, 500, 0, 100, 600, -50, -200, 700, 100]);
        let r = req(&p, 10);
        drain_retaliate(&mut p, &mut pool, &mut npcs, r, 0, false, &mut rng);
        assert_eq!(
            rng.draws,
            vec![
                (-400, 400), (500, 800), (-100, 101),
                (-400, 400), (500, 800), (-100, 101),
                (-400, 400), (500, 800), (-100, 101),
            ]
        );
        assert_eq!(pool.active_count(), 3);
        let s0 = &pool.slots()[0];
        assert_eq!(s0.kind, 726);
        assert_eq!(s0.damage, 75);
        assert_eq!(s0.knock_back, 5.0);
        assert_eq!(s0.pos, Vec2::new(p.x, p.y - 500.0));
        // W4-f48: the data-carrying spawn (`Projectile.cs:7646-7658`) —
        // the 24x24 box, penetrate 2, alpha 50, tileCollide off, the local
        // immunity 10 and the arpen-25 setter (`:7657`).
        assert_eq!((s0.w, s0.h), (24.0, 24.0));
        assert_eq!(s0.penetrate, 2);
        assert_eq!(s0.alpha, 50);
        assert!(!s0.tile_collide);
        assert!(s0.uses_local_immunity);
        assert_eq!(s0.local_hit_cooldown, 10);
        assert_eq!(s0.armor_penetration, 25);
        // dx = center.x - x, dy = center.y - (y-500); speed 23.
        let (pcx, pcy) = p.center();
        let dx = pcx - p.x;
        let dy = pcy - (p.y - 500.0);
        let scale = 23.0 / (dx * dx + dy * dy).sqrt();
        assert!((s0.vel.x - dx * scale).abs() < 1e-4);
        assert!((s0.vel.y - dy * scale).abs() < 1e-4);
        assert!((s0.vel.length() - 23.0).abs() < 1e-3, "speed 23");
        assert_eq!(s0.ai[1], p.y, "ai1 = position.Y");
        assert_eq!(pool.slots()[1].pos, Vec2::new(p.x + 100.0, p.y - 600.0));
        assert_eq!(pool.slots()[2].pos, Vec2::new(p.x - 200.0, p.y - 700.0));
    }

    #[test]
    fn s1_c_7b_the_star_type_overrides_resolve_last_wins() {
        // `Player.cs:38437-38456`: starVeil 725 → beeCloak 724 → manaCloak
        // 723 → snakeCloak 1133, each check overwrites the previous.
        for (flags, expect) in [
            ((true, false, false, false), 725u16),
            ((false, true, false, false), 724),
            ((false, false, true, false), 723),
            ((false, false, false, true), 1133),
            ((true, true, true, true), 1133),
            ((true, true, false, false), 724),
        ] {
            let mut p = player();
            p.star_cloak = true;
            (p.star_veil_override, p.bee_cloak_override, p.mana_cloak_override, p.snake_cloak_override) = flags;
            let mut pool = ProjectilePool::new();
            let mut npcs: [Npc; 0] = [];
            let mut rng = ScriptRng::with_rolls(&[0; 9]);
            let r = req(&p, 10);
            drain_retaliate(&mut p, &mut pool, &mut npcs, r, 0, false, &mut rng);
            assert_eq!(pool.slots()[0].kind, expect, "flags {flags:?}");
        }
    }

    #[test]
    fn s1_c_7b_the_star_damage_doubles_in_expert() {
        // `Player.cs:38457-38465`: 75 classic, ×2 expert (×3 master is
        // unported).
        let mut p = player();
        p.star_cloak = true;
        let mut pool = ProjectilePool::new();
        let mut npcs: [Npc; 0] = [];
        let mut rng = ScriptRng::with_rolls(&[0; 9]);
        let r = req(&p, 10);
        drain_retaliate(&mut p, &mut pool, &mut npcs, r, 0, true, &mut rng);
        assert_eq!(pool.slots()[0].damage, 150);
    }

    #[test]
    fn s1_c_7b_the_honeycomb_bee_count_rolls_and_buff() {
        // `Player.cs:38484-38515`: two `Next(3)` rolls (no strongBees third
        // draw), per bee vx/vy then beeType (no draw without strongBees)
        // then beeDamage `Next(2)`; the Honey buff 48 (300) after the loop.
        let mut p = player();
        p.honey_comb = true;
        let mut pool = ProjectilePool::new();
        let mut npcs: [Npc; 0] = [];
        // count rolls 0, 1 → 2 bees; bee 0: vx 5, vy -5, dmg +1; bee 1:
        // vx 0, vy 0, dmg +0.
        let mut rng = ScriptRng::with_rolls(&[0, 1, 5, -5, 1, 0, 0, 0]);
        let r = req(&p, 10);
        drain_retaliate(&mut p, &mut pool, &mut npcs, r, 0, false, &mut rng);
        assert_eq!(
            rng.draws,
            vec![
                (0, 3), (0, 3),
                (-35, 36), (-35, 36), (0, 2),
                (-35, 36), (-35, 36), (0, 2),
            ],
            "no strongBees: no third count roll, no beeType flip"
        );
        assert_eq!(pool.active_count(), 2);
        let b0 = &pool.slots()[0];
        assert_eq!(b0.kind, 181);
        assert_eq!(b0.damage, 14, "13 + Next(2)=1");
        assert_eq!(b0.knock_back, 0.0);
        assert_eq!(b0.pos, Vec2::new(p.x, p.y));
        assert!((b0.vel.x - 0.1).abs() < 1e-6, "5 * 0.02");
        assert!((b0.vel.y + 0.1).abs() < 1e-6);
        assert_eq!(pool.slots()[1].damage, 13);
        assert_eq!(
            p.buffs.find_buff_index(48).map(|i| p.buffs.buff_time[i]),
            Some(300),
            "AddBuff(48, 300)"
        );
    }

    #[test]
    fn s1_c_7b_strong_bees_add_the_third_roll_the_flip_and_the_damage() {
        // `Player.cs:38493-38496` (third count roll), `:12417-12444`
        // (beeType flip → 566, beeDamage Next(1,4), beeKB 0.5), base 18.
        let mut p = player();
        p.honey_comb = true;
        p.strong_bees = true;
        let mut pool = ProjectilePool::new();
        let mut npcs: [Npc; 0] = [];
        // count rolls 1, 1, 0 → 2 bees; bee 0: vx 0, vy 0, flip 0 (strong),
        // dmg +2; bee 1: vx 0, vy 0, flip 1 (plain), dmg +1.
        let mut rng = ScriptRng::with_rolls(&[1, 1, 0, 0, 0, 0, 2, 0, 0, 1, 1]);
        let r = req(&p, 10);
        drain_retaliate(&mut p, &mut pool, &mut npcs, r, 0, false, &mut rng);
        assert_eq!(
            rng.draws,
            vec![
                (0, 3), (0, 3), (0, 3),
                (-35, 36), (-35, 36), (0, 2), (1, 4),
                (-35, 36), (-35, 36), (0, 2), (0, 2),
            ],
            "the velocity rolls precede the beeType flip; strong dmg Next(1,4)"
        );
        let b0 = &pool.slots()[0];
        assert_eq!(b0.kind, 566);
        assert_eq!(b0.damage, 20, "18 + 2");
        assert_eq!(b0.knock_back, 0.5, "beeKB(0) strong = 0.5");
        let b1 = &pool.slots()[1];
        assert_eq!(b1.kind, 181);
        assert_eq!(b1.damage, 19, "18 + Next(2)=1");
        assert_eq!(b1.knock_back, 0.0);
    }

    #[test]
    fn s1_c_7b_the_honeycomb_base_scales_in_expert() {
        // `Player.cs:38497-38509`: 13 × 1.5 = 19.5 → (int) 19; strong 18 ×
        // 1.5 = 27.
        let mut p = player();
        p.honey_comb = true;
        let mut pool = ProjectilePool::new();
        let mut npcs: [Npc; 0] = [];
        let mut rng = ScriptRng::with_rolls(&[1, 1, 0, 0, 0]);
        let r = req(&p, 10);
        drain_retaliate(&mut p, &mut pool, &mut npcs, r, 0, true, &mut rng);
        assert_eq!(pool.slots()[0].damage, 19);
        let mut p = player();
        p.honey_comb = true;
        p.strong_bees = true;
        let mut pool = ProjectilePool::new();
        // count 1,1,1 → 1 bee; vx 0, vy 0, flip 1 (plain), dmg +0.
        let mut rng = ScriptRng::with_rolls(&[1, 1, 1, 0, 0, 1, 0]);
        let r = req(&p, 10);
        drain_retaliate(&mut p, &mut pool, &mut npcs, r, 0, true, &mut rng);
        assert_eq!(pool.slots()[0].damage, 27);
    }

    #[test]
    fn s1_c_7b_the_three_sources_run_in_the_c_order() {
        // brain → star cloak → honeycomb (`:38403/:38432/:38482`).
        let mut p = player();
        p.brain_of_confusion = true;
        p.star_cloak = true;
        p.honey_comb = true;
        let mut pool = ProjectilePool::new();
        let mut npcs: [Npc; 0] = [];
        let mut rolls = vec![0, 20]; // dart
        rolls.extend([0; 9]); // 3 stars
        rolls.extend([1, 1, 0, 0, 0]); // 1 bee
        let mut rng = ScriptRng::with_rolls(&rolls);
        let r = req(&p, 10);
        drain_retaliate(&mut p, &mut pool, &mut npcs, r, 0, false, &mut rng);
        let kinds: Vec<u16> = pool.slots().iter().filter(|s| s.active).map(|s| s.kind).collect();
        assert_eq!(kinds, vec![565, 726, 726, 726, 181]);
    }

    #[test]
    fn s1_c_7b_the_dismount_request_drains_through_try_dismount() {
        // `Player.cs:38396-38400`: the queued request dismounts the mount.
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        assert!(p.set_mount(2, false), "the slime mount equips");
        assert!(p.mount.is_active());
        // The request path: a DismountsPlayersOnHit source (877) through an
        // accepted hurt queues it (`Player.cs:38396-38400`).
        assert!(p.hurt_with_source(
            10,
            p.x + 100.0,
            &mut tr_core::UnifiedRandom::default(),
            true,
            877
        ));
        drain_dismount(&mut p, &w);
        assert!(!p.mount.is_active(), "TryDismount ran");
        assert!(!p.take_dismount_request(), "the request was taken");
        // No request → the mount stays.
        assert!(p.set_mount(2, false));
        drain_dismount(&mut p, &w);
        assert!(p.mount.is_active());
    }

    #[test]
    fn w4_f50_the_football_drain_spawns_the_return_and_clears_the_slot() {
        // `Player.cs:38255-38269`: the Football (4743) spawns the 861 return
        // at the player's center with `(-hitDirection * 4, -6)`, damage 0,
        // ai1 = 1, ai2 = stack, then clears the slot.
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.has_football = true;
        p.hurt_cd = 0.0;
        p.immune = false;
        assert!(p.hurt(10, p.x + 100.0, &mut tr_core::UnifiedRandom::default(), true));
        let mut pool = ProjectilePool::new();
        let mut inv = [crate::item::EMPTY_SLOT; 50];
        inv[3] = crate::item::ItemSlot { kind: 4743, name: "Football", count: 5, prefix: 0 };
        let mut mouse = crate::item::EMPTY_SLOT;
        drain_football(&mut p, &mut pool, &mut inv, &mut mouse, 0);
        let slot = pool.slots().iter().find(|s| s.active).expect("the 861 return");
        assert_eq!(slot.kind, 861);
        assert_eq!(slot.damage, 0);
        // hitDirection: the player's center >= from_x (p.x + 100) is false,
        // so hit_dir = -1 → velocity.X = -(-1)*4 = 4.
        assert_eq!(slot.vel.x, 4.0);
        assert_eq!(slot.vel.y, -6.0);
        assert_eq!(slot.ai[1], 1.0);
        assert_eq!(slot.ai[2], 5.0, "ai2 = the stack");
        assert_eq!(inv[3].kind, 0, "the slot cleared");
        assert!(p.take_football().is_none(), "the request is a one-shot");
    }

    #[test]
    fn w4_f50_the_golf_club_drain_kills_the_owned_lances() {
        // `Player.cs:38271-38280`: the held jousting lance (4788/4789/4790)
        // kills every owned 877/878/879 projectile.
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.hurt_cd = 0.0;
        p.immune = false;
        assert!(p.hurt(10, p.x + 100.0, &mut tr_core::UnifiedRandom::default(), true));
        let mut pool = ProjectilePool::new();
        pool.spawn(877, 0, glam::Vec2::ZERO, glam::Vec2::ZERO, 10, 0.0, false);
        pool.spawn(878, 0, glam::Vec2::ZERO, glam::Vec2::ZERO, 10, 0.0, false);
        pool.spawn(879, 1, glam::Vec2::ZERO, glam::Vec2::ZERO, 10, 0.0, false); // other owner
        pool.spawn(1, 0, glam::Vec2::ZERO, glam::Vec2::ZERO, 10, 0.0, false); // not a lance
        drain_golf_club_kill(&mut p, &mut pool, 4788, 0);
        assert!(!pool.slots()[0].active, "the owned 877 dies");
        assert!(!pool.slots()[1].active, "the owned 878 dies");
        assert!(pool.slots()[2].active, "the other-owner 879 stays");
        assert!(pool.slots()[3].active, "the non-lance stays");
        // A non-lance held item kills nothing.
        let mut q = Player::spawn(&w);
        q.hurt_cd = 0.0;
        q.immune = false;
        assert!(q.hurt(10, q.x + 100.0, &mut tr_core::UnifiedRandom::default(), true));
        let mut pool2 = ProjectilePool::new();
        pool2.spawn(877, 0, glam::Vec2::ZERO, glam::Vec2::ZERO, 10, 0.0, false);
        drain_golf_club_kill(&mut q, &mut pool2, 1, 0);
        assert!(pool2.slots()[0].active, "a non-lance held item kills nothing");
    }

    #[test]
    fn w4_f50_the_hit_dust_drain_spawns_the_burst() {
        // `Player.cs:38527-38553`: `num2 / statLifeMax2 * 100` particles.
        // 10 damage / 100 max * 100 = 10 particles, default dust 5.
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.hurt_cd = 0.0;
        p.immune = false;
        assert!(p.hurt(10, p.x + 100.0, &mut tr_core::UnifiedRandom::default(), true));
        let mut dust = tr_core::particles::DustPool::new();
        let ctx = tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: glam::Vec2::new(p.x - 400.0, p.y - 400.0),
            screen_size: glam::Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        };
        let mut rng = tr_core::UnifiedRandom::new(7);
        drain_hurt_dust(&mut p, &mut dust, &ctx, &w, &mut rng);
        let active = dust.slots().iter().filter(|d| d.active).count();
        assert_eq!(active, 10, "10 damage / 100 max * 100 = 10 particles");
        assert!(p.take_hurt_dust().is_none(), "the request is a one-shot");
    }

    #[test]
    fn w4_f50_the_hit_dust_keys_on_the_frost_and_bone_armor_sets() {
        // `Player.cs:38534-38551`: stoned 1, frost 135, bone 26, default 5.
        // Deviation 4 wires the frost/bone texture-id keys.
        let w = World::new("t".into(), 64, 800);
        let ctx = tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: glam::Vec2::ZERO,
            screen_size: glam::Vec2::new(4800.0, 3200.0),
            max_dust_to_draw: 6000,
        };
        let key = |frost: bool, bone: bool, stoned: bool| -> i32 {
            let mut p = Player::spawn(&w);
            p.frost_armor = frost;
            p.bone_armor = bone;
            if stoned {
                p.buffs.add_buff(156, 100);
            }
            p.hurt_cd = 0.0;
            p.immune = false;
            assert!(p.hurt(10, p.x + 100.0, &mut tr_core::UnifiedRandom::default(), true));
            let mut dust = tr_core::particles::DustPool::new();
            let mut rng = tr_core::UnifiedRandom::new(7);
            drain_hurt_dust(&mut p, &mut dust, &ctx, &w, &mut rng);
            dust.slots().iter().find(|d| d.active).unwrap().dust_type
        };
        assert_eq!(key(false, false, false), 5, "default");
        assert_eq!(key(true, false, false), 135, "frost");
        assert_eq!(key(false, true, false), 26, "bone");
        assert_eq!(key(true, true, false), 135, "frost before bone");
        assert_eq!(key(true, true, true), 1, "stoned before everything");
    }

    #[test]
    fn w4_f50_the_armor_dust_scan_derives_the_necro_and_frost_sets() {
        // `ApplyArmorSoundAndDustChanges` (`Player.cs:38167-38186`): the
        // Necro set (head 7/75, body 7, legs 7) → boneArmor; the Frost set
        // (head 46, body 27, legs 26) → frostArmor. Deviation 4.
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // Necro: 151 (head 7), 152 (body 7), 153 (legs 7).
        p.equipped[0] = 151;
        p.equipped[1] = 152;
        p.equipped[2] = 153;
        p.apply_armor_sound_and_dust_changes();
        assert!(p.bone_armor, "the Necro set arms boneArmor");
        assert!(!p.frost_armor);
        // Ancient Necro Helmet 959 (head 75) also counts.
        let mut q = Player::spawn(&w);
        q.equipped[0] = 959;
        q.equipped[1] = 152;
        q.equipped[2] = 153;
        q.apply_armor_sound_and_dust_changes();
        assert!(q.bone_armor, "the Ancient Necro Helmet (75) counts");
        // Frost: 684 (head 46), 685 (body 27), 686 (legs 26).
        let mut r = Player::spawn(&w);
        r.equipped[0] = 684;
        r.equipped[1] = 685;
        r.equipped[2] = 686;
        r.apply_armor_sound_and_dust_changes();
        assert!(r.frost_armor, "the Frost set arms frostArmor");
        assert!(!r.bone_armor);
        // A mixed set arms neither.
        let mut s = Player::spawn(&w);
        s.equipped[0] = 151;
        s.equipped[1] = 685;
        s.equipped[2] = 153;
        s.apply_armor_sound_and_dust_changes();
        assert!(!s.bone_armor && !s.frost_armor);
    }
}
