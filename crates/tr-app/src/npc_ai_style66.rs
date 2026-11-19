//! The aiStyle-66 family — the crawling critters (`NPC.AI()`'s
//! `else if (this.aiStyle == 66)` arm, `NPC.cs:34253-34347`). S1-D-3,
//! aiStyle-66 round, CODE-ONLY MODE (no compiler invoked; the witness
//! tests below are written, not run).
//!
//! NOT the projectile aiStyle 66 — that is the Twins-minion arm
//! (`Projectile.cs:29604-30161`, landed S1-D-4b-6,
//! `projectile.rs::ai_twins_minion`). This module owns the NPC style.
//!
//! ## Census (every `this.aiStyle = 66;` row in `NPC.cs`)
//!
//! Eight kinds, all `SetDefaults`-verified:
//! * **Worm 357** (`NPCID.Worm`, `ID/NPCID.cs:11391`; row
//!   `NPC.cs:13545-13558`) — 10×4, slots 0.1, catchItem 2002, friendly.
//! * **TruffleWorm 374** (`ID/NPCID.cs:11425`; row `:13762-13776`) —
//!   20×10, slots 0.15, catchItem 2673, rarity 4,
//!   `dontTakeDamageFromHostiles = true`, NOT friendly. The only kind
//!   with the dig tail.
//! * **GoldWorm 448** (`ID/NPCID.cs:11569`; row `:14332-14346`) — 10×4,
//!   slots 0.1, catchItem 2895, friendly, rarity 3. The only
//!   `NPCID.Sets.IsGoldCritter` member of the family
//!   (`ID/NPCID.cs:10126`).
//! * **EnchantedNightcrawler 484 / Grubby 485 / Sluggy 486 / Buggy 487**
//!   (`ID/NPCID.cs:11641-11647`; the RANGE row
//!   `this.type >= 484 && this.type <= 487`, `:14576-14589`) — 10×4,
//!   slots 0.1, `catchItem = 3191 + type - 484`, friendly. (GraniteFlyer
//!   483 is aiStyle **91**, `:14567` — outside the family.)
//! * **Maggot 606** (`ID/NPCID.cs:11889`; row `:16781-16794`) — 10×4,
//!   slots 0.1, catchItem 4363, friendly.
//!
//! The transform target **TruffleWormDigger 375** (`ID/NPCID.cs:11427`,
//! row `:13777-13795`) is aiStyle **6** (the worm pass) — it is NOT a
//! style-66 kind; only the dig request is this arm's.
//!
//! ## The machine
//!
//! A ground crawler, not a hopper: the arm never writes `velocity.Y` —
//! gravity/integration are `UpdateNPC`'s post-AI step (`NPC.cs:90465+`).
//! In C# order:
//! 1. `:34255-34259` — type 484 ONLY: one `Main.rand.Next(90, 111)` per
//!    tick (ungated), the essScale-flicker light
//!    `num983 = Next(90,111) * 0.01f * ((Main.essScale + 0.5f) / 2f)`,
//!    `AddLight` at `(int)((X + width/2) / 16f)` (the quotient-truncated
//!    `XAnchor::Center` form; `width/2` is INTEGER division — 10/2 = 5,
//!    4/2 = 2, exact for this family), rgb `(0.3, 0.1, 0.25) * num983`.
//! 2. `:34260-34313` — grounded (`velocity.Y == 0f`):
//!    * `ai[0] == 1` (walk): `direction == 0` → `TargetClosest()`
//!      (`:34266`, faceTarget true — the shared `npc_target.rs` port,
//!      RNG-free); `collideX` → `direction *= -1` (`:34270`); the speed
//!      ladder `num984` = 0.2 default, 485 → 0.25, 486 → 0.325,
//!      487 → 0.4 (`:34272-34284`); `velocity.X = num984 * direction`,
//!      then 374 ×3 (`:34286-34289` → 0.6).
//!    * else `velocity.X = 0` (`:34293`).
//!    * `Main.netMode != 1` (`:34295`, folds to true — the port is
//!      single-player, the style50/55 precedent): `localAI[1]--`; at
//!      `<= 0` the toggle: walk→idle draws `Next(300, 900)` (`:34303`),
//!      idle→walk draws `Next(600, 1800)` (`:34308`); `netUpdate = true`
//!      (`:34310`). The velocity write uses the OLD `ai[0]` — the toggle
//!      tick itself keeps the old gait's velocity.
//! 3. `:34314-34321` — airborne with `direction == 0`: latch ±1 from the
//!    sign of `velocity.X` (negative → -1, else +1).
//! 4. `:34322-34325` — `type != 374` RETURNS. Everything below is the
//!    TruffleWorm's dig tail only.
//! 5. `:34326` — `spriteDirection = direction`.
//! 6. `:34327-34336` — `flag69`: ANY `Main.player[i]` (`i < 255`) with
//!    `active && !dead` and `Vector2.Distance(Center, Center) <= 160f`
//!    (EUCLIDEAN — not the Manhattan target metric; NO ghost check,
//!    unlike `TargetClosest`).
//! 7. `:34337-34341` — `num986 = 90`; `flag69 && ai[1] < 90` → `ai[1]++`
//!    (the counter never decays — a worm that was approached keeps its
//!    progress).
//! 8. `:34342-34346` — `ai[1] == 90 && Main.netMode != 1`:
//!    `position.Y += 16f`, `Transform(375)`.
//!
//! ## The Transform(375) seam (`NPC.cs:81084-81145`)
//!
//! Returned as [`Style66Outcome::transform_to_digger`] — the caller owes
//! the whole `Transform` (the roster has neither kind yet). Its
//! semantics for the 374→375 pair, all RNG-free:
//! * `value == 0` flag (`:81088`): 374's `value` is the head default 0
//!   (`:8544`, the row sets none) → stays 0 (`:81102-81105`).
//! * Reposition (`:81092-81100`): `Y += height` (OLD 374 height 10),
//!   `SetDefaults(375)` (10×10), `Y -= height` (NEW 10) — net 0; with
//!   the arm's `+16` the worm sinks one tile, bottom-aligned.
//! * `SetDefaults(375)` (`:13777-13795`): aiStyle 6, `noGravity` /
//!   `noTileCollide` / `behindTiles` true, kbResist 0, slots 0.15,
//!   catchItem 2673, rarity 4, `dontTakeDamageFromHostiles` true. The
//!   whole `SetDefaults` body (`:8426-17859`) contains NO `rand.` draw
//!   (scanned), so the re-default is stream-neutral.
//! * `TargetClosest()` (`:81101`) — RNG-free (`npc_target.rs`).
//! * Life rescale (`:81106-81110`): `life = old_life * new_max /
//!   old_max` (INTEGER arithmetic), floored at 1 —
//!   [`transform_life`]. Both kinds are 5-HP so an undamaged worm keeps
//!   5; a 3-HP worm keeps 3.
//! * `ai[0..3] = 0` (`:81111-81114`) — the 90-counter is cleared.
//! * The `switch (newType)` home-tile arm (`:81115-81130`) has no case
//!   375; `TransformVisuals(374, 375)` (`:80963-81033`) matches NONE of
//!   its branches (158/159, 478/479, 406/405, 428/427, 427/426) — only
//!   the `netOffset` add/subtract, no draws;
//!   `GiveTownUniqueDataToNPCsThatNeedIt` (`:80839-80856`) is inert
//!   (`isLikeATownNPC` false for 375); `townNPC` false → the homeless
//!   block skipped; `altTexture = 0` (`:81143`).
//!
//! ## RNG order (by construction)
//!
//! The arm's ONLY `Main.rand` draws, in per-tick C# order:
//! 1. type 484: `Next(90, 111)` (`:34257`) — EVERY tick, before any
//!    branch, airborne or grounded.
//! 2. grounded timer expiry: exactly ONE of `Next(300, 900)` (walk→idle)
//!    or `Next(600, 1800)` (idle→walk) (`:34303`/`:34308`). No expiry →
//!    no draw. Airborne → no draw (the whole timer block is inside the
//!    `velocity.Y == 0f` branch).
//! `TargetClosest` and the dig tail draw NOTHING; `Transform(375)`
//! draws nothing (above). All draws ride the caller's `UnifiedRandom`
//! (the `Main.rand` analogue); the witnesses replay the cited draws on
//! a cloned stream and compare the full state.
//!
//! The per-tick draw OUTSIDE the arm: `UpdateNPC_CastLights`
//! (`NPC.cs:92343-92414`, called BEFORE `AI()` in `UpdateNPC`) arms the
//! `IsGoldCritter` set (`:92345-92368`) — of this family only **448** is
//! a member (`ID/NPCID.cs:10126`). [`gold_worm_sparkle`] ports that
//! gate: the `Next(300)` (`:92361`) is drawn whenever the tile light's
//! max channel > 20 (`:92349`), and succeeds when `< max/30` (integer
//! divide, `:92360`). On success the C# spawns ONE dust 43
//! (`:92363-92364`, alpha 254, yellow, scale 0.5, velocity zeroed) —
//! the dust spawn is the wiring caller's (the `Dust.NewDust` internals
//! draw; the caller must run them in C# order when the dust feed lands,
//! the `npc_ai_style50.rs` light-seam precedent). None of the other
//! CastLights arms (160, 254..=261/634/635, 209, 44) matches a family
//! kind.
//!
//! ## Wiring deferred (the BlazingWheel / style72 precedent)
//!
//! The roster has NO `NpcKind` for any of the eight kinds (only the
//! lang/sacrifice tables name them), so `update_rng`'s aiStyle axis
//! (`npc.rs:3651`) has no style-66 arm to claim. This module is the pure
//! port + the minimal seams:
//! * `Main.essScale` is the caller's `App::ess_scale`
//!   (`item_lights.rs::update_ess_scale`, S1-E-5) — passed in.
//! * `Main.netMode != 1` folds to true (single-player port, the
//!   style50/55 precedent) — on a client the C# draws no timer RNG and
//!   never digs.
//! * The 484 light is RETURNED in [`Style66Outcome::light`] for the
//!   caller's `EntityLights` feed.
//! * `FindFrame` (`NPC.cs:66684-77370`) is a separate system (the S1-D-3
//!   FindFrame waves) — its family cases are cited here as the wiring
//!   obligation, NOT ported: case 374 (`:71121-71155`, the 6-tick ground
//!   cycle wrapping to frame 1, the airborne 3-tick spin), case 375
//!   (`:71109-71120`, the digger's 4-tick cycle), the 357/448/484/606
//!   group (`:72774-72828`, the 12-tick two-frame crawl, the airborne
//!   `direction * 0.1f` tilt, the slope-1/2 ±0.785f rotation + the
//!   `localAI[0]` -2/0 marker) and the 485/486/487 group
//!   (`:72829-72883`, `spriteDirection = direction`, the
//!   `|vx| * 5`-weighted 10-tick four-frame cycle, the same slope tail).
//!   `Main.npcFrameCount` (`Main.cs:67082-67153`, counted entry-by-entry):
//!   357→2, 374→4, 375→4, 448→2, 484→2, 485→4, 486→4, 487→4, 606→2 —
//!   pinned as consts below.
//! * `HitEffect`/death are NOT in this arm (it never kills) — the
//!   caller's strike path owns them.
//!
//! See docs/ANALYSIS.md "The crawling critters (S1-D-3 aiStyle-66,
//! code-only)".
#![allow(dead_code)] // wiring deferred — the kinds + dispatch arm land with the roster wave

use glam::Vec2;

use crate::entity_lights::{tile_x, tile_y, XAnchor};
use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::unified_random::UnifiedRandom;
use tr_core::World;

/// One family `SetDefaults` row, verbatim, with the head defaults the
/// branch leaves untouched (`:8510` npcSlots 1 → overridden by every
/// row, `:8519` friendly false, `:8533` knockBackResist 1, `:8535`
/// noGravity false, `:8443` rarity 0, `:8501`
/// dontTakeDamageFromHostiles false, `:8544` value 0).
pub struct CrawlerDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — every row `SoundID.NPCHit1` = (3, 1)
    /// (`ID/SoundID.cs:1257`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — every row `SoundID.NPCDeath1` = (4, 1)
    /// (`ID/SoundID.cs:1315`).
    pub death_sound: (i32, i32),
    pub npc_slots: f32,
    pub catch_item: i32,
    pub friendly: bool,
    pub rarity: i32,
    pub knock_back_resist: f32,
    pub no_gravity: bool,
    pub dont_take_damage_from_hostiles: bool,
}

/// `NPC.cs:13545-13558` — the Worm 357.
pub const WORM: CrawlerDefaults = CrawlerDefaults {
    width: 10,
    height: 4,
    ai_style: 66,
    damage: 0,
    defense: 0,
    life_max: 5,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    npc_slots: 0.1,
    catch_item: 2002,
    friendly: true,
    rarity: 0, // the head default (:8443) — the row sets none
    knock_back_resist: 1.0,
    no_gravity: false,
    dont_take_damage_from_hostiles: false,
};

/// `NPC.cs:13762-13776` — the TruffleWorm 374. NOT friendly (the head
/// default `:8519`); the only `dontTakeDamageFromHostiles` row.
pub const TRUFFLE_WORM: CrawlerDefaults = CrawlerDefaults {
    width: 20,
    height: 10,
    ai_style: 66,
    damage: 0,
    defense: 0,
    life_max: 5,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    npc_slots: 0.15,
    catch_item: 2673,
    friendly: false,
    rarity: 4,
    knock_back_resist: 1.0,
    no_gravity: false,
    dont_take_damage_from_hostiles: true,
};

/// `NPC.cs:14332-14346` — the GoldWorm 448.
pub const GOLD_WORM: CrawlerDefaults = CrawlerDefaults {
    width: 10,
    height: 4,
    ai_style: 66,
    damage: 0,
    defense: 0,
    life_max: 5,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    npc_slots: 0.1,
    catch_item: 2895,
    friendly: true,
    rarity: 3,
    knock_back_resist: 1.0,
    no_gravity: false,
    dont_take_damage_from_hostiles: false,
};

/// `NPC.cs:14576-14589` — the 484..=487 range row. `catch_item` is
/// `3191 + type - 484`; the const carries the 484 base
/// (EnchantedNightcrawler) — [`grub_row_catch_item`] derives the rest.
pub const ENCHANTED_NIGHTCRAWLER: CrawlerDefaults = CrawlerDefaults {
    width: 10,
    height: 4,
    ai_style: 66,
    damage: 0,
    defense: 0,
    life_max: 5,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    npc_slots: 0.1,
    catch_item: 3191,
    friendly: true,
    rarity: 0,
    knock_back_resist: 1.0,
    no_gravity: false,
    dont_take_damage_from_hostiles: false,
};

/// `NPC.cs:16781-16794` — the Maggot 606.
pub const MAGGOT: CrawlerDefaults = CrawlerDefaults {
    width: 10,
    height: 4,
    ai_style: 66,
    damage: 0,
    defense: 0,
    life_max: 5,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    npc_slots: 0.1,
    catch_item: 4363,
    friendly: true,
    rarity: 0,
    knock_back_resist: 1.0,
    no_gravity: false,
    dont_take_damage_from_hostiles: false,
};

/// `NPC.cs:14587` — `catchItem = (short)(3191 + this.type - 484)`:
/// Grubby 485 → 3192, Sluggy 486 → 3193, Buggy 487 → 3194.
pub fn grub_row_catch_item(npc_type: i32) -> i32 {
    3191 + npc_type - 484
}

/// `Main.npcFrameCount` rows (`Main.cs:67082-67153`, counted
/// entry-by-entry) — the FindFrame wiring obligation's frame budgets.
pub const FRAME_COUNTS: [(i32, u32); 9] = [
    (357, 2),
    (374, 4),
    (375, 4), // the digger (aiStyle 6), cited for the Transform seam
    (448, 2),
    (484, 2),
    (485, 4),
    (486, 4),
    (487, 4),
    (606, 2),
];

/// The arm's per-tick side effects beyond the NPC state — the wiring
/// seams (module docs).
pub struct Style66Outcome {
    /// The 484-only essScale flicker (`:34257-34258`):
    /// `(tileX, tileY, [r, g, b])` for the caller's `EntityLights`.
    /// `None` for every other kind.
    pub light: Option<(i32, i32, [f32; 3])>,
    /// The TruffleWorm dig (`:34342-34346`): `npc.y` is already +16;
    /// the caller owes the whole `Transform(375)` (module docs — the
    /// reposition/life/ai-zeroing semantics, all RNG-free).
    pub transform_to_digger: bool,
}

/// One fixed step of the aiStyle-66 arm (`NPC.cs:34253-34347`), in C#
/// order. `npc_type` is the vanilla id until the roster kinds land;
/// `ess_scale` is `Main.essScale` (`App::ess_scale`); `players`/`ticker`
/// feed the `:34266` `TargetClosest()` through the shared
/// `npc_target.rs` port and the `:34328-34336` proximity scan (the C#
/// loops the 255 `Main.player` slots; the port's absent slots are
/// simply absent from the slice). `rng` is the caller's `Main.rand`
/// stream — the ONLY draws are the 484 flicker and the timer expiry
/// (module docs).
pub fn ai_style66(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    npc_type: i32,
    ess_scale: f32,
    ticker: u64,
    rng: &mut UnifiedRandom,
) -> Style66Outcome {
    // `:34255-34259` — the EnchantedNightcrawler's flicker, FIRST and
    // ungated (airborne ticks draw too).
    let light = if npc_type == 484 {
        // `:34257` — num983 = (float)Next(90, 111) * 0.01f
        //                    * ((Main.essScale + 0.5f) / 2f).
        let num983 = rng.next_range(90, 111) as f32 * 0.01 * ((ess_scale + 0.5) / 2.0);
        let (w, h) = npc.size();
        let pos = Vec2::new(npc.x, npc.y);
        // `:34258` — (int)((X + (float)(width / 2)) / 16f): the
        // quotient-truncated centre. width/2 is INTEGER division in
        // the C#; 10/2 = 5 and 4/2 = 2 are exact, so XAnchor::Center's
        // float half matches for this family.
        Some((
            tile_x(pos, w, XAnchor::Center),
            tile_y(pos, h, XAnchor::Center),
            [0.3 * num983, 0.1 * num983, 0.25 * num983],
        ))
    } else {
        None
    };
    // `:34260` — the grounded branch.
    if npc.vy == 0.0 {
        // `:34262-34290` — the walk gait.
        if npc.ai[0] == 1.0 {
            // `:34264-34267` — face the player only while directionless.
            if npc.facing == 0 {
                target_closest(npc, world, players, true, 66, ticker);
            }
            // `:34268-34271` — the wall bounce.
            if npc.collide_x {
                npc.facing *= -1;
            }
            // `:34272-34284` — the speed ladder.
            let mut num984 = 0.2f32;
            if npc_type == 485 {
                num984 = 0.25;
            }
            if npc_type == 486 {
                num984 = 0.325;
            }
            if npc_type == 487 {
                num984 = 0.4;
            }
            // `:34285` — velocity.X = num984 * direction.
            npc.vx = num984 * npc.facing as f32;
            // `:34286-34289` — the TruffleWorm's ×3 sprint.
            if npc_type == 374 {
                npc.vx *= 3.0;
            }
        } else {
            // `:34293` — idle: velocity.X = 0.
            npc.vx = 0.0;
        }
        // `:34295-34312` — the gait timer (netMode != 1 folds to true,
        // module docs). The velocity above used the OLD ai[0].
        npc.local_ai[1] -= 1.0;
        if npc.local_ai[1] <= 0.0 {
            if npc.ai[0] == 1.0 {
                // `:34302-34303` — walk → idle, Next(300, 900).
                npc.ai[0] = 0.0;
                npc.local_ai[1] = rng.next_range(300, 900) as f32;
            } else {
                // `:34307-34308` — idle → walk, Next(600, 1800).
                npc.ai[0] = 1.0;
                npc.local_ai[1] = rng.next_range(600, 1800) as f32;
            }
            // `:34310`.
            npc.net_update = true;
        }
    } else if npc.facing == 0 {
        // `:34314-34321` — the airborne direction latch.
        npc.facing = 1;
        if npc.vx < 0.0 {
            npc.facing = -1;
        }
    }
    // `:34322-34325` — only the TruffleWorm has the dig tail.
    if npc_type != 374 {
        return Style66Outcome {
            light,
            transform_to_digger: false,
        };
    }
    // `:34326` — spriteDirection = direction.
    npc.sprite_dir = npc.facing;
    // `:34327-34336` — flag69: any live player within EUCLIDEAN 160 of
    // the centre. No ghost check (unlike TargetClosest); the C# gate is
    // `!(Distance > 160f)` — exactly 160 counts.
    let (cx, cy) = npc.center();
    let our_center = Vec2::new(cx, cy);
    let mut flag69 = false;
    for p in players {
        if p.active && !p.dead {
            let pc = Vec2::new(
                p.x + p.width as f32 / 2.0,
                p.y + p.height as f32 / 2.0,
            );
            if !(pc.distance(our_center) > 160.0) {
                flag69 = true;
                break;
            }
        }
    }
    // `:34337-34341` — the 90-tick approach counter (never decays).
    let num986 = 90.0f32;
    if flag69 && npc.ai[1] < num986 {
        npc.ai[1] += 1.0;
    }
    // `:34342-34346` — the dig: Y += 16, Transform(375) (the caller's
    // seam; netMode != 1 folds to true).
    let mut transform_to_digger = false;
    if npc.ai[1] == num986 {
        npc.y += 16.0;
        transform_to_digger = true;
    }
    Style66Outcome {
        light,
        transform_to_digger,
    }
}

/// `NPC.Transform`'s life rescale (`NPC.cs:81106-81110`):
/// `life = old_life * new_life_max / old_life_max` — INTEGER multiply
/// then divide — floored at 1. For the 374→375 pair both maxima are 5,
/// so the life carries over unchanged.
pub fn transform_life(old_life: i32, old_life_max: i32, new_life_max: i32) -> i32 {
    let life = old_life * new_life_max / old_life_max;
    if life <= 0 {
        1
    } else {
        life
    }
}

/// The GoldWorm's per-tick sparkle gate — `UpdateNPC_CastLights`'s
/// `IsGoldCritter` arm (`NPC.cs:92345-92368`), called BEFORE `AI()` in
/// `UpdateNPC`. Of the family only 448 is in the set
/// (`ID/NPCID.cs:10126`); the caller gates on the kind.
///
/// `tile_light` is `Lighting.GetColor((int)Center.X / 16,
/// (int)Center.Y / 16)` (`:92348`, the truncate-before-divide form) as
/// RGB bytes. Returns `true` when the C# would spawn the ONE dust 43
/// (`:92363-92364`: alpha 254, `new Color(255, 255, 0)`, scale 0.5,
/// velocity zeroed) — the spawn itself (and its `Dust.NewDust` internal
/// draws) is the wiring caller's, in C# order (module docs).
///
/// RNG: the `Next(300)` (`:92361`) is drawn ONLY when some channel
/// exceeds 20 (`:92349` — `R > 20 || B > 20 || G > 20`); a dark tile
/// draws NOTHING. The threshold is `max_channel / 30` (integer divide,
/// `:92351-92360`).
pub fn gold_worm_sparkle(tile_light: [u8; 3], rng: &mut UnifiedRandom) -> bool {
    let [r, g, b] = tile_light;
    // `:92349` — the any-channel gate.
    if r > 20 || b > 20 || g > 20 {
        // `:92351-92360` — num = max(R, G, B) / 30.
        let num = r.max(g).max(b) as i32 / 30;
        // `:92361` — the gate draw, consumed even on failure.
        return rng.next_max(300) < num;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A crawler-shaped NPC: the roster has no family kind yet, so the
    /// fixture rides a placeholder kind with the C# box through
    /// `size_override` and the 5-HP life. `facing` 0 mirrors the C#
    /// constructor (`NPC.cs:8354`).
    fn crawler(npc_type: i32, x: f32, y: f32) -> Npc {
        let (w, h) = match npc_type {
            374 => (TRUFFLE_WORM.width, TRUFFLE_WORM.height),
            _ => (WORM.width, WORM.height),
        };
        let mut n = Npc::new(NpcKind::Zombie, x, y, 0, 0.0);
        n.size_override = Some((w as f32, h as f32));
        n.hp = 5;
        n.max_hp = 5;
        n
    }

    fn cand(x: f32, y: f32) -> TargetCandidate {
        TargetCandidate {
            active: true,
            dead: false,
            ghost: false,
            x,
            y,
            width: 20,
            height: 42,
            aggro: 0,
            item_animation: 0,
            npc_type_no_aggro: false,
            tank_pet: None,
        }
    }

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    /// The five SetDefaults rows, field by field (`NPC.cs:13545-13558`,
    /// `:13762-13776`, `:14332-14346`, `:14576-14589`, `:16781-16794`).
    #[test]
    fn the_defaults_rows_match_the_csharp() {
        for d in [&WORM, &TRUFFLE_WORM, &GOLD_WORM, &ENCHANTED_NIGHTCRAWLER, &MAGGOT] {
            assert_eq!(d.ai_style, 66);
            assert_eq!((d.damage, d.defense, d.life_max), (0, 0, 5));
            assert_eq!(d.hit_sound, (3, 1), "SoundID.NPCHit1");
            assert_eq!(d.death_sound, (4, 1), "SoundID.NPCDeath1");
            assert_eq!(d.knock_back_resist, 1.0, ":8533 head default");
            assert!(!d.no_gravity, ":8535 head default");
        }
        assert_eq!((WORM.width, WORM.height), (10, 4));
        assert_eq!(WORM.catch_item, 2002);
        assert!(WORM.friendly);
        assert_eq!((TRUFFLE_WORM.width, TRUFFLE_WORM.height), (20, 10));
        assert_eq!(TRUFFLE_WORM.npc_slots, 0.15);
        assert_eq!(TRUFFLE_WORM.catch_item, 2673);
        assert_eq!(TRUFFLE_WORM.rarity, 4);
        assert!(!TRUFFLE_WORM.friendly, "the head default (:8519)");
        assert!(TRUFFLE_WORM.dont_take_damage_from_hostiles, ":13775");
        assert_eq!(GOLD_WORM.catch_item, 2895);
        assert_eq!(GOLD_WORM.rarity, 3);
        assert_eq!(MAGGOT.catch_item, 4363);
        // `:14587` — the range row's catchItem ladder.
        assert_eq!(grub_row_catch_item(484), 3191);
        assert_eq!(grub_row_catch_item(485), 3192);
        assert_eq!(grub_row_catch_item(486), 3193);
        assert_eq!(grub_row_catch_item(487), 3194);
        // Main.cs:67082-67153 — the frame budgets.
        assert_eq!(FRAME_COUNTS.len(), 9);
        assert!(FRAME_COUNTS.contains(&(374, 4)));
        assert!(FRAME_COUNTS.contains(&(357, 2)));
        assert!(FRAME_COUNTS.contains(&(485, 4)));
    }

    /// `NPC.cs:34272-34289` — the walk speed ladder: 0.2 default,
    /// 0.25/0.325/0.4 for 485/486/487, ×3 for 374.
    #[test]
    fn the_walk_speed_ladder_matches_the_csharp() {
        let w = air();
        let players = [cand(1200.0, 1000.0)];
        for (ty, expect) in [
            (357, 0.2f32),
            (448, 0.2),
            (484, 0.2),
            (606, 0.2),
            (485, 0.25),
            (486, 0.325),
            (487, 0.4),
            (374, 0.6),
        ] {
            let mut n = crawler(ty, 1000.0, 1000.0);
            n.ai[0] = 1.0;
            n.facing = 1;
            n.local_ai[1] = 500.0; // no expiry this tick
            let mut rng = UnifiedRandom::new(7);
            let _ = ai_style66(&mut n, &w, &players, ty, 1.0, 0, &mut rng);
            assert_eq!(n.vx, expect, "type {ty} (:34272-34289)");
            // The mirror direction.
            n.facing = -1;
            let _ = ai_style66(&mut n, &w, &players, ty, 1.0, 0, &mut rng);
            assert_eq!(n.vx, -expect, "type {ty} facing -1");
        }
    }

    /// `NPC.cs:34293` — the idle gait zeroes velocity.X; `:34268-34271`
    /// — collideX flips the facing BEFORE the velocity write.
    #[test]
    fn idle_zeroes_and_collide_x_flips() {
        let w = air();
        let players = [cand(1200.0, 1000.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut n = crawler(357, 1000.0, 1000.0);
        n.ai[0] = 0.0;
        n.vx = 3.0;
        n.local_ai[1] = 500.0;
        let _ = ai_style66(&mut n, &w, &players, 357, 1.0, 0, &mut rng);
        assert_eq!(n.vx, 0.0, ":34293");
        let mut n2 = crawler(357, 1000.0, 1000.0);
        n2.ai[0] = 1.0;
        n2.facing = 1;
        n2.collide_x = true;
        n2.local_ai[1] = 500.0;
        let _ = ai_style66(&mut n2, &w, &players, 357, 1.0, 0, &mut rng);
        assert_eq!(n2.facing, -1, ":34270");
        assert_eq!(n2.vx, -0.2, "the velocity uses the FLIPPED facing");
    }

    /// `NPC.cs:34297-34310` — the timer: localAI[1] decrements every
    /// grounded tick; at <= 0 the gait toggles and draws exactly one
    /// range roll — Next(600, 1800) idle→walk (the FIRST expiry, from
    /// the SetDefaults 0), Next(300, 900) walk→idle. The toggle tick's
    /// velocity keeps the OLD gait. netUpdate arms.
    #[test]
    fn the_gait_timer_toggles_and_draws_in_c_order() {
        let w = air();
        let players = [cand(1200.0, 1000.0)];
        let mut n = crawler(357, 1000.0, 1000.0);
        n.facing = 1;
        // localAI[1] starts 0 (SetDefaults :8570) → first tick expires.
        let mut rng = UnifiedRandom::new(42);
        let witness = rng.clone();
        let first = rng.next_range(600, 1800); // the idle→walk draw
        let out = ai_style66(&mut n, &w, &players, 357, 1.0, 0, &mut rng);
        assert!(!out.transform_to_digger);
        assert_eq!(n.ai[0], 1.0, ":34307");
        assert_eq!(n.local_ai[1], first as f32, ":34308 Next(600,1800)");
        assert_eq!(n.vx, 0.0, "the toggle tick keeps the OLD idle gait");
        assert!(n.net_update, ":34310");
        assert_eq!(rng, witness, "exactly one Next(600,1800), nothing else");
        // Run the walk to expiry; the next draw is Next(300, 900).
        n.local_ai[1] = 1.0;
        n.net_update = false;
        let witness2 = rng.clone();
        let second = rng.next_range(300, 900);
        let _ = ai_style66(&mut n, &w, &players, 357, 1.0, 0, &mut rng);
        assert_eq!(n.ai[0], 0.0, ":34302");
        assert_eq!(n.local_ai[1], second as f32, ":34303 Next(300,900)");
        assert_eq!(n.vx, 0.2, "the toggle tick keeps the OLD walk gait");
        assert!(n.net_update);
        assert_eq!(rng, witness2, "exactly one Next(300,900)");
        // A non-expiry tick draws NOTHING.
        n.local_ai[1] = 100.0;
        let witness3 = rng.clone();
        let _ = ai_style66(&mut n, &w, &players, 357, 1.0, 0, &mut rng);
        assert_eq!(n.local_ai[1], 99.0, ":34297 the decrement");
        assert_eq!(rng, witness3);
    }

    /// `NPC.cs:34314-34321` — airborne: no timer, no velocity write;
    /// a directionless NPC latches ±1 from the sign of vx.
    #[test]
    fn the_airborne_latch_follows_the_velocity_sign() {
        let w = air();
        let players = [cand(1200.0, 1000.0)];
        let mut rng = UnifiedRandom::new(9);
        for (vx, expect) in [(-2.0f32, -1i32), (0.0, 1), (2.0, 1)] {
            let mut n = crawler(357, 1000.0, 1000.0);
            n.vy = 1.5; // airborne
            n.vx = vx;
            n.local_ai[1] = 1.0; // would expire if grounded
            let witness = rng.clone();
            let _ = ai_style66(&mut n, &w, &players, 357, 1.0, 0, &mut rng);
            assert_eq!(n.facing, expect, "vx {vx} (:34317-34320)");
            assert_eq!(n.local_ai[1], 1.0, "the timer is ground-only");
            assert_eq!(n.vx, vx, "no airborne velocity write");
            assert_eq!(rng, witness, "airborne non-484 draws nothing");
        }
        // An already-facing airborne NPC keeps its facing.
        let mut n = crawler(357, 1000.0, 1000.0);
        n.vy = 1.5;
        n.vx = 2.0;
        n.facing = -1;
        let _ = ai_style66(&mut n, &w, &players, 357, 1.0, 0, &mut rng);
        assert_eq!(n.facing, -1, "the latch is direction == 0 only");
    }

    /// `NPC.cs:34264-34267` — the walk's TargetClosest fires only while
    /// directionless; it faces the player through the shared port.
    #[test]
    fn the_directionless_walk_targets_the_player() {
        let w = air();
        // Player to the LEFT of the worm.
        let players = [cand(800.0, 979.0)];
        let mut rng = UnifiedRandom::new(3);
        let mut n = crawler(357, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.local_ai[1] = 500.0;
        let witness = rng.clone();
        let _ = ai_style66(&mut n, &w, &players, 357, 1.0, 11, &mut rng);
        assert_eq!(n.target, 0);
        assert_eq!(n.facing, -1, "TargetClosest faces the player");
        assert_eq!(n.vx, -0.2);
        assert_eq!(n.target_set_frame, 11, "the ticker stamp");
        assert_eq!(rng, witness, "TargetClosest draws nothing");
    }

    /// `NPC.cs:34255-34259` — the 484 flicker: one Next(90, 111) EVERY
    /// tick (grounded or airborne), the essScale factor, the
    /// (0.3, 0.1, 0.25) tint at the quotient-truncated centre tile.
    #[test]
    fn the_nightcrawler_flicker_draws_every_tick() {
        let w = air();
        let players = [cand(1200.0, 1000.0)];
        let mut n = crawler(484, 1000.0, 1000.0);
        n.vy = 2.0; // airborne — the flicker still draws
        let mut rng = UnifiedRandom::new(77);
        let witness = rng.clone();
        let roll = rng.next_range(90, 111);
        let out = ai_style66(&mut n, &w, &players, 484, 1.0, 0, &mut rng);
        assert_eq!(rng, witness, "exactly one Next(90,111)");
        let num983 = roll as f32 * 0.01 * ((1.0f32 + 0.5) / 2.0);
        let (tx, ty, rgb) = out.light.expect("484 lights");
        // Centre (1005, 1002): (int)(1005/16) = 62, (int)(1002/16) = 62.
        assert_eq!(tx, (1005.0f32 / 16.0) as i32);
        assert_eq!(ty, (1002.0f32 / 16.0) as i32);
        assert_eq!(rgb, [0.3 * num983, 0.1 * num983, 0.25 * num983]);
        // essScale 0.7 shrinks the ((ess + 0.5) / 2) factor vs 1.0 —
        // the expected value rides the SAME f32 expression order.
        let mut n2 = crawler(484, 1000.0, 1000.0);
        let mut rng2 = UnifiedRandom::new(77); // the same roll
        let out2 = ai_style66(&mut n2, &w, &players, 484, 0.7, 0, &mut rng2);
        let num983b = roll as f32 * 0.01 * ((0.7f32 + 0.5) / 2.0);
        assert_eq!(out2.light.unwrap().2[0], 0.3 * num983b);
        // A non-484 kind has no light and no flicker draw.
        let mut n3 = crawler(357, 1000.0, 1000.0);
        let witness3 = rng.clone();
        let out3 = ai_style66(&mut n3, &w, &players, 357, 1.0, 0, &mut rng);
        assert!(out3.light.is_none());
        assert_eq!(rng, witness3);
    }

    /// `NPC.cs:34322-34341` — the dig tail is 374-only: the counter
    /// increments once per tick while a live player is within 160
    /// (Euclidean), never decays, and caps at 90.
    #[test]
    fn the_dig_counter_is_truffle_worm_only_and_never_decays() {
        let w = air();
        let mut rng = UnifiedRandom::new(5);
        // A non-374 kind: no sprite_dir write, no counter.
        let near = [cand(1010.0, 979.0)];
        let mut n = crawler(357, 1000.0, 1000.0);
        n.local_ai[1] = 500.0;
        let out = ai_style66(&mut n, &w, &near, 357, 1.0, 0, &mut rng);
        assert!(!out.transform_to_digger);
        assert_eq!(n.ai[1], 0.0, ":34322 the early return");
        // The TruffleWorm: 89 ticks near, then away — the counter holds.
        let mut t = crawler(374, 1000.0, 1000.0);
        t.local_ai[1] = 5000.0;
        t.facing = 1;
        for _ in 0..89 {
            let _ = ai_style66(&mut t, &w, &near, 374, 1.0, 0, &mut rng);
        }
        assert_eq!(t.ai[1], 89.0);
        assert_eq!(t.sprite_dir, t.facing, ":34326");
        let far = [cand(5000.0, 979.0)];
        for _ in 0..10 {
            let out = ai_style66(&mut t, &w, &far, 374, 1.0, 0, &mut rng);
            assert!(!out.transform_to_digger);
        }
        assert_eq!(t.ai[1], 89.0, "no decay (:34338 has no else)");
    }

    /// `NPC.cs:34331` — the proximity gate is EUCLIDEAN 160 inclusive,
    /// with no ghost check; dead/inactive players never count.
    #[test]
    fn the_proximity_gate_is_euclidean_160_inclusive() {
        let w = air();
        let mut rng = UnifiedRandom::new(5);
        // Worm centre (1005, 1002). Player centre exactly 160 to the
        // right: (1165, 1002) → position (1155, 981).
        let edge = [cand(1155.0, 981.0)];
        let mut t = crawler(374, 1000.0, 1000.0);
        t.local_ai[1] = 5000.0;
        let _ = ai_style66(&mut t, &w, &edge, 374, 1.0, 0, &mut rng);
        assert_eq!(t.ai[1], 1.0, "exactly 160 counts (the !(> 160) gate)");
        // One px farther: 161 > 160 → no increment.
        let out = [cand(1156.0, 981.0)];
        let mut t2 = crawler(374, 1000.0, 1000.0);
        t2.local_ai[1] = 5000.0;
        let _ = ai_style66(&mut t2, &w, &out, 374, 1.0, 0, &mut rng);
        assert_eq!(t2.ai[1], 0.0);
        // Manhattan would count a diagonal (120, 120) = 240 > 160
        // Euclidean ≈ 169.7 — also out; use (113, 113) ≈ 159.8 IN while
        // Manhattan 226 would be out: the gate is Euclidean.
        let diag = [cand(1005.0 + 113.0 - 10.0, 1002.0 + 113.0 - 21.0)];
        let mut t3 = crawler(374, 1000.0, 1000.0);
        t3.local_ai[1] = 5000.0;
        let _ = ai_style66(&mut t3, &w, &diag, 374, 1.0, 0, &mut rng);
        assert_eq!(t3.ai[1], 1.0, "Euclidean, not Manhattan");
        // Dead and inactive players are skipped; a ghost still counts
        // (no ghost check at :34331).
        let mut dead = cand(1010.0, 979.0);
        dead.dead = true;
        let mut inactive = cand(1010.0, 979.0);
        inactive.active = false;
        let mut ghost = cand(1010.0, 979.0);
        ghost.ghost = true;
        for (p, expect) in [(dead, 0.0), (inactive, 0.0), (ghost, 1.0)] {
            let mut tt = crawler(374, 1000.0, 1000.0);
            tt.local_ai[1] = 5000.0;
            let _ = ai_style66(&mut tt, &w, &[p], 374, 1.0, 0, &mut rng);
            assert_eq!(tt.ai[1], expect);
        }
    }

    /// `NPC.cs:34342-34346` — at exactly ai[1] == 90 the worm sinks
    /// 16 px and requests the Transform(375); the request repeats while
    /// the type is still 374 (the C# re-checks every tick).
    #[test]
    fn the_dig_fires_at_exactly_ninety() {
        let w = air();
        let near = [cand(1010.0, 979.0)];
        let mut rng = UnifiedRandom::new(5);
        let mut t = crawler(374, 1000.0, 1000.0);
        t.local_ai[1] = 5000.0;
        t.ai[1] = 89.0;
        let y0 = t.y;
        let out = ai_style66(&mut t, &w, &near, 374, 1.0, 0, &mut rng);
        assert_eq!(t.ai[1], 90.0);
        assert!(out.transform_to_digger);
        assert_eq!(t.y, y0 + 16.0, ":34344");
        // The Transform seam's life rescale (:81106-81110): 5→5 keeps
        // the life; integer math; floor 1.
        assert_eq!(transform_life(5, 5, 5), 5);
        assert_eq!(transform_life(3, 5, 5), 3);
        assert_eq!(transform_life(0, 5, 5), 1, "the <= 0 floor");
        assert_eq!(transform_life(7, 10, 3), 2, "7*3/10 = 2 (int)");
    }

    /// `NPC.cs:92345-92368` — the gold sparkle: dark tiles draw
    /// NOTHING; a lit tile draws exactly one Next(300) (even on
    /// failure) and succeeds below max/30.
    #[test]
    fn the_gold_sparkle_gates_on_light_then_draws_once() {
        let mut rng = UnifiedRandom::new(11);
        // Dark: all channels <= 20 → no draw.
        let witness = rng.clone();
        assert!(!gold_worm_sparkle([20, 20, 20], &mut rng));
        assert_eq!(rng, witness, "the gate short-circuits the draw");
        // Lit: max 255 → threshold 255/30 = 8; the draw is consumed.
        let witness2 = rng.clone();
        let roll = rng.next_max(300);
        let got = gold_worm_sparkle([255, 0, 0], &mut rng);
        assert_eq!(got, roll < 8);
        assert_eq!(rng, witness2, "exactly one Next(300)");
        // The max channel picks the threshold: G 90 → 3.
        let witness3 = rng.clone();
        let roll3 = rng.next_max(300);
        let got3 = gold_worm_sparkle([21, 90, 30], &mut rng);
        assert_eq!(got3, roll3 < 3);
        assert_eq!(rng, witness3);
    }

    /// The whole-arm RNG witness: a mixed 120-tick run (grounded walk /
    /// idle / airborne, 484 + 374 + 357) replays EXACTLY the cited
    /// draws on a cloned stream — the 484 flicker every tick, one timer
    /// range roll per expiry, nothing else (no TargetClosest / dig /
    /// Transform draws).
    #[test]
    fn the_style66_arm_draws_only_the_cited_rolls() {
        let w = air();
        let players = [cand(1010.0, 979.0)];
        let mut rng = UnifiedRandom::new(2026);
        let witness = rng.clone();
        let mut night = crawler(484, 1000.0, 1000.0);
        let mut truffle = crawler(374, 1000.0, 1000.0);
        let mut worm = crawler(357, 1000.0, 1000.0);
        night.facing = 1;
        truffle.facing = 1;
        worm.facing = 1;
        for tick in 0..120 {
            // Airborne every 7th tick for the worm.
            worm.vy = if tick % 7 == 0 { 1.0 } else { 0.0 };
            for (n, ty) in [(&mut night, 484), (&mut truffle, 374), (&mut worm, 357)] {
                // Mirror the C# draws on the witness: the 484 flicker
                // first, then the grounded timer expiry.
                if ty == 484 {
                    let _ = witness.next_range(90, 111);
                }
                if n.vy == 0.0 && n.local_ai[1] - 1.0 <= 0.0 {
                    if n.ai[0] == 1.0 {
                        let _ = witness.next_range(300, 900);
                    } else {
                        let _ = witness.next_range(600, 1800);
                    }
                }
                let _ = ai_style66(n, &w, &players, ty, 1.0, tick as u64, &mut rng);
            }
        }
        assert_eq!(rng, witness, "the arm draws exactly the cited rolls");
        // The truffle dug long ago (the player never leaves).
        assert!(truffle.ai[1] >= 90.0);
    }
}
