//! S1-E-7 wave E — the `Player.UpdateBuffs` fourth missing block
//! (`Player.cs:11406-12012`, 37 ids: the late pet family 84…136, the
//! DoT/status tail 80/320/353/163/164/194/195/196/197/198/145/43, Werewolf
//! 28, the hunger tiers 333/334 and Hearty Meal 336).
//!
//! Claim + live-set census: `docs/research/28e-wave-e-claim.md` (c16 owns
//! `:9914-10048`, c17 `:10318-10461`, c18 `:10462-10976`; wave B's
//! `:10049-10317` rows stay wave-B-owned even where still unwired).
//!
//! Disjoint-wave design (the wave-B seam pattern): this module is a PURE
//! effect table — `apply_status_effect_arm` matches one `buffType[j]` id
//! and folds its writes into a `StatusEffects` accumulator, exactly like
//! the C# arm writes into the `Player` fields. `run_status_buff_chain` is
//! the slot-ordered driver over `&mut BuffSlots` mirroring the C# `for j`
//! loop with the `DelBuff(j); j--` rescan (Werewolf `:11907-11908`) and
//! the delete-without-rescan `continue` (Shimmering `:11797`/`:11823`).
//! Wiring into the live tick is the waves' merge step, not this wave's —
//! until then the module is test-only: no runtime behaviour change, no
//! RNG or fixed-step-order change. `buffs.rs`/`player.rs` are untouched.
//!
//! RNG census of the block: ZERO direct `Main.rand` draws. The only
//! indirect rolls are the pet `Projectile.NewProjectile` init draws
//! (`:11417` etc.) — recorded as spawn REQUESTS, the pool seam owns them
//! (the wired accessory module's `pool.spawn` precedent).

use crate::buffs::{BuffSlots, MAX_BUFFS};

/// The per-tick context the arms read besides `buffType`/`buffTime`.
/// `whoAmI == Main.myPlayer` collapses to true in the single-player port
/// (Shimmering's remote-player early `continue` `:11789-11792` never fires),
/// matching every other ported `UpdateBuffs` arm.
pub struct StatusEffectContext<'a> {
    /// `ownedProjectileCounts` (`Player.cs:12184-12215` recount) — the pet
    /// spawn gates read it (`:11411` etc.).
    pub owned_projectile_counts: &'a [i32],
    /// `Main.dayTime` — the Werewolf arm's gate (`:11895`).
    pub day_time: bool,
    /// `wolfAcc` / `merman` (`:11895`) — the Moon Charm / Neptune's Shell
    /// accessory flags (`UpdateEquips` writes them, `:15136-15137`).
    pub wolf_acc: bool,
    pub merman: bool,
    /// `shimmerWet` (`:11801`).
    pub shimmer_wet: bool,
    /// The entity box — the pet spawn centre (`base.position + width/2,
    /// height/2`, `:11417`) and Shimmering's tile scan / fallStart
    /// (`:11788-11816`).
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
    /// `Main.UnderworldLayer` (`:11793`).
    pub underworld_layer: f32,
    /// `WorldGen.SolidTile3(x, y)` (`:11811`) — supplied by the caller so
    /// this table stays world-free; the scan loop itself is ported here.
    pub solid3: &'a dyn Fn(i32, i32) -> bool,
}

/// One late-pet row: the buff id, the `Player` pet-flag field name (the
/// C# bool the arm sets), the pet projectile kind and the C# arm head.
/// The 20 plain rows are verbatim copies of the
/// `BuffHandle_SpawnPetIfNeededAndSetTime` shape (`Player.cs:12227-12258`):
/// pin 18000, set the flag, spawn at the entity centre only while the
/// owned count is 0. Crystal Leaf (60) has its own arm (the dedupe walk).
pub struct StatusPetArm {
    pub buff: i32,
    pub flag: &'static str,
    pub proj: u16,
}

/// The late pet family (`Player.cs:11406-11706`), in C# chain order.
pub const STATUS_PET_ARMS: &[StatusPetArm] = &[
    StatusPetArm { buff: 84, flag: "blackCat", proj: 319 },        // `:11406-11419`
    StatusPetArm { buff: 61, flag: "dino", proj: 236 },            // `:11420-11433`
    StatusPetArm { buff: 154, flag: "babyFaceMonster", proj: 499 }, // `:11434-11447`
    StatusPetArm { buff: 65, flag: "eyeSpring", proj: 268 },       // `:11448-11461`
    StatusPetArm { buff: 66, flag: "snowman", proj: 269 },         // `:11462-11475`
    StatusPetArm { buff: 42, flag: "turtle", proj: 127 },          // `:11476-11489`
    StatusPetArm { buff: 45, flag: "eater", proj: 175 },           // `:11490-11503`
    StatusPetArm { buff: 50, flag: "skeletron", proj: 197 },       // `:11504-11517`
    StatusPetArm { buff: 51, flag: "hornet", proj: 198 },          // `:11518-11531`
    StatusPetArm { buff: 52, flag: "tiki", proj: 199 },            // `:11532-11545`
    StatusPetArm { buff: 53, flag: "lizard", proj: 200 },          // `:11546-11559`
    StatusPetArm { buff: 54, flag: "parrot", proj: 208 },          // `:11560-11573`
    StatusPetArm { buff: 55, flag: "truffle", proj: 209 },         // `:11574-11587`
    StatusPetArm { buff: 56, flag: "sapling", proj: 210 },         // `:11588-11601`
    StatusPetArm { buff: 85, flag: "cSapling", proj: 324 },        // `:11602-11615`
    StatusPetArm { buff: 81, flag: "spider", proj: 313 },          // `:11616-11629`
    StatusPetArm { buff: 82, flag: "squashling", proj: 314 },      // `:11630-11643`
    StatusPetArm { buff: 57, flag: "wisp", proj: 211 },            // `:11644-11657`
    StatusPetArm { buff: 127, flag: "zephyrfish", proj: 380 },     // `:11679-11692`
    StatusPetArm { buff: 136, flag: "miniMinotaur", proj: 398 },   // `:11693-11706`
];

/// One tick's accumulated writes of the block. Flags OR, stats ADD per
/// slot exactly like the C# chain (two Hungry slots stack); the `Option`
/// setters are assignments (`buffTime[j] = …`, `fallStart = …`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatusEffects {
    /// `DelBuff(j)` (`:11797`, `:11823`, `:11907`).
    pub delete_buff: bool,
    /// The `j--` rescan after the delete — only Werewolf does it
    /// (`:11908`); Shimmering's deletes `continue` WITHOUT the rescan.
    pub rescan_slot: bool,
    /// `buffTime[j] = …` — 18000 (pets), 60/6 (Shimmering).
    pub buff_time_set: Option<i32>,
    /// The pet flags the arms set (the C# bool field name, one per row).
    pub pet_flags: Vec<&'static str>,
    /// `Projectile.NewProjectile(source, position.X + width/2,
    /// position.Y + height/2, 0, 0, kind, 0, 0f, whoAmI)` requests
    /// (`:11417` etc.) — only when the owned count is 0.
    pub pet_spawns: Vec<u16>,
    /// Crystal Leaf (60): kill every owned 226 beyond the first
    /// (`:11663-11673`); the pool walk itself rides the wiring step.
    pub crystal_leaf_dedupe: bool,
    // DoT / status flags (`:11727-11885`).
    pub blackout: bool,               // 80, `:11729`
    pub dripping_sparkle_slime: bool, // 320, `:11758`
    pub shimmering: bool,             // 353, `:11786`
    pub frozen: bool,                 // 353, `:11787`
    pub headcovered: bool,            // 163, `:11828`
    pub bleed: bool,                  // 163, `:11829`
    pub vortex_debuff: bool,          // 164, `:11833`
    pub wind_pushed: bool,            // 194, `:11837`
    pub withered_armor: bool,         // 195, `:11841`
    pub withered_weapon: bool,        // 196, `:11849`
    pub slow_ogre_spit: bool,         // 197, `:11853`
    pub parry_damage_buff: bool,      // 198, `:11857`
    pub moon_leech: bool,             // 145, `:11861`
    pub defended_by_paladin: bool,    // 43, `:11884`
    pub were_wolf: bool,              // 28, `:11898`
    pub hungry: bool,                 // 333, `:11981`
    pub starving: bool,               // 334, `:11996`
    pub hearty_meal: bool,            // 336, `:12011`
    /// Shimmering's `fallStart = (int)(position.Y / 16f)` (`:11788`).
    pub fall_start_set: Option<i32>,
    // Stat adds (per-slot, summed like the C# chain).
    pub stat_defense: i32,
    pub melee_crit: i32,
    pub ranged_crit: i32,
    pub magic_crit: i32,
    pub melee_damage: f32,
    pub ranged_damage: f32,
    pub magic_damage: f32,
    pub minion_damage: f32,
    pub minion_kb: f32,
    pub melee_speed: f32,
    pub move_speed: f32,
    /// Signed like the C#: Hungry/Starving ADD to `pickSpeed` (a higher
    /// pickSpeed mines slower); the wiring maps it onto the port's
    /// positive-bonus convention with the sign flip.
    pub pick_speed: f32,
    pub life_regen: i32,
}

/// The plain late-pet arm (`Player.cs:11406-11419` shape, the inline copy
/// of `BuffHandle_SpawnPetIfNeededAndSetTime` `:12227-12258`): pin the
/// time, set the flag, spawn at the entity centre when the owned count
/// is 0. The `whoAmI == Main.myPlayer` gate (`:11415`) is always true in
/// the single-player port.
fn apply_plain_pet_arm(e: &mut StatusEffects, ctx: &StatusEffectContext, arm: &StatusPetArm) {
    e.buff_time_set = Some(18000); // `:11408`
    e.pet_flags.push(arm.flag); // `:11409`
    let owned = ctx
        .owned_projectile_counts
        .get(usize::from(arm.proj))
        .copied()
        .unwrap_or(0);
    if owned == 0 {
        e.pet_spawns.push(arm.proj); // `:11415-11418`
    }
}

/// The Crystal Leaf arm (`Player.cs:11658-11678`): unlike the other pets
/// it walks the pool and kills every owned 226 beyond the first
/// (`:11663-11673`); it spawns only when NONE is owned. The kill walk
/// rides the wiring step (`crystal_leaf_dedupe`); the count gates are
/// exact from the `UpdateProjectileCaches` recount.
fn apply_crystal_leaf_arm(e: &mut StatusEffects, ctx: &StatusEffectContext) {
    e.buff_time_set = Some(18000); // `:11660`
    e.pet_flags.push("crystalLeaf"); // `:11661`
    let owned = ctx.owned_projectile_counts.get(226).copied().unwrap_or(0);
    if owned > 1 {
        e.crystal_leaf_dedupe = true; // `:11667-11670`
    }
    if owned == 0 {
        e.pet_spawns.push(226); // `:11674-11677`
    }
}

/// The Shimmering arm (`Player.cs:11784-11825`): the absorbed state —
/// `shimmering + frozen`, `fallStart` reset; below the underworld layer
/// the buff dies, in shimmer it holds 60 ticks, overlapping a solid tile
/// holds 6, otherwise it dies. The `SolidTile3` box scan
/// (`:11807-11816`) is ported verbatim against the caller's predicate —
/// the loop bounds keep the C#'s float comparison (`(float)num <=
/// (pos + size) / 16f`), inclusive on the far edge. None of the three
/// deletes rescans the slot (the C# `continue`s, unlike Werewolf).
fn apply_shimmering_arm(e: &mut StatusEffects, ctx: &StatusEffectContext) {
    e.shimmering = true; // `:11786`
    e.frozen = true; // `:11787`
    e.fall_start_set = Some((ctx.y / 16.0) as i32); // `:11788`
    if ctx.y / 16.0 > ctx.underworld_layer {
        // `:11793-11800`
        e.delete_buff = true;
        return;
    }
    if ctx.shimmer_wet {
        // `:11801-11805`
        e.buff_time_set = Some(60);
        return;
    }
    let mut flag32 = false; // `:11806`
    let mut num20 = (ctx.x / 16.0) as i32;
    while num20 as f32 <= (ctx.x + ctx.width as f32) / 16.0 {
        let mut num21 = (ctx.y / 16.0) as i32;
        while num21 as f32 <= (ctx.y + ctx.height as f32) / 16.0 {
            if (ctx.solid3)(num20, num21) {
                flag32 = true; // `:11811-11815`
            }
            num21 += 1;
        }
        num20 += 1;
    }
    if flag32 {
        e.buff_time_set = Some(6); // `:11817-11820`
    } else {
        e.delete_buff = true; // `:11821-11824`
    }
}

/// The Werewolf arm (`Player.cs:11893-11910`): night + Moon Charm + not
/// merman grants the stat block, else the buff dies AND the slot rescans
/// (`DelBuff(j); j--`, `:11907-11908`). The `meleeSpeed` add is 0.051f —
/// NOT the damage 0.05f of the fed tiers.
fn apply_werewolf_arm(e: &mut StatusEffects, ctx: &StatusEffectContext) {
    if !ctx.day_time && ctx.wolf_acc && !ctx.merman {
        // `:11895-11904`
        e.life_regen += 1;
        e.were_wolf = true;
        e.melee_crit += 2;
        e.melee_damage += 0.051;
        e.melee_speed += 0.051;
        e.stat_defense += 3;
        e.move_speed += 0.05;
    } else {
        e.delete_buff = true; // `:11907`
        e.rescan_slot = true; // `:11908`
    }
}

/// The hunger tiers (`Player.cs:11979-12008`): Hungry (333) and Starving
/// (334) subtract the fed-tier stat block — no `moveSpeed` line in either
/// arm (the C# omits it), and `pickSpeed` ADDS (mines slower).
fn apply_hunger_arm(e: &mut StatusEffects, starving: bool) {
    let (def, crit, dmg, kb, pick) = if starving {
        (4, 4, 0.1, 1.0, 0.15) // `:11994-12008`
    } else {
        (2, 2, 0.05, 0.5, 0.05) // `:11979-11993`
    };
    if starving {
        e.starving = true; // `:11996`
    } else {
        e.hungry = true; // `:11981`
    }
    e.stat_defense -= def;
    e.melee_crit -= crit;
    e.melee_damage -= dmg;
    e.melee_speed -= dmg;
    e.magic_crit -= crit;
    e.magic_damage -= dmg;
    e.ranged_crit -= crit;
    e.ranged_damage -= dmg;
    e.minion_damage -= dmg;
    e.minion_kb -= kb;
    e.pick_speed += pick;
}

/// One arm of the block (`Player.cs:11406-12012`). Returns `true` when the
/// id is handled (the caller's `else if` chain stops), `false` for ids
/// outside this wave's range — the dispatch hook the merge-step wiring
/// calls per active slot, in slot order. `buff_time` is unused: no arm in
/// the block reads the remaining time (the pets pin 18000, Shimmering
/// assigns 60/6, the stat arms are time-free); the parameter keeps the
/// wave-B/C/D signature shape for the merge.
pub fn apply_status_effect_arm(
    e: &mut StatusEffects,
    ctx: &StatusEffectContext,
    buff_type: i32,
    _buff_time: i32,
) -> bool {
    if let Some(arm) = STATUS_PET_ARMS.iter().find(|a| a.buff == buff_type) {
        apply_plain_pet_arm(e, ctx, arm);
        return true;
    }
    match buff_type {
        60 => apply_crystal_leaf_arm(e, ctx), // `:11658-11678`
        80 => e.blackout = true,              // `:11727-11730`
        320 => e.dripping_sparkle_slime = true, // `:11756-11759`
        353 => apply_shimmering_arm(e, ctx),  // `:11784-11825`
        163 => {
            // Obstructed (`:11826-11830`) — the C# sets BOTH fields.
            e.headcovered = true;
            e.bleed = true;
        }
        164 => e.vortex_debuff = true,     // `:11831-11834`
        194 => e.wind_pushed = true,       // `:11835-11838`
        195 => e.withered_armor = true,    // `:11839-11842`
        196 => e.withered_weapon = true,   // `:11847-11850`
        197 => e.slow_ogre_spit = true,    // `:11851-11854`
        198 => e.parry_damage_buff = true, // `:11855-11858`
        145 => e.moon_leech = true,        // `:11859-11862`
        43 => e.defended_by_paladin = true, // `:11882-11885`
        28 => apply_werewolf_arm(e, ctx),  // `:11893-11910`
        333 => apply_hunger_arm(e, false), // `:11979-11993`
        334 => apply_hunger_arm(e, true),  // `:11994-12008`
        336 => e.hearty_meal = true,       // `:12009-12012`
        _ => return false,
    }
    true
}

/// The slot-ordered driver — the block's slice of the C# `for j` loop
/// (`Player.cs:9865-12069`). Per slot: skip empty/expired (`:9867`), run
/// the arm; on a delete, `DelBuff(j)` compacts the bar and the loop either
/// rescans the slot (Werewolf's `j--`, `:11908`) or advances (Shimmering's
/// `continue`). The timer head's decrement is NOT here — `buffs.rs::
/// update_buffs_tick` owns it (`:9871-9874`), exactly like the live scan.
/// Returns the per-slot accumulators in visit order (the wiring step
/// folds them onto the live fields; the tests read them directly).
pub fn run_status_buff_chain(
    slots: &mut BuffSlots,
    ctx: &StatusEffectContext,
) -> Vec<StatusEffects> {
    let mut out = Vec::new();
    let mut j = 0usize;
    while j < MAX_BUFFS {
        if slots.buff_type[j] <= 0 || slots.buff_time[j] <= 0 {
            j += 1; // `:9867-9870`
            continue;
        }
        let mut e = StatusEffects::default();
        if apply_status_effect_arm(&mut e, ctx, slots.buff_type[j], slots.buff_time[j]) {
            if let Some(t) = e.buff_time_set {
                slots.buff_time[j] = t; // `buffTime[j] = …`
            }
            if e.delete_buff {
                slots.del_buff(j); // `DelBuff(j)`
                if !e.rescan_slot {
                    j += 1; // the `continue` arms do NOT rescan
                }
                // Werewolf's `j--` then the loop's `j++` net to a rescan
                // of the same index — the compacted buff runs this pass.
            } else {
                j += 1;
            }
            out.push(e);
        } else {
            j += 1;
        }
    }
    out
}