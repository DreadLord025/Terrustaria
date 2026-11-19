//! S1-D-3 (aiStyle-42) — the Lost Girl's machine: `NPC.AI()`'s
//! `else if (this.aiStyle == 42)` arm (`NPC.cs:30160-30189`), the ONLY
//! user of the style in the C# (`SetDefaults` case 195, `NPC.cs:11496`;
//! `NPCID.LostGirl = 195`, `NPCID.cs:11067`). The girl stands still until
//! a player comes within 200 px with a clear sight line, or until she is
//! disturbed (any X velocity, rising/fast-falling Y velocity, or any
//! damage taken); then `ai[0]` counts 1 → 21 and `Transform(196)` turns
//! her into the Nymph (`NPCID.Nymph = 196`, `NPCID.cs:11069`, aiStyle 3,
//! `NPC.cs:11510`).
//!
//! **UNWIRED module (code-only mode, 2026-11-18).** The port's `NpcKind`
//! has no LostGirl/Nymph variants yet and `npc/data.rs`'s id/ai_style/
//! stat tables are contested by the in-flight S1-D-3 sibling branches,
//! so this slice lands as pure functions with the kind-table rows +
//! dispatch wiring deferred to the family's merge step (the
//! `drawdata_anim.rs` / `render/special_tiles.rs` pattern). Intended
//! call sites:
//! * [`lost_girl_ai`] — `update_rng`'s `match self.kind.ai_style()` axis
//!   (`npc.rs`), a `42 =>` arm once `NpcKind::LostGirl` carries
//!   `ai_style() == 42` (`NPC.cs:11496`).
//! * [`transform_to`] with [`NYMPH`] — the arm's `Transform(196)`
//!   (`NPC.cs:30186`); the wiring step also swaps `npc.kind` to the new
//!   `NpcKind::Nymph` (the port's in-place kind-swap convention, the
//!   Stardust CellSmall's `Transform(405)`, `npc.rs::update_stardust_cell`).
//!
//! RNG by construction: the whole path is draw-free. The arm
//! (`NPC.cs:30160-30189`) contains no `Main.rand` call; `TargetClosest`
//! (`NPC.cs:77652-77781`) is a deterministic scan; `Transform(196)`
//! (`NPC.cs:81084-81145`) draws nothing on the 195→196 path —
//! `SetDefaults` case 196 (`NPC.cs:11506-11519`) has no rand row,
//! `TransformVisuals` (`NPC.cs:80963-81033`) has NO (195,196) arm (its
//! rand-drawing arms are 478→479, 406→405, 428→427, 427→426 only), and
//! `GiveTownUniqueDataToNPCsThatNeedIt` (`:80839`) is town-only (196 is
//! not a town NPC). The functions therefore take no RNG parameter at
//! all — a draw is impossible by signature; the witness test pins the
//! untouched stream across 600 ticks.
//!
//! Deliberate stand-ins (recorded in docs/ANALYSIS.md "The Lost Girl's
//! aiStyle-42 machine"):
//! * The C# `Main.netMode != 1` gate (`NPC.cs:81086`) folds to the host
//!   path (the port is single-player/host); the `netMode == 2` SendData
//!   pair (`:81131-81135`) is unported wire work.
//! * `TransformVisuals`' `position += netOffset` / `-= netOffset` pair
//!   (`:80965`, `:81032`) is a no-op in single-player (`net_offset` is
//!   zero on the host, `npc.rs`'s field doc) and there is no (195,196)
//!   visual arm — dropped whole.
//! * `altTexture = 0` (`NPC.cs:81143`) has no port field (no altTexture
//!   system yet) — dropped.
//! * `frameCounter = 0` (`SetDefaults`, `NPC.cs:8529`) rides the private
//!   `Npc::frame_counter`; the wiring step resets it inside `npc.rs`
//!   (this module resets the public `frame` row, the
//!   `update_stardust_cell` convention).
//! * The stat swap (damage 10→35, defense 30→16, kb 0.5→0.4, rarity 2,
//!   value 2 gold both) rides the kind tables — [`LOST_GIRL`]/[`NYMPH`]
//!   carry the cited rows for the wiring step; only `lifeMax` (the
//!   Transform life-share input, `NPC.cs:81106`) and the hitbox height
//!   (the reposition, `:81094`/`:81099`) are applied here.
#![allow(dead_code)] // UNWIRED — see the module doc; the wiring is the S1-D-3 merge step.

use crate::npc::Npc;
use crate::player::Player;
use tr_core::World;

/// One `SetDefaults` row as `Transform` reads it (`NPC.cs:81084-81145`):
/// the hitbox (the `withReposition` shift, `:81094`/`:81099`), `lifeMax`
/// (the life share, `:81106`) and the identity/stat constants the
/// wiring step folds into the kind tables.
#[derive(Clone, Copy, Debug)]
pub struct KindDefaults {
    pub width: f32,
    pub height: f32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub knock_back_resist: f32,
    /// `NPC.value` in copper — `Item.buyPrice(0, 2)` = 2 gold = 20000
    /// (`Item.cs:49417-49420`).
    pub value: f32,
    pub rarity: i32,
}

/// `SetDefaults` case 195 (`NPC.cs:11492-11505`): 18x40, aiStyle 42,
/// dmg 10, def 30, 250 HP, kb 0.5, rarity 2, value 2 gold;
/// HitSound NPCHit1 / DeathSound NPCDeath1 (`:11500-11501`).
pub const LOST_GIRL: KindDefaults = KindDefaults {
    width: 18.0,
    height: 40.0,
    ai_style: 42,
    damage: 10,
    defense: 30,
    life_max: 250,
    knock_back_resist: 0.5,
    value: 20000.0,
    rarity: 2,
};

/// `SetDefaults` case 196 (`NPC.cs:11506-11519`): 18x40, aiStyle 3,
/// dmg 35, def 16, 300 HP, kb 0.4, rarity 2, value 2 gold;
/// HitSound NPCHit1 / DeathSound NPCDeath2 (`:11514-11515`).
pub const NYMPH: KindDefaults = KindDefaults {
    width: 18.0,
    height: 40.0,
    ai_style: 3,
    damage: 35,
    defense: 16,
    life_max: 300,
    knock_back_resist: 0.4,
    value: 20000.0,
    rarity: 2,
};

/// `NPC.TargetClosest()` (`NPC.cs:30162`, the arm's head) through the
/// shared selection (`npc_target.rs`, `NPC.cs:77652-77781`) — the single
/// session player as slot 0, the stock aggro/no-aggro/tank values (the
/// S1-D-3p convention). RNG-free (`:77652-77781` has no `Next`).
fn target_closest(npc: &mut Npc, world: &World, player: &Player) {
    let cand = crate::npc_target::session_candidate(player);
    // The `EverLastingTicker` stamp's only reader is the unported
    // target-line fade — 0 until the caller migration (S1-D-3p).
    crate::npc_target::target_closest(npc, world, &[cand], true, 42, 0);
}

/// `NPC.Transform(int newType, ...)` (`NPC.cs:81084-81145`) with the
/// default arguments the arm passes (`Transform(196)`: ai0..3 = 0,
/// `withReposition = true`), on the host path (`Main.netMode != 1`,
/// `:81086`). The kind/stat-table swap is the wiring step (see the
/// module doc); this applies the generic Transform maths the C# runs
/// around `SetDefaults`.
pub fn transform_to(npc: &mut Npc, world: &World, player: &Player, new: &KindDefaults) {
    // `:81088-81091` — the value-zero flag and the life share inputs.
    // `value` rides the kind table (both 195 and 196 carry 20000, so
    // `flag` is false for this pair — the `:81102-81105` reset is a
    // no-op here); `num`/`num2` are the PRE-defaults life pair.
    let num = npc.hp;
    let num2 = npc.max_hp;
    // `:81092-81095` — `withReposition`: the bottom edge is held across
    // the defaults swap (`position.Y += height` BEFORE, `-= height`
    // AFTER, `:81097-81100`). Both rows are 18x40, so the net shift is
    // zero — the maths is kept verbatim for a future unequal pair.
    let (_, old_h) = npc.size();
    npc.y += old_h;
    // `SetDefaults(newType)` (`:81096`) — the fields this module owns:
    // the hitbox (`size_override`, the King Slime convention), lifeMax,
    // the `SetDefaults` head's scale 1 / alpha 0 / netUpdate true
    // (`NPC.cs:8525`/`:8530`/`:8536`), the ai/localAI zero loops
    // (`:8564-8571`), `ClearTarget()`'s 255 + `oldTarget` latch
    // (`:8539-8540`) and the frame reset (the `update_stardust_cell`
    // convention; `frameCounter` is private — the wiring step).
    npc.size_override = Some((new.width, new.height));
    npc.max_hp = new.life_max;
    npc.scale = 1.0;
    npc.alpha = 0;
    npc.net_update = true;
    npc.ai = [0.0; 4];
    npc.local_ai = [0.0; 4];
    npc.target = 255;
    npc.old_target = 255;
    npc.target_rect = (0, 0, 0, 0);
    npc.frame = 0;
    npc.y -= new.height;
    // `:81101` — Transform re-picks the target after the defaults.
    target_closest(npc, world, player);
    // `:81106-81110` — the life share: INTEGER `num * lifeMax / num2`
    // (C# int multiply then truncate-toward-zero divide), floored at 1.
    npc.hp = num * npc.max_hp / num2;
    if npc.hp <= 0 {
        npc.hp = 1;
    }
    // `:81111-81114` — the ai0..3 arguments (all 0 for `Transform(196)`;
    // the SetDefaults zero loop already wrote them — kept explicit).
    npc.ai = [0.0; 4];
    // `:81115-81130` — the newType switch: 196 is NOT in the town list
    // (107/108/124/353/369/550/588/679/680/683/684), no home-tile write.
    // `:81131-81135` — the netMode==2 SendData pair: unported wire work.
    // `:81136` — TransformVisuals(195, 196): no arm matches (see the
    // module doc); the netOffset pair is a single-player no-op.
    // `:81137-81142` — town-only data + homeless/achievement: 196 is
    // not a town NPC. `:81143` altTexture: no port field.
}

/// One fixed step of the aiStyle-42 arm (`NPC.cs:30160-30189`). Returns
/// `true` on the tick `Transform(196)` fires (`:30186`) — the transform
/// maths is applied via [`transform_to`] with [`NYMPH`]; the wiring step
/// additionally swaps `npc.kind`. No RNG parameter: the arm is draw-free
/// by construction (see the module doc).
pub fn lost_girl_ai(npc: &mut Npc, world: &World, player: &Player) -> bool {
    // `:30162` — TargetClosest runs EVERY tick, before the state split.
    target_closest(npc, world, player);
    if npc.ai[0] == 0.0 {
        // `:30165-30174` — the calm-state trigger: a picked target
        // within 200 px of the NPC's centre with a clear sight line.
        if npc.target >= 0 {
            let (w, h) = npc.size();
            // `:30167` — vector74 = position + size * 0.5f.
            let cx = npc.x + w * 0.5;
            let cy = npc.y + h * 0.5;
            // `:30168-30169` — num616 aims at the player's centre X,
            // num617 at the player's TOP edge (`position.Y`, NOT the
            // centre — the C# asymmetry is kept verbatim).
            let dx = player.x + crate::player::WIDTH * 0.5 - cx;
            let dy = player.y - cy;
            // `:30170` — `(float)Math.Sqrt(dx*dx + dy*dy) < 200f` AND
            // `Collision.CanHit(position, width, height, player.position,
            // player.width, player.height)` (`Collision.cs:381-479`,
            // the port's Manhattan walk in `projectile::can_hit`).
            if (dx * dx + dy * dy).sqrt() < 200.0
                && crate::projectile::can_hit(
                    world,
                    npc.x,
                    npc.y,
                    w,
                    h,
                    player.x,
                    player.y,
                    crate::player::WIDTH,
                    crate::player::HEIGHT,
                )
            {
                npc.ai[0] = 1.0;
            }
        }
        // `:30175-30178` — the disturbance trigger: ANY X velocity, a
        // rising or faster-than-2 falling Y velocity, or damage taken.
        // (Runs even when the sight trigger above already fired — the
        // C# has no else; the write is idempotent.)
        if npc.vx != 0.0 || npc.vy < 0.0 || npc.vy > 2.0 || npc.hp != npc.max_hp {
            npc.ai[0] = 1.0;
        }
    } else {
        // `:30182-30187` — the count-up: 1 → 21, clamped at 21, then
        // `Transform(196)` (which zeroes ai[0] again, `:81111`).
        npc.ai[0] += 1.0;
        if npc.ai[0] >= 21.0 {
            npc.ai[0] = 21.0;
            transform_to(npc, world, player, &NYMPH);
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::{UnifiedRandom, TILE_PX};

    // The kind-table rows are the wiring step; the tests ride a stand-in
    // kind with the Lost Girl's 18x40 hitbox (the Zombie row) and pin
    // the machine's own fields. `size_override` carries the cited box.
    fn girl(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((LOST_GIRL.width, LOST_GIRL.height));
        n.max_hp = LOST_GIRL.life_max;
        n.hp = n.max_hp;
        n
    }

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    fn player_at(x: f32, y: f32) -> Player {
        let w = air();
        let mut p = Player::spawn(&w);
        p.x = x;
        p.y = y;
        p
    }

    /// `NPC.cs:30163-30178` — the calm state holds: no player in range,
    /// zero X velocity, Y velocity inside [0, 2], full life.
    #[test]
    fn s1_d_3_s42_the_calm_state_holds_while_undisturbed() {
        let w = air();
        let p = player_at(100.0 * TILE_PX + 500.0, 40.0 * TILE_PX);
        let mut n = girl(100.0 * TILE_PX, 40.0 * TILE_PX);
        n.vy = 2.0; // the `> 2f` bound is strict (`:30175`)
        for _ in 0..10 {
            assert!(!lost_girl_ai(&mut n, &w, &p));
            assert_eq!(n.ai[0], 0.0);
        }
    }

    /// `NPC.cs:30167-30173` — a player within 200 px with a clear sight
    /// line flips ai[0] to 1. num617 aims at the player's TOP edge: with
    /// equal y the offset is -20 (the NPC's half-height), not 0.
    #[test]
    fn s1_d_3_s42_the_close_visible_player_triggers_the_count() {
        let w = air();
        // dx = 151, dy = -20 → sqrt(151² + 20²) ≈ 152.3 < 200.
        let p = player_at(100.0 * TILE_PX + 150.0, 40.0 * TILE_PX);
        let mut n = girl(100.0 * TILE_PX, 40.0 * TILE_PX);
        assert!(!lost_girl_ai(&mut n, &w, &p));
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.target, 0, "TargetClosest picked the session player");
    }

    /// `NPC.cs:30170` — the range test is STRICT `< 200f`: at exactly
    /// 200 px (dy = 0 via the player's top edge) the girl stays calm;
    /// one pixel closer triggers.
    #[test]
    fn s1_d_3_s42_the_range_bound_is_strict() {
        let w = air();
        let gx = 100.0 * TILE_PX;
        let gy = 40.0 * TILE_PX;
        // dy = player.y - (gy + 20) = 0 → player.y = gy + 20.
        // dx = player.x + 10 - (gx + 9) = 200 → player.x = gx + 199.
        let p = player_at(gx + 199.0, gy + 20.0);
        let mut n = girl(gx, gy);
        lost_girl_ai(&mut n, &w, &p);
        assert_eq!(n.ai[0], 0.0, "exactly 200 is not < 200");
        let p2 = player_at(gx + 198.0, gy + 20.0);
        let mut n2 = girl(gx, gy);
        lost_girl_ai(&mut n2, &w, &p2);
        assert_eq!(n2.ai[0], 1.0, "199 px triggers");
    }

    /// `NPC.cs:30170` — `Collision.CanHit` gates the trigger: a solid
    /// wall column between the two boxes blocks the sight line even
    /// inside 200 px (the Manhattan walk, `Collision.cs:381-479`).
    #[test]
    fn s1_d_3_s42_the_wall_blocks_the_sight_trigger() {
        let mut w = air();
        for y in 30..50 {
            let i = w.index(105, y);
            w.tiles[i] = tr_core::tile::Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let p = player_at(100.0 * TILE_PX + 150.0, 40.0 * TILE_PX);
        let mut n = girl(100.0 * TILE_PX, 40.0 * TILE_PX);
        lost_girl_ai(&mut n, &w, &p);
        assert_eq!(n.ai[0], 0.0, "the wall blocks CanHit");
        // The disturbance trigger still fires through the wall.
        n.hp -= 1;
        lost_girl_ai(&mut n, &w, &p);
        assert_eq!(n.ai[0], 1.0, "damage bypasses the sight gate (:30175)");
    }

    /// `NPC.cs:30175-30178` — each disturbance term alone flips the
    /// state: vx != 0, vy < 0, vy > 2, life != lifeMax. The vy == 2
    /// boundary stays calm.
    #[test]
    fn s1_d_3_s42_every_disturbance_term_triggers() {
        let w = air();
        let p = player_at(100.0 * TILE_PX + 500.0, 40.0 * TILE_PX);
        for (vx, vy, hurt) in [(0.5, 0.0, false), (-0.5, 0.0, false), (0.0, -0.1, false), (0.0, 2.1, false), (0.0, 0.0, true)] {
            let mut n = girl(100.0 * TILE_PX, 40.0 * TILE_PX);
            n.vx = vx;
            n.vy = vy;
            if hurt {
                n.hp -= 1;
            }
            lost_girl_ai(&mut n, &w, &p);
            assert_eq!(n.ai[0], 1.0, "vx {vx} vy {vy} hurt {hurt}");
        }
    }

    /// `NPC.cs:30182-30187` — the count climbs 1 per tick and the
    /// transform fires on the tick ai[0] reaches 21 (20 ticks after the
    /// trigger). Transform zeroes ai[0] (`:81111`).
    #[test]
    fn s1_d_3_s42_the_count_reaches_21_and_transforms() {
        let w = air();
        let p = player_at(100.0 * TILE_PX + 500.0, 40.0 * TILE_PX);
        let mut n = girl(100.0 * TILE_PX, 40.0 * TILE_PX);
        n.ai[0] = 1.0;
        for tick in 2..=20 {
            assert!(!lost_girl_ai(&mut n, &w, &p));
            assert_eq!(n.ai[0], tick as f32);
        }
        assert!(lost_girl_ai(&mut n, &w, &p), "the 21st tick transforms");
        assert_eq!(n.ai[0], 0.0, "Transform(196) zeroes ai (:81111)");
        assert_eq!(n.max_hp, NYMPH.life_max, "the Nymph's 300 HP row");
    }

    /// `NPC.cs:81092-81110` — the Transform maths: the bottom-edge
    /// reposition nets zero for the equal 18x40 boxes, the life share is
    /// the INTEGER `num * 300 / 250` (125 → 150), floored at 1 (1 → 1),
    /// and TargetClosest re-runs after the defaults (`:81101`).
    #[test]
    fn s1_d_3_s42_the_transform_shares_life_and_holds_the_bottom() {
        let w = air();
        let p = player_at(100.0 * TILE_PX + 500.0, 40.0 * TILE_PX);
        let mut n = girl(100.0 * TILE_PX, 40.0 * TILE_PX);
        let y0 = n.y;
        n.hp = 125;
        n.ai = [21.0, 3.0, 4.0, 5.0];
        transform_to(&mut n, &w, &p, &NYMPH);
        assert_eq!(n.y, y0, "18x40 → 18x40: the reposition nets zero");
        assert_eq!(n.hp, 150, "125 * 300 / 250 = 150 (int maths)");
        assert_eq!(n.ai, [0.0; 4]);
        assert_eq!(n.local_ai, [0.0; 4]);
        assert_eq!(n.target, 0, "the post-defaults TargetClosest (:81101)");
        assert!(n.net_update, "SetDefaults arms netUpdate (:8530)");
        // The floor: 1 * 300 / 250 = 1 (already >= 1); a 0-life input
        // would floor to 1 (`:81107-81110`).
        let mut n2 = girl(100.0 * TILE_PX, 40.0 * TILE_PX);
        n2.hp = 1;
        transform_to(&mut n2, &w, &p, &NYMPH);
        assert_eq!(n2.hp, 1);
    }

    /// `NPC.cs:30162` — TargetClosest runs every tick, including the
    /// counting ticks: the facing tracks the player while ai[0] climbs.
    #[test]
    fn s1_d_3_s42_the_target_tracks_while_counting() {
        let w = air();
        let mut n = girl(100.0 * TILE_PX, 40.0 * TILE_PX);
        n.ai[0] = 5.0;
        let left = player_at(100.0 * TILE_PX - 300.0, 40.0 * TILE_PX);
        lost_girl_ai(&mut n, &w, &left);
        assert_eq!(n.facing, -1, "the player is left");
        assert_eq!(n.ai[0], 6.0);
        let right = player_at(100.0 * TILE_PX + 300.0, 40.0 * TILE_PX);
        lost_girl_ai(&mut n, &w, &right);
        assert_eq!(n.facing, 1, "the facing follows on the next tick");
        assert_eq!(n.ai[0], 7.0);
    }

    /// RNG witness (written, NOT run — code-only mode): the arm
    /// (`NPC.cs:30160-30189`), `TargetClosest` (`:77652-77781`) and the
    /// 195→196 `Transform` path (`:81084-81145`, no TransformVisuals arm,
    /// no rand in `SetDefaults` case 196 `:11506-11519`) contain no
    /// `Main.rand.Next`. The functions take no RNG parameter, so the
    /// `UpdateNPCs` stream cannot advance: two identically seeded
    /// streams, one carried alongside 600 machine ticks, must yield the
    /// same draw sequence.
    #[test]
    fn s1_d_3_s42_the_machine_never_draws_from_the_stream() {
        let w = air();
        let p = player_at(100.0 * TILE_PX + 150.0, 40.0 * TILE_PX);
        let mut carried = UnifiedRandom::new(123);
        let mut control = UnifiedRandom::new(123);
        let mut n = girl(100.0 * TILE_PX, 40.0 * TILE_PX);
        for tick in 0..600 {
            lost_girl_ai(&mut n, &w, &p);
            if tick % 100 == 99 {
                // Re-arm the calm state so both branches keep running.
                n.ai[0] = 0.0;
                n.hp = n.max_hp;
            }
        }
        for _ in 0..8 {
            assert_eq!(carried.next_max(1000), control.next_max(1000));
        }
    }

    /// The cited `SetDefaults` rows (`NPC.cs:11492-11519`) — the wiring
    /// step's kind-table pin: the pair shares the 18x40 box and the
    /// 2-gold value; the Nymph swaps aiStyle 42 → 3, 250 → 300 HP,
    /// 10 → 35 dmg, 30 → 16 def, 0.5 → 0.4 kb.
    #[test]
    fn s1_d_3_s42_the_defaults_rows_match_the_csharp_table() {
        assert_eq!((LOST_GIRL.width, LOST_GIRL.height), (18.0, 40.0));
        assert_eq!((NYMPH.width, NYMPH.height), (18.0, 40.0));
        assert_eq!(LOST_GIRL.ai_style, 42);
        assert_eq!(NYMPH.ai_style, 3);
        assert_eq!(LOST_GIRL.life_max, 250);
        assert_eq!(NYMPH.life_max, 300);
        assert_eq!(LOST_GIRL.damage, 10);
        assert_eq!(NYMPH.damage, 35);
        assert_eq!(LOST_GIRL.defense, 30);
        assert_eq!(NYMPH.defense, 16);
        assert_eq!(LOST_GIRL.knock_back_resist, 0.5);
        assert_eq!(NYMPH.knock_back_resist, 0.4);
        assert_eq!(LOST_GIRL.value, 20000.0, "Item.buyPrice(0, 2)");
        assert_eq!(NYMPH.value, 20000.0);
        assert_eq!(LOST_GIRL.rarity, 2);
        assert_eq!(NYMPH.rarity, 2);
    }
}
