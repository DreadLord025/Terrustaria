//! S1-P-i28 — the thrown-consumable projectile family's missing flight arms
//! and death hooks. The census (ANALYSIS "The thrown consumables (S1-P-i28)")
//! found the twelve thrown weapons' item rows already faithful, the throwing
//! arm's consumable/spawn defects fixed in `world_sim/editing.rs`, and the
//! aiStyle-2 knives/waters + aiStyle-14 beenade already riding the ported
//! `ai_style2`/`ai_spear` arms once their `proj_data` rows land. What was
//! MISSING on the projectile side:
//!
//! * **aiStyle 68 — the Molotov cocktail (399)** (`Projectile.cs:30166-30239`):
//!   the spin, the 3-tick alpha fade-in, the 15-tick gravity, the `timeLeft<=3`
//!   resize-to-80 prime and the wet prime; the `HandleMovement`-68 collision
//!   (`:19780-19786`) stops the bottle and primes it instead of killing it.
//!   The death burst (`Kill()` `:73921-73951`) spawns the three greek fires.
//! * **aiStyle 113 — the Bone javelin (598)** (`AI_113_TargetSticker`
//!   `Projectile.cs:40562-40899`): the flying arc (45-tick then 0.98/0.35
//!   gravity), the stick state (`ai[0]==1`) that pins the javelin to its NPC
//!   for 900 ticks, and the kill on expiry / dead target. The stick itself,
//!   the Daybroken debuff and the per-NPC 6-javelin cap ride the pool's
//!   on-hit seam ([`crate::projectile::ProjectilePool`]).
//! * **aiStyle 36 — the bees (181/566)** (`Projectile.cs:27292-27471`): the
//!   beenade's children. The alpha fade-in, the `extraUpdates` collapse, the
//!   3-frame flutter, the 30-tick then 800-Manhattan homing scan and the
//!   per-axis accel steering (the giant bee's 6.8/0.14 vs the bee's 6/0.1).
//! * **aiStyle 1 — the Javelin (507)** (`AI_001`'s `:57471-57478` gravity arm,
//!   the 507/508/662/680/685 group): the 60-tick then 0.98/0.3 fall, the
//!   default rotation and the +16 clamp. Kind-disjoint from the ai001 A/B/C
//!   slices, the riders and the frostmoon/moonlord sets.
//! * **The death hooks**: the water bottles' splash conversion
//!   (`Kill()` `:77629-77645` → `WorldGen.Convert`), the beenade's bee burst
//!   (`Kill()` `:76431-76463`), the molotov's greek-fire burst.
//! * **The on-hit debuffs** (`StatusNPC`): the poisoned knife's 1/2 Poisoned
//!   (`:11628-11634`), the molotov + greek fires' OnFire (`:11506-11509`), the
//!   bone javelin's Daybroken (`:11335-11338`).
//!
//! Movement is the port's documented `HandleMovement` stand-in: the bees and
//! the flying javelin ride the kill-on-tile default arm (`:19849-19853`, the
//! `kill_step` shape); the molotov rides its own stop-and-prime arm; the stuck
//! javelin has `tileCollide=false` so it integrates freely.
//!
//! RNG by construction (ANALYSIS "The thrown consumables (S1-P-i28)"):
//! - The flight arms (68/113/36/507) draw ZERO `Main.rand` — their only rand
//!   riders are dust/sound (the molotov's `alpha==0` dust `:30191-30220`, the
//!   bee's frame flutter rolls nothing), skipped whole with their draws (the
//!   `ai_style2`/frostmoon precedent).
//! - The on-hit debuffs KEEP their explicit rolls in C# order: the poisoned
//!   knife's `Next(2)` (`:11630`), the molotov/fire's `Next(3,7)` (`:11508`).
//!   The bone javelin's Daybroken is unconditional (no roll).
//! - The beenade death burst KEEPS its draws: the `Next(15,25)` bee count
//!   (`:76457`) then, per bee, the velocity `Next(-35,36)` X-then-Y pair
//!   (`:76460`), the `beeType()` coin flip, `beeDamage` and `beeKB` (the
//!   `hurt_modifiers` order). The four gore type picks `Next(61,64)` +
//!   `NewGore` kicks are the G3 draw-census (kept, bodies skipped).
//! - The molotov death burst KEEPS its draws: per fire the X offset
//!   `Next(20,50)` + `Next(-20,21)`, the Y offset `Next(30,50)` +
//!   `Next(-20,5)`, then the fire-type `Next(3)` (`:73946-73948`), six fires.
//!
//! CODE-ONLY MODE (S1-P-i28): the witness tests at the bottom are WRITTEN NOT
//! RUN (no compiler invoked on this branch); verified by re-reading the cited
//! C# blocks and hand arithmetic.
//!
//! Deliberate skips (ANALYSIS): dust, gore bodies, `Lighting.AddLight`, sound
//! bodies (the cues ride `wants_sound` where the pool drains them),
//! `netUpdate`/`NetMessage.SendData`, the `HitEffect` visual on the stuck
//! javelin's 30-tick pulse. The bee group-static immunity identity (both 181
//! and 566 use `UseGroupStaticImmunity(181)`, `Projectile.cs:2684`/`:6183`)
//! folds to the port's kind-keyed `id_static` map — a recorded deviation (the
//! two bee kinds do not share one immunity clock in the port).

use glam::Vec2;
use tr_core::loot::DropRng;
use tr_core::World;

use crate::npc::Npc;
use crate::projectile::{
    can_hit, chaseable, Projectile, BEE, BEENADE, BLOOD_WATER, BONE_JAVELIN, GIANT_BEE,
    GRENADE, HOLY_WATER, MOLOTOV_COCKTAIL, MOLOTOV_FIRE_1, MOLOTOV_FIRE_2, MOLOTOV_FIRE_3,
    POISONED_KNIFE, STICKY_GRENADE, UNHOLY_WATER,
};

/// The aiStyle-1 kind this module owns (the javelin 507). Kind-disjoint from
/// `projectile_ai001_a/b/c`, the riders, the Book of Skulls and the
/// frostmoon/moonlord sets by construction (507 is in none of them).
pub(crate) fn covers_ai001(kind: u16) -> bool {
    kind == 507
}

/// `AI_001`'s javelin (507) body. No head hook references 507 (verified by
/// reading `:54002-55945`); 507 is NOT on the `flag3` skip list
/// (`:55947-56032`) so the counter runs; the gravity arm is the shared
/// 507/508/662/680/685 group (`:57471-57478`); the rotation tail is the
/// default (`:57748-57751`, 507 is not 344/498); the clamp is the +16 bound
/// (`:57758-57768`, `flag4` false). Movement is the kill-on-tile stand-in.
pub(crate) fn javelin_friendly_step(p: &mut Projectile, world: &World) {
    // `:56038-56041` — `if (flag3) ai[0]++`.
    p.ai[0] += 1.0;
    // `:57473-57477` — from `ai[0] > 60`: `velocity.X *= 0.98; Y += 0.3`.
    if p.ai[0] > 60.0 {
        p.vel.x *= 0.98;
        p.vel.y += 0.3;
    }
    // `:57750` — the default rotation (the literal 1.57).
    p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    // `:57764-57767` — the +16 Y bound.
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    kill_step(p, world);
}

/// The kill-on-tile movement stand-in (`HandleMovement`'s default arm
/// `Projectile.cs:19849-19853`, the `ai_style1_flight`/`kill_step` shape):
/// integrate one step, die on a solid tile. `tileCollide=false` slots
/// integrate freely (the stuck javelin).
fn kill_step(p: &mut Projectile, world: &World) {
    if !p.tile_collide {
        p.pos += p.vel;
        return;
    }
    let nx = p.pos.x + p.vel.x;
    let ny = p.pos.y + p.vel.y;
    let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
    let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
    if hit_x || hit_y {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    } else {
        p.pos.x = nx;
        p.pos.y = ny;
    }
}

/// `AI()`'s aiStyle-68 arm (`Projectile.cs:30166-30239`) for the molotov
/// cocktail (399), then the `HandleMovement`-68 collision
/// (`:19780-19786`). The 669 Ale-Throwing-Glove dust branch (`flag35`) is a
/// different kind and stays out. Zero RNG (the `alpha==0` dust `:30191-30207`
/// is skipped whole, its draws with it).
pub(crate) fn molotov_step(p: &mut Projectile, world: &World) {
    // `:30168` — `rotation += 0.25 * direction`.
    p.rot += 0.25 * p.direction as f32;
    // `:30171` — `ai[0]++`.
    p.ai[0] += 1.0;
    // `:30172-30179` — from `ai[0] >= 3`: `alpha -= 40`, clamp 0.
    if p.ai[0] >= 3.0 {
        p.alpha -= 40;
        if p.alpha < 0 {
            p.alpha = 0;
        }
    }
    // `:30180-30188` — from `ai[0] >= 15`: `Y += 0.2` (clamp 16), `X *= 0.99`.
    if p.ai[0] >= 15.0 {
        p.vel.y += 0.2;
        if p.vel.y > 16.0 {
            p.vel.y = 16.0;
        }
        p.vel.x *= 0.99;
    }
    // `:30189-30221` — the `alpha == 0` dust window (399's single fire dust,
    // `:30191-30207`) is SKIPPED (no dust layer, its draws with it).
    // `:30222` — `spriteDirection = direction`.
    p.sprite_direction = p.direction;
    // `:30223-30234` — `timeLeft <= 3`: the prime — `tileCollide=false`,
    // `alpha=255`, resize to 80x80 about the centre, `knockBack=8`.
    if p.time_left <= 3 {
        p.tile_collide = false;
        p.alpha = 255;
        let cx = p.pos.x + p.w * 0.5;
        let cy = p.pos.y + p.h * 0.5;
        p.w = 80.0;
        p.h = 80.0;
        p.pos = Vec2::new(cx - 40.0, cy - 40.0);
        p.knock_back = 8.0;
    }
    // `:30235-30238` — `wet && timeLeft > 3`: prime early. The port's slot
    // carries no liquid-wet flag (the pool-wide convention), so this folds
    // dry — recorded in ANALYSIS.
    // The `HandleMovement`-68 collision (`:19780-19786`): on a solid tile the
    // bottle stops (`velocity *= 0`), goes opaque and primes (`timeLeft=3`).
    // It does NOT die on the tile — the primed bottle bursts via the AI above.
    if p.tile_collide {
        let nx = p.pos.x + p.vel.x;
        let ny = p.pos.y + p.vel.y;
        let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
        let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
        if hit_x || hit_y {
            p.vel = Vec2::ZERO;
            p.alpha = 255;
            p.time_left = 3;
        } else {
            p.pos.x = nx;
            p.pos.y = ny;
        }
    } else {
        p.pos += p.vel;
    }
}

/// `AI_113_TargetSticker` (`Projectile.cs:40562-40899`) for the bone javelin
/// (598). `npcs` is the pool's NPC slice (the stick state reads its pinned
/// target); the stick's `ai[1]` is the NPC's index in that slice (the port's
/// stable-index convention, ANALYSIS). Zero RNG.
pub(crate) fn bone_javelin_step(p: &mut Projectile, world: &World, npcs: &[Npc]) {
    // `:40564-40576` — the alpha fade: `num=25` (598 is not 614), `alpha-=25`,
    // clamp 0.
    if p.alpha > 0 {
        p.alpha -= 25;
    }
    if p.alpha < 0 {
        p.alpha = 0;
    }
    if p.ai[0] == 0.0 {
        // `:40610-40664` — the flying arc (598 is neither 1024 nor 614).
        // `:40612` — `ai[1]++` (the flight-age counter).
        p.ai[1] += 1.0;
        // `:40613-40629` — `num4=45`; from `ai[1] >= 45`: `num5=0.98`,
        // `num6=0.35` (598 is not 636), `X *= 0.98`, `Y += 0.35`.
        if p.ai[1] >= 45.0 {
            p.vel.x *= 0.98;
            p.vel.y += 0.35;
        }
        // `:40648` — `rotation = velocity.ToRotation() + π/2` (598 not 1024).
        p.rot = p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2;
    }
    if p.ai[0] == 1.0 {
        // `:40666-40848` — the stick state.
        // `:40668` — `frame = 0`.
        p.frame = 0;
        // `:40671-40672` — `ignoreWater=true`, `tileCollide=false`.
        p.ignore_water = true;
        p.tile_collide = false;
        // `:40673-40689` — `num11=15` (598 is none of 636/971/975/1024).
        let num11 = 15.0f32;
        // `:40702` — `localAI[0]++` (the stick-age counter).
        p.local_ai[0] += 1.0;
        // `:40703-40706` — `localAI[0] % 30 == 0` → the HitEffect pulse
        // (SKIPPED — a visual, no state the port reads).
        // `:40711` — `num12 = (int)ai[1]` (the pinned NPC index).
        let num12 = p.ai[1] as i32;
        // `:40753-40773` — the expiry / target-validity ladder (598 not 1024).
        let mut kill = false;
        if p.local_ai[0] >= 60.0 * num11 {
            kill = true;
        } else if num12 < 0 || num12 as usize >= npcs.len() {
            kill = true;
        } else {
            let n = &npcs[num12 as usize];
            // `Main.npc[num12].active && !dontTakeDamage` — the port's live
            // list stands in for `active` (the `hp > 0` scan convention).
            if n.hp > 0 && !n.dont_take_damage {
                // `:40763` — `Center = npc.Center - velocity * 2`.
                let (ncx, ncy) = n.center();
                let cx = ncx - p.vel.x * 2.0;
                let cy = ncy - p.vel.y * 2.0;
                p.pos = Vec2::new(cx - p.w * 0.5, cy - p.h * 0.5);
                // `:40764` — `gfxOffY = npc.gfxOffY`.
                p.gfx_off_y = n.gfx_off_y;
                // `:40765-40768` — the 30-tick HitEffect (SKIPPED, visual).
            } else {
                kill = true;
            }
        }
        // `:40774-40791` — `flag` → `Kill()` (598 not 1024, so no ai[0]=2).
        if kill {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
            return;
        }
    }
    // `:40849-40889` — the `ai[0]==2` return-to-owner state is 1024-only.
    // Movement: the flying javelin rides the kill-on-tile default arm; the
    // stuck javelin has `tileCollide=false` so it integrates freely.
    kill_step(p, world);
}

/// `AI()`'s aiStyle-36 arm (`Projectile.cs:27292-27471`) for the bees
/// (181/566). `npcs` feeds the homing scan. Zero RNG (the arm's only rand
/// riders are the 307/316 dust, other kinds). The wet-kill head
/// (`:27294-27297`) folds dry (no liquid-wet flag on the slot, ANALYSIS).
pub(crate) fn bee_step(p: &mut Projectile, world: &World, npcs: &[Npc]) {
    // `:27298-27309` — the alpha fade-in: `alpha>0` → `alpha-=50`, else
    // `extraUpdates=0`; clamp 0.
    if p.alpha > 0 {
        p.alpha -= 50;
    } else {
        p.extra_updates = 0;
    }
    if p.alpha < 0 {
        p.alpha = 0;
    }
    // `:27334-27366` — the non-307 branch (181/566 are not 307/316): the
    // sprite direction off the velocity sign, `rotation = velocity.X * 0.1`,
    // the 3-tick frame flutter wrapping at 3.
    if p.vel.x > 0.0 {
        p.sprite_direction = 1;
    } else if p.vel.x < 0.0 {
        p.sprite_direction = -1;
    }
    p.rot = p.vel.x * 0.1;
    p.frame_counter += 1;
    if p.frame_counter >= 3 {
        p.frame += 1;
        p.frame_counter = 0;
    }
    if p.frame >= 3 {
        p.frame = 0;
    }
    // `:27367-27391` — the homing scan: `ai[0]++`, pin at 30, then the
    // closest chaseable NPC within 800 Manhattan with a clear line.
    let mut tx = p.pos.x;
    let mut ty = p.pos.y;
    let mut best = 100000.0f32;
    let mut found = false;
    p.ai[0] += 1.0;
    if p.ai[0] > 30.0 {
        p.ai[0] = 30.0;
        let cx = p.pos.x + p.w * 0.5;
        let cy = p.pos.y + p.h * 0.5;
        for n in npcs {
            // `CanBeChasedBy(this) && (!npc.wet || npc.type==370 || this.type==307)`
            // (`:27377`) — the bees are not 307, so a wet NPC is refused
            // UNLESS it is Duke Fishron (370), which the bees may chase while
            // wet (the port's `Npc.wet` carries the flag, `NpcKind::DukeFishron`
            // the type).
            if !chaseable(n) || (n.wet && n.kind != crate::npc::NpcKind::DukeFishron) {
                continue;
            }
            let (nw, nh) = n.size();
            let (nx, ny) = (n.x + nw * 0.5, n.y + nh * 0.5);
            let d = (cx - nx).abs() + (cy - ny).abs();
            if d < 800.0 && d < best && can_hit(world, p.pos.x, p.pos.y, p.w, p.h, n.x, n.y, nw, nh) {
                best = d;
                tx = nx;
                ty = ny;
                found = true;
            }
        }
    }
    // `:27392-27396` — no target: aim 100 velocity-steps ahead.
    if !found {
        tx = p.pos.x + p.w * 0.5 + p.vel.x * 100.0;
        ty = p.pos.y + p.h * 0.5 + p.vel.y * 100.0;
    }
    // `:27401-27430` — the per-kind speed/accel: the bee 6/0.1, the giant bee
    // 6.8/0.14 (`:27426-27430`); 189/307/316 are other kinds.
    let (speed, accel) = if p.kind == GIANT_BEE { (6.8, 0.14) } else { (6.0, 0.1) };
    // `:27431-27438` — the aim vector normalized to `speed`.
    let cx = p.pos.x + p.w * 0.5;
    let cy = p.pos.y + p.h * 0.5;
    let mut dx = tx - cx;
    let mut dy = ty - cy;
    let len = (dx * dx + dy * dy).sqrt();
    let scale = speed / len;
    dx *= scale;
    dy *= scale;
    // `:27439-27470` — the per-axis accel steering with the crossing boost.
    if p.vel.x < dx {
        p.vel.x += accel;
        if p.vel.x < 0.0 && dx > 0.0 {
            p.vel.x += accel * 2.0;
        }
    } else if p.vel.x > dx {
        p.vel.x -= accel;
        if p.vel.x > 0.0 && dx < 0.0 {
            p.vel.x -= accel * 2.0;
        }
    }
    if p.vel.y < dy {
        p.vel.y += accel;
        if p.vel.y < 0.0 && dy > 0.0 {
            p.vel.y += accel * 2.0;
        }
    } else if p.vel.y > dy {
        p.vel.y -= accel;
        if p.vel.y > 0.0 && dy < 0.0 {
            p.vel.y -= accel * 2.0;
        }
    }
    // Movement: the kill-on-tile default arm (no aiStyle-36 movement arm).
    kill_step(p, world);
}

/// `StatusNPC`'s thrown-consumable debuffs, called on a landed hit
/// (`Projectile.cs:13580`). The explicit rolls stay in C# order on the pass's
/// stream. The bone javelin's Daybroken is gated on `ai[1] == npc_idx` (the
/// stick pin, `:11335`) — the pool sets the pin BEFORE this call (the
/// `:13210-13217` stick runs ahead of `StatusNPC`).
pub(crate) fn thrown_on_hit(p: &Projectile, npc: &mut Npc, npc_idx: u16, rng: &mut dyn DropRng) {
    // `:11628-11634` — the poisoned knife (54): 1/2 Poisoned 600.
    if p.kind == POISONED_KNIFE && rng.next_range(0, 2) == 0 {
        npc.buffs.add_buff(20, 600);
    }
    // `:11506-11509` — the molotov (399) + the greek fires (400-402):
    // OnFire `60 * Next(3,7)`.
    if matches!(p.kind, MOLOTOV_COCKTAIL | MOLOTOV_FIRE_1 | MOLOTOV_FIRE_2 | MOLOTOV_FIRE_3) {
        npc.buffs.add_buff(24, 60 * rng.next_range(3, 7));
    }
    // `:11335-11338` — the bone javelin (598): Daybroken 900 when pinned to
    // this NPC (`ai[1] == i`).
    if p.kind == BONE_JAVELIN && p.ai[1] == npc_idx as f32 {
        npc.buffs.add_buff(169, 900);
    }
}

/// The water bottles' splash conversion — `Kill()`'s `:77629-77645` arm:
/// `WorldGen.Convert(centerTile, conversionType, 4)` (the size-4 overload
/// `WorldGen.cs:55067-55083`, the Manhattan<6 diamond). The holy water
/// hallow (type 2), the unholy water corruption (type 1), the blood water
/// crimson (type 4). The port's `convert_cell_flavor` is the per-cell map
/// (the `convert_cleanse` precedent); the reframe rides its documented
/// `reframe_area` superset.
pub(crate) fn water_death(p: &Projectile, world: &mut World) {
    let flavor = match p.kind {
        HOLY_WATER => tr_core::spread::ConversionFlavor::Hallow,
        UNHOLY_WATER => tr_core::spread::ConversionFlavor::Corruption,
        BLOOD_WATER => tr_core::spread::ConversionFlavor::Crimson,
        _ => return,
    };
    // `:77631-77632` — the centre tile (`(position + width/2) / 16`).
    let i = (p.pos.x + p.w * 0.5) as i32 / 16;
    let j = (p.pos.y + p.h * 0.5) as i32 / 16;
    // `WorldGen.Convert(i, j, type, 4)` (`:55073-55082`): the size-4 box with
    // the `|k-i| + |l-j| < 6` diamond and the `InWorld(k, l, 1)` guard.
    for k in (i - 4)..=(i + 4) {
        for l in (j - 4)..=(j + 4) {
            if (k - i).abs() + (l - j).abs() < 6
                && k >= 1
                && l >= 1
                && k < world.width - 1
                && l < world.height - 1
            {
                tr_core::spread::convert_cell_flavor(world, k, l, flavor, true);
            }
        }
    }
}

/// The beenade's bee-burst spawn request — one bee's resolved stats. The pool
/// resolves it through `spawn_with_data_ai_owned` (the beenade's owner). The
/// draws that produced it are consumed by [`beenade_death_draws`] in C# order.
pub(crate) struct BeeSpawn {
    pub kind: u16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub knock_back: f32,
}

/// `Kill()`'s beenade arm (`Projectile.cs:76431-76463`): the bee burst. The
/// four gore type picks + `NewGore` kicks are the G3 draw-census (kept, bodies
/// skipped); then the owner-gated `Next(15,25)` bee count and, per bee, the
/// velocity pair + `beeType`/`beeDamage`/`beeKB`. Returns the resolved bee
/// spawns. `strong_bees` mirrors `Player.strongBees` (the Hive Pack accessory,
/// `Player.cs:12417-12426`); `base_damage` is the beenade's `this.damage`.
///
/// The C# named-argument evaluation order (`:76460`): SpeedX, SpeedY, then
/// Type (`beeType()`), Damage (`beeDamage`), KnockBack (`beeKB(0f)`) — the
/// velocity rolls precede the bee-type coin flip (the `hurt_modifiers` order).
pub(crate) fn beenade_death(
    p: &Projectile,
    strong_bees: bool,
    rng: &mut dyn DropRng,
) -> Vec<BeeSpawn> {
    // `:76439-76454` — the four gores: each `Next(61,64)` type + the NewGore
    // kicks `Next(10,31)` Y-then-`Next(-20,21)` X (`Gore.cs:1327-1328`). The
    // bodies are skipped; the draws stay (the G3 standard).
    for _ in 0..4 {
        let _ = rng.next_range(61, 64);
        let _ = rng.next_range(10, 31);
        let _ = rng.next_range(-20, 21);
    }
    let mut out = Vec::new();
    // `:76455-76462` — the owner-gated bee burst. The pool only dispatches
    // locally-owned slots, so the gate folds true (the saucer precedent).
    // `:76457` — `num832 = Next(15,25)`.
    let count = rng.next_range(15, 25);
    for _ in 0..count {
        // `:76460` — SpeedX then SpeedY: `Next(-35,36) * 0.02` each.
        let vx = rng.next_range(-35, 36) as f32 * 0.02;
        let vy = rng.next_range(-35, 36) as f32 * 0.02;
        // `beeType()` (`Player.cs:12417-12426`): the strong-bee coin flip.
        let (kind, strong) = if strong_bees && rng.next_range(0, 2) == 0 {
            (GIANT_BEE, true)
        } else {
            (BEE, false)
        };
        // `beeDamage(this.damage)` (`Player.cs:12428-12435`).
        let damage = if strong {
            p.damage + rng.next_range(1, 4)
        } else {
            p.damage + rng.next_range(0, 2)
        };
        // `beeKB(0f)` (`Player.cs:12437-12444`): strong `0.5 + 0*1.1 = 0.5`.
        let knock_back = if strong { 0.5 } else { 0.0 };
        out.push(BeeSpawn {
            kind,
            x: p.pos.x,
            y: p.pos.y,
            vx,
            vy,
            damage,
            knock_back,
        });
    }
    out
}

/// The molotov's greek-fire spawn request — one fire's resolved stats.
pub(crate) struct FireSpawn {
    pub kind: u16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
}

/// `Kill()`'s molotov arm (`Projectile.cs:73921-73951`): the six greek fires.
/// The dust bodies are skipped (their `Dust.NewDust` internal rolls go with
/// the dust pool); the owner-gated fire loop's explicit draws stay. Per fire
/// (`:73946-73948`): the X offset `(-velocity.X) * Next(20,50) * 0.01 +
/// Next(-20,21) * 0.4`, the Y offset `(-|velocity.Y|) * Next(30,50) * 0.01 +
/// Next(-20,5) * 0.4`, the spawn at `Center + (offX, offY)` with velocity
/// `(offX, offY)`, type `400 + Next(3)`, damage `(int)(damage * 0.5)`, kb 0.
pub(crate) fn molotov_death(p: &Projectile, rng: &mut dyn DropRng) -> Vec<FireSpawn> {
    let mut out = Vec::new();
    let cx = p.pos.x + p.w * 0.5;
    let cy = p.pos.y + p.h * 0.5;
    // `:73942-73950` — the owner-gated six-fire loop (folds true, the pool
    // dispatches locally-owned slots).
    for _ in 0..6 {
        // `:73946` — the X offset: two draws, `Next(20,50)` then `Next(-20,21)`.
        let off_x = (-p.vel.x) * rng.next_range(20, 50) as f32 * 0.01
            + rng.next_range(-20, 21) as f32 * 0.4;
        // `:73947` — the Y offset: `Next(30,50)` then `Next(-20,5)`.
        let off_y = (-p.vel.y.abs()) * rng.next_range(30, 50) as f32 * 0.01
            + rng.next_range(-20, 5) as f32 * 0.4;
        // `:73948` — the fire type `400 + Next(3)` (the THIRD draw of the row).
        let kind = 400 + rng.next_range(0, 3) as u16;
        out.push(FireSpawn {
            kind,
            x: cx + off_x,
            y: cy + off_y,
            vx: off_x,
            vy: off_y,
            damage: (p.damage as f64 * 0.5) as i32,
        });
    }
    out
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value below is hand-derived from the cited C# lines.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;

    /// A scripted `DropRng` recording every `(min,max)` draw — the draw-order
    /// contract mirror (the `hurt_modifiers`/`buff_effects_accessory` pattern).
    struct ScriptRng {
        rolls: Vec<i32>,
        draws: Vec<(i32, i32)>,
    }
    impl ScriptRng {
        fn with_rolls(rolls: &[i32]) -> Self {
            Self { rolls: rolls.to_vec(), draws: Vec::new() }
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

    fn proj(kind: u16) -> Projectile {
        let mut p = Projectile { kind, ..Default::default() };
        if let Some(d) = proj_data(kind) {
            p.w = d.w;
            p.h = d.h;
            p.alpha = d.alpha;
            p.time_left = d.time_left;
            p.penetrate = d.penetrate;
            p.max_penetrate = d.penetrate;
            p.extra_updates = d.extra_updates;
            p.tile_collide = d.tile_collide;
        }
        p
    }

    fn empty_world() -> World {
        World::new("t".into(), 64, 800)
    }

    /// The javelin (507) flies straight for 60 ticks, then falls 0.3/tick with
    /// the 0.98 X decay (`Projectile.cs:57473-57477`).
    #[test]
    fn s1_p_i28_javelin_falls_after_sixty_ticks() {
        let w = empty_world();
        let mut p = proj(507);
        p.vel = Vec2::new(10.0, 0.0);
        p.pos = Vec2::new(500.0, 500.0);
        for _ in 0..60 {
            javelin_friendly_step(&mut p, &w);
        }
        assert_eq!(p.ai[0], 60.0, "the counter runs every tick");
        assert_eq!(p.vel.y, 0.0, "no gravity through tick 60");
        javelin_friendly_step(&mut p, &w);
        assert_eq!(p.ai[0], 61.0);
        assert!((p.vel.y - 0.3).abs() < 1e-6, "tick 61 adds 0.3");
        assert!((p.vel.x - 9.8).abs() < 1e-6, "tick 61 decays X by 0.98");
    }

    /// The molotov fades in over 3 ticks then falls after 15
    /// (`Projectile.cs:30172-30188`).
    #[test]
    fn s1_p_i28_molotov_fade_and_gravity() {
        let w = empty_world();
        let mut p = proj(MOLOTOV_COCKTAIL);
        p.pos = Vec2::new(500.0, 500.0);
        p.vel = Vec2::new(4.0, 0.0);
        p.direction = 1;
        // alpha starts 255; tick 1/2 no fade (ai[0] < 3), tick 3 fades 40.
        molotov_step(&mut p, &w);
        assert_eq!(p.alpha, 255, "ai[0]=1 < 3, no fade");
        molotov_step(&mut p, &w);
        assert_eq!(p.alpha, 255, "ai[0]=2 < 3, no fade");
        molotov_step(&mut p, &w);
        assert_eq!(p.alpha, 215, "ai[0]=3 fades 40");
        assert!((p.rot - 0.75).abs() < 1e-6, "3 ticks of 0.25 spin");
        for _ in 0..12 {
            molotov_step(&mut p, &w);
        }
        // ai[0]=15 now: gravity armed.
        assert!(p.vel.y > 0.0, "tick 15 adds 0.2 gravity");
    }

    /// The molotov primes (resize 80, kb 8, alpha 255) at timeLeft<=3
    /// (`Projectile.cs:30223-30234`).
    #[test]
    fn s1_p_i28_molotov_primes_at_low_timeleft() {
        let w = empty_world();
        let mut p = proj(MOLOTOV_COCKTAIL);
        p.pos = Vec2::new(500.0, 500.0);
        p.time_left = 3;
        molotov_step(&mut p, &w);
        assert_eq!(p.w, 80.0);
        assert_eq!(p.h, 80.0);
        assert_eq!(p.knock_back, 8.0);
        assert_eq!(p.alpha, 255);
        assert!(!p.tile_collide);
        // The centre is preserved: 500 + 7 = 507 → 507 - 40 = 467.
        assert!((p.pos.x - 467.0).abs() < 1e-6);
    }

    /// The bone javelin's flying arc falls FROM age 45 (`Projectile.cs:40618`
    /// `if (this.ai[1] >= (float)num4)` with `num4 = 45`): `ai[1]++` runs
    /// first, so the 45th tick already reaches 45 and applies the gravity.
    #[test]
    fn s1_p_i28_bone_javelin_flying_arc() {
        let w = empty_world();
        let mut p = proj(BONE_JAVELIN);
        p.pos = Vec2::new(500.0, 500.0);
        p.vel = Vec2::new(10.0, 0.0);
        p.ai[0] = 0.0;
        for _ in 0..44 {
            bone_javelin_step(&mut p, &w, &[]);
        }
        assert_eq!(p.ai[1], 44.0, "the flight-age counter");
        assert_eq!(p.vel.y, 0.0, "no gravity through age 44");
        bone_javelin_step(&mut p, &w, &[]);
        assert_eq!(p.ai[1], 45.0);
        assert!((p.vel.y - 0.35).abs() < 1e-6, "age 45 adds 0.35 (>= 45)");
        assert!((p.vel.x - 9.8).abs() < 1e-6, "age 45 decays X by 0.98");
    }

    /// The stuck javelin expires at localAI[0] >= 900 (60*15)
    /// (`Projectile.cs:40753`).
    #[test]
    fn s1_p_i28_bone_javelin_stick_expires() {
        let w = empty_world();
        let mut p = proj(BONE_JAVELIN);
        p.ai[0] = 1.0;
        p.ai[1] = 0.0; // pinned to NPC 0
        p.vel = Vec2::new(2.0, 0.0);
        let mut n = Npc::new(crate::npc::NpcKind::Zombie, 600.0, 500.0, 0, 0.0);
        n.hp = n.max_hp;
        p.local_ai[0] = 898.0;
        bone_javelin_step(&mut p, &w, std::slice::from_ref(&n));
        assert!(p.active, "899 < 900 still stuck");
        bone_javelin_step(&mut p, &w, std::slice::from_ref(&n));
        assert!(!p.active, "900 >= 900 expires");
    }

    /// The bee's homing steering accelerates toward the target
    /// (`Projectile.cs:27439-27470`); the giant bee uses 6.8/0.14.
    #[test]
    fn s1_p_i28_bee_steers_toward_target() {
        let w = empty_world();
        let mut p = proj(BEE);
        p.pos = Vec2::new(500.0, 500.0);
        p.vel = Vec2::ZERO;
        p.alpha = 0;
        p.ai[0] = 30.0; // the scan is armed
        let mut n = Npc::new(crate::npc::NpcKind::Zombie, 600.0, 500.0, 0, 0.0);
        n.hp = n.max_hp;
        n.max_hp = 100;
        bee_step(&mut p, &w, &[n]);
        // The target is to the right (+X): the bee accelerates +0.1.
        assert!(p.vel.x > 0.0, "the bee steers toward the NPC");
        assert_eq!(p.extra_updates, 0, "alpha 0 collapses extraUpdates");
    }

    /// The poisoned knife's 1/2 Poisoned roll (`Projectile.cs:11630-11632`).
    #[test]
    fn s1_p_i28_poisoned_knife_half_roll() {
        let p = proj(POISONED_KNIFE);
        let mut n = Npc::new(crate::npc::NpcKind::Zombie, 0.0, 0.0, 0, 0.0);
        // roll 0 → Poisoned 600.
        let mut rng = ScriptRng::with_rolls(&[0]);
        thrown_on_hit(&p, &mut n, 0, &mut rng);
        assert_eq!(rng.draws, vec![(0, 2)]);
        assert!(n.buffs.find_buff_index(20).is_some(), "roll 0 applies Poisoned");
        // roll 1 → no debuff.
        let mut n2 = Npc::new(crate::npc::NpcKind::Zombie, 0.0, 0.0, 0, 0.0);
        let mut rng2 = ScriptRng::with_rolls(&[1]);
        thrown_on_hit(&p, &mut n2, 0, &mut rng2);
        assert!(n2.buffs.find_buff_index(20).is_none(), "roll 1 saves");
    }

    /// The molotov's OnFire `60 * Next(3,7)` (`Projectile.cs:11508`).
    #[test]
    fn s1_p_i28_molotov_onfire_roll() {
        let p = proj(MOLOTOV_COCKTAIL);
        let mut n = Npc::new(crate::npc::NpcKind::Zombie, 0.0, 0.0, 0, 0.0);
        let mut rng = ScriptRng::with_rolls(&[5]);
        thrown_on_hit(&p, &mut n, 0, &mut rng);
        assert_eq!(rng.draws, vec![(3, 7)]);
        assert!(n.buffs.find_buff_index(24).is_some(), "OnFire applied");
    }

    /// The bone javelin's Daybroken gates on the stick pin
    /// (`Projectile.cs:11335`).
    #[test]
    fn s1_p_i28_bone_javelin_daybroken_pin_gate() {
        let mut p = proj(BONE_JAVELIN);
        p.ai[1] = 3.0; // pinned to NPC 3
        let mut n = Npc::new(crate::npc::NpcKind::Zombie, 0.0, 0.0, 0, 0.0);
        let mut rng = ScriptRng::with_rolls(&[]);
        thrown_on_hit(&p, &mut n, 3, &mut rng);
        assert!(n.buffs.find_buff_index(169).is_some(), "pin matches → Daybroken");
        assert!(rng.draws.is_empty(), "Daybroken is unconditional");
        // A different NPC index → no Daybroken.
        let mut n2 = Npc::new(crate::npc::NpcKind::Zombie, 0.0, 0.0, 0, 0.0);
        thrown_on_hit(&p, &mut n2, 4, &mut rng);
        assert!(n2.buffs.find_buff_index(169).is_none(), "pin mismatch → none");
    }

    /// The beenade death burst's draw order: 4 gore rows, then Next(15,25),
    /// then per bee the velocity pair + beeType flip + beeDamage
    /// (`Projectile.cs:76439-76460`). With `strongBees` OFF the `beeType()`
    /// coin flip short-circuits (`strongBees && Next(2)==0`, `Player.cs:12419`)
    /// — it draws NOTHING, so each plain bee is only vx + vy + beeDamage = 3
    /// draws (the `hurt_modifiers` bee_type order).
    #[test]
    fn s1_p_i28_beenade_death_draw_order() {
        let mut p = proj(BEENADE);
        p.damage = 12;
        // 4 gore rows (3 draws each) = 12, then count=15 (roll 15).
        let mut rolls = vec![61, 10, -20, 62, 11, -19, 63, 12, -18, 61, 13, -17];
        rolls.push(15); // Next(15,25) → 15 bees
        // Each plain bee (strongBees off): vx=0, vy=0, NO coin flip, beeDamage=0.
        for _ in 0..15 {
            rolls.extend_from_slice(&[0, 0, 0]);
        }
        let mut rng = ScriptRng::with_rolls(&rolls);
        let bees = beenade_death(&p, false, &mut rng);
        assert_eq!(bees.len(), 15, "Next(15,25) rolled 15");
        // The first 12 draws are the gore rows.
        assert_eq!(&rng.draws[..12], &[
            (61, 64), (10, 31), (-20, 21),
            (61, 64), (10, 31), (-20, 21),
            (61, 64), (10, 31), (-20, 21),
            (61, 64), (10, 31), (-20, 21),
        ]);
        assert_eq!(rng.draws[12], (15, 25), "the bee count draw");
        assert_eq!(rng.draws[13], (-35, 36), "first bee SpeedX");
        assert_eq!(rng.draws[14], (-35, 36), "first bee SpeedY");
        // strongBees off → the coin flip short-circuits, the next draw is beeDamage.
        assert_eq!(rng.draws[15], (0, 2), "first bee beeDamage (plain, no flip draw)");
        assert_eq!(rng.draws.len(), 12 + 1 + 15 * 3, "3 draws per plain bee");
        assert_eq!(bees[0].kind, BEE, "no strong bees → plain bee");
        assert_eq!(bees[0].damage, 12, "12 + Next(0,2)=0");
        assert_eq!(bees[0].knock_back, 0.0, "beeKB(0) plain");
    }

    /// The strong-bee beenade burst (`strongBees` ON, the Hive Pack): each bee
    /// now draws the coin flip (`Next(2)`), and a flip of 0 makes a giant bee
    /// (566) with `beeDamage = damage + Next(1,4)` and `beeKB(0) = 0.5`
    /// (`Player.cs:12417-12444`). Per bee: vx, vy, flip, beeDamage = 4 draws.
    #[test]
    fn s1_p_i28_beenade_death_strong_bee_draw_order() {
        let mut p = proj(BEENADE);
        p.damage = 12;
        let mut rolls = vec![61, 10, -20, 62, 11, -19, 63, 12, -18, 61, 13, -17];
        rolls.push(15); // count = 15 bees
        // Each strong bee: vx=0, vy=0, flip=0 (→ giant), beeDamage=2.
        for _ in 0..15 {
            rolls.extend_from_slice(&[0, 0, 0, 2]);
        }
        let mut rng = ScriptRng::with_rolls(&rolls);
        let bees = beenade_death(&p, true, &mut rng);
        assert_eq!(bees.len(), 15);
        assert_eq!(rng.draws[13], (-35, 36), "first bee SpeedX");
        assert_eq!(rng.draws[14], (-35, 36), "first bee SpeedY");
        assert_eq!(rng.draws[15], (0, 2), "first bee beeType flip (strongBees on)");
        assert_eq!(rng.draws[16], (1, 4), "first bee beeDamage (strong)");
        assert_eq!(rng.draws.len(), 12 + 1 + 15 * 4, "4 draws per strong bee");
        assert_eq!(bees[0].kind, GIANT_BEE, "flip 0 → giant bee");
        assert_eq!(bees[0].damage, 14, "12 + Next(1,4)=2");
        assert_eq!(bees[0].knock_back, 0.5, "beeKB(0) strong = 0.5");
    }

    /// The molotov death burst's per-fire draw order: X pair, Y pair, type
    /// (`Projectile.cs:73946-73948`), six fires.
    #[test]
    fn s1_p_i28_molotov_death_draw_order() {
        let mut p = proj(MOLOTOV_COCKTAIL);
        p.damage = 23;
        p.vel = Vec2::new(2.0, -3.0);
        // Per fire: Next(20,50), Next(-20,21), Next(30,50), Next(-20,5),
        // Next(0,3). Script 6 fires' worth.
        let mut rolls = Vec::new();
        for _ in 0..6 {
            rolls.extend_from_slice(&[20, 0, 30, 0, 0]);
        }
        let mut rng = ScriptRng::with_rolls(&rolls);
        let fires = molotov_death(&p, &mut rng);
        assert_eq!(fires.len(), 6);
        assert_eq!(&rng.draws[..5], &[
            (20, 50), (-20, 21), (30, 50), (-20, 5), (0, 3),
        ]);
        assert_eq!(fires[0].kind, 400, "400 + Next(0,3)=0");
        assert_eq!(fires[0].damage, 11, "(int)(23 * 0.5) = 11");
        // off_x = (-2)*20*0.01 + 0*0.4 = -0.4; off_y = (-3)*30*0.01 + 0 = -0.9.
        assert!((fires[0].vx - (-0.4)).abs() < 1e-6);
        assert!((fires[0].vy - (-0.9)).abs() < 1e-6);
    }

    /// The water splash converts the Manhattan<6 diamond
    /// (`WorldGen.cs:55073-55082`).
    #[test]
    fn s1_p_i28_holy_water_splash_hallows() {
        let mut w = empty_world();
        // A dirt tile (kind 0) at the centre and a stone (1) nearby.
        let i = w.index(30, 300);
        w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = proj(HOLY_WATER);
        p.pos = Vec2::new(30.0 * 16.0, 300.0 * 16.0);
        water_death(&p, &mut w);
        // The stone at the centre hallow-converts (1 → 117 pearlstone).
        let t = w.tile(30, 300).unwrap();
        assert_eq!(t.kind, 117, "the centre stone hallow-converts");
    }

    /// The data rows land with the C# SetDefaults values.
    #[test]
    fn s1_p_i28_thrown_rows_match_setdefaults() {
        assert_eq!(proj_data(POISONED_KNIFE).unwrap().ai_style, 2);
        assert_eq!(proj_data(GRENADE).unwrap().time_left, 180);
        assert_eq!(proj_data(STICKY_GRENADE).unwrap().tile_collide, false);
        assert_eq!(proj_data(MOLOTOV_COCKTAIL).unwrap().ai_style, 68);
        assert_eq!(proj_data(MOLOTOV_COCKTAIL).unwrap().alpha, 255);
        assert_eq!(proj_data(BEENADE).unwrap().ai_style, 14);
        assert_eq!(proj_data(BEENADE).unwrap().time_left, 180);
        assert_eq!(proj_data(BONE_JAVELIN).unwrap().ai_style, 113);
        assert_eq!(proj_data(BEE).unwrap().ai_style, 36);
        assert_eq!(proj_data(BEE).unwrap().extra_updates, 3);
        assert_eq!(proj_data(GIANT_BEE).unwrap().penetrate, 4);
        assert_eq!(proj_data(MOLOTOV_FIRE_1).unwrap().penetrate, 3);
        // The Javelin (507) row — the Javelin item's (3094) shot. Without it
        // `spawn_with_data_ai` no-ops and the javelin never spawns; the flight
        // arm (`covers_ai001`) only runs once the row lands.
        let jav = proj_data(crate::projectile::JAVELIN).expect("the javelin row");
        assert_eq!(jav.ai_style, 1);
        assert_eq!(jav.penetrate, 3);
        assert_eq!((jav.w, jav.h), (16.0, 16.0));
        assert!(crate::projectile_ai_thrown::covers_ai001(507));
    }
}