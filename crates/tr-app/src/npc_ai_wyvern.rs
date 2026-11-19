//! The Wyvern's head machine — the type-87 arms of `NPC.AI_006_Worms`
//! (`NPC.cs:50963-52273`, dispatched by `aiStyle == 6`, `SetDefaults`
//! `NPC.cs:9945`). S1-D-3 (wyvern), **code-only mode**: the machine is
//! ported here as pure functions over the NPC pool; the sim wiring is
//! deferred — the shared worm pass (`world_sim/npcs.rs::update_worms_in`)
//! is R77's lane and stays untouched. See docs/ANALYSIS.md "The Wyvern
//! family (S1-D-3, code-only mode)".
//!
//! The C# family is six types — the head (87), the legs (88, chain
//! indices 1/8), the body (89, the default), body2 (90, index 11), body3
//! (91, index 12) and the tail (92, index 13), all aiStyle 6, 32×32,
//! lifeMax 4000, noGravity/noTileCollide/netAlways, kb 0
//! (`SetDefaults`, `NPC.cs:9939-10046`; the head dmg 80 / def 10 /
//! npcSlots 5 / value 10000, the segments dmg 40 / def 20 / dontCountMe,
//! value 10000 except the plain body's 2000). The port has all six
//! `NpcKind`s; the stats ride `npc/data.rs`'s tables.
//!
//! What is Wyvern-specific vs the shared worm sim-pass (the audit
//! `docs/research/55-audit-worms.md` T3a, distrust-verbatim):
//! - the **fixed 14-link chain** with per-index kinds (`:51156-51190`) —
//!   NOT the budget model; [`wyvern_chain_kind`] + [`assemble_wyvern_chain`]
//!   port the spawn block whole;
//! - `flag3 = true` always (`:51548-51551`) — the sky flyer never digs, so
//!   the tile scan (`:51573-51597`), the 1000-px dig override
//!   (`:51598-51615`) and the air-stumble branch (`:51925-51973`) are all
//!   unreachable for 87-92;
//! - the spriteDirection flips from the velocity sign (`:51616-51626`)
//!   and, on the followers, from the step sign (`:51911-51921`);
//! - the tunables 11/0.25 (`:51720-51724`), the segment spacing 42
//!   (`:51881-51884`);
//! - the **altitude gates** (`flag8`, `:52053-52078`): the reversal
//!   turn-boost inside 300 px and the climb rule (head below the player's
//!   centre, or the player below `worldSurface`, or dead);
//! - the sound gate `this.type != 87` (`:51976`) — the wyvern head plays
//!   no dig sound.
//! The shared three-arm accel steer (`:52112-52198`) and the head rotation
//! (`:52201`) are NOT wyvern-specific — they serve every AI_006 head; they
//! are ported here verbatim for the wyvern path because the live
//! `worm_head_step` is a velocity-snap stand-in (audit T1b/T1a). The
//! shared pass's own repair is R77's; this module duplicates nothing it
//! owns at runtime (UNWIRED).
//!
//! Determinism — **RNG by construction**: the 87-92 path through
//! `AI_006_Worms` draws NOTHING from `Main.rand`. Census of every draw in
//! the whole arm (`NPC.cs:50963-52273`): the 117 leech dust (`:50974`),
//! the 454-459/621-623 fade dust (`:50993`/`:51015`/`:51024`), the 402
//! stardust burst (`:51043-51058`), the expert EoW VileSpit rolls
//! (`:51087`/`:51100`), the 454 chain's `Next(6,10)` (`:51235`), the 513
//! chain's `Next(12,21)` (`:51259`), the budget rolls `Next(8,13)` etc.
//! (`:51310-51341` — the type list `:51303` excludes 87), the tile-scan
//! crumble `Next(100)` (`:51590` — unreachable, flag3 true) and the 412
//! crawltipede dust (`:51656`/`:51662`). None fires for a wyvern. The
//! `NewNPC` getGoodWorld roll (`:80752`) is gated on `Main.getGoodWorld`
//! (false in the port). So neither [`assemble_wyvern_chain`] nor
//! [`wyvern_head_step`] takes an RNG parameter; the witness tests pin a
//! seeded `UnifiedRandom` bit-identical across both.
//!
//! Pool indices: the C# links are `Main.npc` slot indices (`ai[0]` the
//! follower, `ai[1]` the leader, `ai[3]`/`realLife` the head). The port's
//! `App::npcs` compacts every frame, so the chain's stable identity is
//! `worm_id` + `seg_index` (the `carrier_index` convention, `npc.rs`);
//! [`assemble_wyvern_chain`] writes both the ai[] links (pool indices at
//! assembly time, the C#'s values) and the stable identity. The follower
//! step resolves its leader through the stable identity, never a stale
//! index.

#![allow(dead_code)] // wiring deferred - the sim dispatch + the chain drain land with R77's wave

use crate::npc::{Npc, NpcKind};
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::World;

/// The chain length — `for (int m = 0; m < 14; m++)` (`NPC.cs:51162`).
pub const WYVERN_CHAIN_LINKS: u32 = 14;

/// The per-index segment kind (`NPC.cs:51164-51180`): the default 89,
/// legs 88 at m = 1/8, body2 90 at m = 11, body3 91 at m = 12, tail 92
/// at m = 13. `m` is the loop index — the head is NOT one of the 14
/// links (the chain is 15 NPCs long, head + 14).
pub fn wyvern_chain_kind(m: u32) -> NpcKind {
    match m {
        1 | 8 => NpcKind::WyvernLegs,
        11 => NpcKind::WyvernBody2,
        12 => NpcKind::WyvernBody3,
        13 => NpcKind::WyvernTail,
        _ => NpcKind::WyvernBody,
    }
}

/// The follower spacing — `num63 = 42f` for types 87-92
/// (`NPC.cs:51880-51884`; the base `num63 = base.width` never applies).
pub const WYVERN_SEGMENT_SPACING: f32 = 42.0;

/// The head's tunables — `num45 = 11f`, `num46 = 0.25f`
/// (`NPC.cs:51720-51724`; `npc/data.rs::worm_tunables` carries the same
/// row for the shared table's readers).
pub const WYVERN_MAX_SPEED: f32 = 11.0;
pub const WYVERN_ACCEL: f32 = 0.25;

/// The fixed 14-link chain assembly (`NPC.cs:51156-51190`), the
/// `netMode != 1 && type == 87 && ai[0] == 0` block ported whole.
///
/// The C# spawns each link with
/// `NewNPC(source, (int)(pos.X + width/2), (int)(pos.Y + height), kind,
/// whoAmI)` — `NewNPC` sets `Bottom = (X, Y)` (`NPC.cs:80770`), i.e. the
/// child's bottom-centre lands on the parent's bottom-centre
/// (`Entity.cs:128-131`: `position = (X - width/2, Y - height)`). Every
/// link therefore starts stacked on the head's bottom-centre; the first
/// follower pass spreads them. The pixel truncation is the C#'s `(int)`
/// casts (toward zero — `.trunc()`, not `.floor()`).
///
/// `ai[3] = realLife = head` on the head and every link (`:51158-51159`,
/// `:51182-51183`); `ai[1]` the leader's slot, `ai[0]` the follower's
/// slot (`:51184`/`:51186`). `CopyInteractions` (`:51185`) copies the
/// player-interaction flags — the port's fresh head has none set at
/// assembly time (the sim assembles on the spawn tick), so the copy is
/// the identity; the wiring re-copies if it ever assembles later.
///
/// Returns the number of links pushed (always 14 — the port's Vec has no
/// slot cap; the C#'s `NewNPC` can return `Main.maxNPCs` on a full pool,
/// `:80794`, which the wiring must guard before calling).
pub fn assemble_wyvern_chain(npcs: &mut Vec<Npc>, head: usize, next_worm_id: &mut u32) -> u32 {
    // The id-less head takes the next chain id (the port's stable
    // identity — the `update_worms_in` init convention).
    if npcs[head].worm_id == 0 {
        npcs[head].worm_id = *next_worm_id;
        *next_worm_id += 1;
    }
    let wid = npcs[head].worm_id;
    // `:51158-51159` — ai[3] = whoAmI; realLife = whoAmI.
    npcs[head].ai[3] = head as f32;
    npcs[head].seg_index = 0;
    npcs[head].worm_inited = true;
    let (hw, hh) = npcs[head].size();
    // The spawn anchor: `(int)(position.X + width/2), (int)(position.Y +
    // height)` — the head's bottom-centre, truncated (`:51181`).
    let spawn_x = (npcs[head].x + hw * 0.5).trunc();
    let spawn_y = (npcs[head].y + hh).trunc();
    let mut leader = head;
    for m in 0..WYVERN_CHAIN_LINKS {
        let kind = wyvern_chain_kind(m);
        let (cw, ch) = kind.size();
        // `NewNPC`'s Bottom placement (`:80770`, `Entity.cs:130`).
        let mut child = Npc::new(kind, spawn_x - cw * 0.5, spawn_y - ch, 1, 0.0);
        child.worm_id = wid;
        child.seg_index = m + 1;
        child.worm_inited = true;
        let slot = npcs.len();
        // `:51182-51184` — ai[3] = head; realLife = head; ai[1] = leader.
        child.ai[3] = head as f32;
        child.ai[1] = leader as f32;
        npcs.push(child);
        // `:51186` — the leader's ai[0] points at the new link.
        npcs[leader].ai[0] = slot as f32;
        leader = slot;
    }
    WYVERN_CHAIN_LINKS
}

/// The per-NPC inputs the machine reads that live on `Main` in the C#.
/// The wiring passes the port's stock values (single-player, classic).
#[derive(Clone, Copy, Debug)]
pub struct WyvernCtx {
    /// `Main.netMode` — the chain-assembly block runs when `!= 1`
    /// (`NPC.cs:51154`); the port is single-player → 0.
    pub net_mode: i32,
    /// `Main.worldSurface` (`Main.cs`'s tile row, `tr_core::World`'s
    /// `world_surface` f64) — the climb gate's player-below-surface test
    /// (`NPC.cs:52063`) compares in TILES against this f64, exactly like
    /// the C#'s `(double)(Center.Y / 16f) > Main.worldSurface`.
    pub world_surface: f64,
}

/// The side effects the machine cannot apply itself. Everything the
/// `Npc` struct carries (position, velocity, ai, rotation, sprite_dir,
/// target, despawn_timer) is written in place, in C# order.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WyvernEvents {
    /// The chain-assembly request (`NPC.cs:51156`: `type == 87 &&
    /// ai[0] == 0` under `netMode != 1`). The caller resolves it through
    /// [`assemble_wyvern_chain`] (the pool + the id counter live outside
    /// the per-NPC step).
    pub wants_chain: bool,
    /// `NPC.realLife` (`NPC.cs:51133-51135`: `ai[3] > 0` mirrors into
    /// realLife). The port's carrier resolution reads the stable
    /// identity instead (`shares_carrier_life` + `worm_id`); the event
    /// carries the C#'s value for the wiring's net sync.
    pub real_life: Option<i32>,
}

/// The `Main.player[this.target]` read after a selection: the C# indexes
/// the 255-slot array (an absent slot is an inactive, non-dead player at
/// the origin); the port's slice holds the session player(s) and
/// `TargetClosest` itself falls back to slot 0 (`NPC.cs:77731-77734`).
/// The style-37 module's convention.
fn target_player<'a>(players: &'a [TargetCandidate], target: i32) -> &'a TargetCandidate {
    players
        .get(target as usize)
        .unwrap_or(&players[0])
}

/// `Entity.Center` (`Entity.cs:54`): `position + (width, height) / 2f` —
/// FLOAT halves of the int box (the 32×32 wyvern halves exactly).
fn center(n: &Npc) -> (f32, f32) {
    let (w, h) = n.size();
    (n.x + w * 0.5, n.y + h * 0.5)
}

/// The wyvern head's per-tick machine — the type-87 path through
/// `AI_006_Worms` (`NPC.cs:50963-52273`), in source order. The follower
/// branch (`ai[1] > 0`) is [`wyvern_follower_step`]; this fn is the
/// no-leader `else` (`:51923-52201`) plus the shared prologue.
///
/// RNG-free by construction (module docs) — no RNG parameter.
pub fn wyvern_head_step(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    ctx: &WyvernCtx,
    ticker: u64,
) -> WyvernEvents {
    let mut ev = WyvernEvents::default();
    // `:51110-51128` — the surface-gate `flag` switch: 87 has no case,
    // flag stays false (the wyvern never despawns for altitude).
    // `:51129-51135` — the realLife mirror (13-15's -1 arm excludes 87).
    if npc.ai[3] > 0.0 {
        ev.real_life = Some(npc.ai[3] as i32);
    }
    // `:51137-51140` — the invalid/dead target re-pick. The `== 255`
    // sentinel short-circuits before the player read (the C#'s `||`
    // order); `flag` false drops the surface term.
    {
        let t = npc.target;
        let invalid =
            t < 0 || t == 255 || players.get(t as usize).map_or(false, |p| p.dead);
        if invalid {
            target_closest(npc, world, players, true, 6, ticker);
        }
    }
    // `:51141-51148` — the dead-target despawn discouragement. `flag`
    // false: no `velocity.Y += num10` (that is the diggers' sink).
    let p = target_player(players, npc.target);
    if p.dead {
        npc.encourage_despawn(300);
    }
    // `:51154-51190` — the chain assembly (the request; the pool lives
    // outside this step). `ai[0] == 0` is the C#'s exact float compare.
    if ctx.net_mode != 1 && npc.ai[0] == 0.0 {
        ev.wants_chain = true;
    }
    // `:51393-51477` — the orphan-death switches: 87 is in the second
    // list (`:51444`) whose body checks `ai[0]` (the FOLLOWER). A head
    // whose follower is gone dies (`life = 0` + HitEffect + checkDead +
    // deactivate, `:51468-51475`). The port's stable identity resolves
    // the follower; a head that has not assembled yet (ai[0] == 0, the
    // spawn tick) reads slot 0 in the C# — the assembly runs FIRST in
    // the same tick, so the live head always has a follower by here.
    // The wiring owns this check (it needs the pool); the pure step
    // carries the aim/steer only. Cited, not folded silently.
    // `:51548-51551` — flag3 = true always: no tile scan, no dig.
    // `:51616-51626` — the spriteDirection flip from the velocity sign.
    if npc.vx < 0.0 {
        npc.sprite_dir = 1;
    } else if npc.vx > 0.0 {
        npc.sprite_dir = -1;
    }
    // `:51670-51671` + `:51720-51724` — the tunables.
    let num45 = WYVERN_MAX_SPEED;
    let num46 = WYVERN_ACCEL;
    // `:51774-51776` — the aim starts at the player's Centre.
    let (mut cx, mut cy) = center(npc);
    let p = target_player(players, npc.target);
    let mut num48 = p.x + p.width as f32 * 0.5;
    let mut num49 = p.y + p.height as f32 * 0.5;
    // `:51855-51860` — the 16-px snap: `(int)(v / 16f) * 16` on both
    // ends, then the delta. The `(int)` casts truncate toward zero.
    num48 = ((num48 / 16.0).trunc() as i32 * 16) as f32;
    num49 = ((num49 / 16.0).trunc() as i32 * 16) as f32;
    cx = ((cx / 16.0).trunc() as i32 * 16) as f32;
    cy = ((cy / 16.0).trunc() as i32 * 16) as f32;
    num48 -= cx;
    num49 -= cy;
    // `:51866` computes num62 here, but the head's fly path reassigns it
    // at `:51997` before any read (the pre-scale value serves only the
    // follower branch + the sound block, both unreachable for 87) — the
    // dead local is elided; the live recompute is below.
    // The no-leader else (`:51923`). flag3 true → the fly branch
    // (`:51974-52200`).
    // `:51976-51996` — the dig-sound block: `this.type != 87` excludes
    // the wyvern head; nothing plays, no soundDelay write.
    // `:51997-52002` — the snapped aim scaled to the max speed. num62
    // is RE-computed from the snapped delta (the C# reassigns it).
    let num62 = (num48 * num48 + num49 * num49).sqrt();
    let num65 = num48.abs();
    let num66 = num49.abs();
    let num67 = num45 / num62;
    num48 *= num67;
    num49 *= num67;
    // `:52003-52051` — flag6 (the corruption/desert escape): the type
    // tests are 7/13/513/510 — none is 87, flag6 stays false.
    // `:52052-52078` — the ALTITUDE GATES (flag8), the wyvern's own arm.
    let mut flag8 = false;
    // `:52055-52062` — the reversal turn-boost: any axis of the velocity
    // opposes the aim, the Manhattan speed exceeds num46/2, and the
    // snapped distance is under 300 px → flag8; below the max speed the
    // whole velocity scales ×1.1 (BOTH axes, one Vector2 multiply).
    if ((npc.vx > 0.0 && num48 < 0.0)
        || (npc.vx < 0.0 && num48 > 0.0)
        || (npc.vy > 0.0 && num49 < 0.0)
        || (npc.vy < 0.0 && num49 > 0.0))
        && npc.vx.abs() + npc.vy.abs() > num46 / 2.0
        && num62 < 300.0
    {
        flag8 = true;
        if npc.vx.abs() + npc.vy.abs() < num45 {
            npc.vx *= 1.1;
            npc.vy *= 1.1;
        }
    }
    // `:52063-52078` — the climb rule: the head's centre below the
    // player's centre, OR the player's centre below the surface (the
    // f64 tile compare), OR the player dead → flag8; keep the horizontal
    // speed under num45/2 (the ×1.1 ramp, the vx == 0 kick `-direction`)
    // else climb while vy > -num45.
    let p = target_player(players, npc.target);
    let (_, hcy) = center(npc);
    let pcy = p.y + p.height as f32 * 0.5;
    if hcy > pcy || ((pcy / 16.0) as f64) > ctx.world_surface || p.dead {
        flag8 = true;
        if npc.vx.abs() < num45 / 2.0 {
            if npc.vx == 0.0 {
                npc.vx -= npc.facing as f32;
            }
            npc.vx *= 1.1;
        } else if npc.vy > 0.0 - num45 {
            npc.vy -= num46;
        }
    }
    // `:52112-52198` — the shared three-arm accel steer, verbatim. The
    // wyvern reaches it whenever neither altitude gate fired.
    if !flag8 {
        steer_toward_aim(npc, num48, num49, num65, num66, num45, num46);
    }
    // `:52201` — the head rotation: atan2(vy, vx) + PI/2 (the C#'s
    // `(float)Math.PI / 2f`, NOT the followers' 1.57f literal — the two
    // constants differ in the last ulp and stay distinct).
    npc.rotation = npc.vy.atan2(npc.vx) + std::f32::consts::FRAC_PI_2;
    // `:52202-52249` — the localAI[0]/netUpdate tail and the 454
    // frameCounter: the type lists exclude 87. `:52251` — the EoW fade
    // gate returns for every non-13-15 type.
    ev
}

/// The shared three-arm accel steer (`NPC.cs:52112-52198`), verbatim —
/// the same-sign arm with the two cross-brakes, the |num48| > |num49|
/// arm and its mirror. `num65`/`num66` are the PRE-scale |aim| halves
/// (`:51998-51999`), captured before the ×num67 scale.
fn steer_toward_aim(
    npc: &mut Npc,
    num48: f32,
    num49: f32,
    num65: f32,
    num66: f32,
    num45: f32,
    num46: f32,
) {
    // `:52114-52153` — any axis already agrees with the aim.
    if (npc.vx > 0.0 && num48 > 0.0)
        || (npc.vx < 0.0 && num48 < 0.0)
        || (npc.vy > 0.0 && num49 > 0.0)
        || (npc.vy < 0.0 && num49 < 0.0)
    {
        // `:52116-52131` — ±num46 per axis toward the aim.
        if npc.vx < num48 {
            npc.vx += num46;
        } else if npc.vx > num48 {
            npc.vx -= num46;
        }
        if npc.vy < num49 {
            npc.vy += num46;
        } else if npc.vy > num49 {
            npc.vy -= num46;
        }
        // `:52132-52142` — the Y cross-brake: the aim is nearly
        // horizontal (|num49| < num45*0.2) and X opposes it → vy ±num46*2
        // along its own sign.
        if num49.abs() < num45 * 0.2
            && ((npc.vx > 0.0 && num48 < 0.0) || (npc.vx < 0.0 && num48 > 0.0))
        {
            if npc.vy > 0.0 {
                npc.vy += num46 * 2.0;
            } else {
                npc.vy -= num46 * 2.0;
            }
        }
        // `:52143-52153` — the X mirror.
        if num48.abs() < num45 * 0.2
            && ((npc.vy > 0.0 && num49 < 0.0) || (npc.vy < 0.0 && num49 > 0.0))
        {
            if npc.vx > 0.0 {
                npc.vx += num46 * 2.0;
            } else {
                npc.vx -= num46 * 2.0;
            }
        }
    }
    // `:52155-52176` — the X-dominant arm: ±num46*1.1 on X, and below
    // half speed the Y sign-assist ±num46.
    else if num65 > num66 {
        if npc.vx < num48 {
            npc.vx += num46 * 1.1;
        } else if npc.vx > num48 {
            npc.vx -= num46 * 1.1;
        }
        if npc.vx.abs() + npc.vy.abs() < num45 * 0.5 {
            if npc.vy > 0.0 {
                npc.vy += num46;
            } else {
                npc.vy -= num46;
            }
        }
    }
    // `:52177-52198` — the Y mirror.
    else {
        if npc.vy < num49 {
            npc.vy += num46 * 1.1;
        } else if npc.vy > num49 {
            npc.vy -= num46 * 1.1;
        }
        if npc.vx.abs() + npc.vy.abs() < num45 * 0.5 {
            if npc.vx > 0.0 {
                npc.vx += num46;
            } else {
                npc.vx -= num46;
            }
        }
    }
}

/// The wyvern follower's per-tick step — the `ai[1] > 0` branch
/// (`NPC.cs:51867-51922`) with the wyvern's spacing 42 and step-sign
/// sprite flip. The shared follow model is FAITHFUL per the audit
/// (T2a: a direct leader snap, not position history); this fn is the
/// wyvern-parameterised copy for the unwired path — the live shared
/// pass keeps its own (R77's lane).
///
/// `leader` is the resolved previous segment (the wiring finds it by
/// `worm_id` + `seg_index - 1`, the `update_worms_in` convention); the
/// C#'s `try/catch` (`:51869-51877`) swallows a missing slot and keeps
/// the pre-branch aim — the caller passes None and this fn skips the
/// snap (the orphan death is the wiring's, `:51425-51433`).
pub fn wyvern_follower_step(npc: &mut Npc, leader: Option<&Npc>) {
    let Some(leader) = leader else { return };
    let (cx, cy) = center(npc);
    let (lcx, lcy) = center(leader);
    let mut num48 = lcx - cx;
    let mut num49 = lcy - cy;
    // `:51878` — the follower rotation: atan2 + the 1.57f LITERAL (the
    // C# uses 1.57f here, Math.PI/2 on the head — distinct constants).
    npc.rotation = num49.atan2(num48) + 1.57;
    // `:51879-51905` — the spacing snap: num63 = 42 for 87-92
    // (`:51881-51884`), num62 = (dist - 42) / dist.
    let num62 = (num48 * num48 + num49 * num49).sqrt();
    let num63 = WYVERN_SEGMENT_SPACING;
    let scale = (num62 - num63) / num62;
    num48 *= scale;
    num49 *= scale;
    // `:51908-51910` — velocity zeroed, position stepped.
    npc.vx = 0.0;
    npc.vy = 0.0;
    npc.x += num48;
    npc.y += num49;
    // `:51911-51921` — the step-sign sprite flip (the wyvern arm of the
    // 87-92/454-459/621-623 group).
    if num48 < 0.0 {
        npc.sprite_dir = 1;
    } else if num48 > 0.0 {
        npc.sprite_dir = -1;
    }
}

/// The wyvern HitEffect plan (`NPC.cs:88909-88933`, the `type >= 87 &&
/// type <= 92` arm) — count-only, the dust/gore visuals are the port's
/// documented skip class. The HIT half (life > 0) is the dmg-scaled
/// dust loop, bound `dmg / lifeMax * 50.0` in DOUBLE precision (the C#'s
/// `(double)num738 < dmg / (double)this.lifeMax * 50.0`); the DEATH half
/// is 10 dusts + `Next(1, 4)` gores, each gore's type a further
/// `Next(11, 14)` draw. The gore draws are REAL RNG — the plan returns
/// the counts and the wiring feeds them to its stream in this order
/// (one Next(1,4), then one Next(11,14) per gore). The AI itself never
/// calls HitEffect; only the strike/death path does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WyvernHitDust {
    /// The hit half (`:88911-88919`): `count` dusts, no gore, no draws.
    Hit { count: u32 },
    /// The death half (`:88921-88932`): 10 dusts + `gores` gore spawns
    /// (the `Next(1, 4)` result; each gore draws `Next(11, 14)`).
    Death { dust: u32, gores: u32 },
}

/// The dust/gore counts for one strike (`NPC.cs:88909-88933`). `life`
/// is `this.life` AT THE CALL (the death path passes <= 0 after
/// `life = 0`); `dmg` the strike's damage, `life_max` the 4000 pool.
/// Pure — the gore RNG draws happen in the wiring, in the C#'s order.
pub fn wyvern_hit_dust_plan(life: i32, dmg: f64, life_max: i32) -> WyvernHitDust {
    if life > 0 {
        // `:88913` — the double-precision loop bound, counted exactly as
        // the C# for-loop runs it.
        let bound = dmg / life_max as f64 * 50.0;
        let mut count = 0u32;
        while (count as f64) < bound {
            count += 1;
        }
        return WyvernHitDust::Hit { count };
    }
    // `:88921-88932` — the flat 10 dusts; the gore count is the wiring's
    // `Next(1, 4)` draw (1..=3). The plan carries the dust count and the
    // gore-count RANGE; the wiring resolves the draw.
    WyvernHitDust::Death { dust: 10, gores: 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::UnifiedRandom;

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
        let mut w = World::new("t".into(), 200, 200);
        w.world_surface = 50.0;
        w
    }

    fn head(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::WyvernHead, x, y, 1, 0.0);
        n.target = 0;
        n
    }

    fn ctx() -> WyvernCtx {
        WyvernCtx { net_mode: 0, world_surface: 50.0 }
    }

    /// `NPC.cs:9939-10046` — the six SetDefaults rows, field by field,
    /// through the port's kind tables.
    #[test]
    fn the_wyvern_defaults_match_the_csharp_rows() {
        let kinds = [
            NpcKind::WyvernHead,
            NpcKind::WyvernLegs,
            NpcKind::WyvernBody,
            NpcKind::WyvernBody2,
            NpcKind::WyvernBody3,
            NpcKind::WyvernTail,
        ];
        for k in kinds {
            assert_eq!(k.size(), (32.0, 32.0), "{k:?} box");
            assert_eq!(k.max_hp(), 4000, "{k:?} lifeMax");
            assert_eq!(k.ai_style(), 6, "{k:?} aiStyle");
            let n = Npc::new(k, 0.0, 0.0, 1, 0.0);
            assert_eq!(n.knock_back_resist(), 0.0, "{k:?} kb");
        }
        assert_eq!(NpcKind::WyvernHead.contact_damage(), 80);
        assert_eq!(Npc::new(NpcKind::WyvernHead, 0.0, 0.0, 1, 0.0).defense(), 10);
        for k in &kinds[1..] {
            assert_eq!(k.contact_damage(), 40, "{k:?} dmg");
            assert_eq!(Npc::new(*k, 0.0, 0.0, 1, 0.0).defense(), 20, "{k:?} def");
        }
        // The value rows: 10000 everywhere except the plain body's 2000
        // (`NPC.cs:9989`).
        assert_eq!(Npc::new(NpcKind::WyvernHead, 0.0, 0.0, 1, 0.0).value(), 10000.0);
        assert_eq!(Npc::new(NpcKind::WyvernBody, 0.0, 0.0, 1, 0.0).value(), 2000.0);
        assert_eq!(Npc::new(NpcKind::WyvernTail, 0.0, 0.0, 1, 0.0).value(), 10000.0);
        // The hit/death cues (`NPC.cs:9950-9951`).
        assert_eq!(NpcKind::WyvernHead.hit_sound(), Some("NPC_Hit_7"));
        assert_eq!(NpcKind::WyvernTail.death_sound(), Some("NPC_Killed_8"));
        // The tunables + spacing (`:51720-51724`, `:51881-51884`).
        assert_eq!(NpcKind::WyvernHead.worm_tunables(), (11.0, 0.25));
        assert_eq!(WYVERN_SEGMENT_SPACING, 42.0);
    }

    /// `NPC.cs:51164-51180` — the per-index kind table: legs at 1/8,
    /// body2 at 11, body3 at 12, tail at 13, body everywhere else.
    #[test]
    fn the_chain_kinds_follow_the_index_table() {
        let expect = [
            NpcKind::WyvernBody,  // m=0
            NpcKind::WyvernLegs,  // m=1
            NpcKind::WyvernBody,  // m=2
            NpcKind::WyvernBody,  // m=3
            NpcKind::WyvernBody,  // m=4
            NpcKind::WyvernBody,  // m=5
            NpcKind::WyvernBody,  // m=6
            NpcKind::WyvernBody,  // m=7
            NpcKind::WyvernLegs,  // m=8
            NpcKind::WyvernBody,  // m=9
            NpcKind::WyvernBody,  // m=10
            NpcKind::WyvernBody2, // m=11
            NpcKind::WyvernBody3, // m=12
            NpcKind::WyvernTail,  // m=13
        ];
        for (m, k) in expect.iter().enumerate() {
            assert_eq!(wyvern_chain_kind(m as u32), *k, "m={m}");
        }
    }

    /// `NPC.cs:51156-51190` — the assembly: 14 links, the ai[]/realLife
    /// links, the Bottom placement (every link stacked on the head's
    /// bottom-centre, `:80770` + `Entity.cs:130`), the stable identity.
    #[test]
    fn the_chain_assembles_fourteen_links_with_the_csharp_links() {
        let mut npcs = vec![head(1000.0, 500.0)];
        let mut next_id = 7u32;
        let n = assemble_wyvern_chain(&mut npcs, 0, &mut next_id);
        assert_eq!(n, 14);
        assert_eq!(npcs.len(), 15);
        assert_eq!(next_id, 8);
        assert_eq!(npcs[0].worm_id, 7);
        assert_eq!(npcs[0].ai[3], 0.0); // ai[3] = whoAmI = 0
        // The head's bottom-centre: (1000 + 16, 500 + 32) = (1016, 532).
        // Every 32x32 link's top-left: (1016 - 16, 532 - 32) = (1000, 500).
        for m in 0..14u32 {
            let i = m as usize + 1;
            assert_eq!(npcs[i].kind, wyvern_chain_kind(m), "link {m}");
            assert_eq!(npcs[i].x, 1000.0, "link {m} x (Bottom placement)");
            assert_eq!(npcs[i].y, 500.0, "link {m} y");
            assert_eq!(npcs[i].worm_id, 7);
            assert_eq!(npcs[i].seg_index, m + 1);
            assert_eq!(npcs[i].ai[3], 0.0, "ai[3] = head slot");
            assert_eq!(npcs[i].ai[1], (i - 1) as f32, "ai[1] = leader slot");
            assert_eq!(npcs[i - 1].ai[0], i as f32, "leader ai[0] = follower");
            assert_eq!(npcs[i].hp, 4000);
        }
        // The tail's ai[0] stays 0 — it ends the chain.
        assert_eq!(npcs[14].ai[0], 0.0);
    }

    /// The assembly truncates the spawn anchor toward zero (the C#'s
    /// `(int)` casts, `:51181`) — a fractional head position lands the
    /// links on the truncated bottom-centre, not the rounded one.
    #[test]
    fn the_chain_spawn_anchor_truncates_like_the_int_casts() {
        let mut npcs = vec![head(1000.9, 500.7)];
        let mut id = 1u32;
        assemble_wyvern_chain(&mut npcs, 0, &mut id);
        // (int)(1000.9 + 16) = 1016; (int)(500.7 + 32) = 532.
        assert_eq!(npcs[1].x, 1016.0 - 16.0);
        assert_eq!(npcs[1].y, 532.0 - 32.0);
    }

    /// `NPC.cs:52055-52062` — the reversal turn-boost: an opposing aim
    /// inside 300 px with Manhattan speed over num46/2 and under num45
    /// scales BOTH velocity axes ×1.1 and skips the steer (flag8).
    #[test]
    fn the_reversal_gate_boosts_the_velocity_inside_300px() {
        let w = air();
        // Head at (1600, 400) centre (1616, 416); player at (1625, 400)
        // centre (1635, 421) — snapped (1632, 416) vs the head's snapped
        // (1616, 416): delta (+16, 0), the aim is +X while vx is -X (a
        // reversal), dist 16 < 300.
        let mut n = head(1600.0, 400.0);
        n.vx = -4.0;
        n.vy = 1.0; // Manhattan 5 > 0.125, < 11
        let players = [cand(1625.0, 400.0)];
        // The climb gate must NOT fire: the head centre (416) is above
        // the player centre (421)? No — 416 < 421, and the player is
        // above the surface (421/16 = 26.3 < 50), alive. Good.
        wyvern_head_step(&mut n, &w, &players, &ctx(), 0);
        assert_eq!(n.vx, -4.0 * 1.1, "the x boost");
        assert_eq!(n.vy, 1.0 * 1.1, "the y boost");
    }

    /// `NPC.cs:52063-52078` — the climb rule: the head below the
    /// player's centre keeps |vx| under 5.5 via the ×1.1 ramp (the
    /// vx == 0 kick is `-direction`), else vy -= 0.25 while vy > -11.
    #[test]
    fn the_climb_gate_ramps_x_then_climbs() {
        let w = air();
        // Head centre y = 416+... put the head BELOW the player: head
        // at (1600, 800) centre 816; player centre 421. 816 > 421 →
        // the climb gate fires.
        let mut n = head(1600.0, 800.0);
        n.vx = 2.0; // |2| < 5.5 → the ramp
        n.vy = 3.0;
        let players = [cand(1610.0, 400.0)];
        wyvern_head_step(&mut n, &w, &players, &ctx(), 0);
        assert_eq!(n.vx, 2.0 * 1.1, "the x ramp");
        assert_eq!(n.vy, 3.0, "no climb while the x ramp runs");
        // At |vx| >= 5.5 the climb arm runs instead.
        let mut n2 = head(1600.0, 800.0);
        n2.vx = 6.0;
        n2.vy = 3.0;
        wyvern_head_step(&mut n2, &w, &players, &ctx(), 0);
        assert_eq!(n2.vx, 6.0, "no x ramp past half speed");
        assert_eq!(n2.vy, 3.0 - 0.25, "the climb");
        // The vx == 0 kick: -direction (facing 1 → vx = -1, then ×1.1).
        let mut n3 = head(1600.0, 800.0);
        n3.vx = 0.0;
        n3.facing = 1;
        wyvern_head_step(&mut n3, &w, &players, &ctx(), 0);
        assert_eq!(n3.vx, -1.0 * 1.1, "the zero kick rides -direction");
    }

    /// `NPC.cs:52063` — the player-below-surface term: the head ABOVE
    /// the player still climbs when the player's centre is below
    /// `worldSurface` (the f64 tile compare).
    #[test]
    fn the_climb_gate_fires_when_the_player_is_below_the_surface() {
        let w = air(); // world_surface = 50 tiles = 800 px
        // Head centre 416 (above); player centre 900 + 21 = 921 →
        // 921/16 = 57.6 > 50 → the gate fires despite the head being
        // above the player.
        let mut n = head(1600.0, 400.0);
        n.vx = 6.0;
        n.vy = 3.0;
        let players = [cand(1610.0, 900.0)];
        wyvern_head_step(&mut n, &w, &players, &ctx(), 0);
        assert_eq!(n.vy, 3.0 - 0.25, "the below-surface climb");
    }

    /// `NPC.cs:52112-52198` — with neither gate firing the shared
    /// three-arm steer runs: the same-sign arm approaches the aim at
    /// ±num46 per axis (NOT a velocity snap — the audit T1b defect).
    #[test]
    fn the_open_sky_steer_approaches_the_aim_by_accel() {
        let w = air();
        // Head at (1600, 400), player far right at (2400, 400): snapped
        // delta (+800, 0) → dist 800 > 300 (no reversal gate); head
        // centre 416 vs player centre 421 — the head is ABOVE, player
        // above surface, alive → no climb gate. The steer runs.
        let mut n = head(1600.0, 400.0);
        n.vx = 1.0;
        n.vy = 0.0;
        let players = [cand(2400.0, 400.0)];
        wyvern_head_step(&mut n, &w, &players, &ctx(), 0);
        // The aim scales to (11, 0); same-sign X arm: vx 1 → 1.25.
        assert_eq!(n.vx, 1.25, "the +num46 approach, not the 11 snap");
        // vy 0 == num49 0: no Y write; the cross-brakes need an opposing
        // sign — none fires.
        assert_eq!(n.vy, 0.0);
        // The head rotation (`:52201`): atan2(0, 1.25) + PI/2.
        assert_eq!(n.rotation, std::f32::consts::FRAC_PI_2);
    }

    /// `NPC.cs:51867-51922` — the follower snap: the 42-px spacing, the
    /// velocity zero, the 1.57f-literal rotation, the step-sign flip.
    #[test]
    fn the_follower_snaps_to_the_42px_spacing() {
        let leader = head(1600.0, 400.0); // centre (1616, 416)
        let mut f = Npc::new(NpcKind::WyvernBody, 1600.0, 500.0, 1, 0.0);
        // Follower centre (1616, 516): delta (0, -100), dist 100.
        f.vx = 5.0;
        f.vy = 5.0;
        wyvern_follower_step(&mut f, Some(&leader));
        // scale = (100 - 42)/100 = 0.58 → step (0, -58). The f32 0.58 is
        // inexact; the C# runs the same f32 ops, so the pin is a tight
        // tolerance, not a decimal literal.
        assert_eq!(f.x, 1600.0);
        assert!((f.y - (500.0 - 58.0)).abs() < 1e-3, "y {}", f.y);
        assert_eq!((f.vx, f.vy), (0.0, 0.0), "velocity zeroed");
        assert_eq!(f.rotation, (-100.0f32).atan2(0.0) + 1.57);
        // num48 == 0: no sprite flip (the C#'s strict </> tests).
        assert_eq!(f.sprite_dir, -1, "the SetDefaults default stands");
        // A missing leader keeps the state (the try/catch swallow).
        let mut g = f.clone();
        wyvern_follower_step(&mut g, None);
        assert_eq!(g.x, f.x);
        assert_eq!(g.rotation, f.rotation);
    }

    /// The follower's step-sign sprite flip (`:51911-51921`): a step
    /// toward -X sets spriteDirection 1, toward +X sets -1.
    #[test]
    fn the_follower_flips_with_the_step_sign() {
        let leader = head(1600.0, 400.0); // centre (1616, 416)
        // Follower to the RIGHT at (1800, 400): centre (1816, 416),
        // delta (-200, 0) → step -X → sprite_dir 1.
        let mut f = Npc::new(NpcKind::WyvernLegs, 1800.0, 400.0, 1, 0.0);
        wyvern_follower_step(&mut f, Some(&leader));
        assert_eq!(f.sprite_dir, 1);
        // To the LEFT: delta (+200, 0) → sprite_dir -1.
        let mut g = Npc::new(NpcKind::WyvernLegs, 1400.0, 400.0, 1, 0.0);
        wyvern_follower_step(&mut g, Some(&leader));
        assert_eq!(g.sprite_dir, -1);
    }

    /// `NPC.cs:51616-51626` — the head's velocity-sign sprite flip runs
    /// before the steer: vx < 0 → 1, vx > 0 → -1, vx == 0 keeps.
    #[test]
    fn the_head_flips_with_the_velocity_sign() {
        let w = air();
        let players = [cand(2400.0, 400.0)];
        let mut n = head(1600.0, 400.0);
        n.vx = -2.0;
        wyvern_head_step(&mut n, &w, &players, &ctx(), 0);
        assert_eq!(n.sprite_dir, 1);
        let mut m = head(1600.0, 400.0);
        m.vx = 2.0;
        wyvern_head_step(&mut m, &w, &players, &ctx(), 0);
        assert_eq!(m.sprite_dir, -1);
    }

    /// `NPC.cs:51141-51148` — a dead target sets EncourageDespawn(300)
    /// and the climb gate fires (the dead term); no flag sink (87 has
    /// no surface-gate case, `:51112-51128`).
    #[test]
    fn the_dead_target_encourages_despawn_and_climbs() {
        let w = air();
        let mut p = cand(2400.0, 400.0);
        p.dead = true;
        let mut n = head(1600.0, 400.0);
        n.vx = 6.0;
        n.vy = 3.0;
        wyvern_head_step(&mut n, &w, &[p], &ctx(), 0);
        assert_eq!(n.despawn_timer, 300);
        assert_eq!(n.vy, 3.0 - 0.25, "the dead-target climb");
    }

    /// The chain request (`:51156`): ai[0] == 0 under netMode != 1 asks
    /// for the assembly; a linked head (ai[0] != 0) or a client
    /// (netMode == 1) does not.
    #[test]
    fn the_chain_request_gates_on_ai0_and_netmode() {
        let w = air();
        let players = [cand(2400.0, 400.0)];
        let mut n = head(1600.0, 400.0);
        let ev = wyvern_head_step(&mut n, &w, &players, &ctx(), 0);
        assert!(ev.wants_chain);
        n.ai[0] = 3.0;
        let ev = wyvern_head_step(&mut n, &w, &players, &ctx(), 0);
        assert!(!ev.wants_chain);
        n.ai[0] = 0.0;
        let client = WyvernCtx { net_mode: 1, world_surface: 50.0 };
        let ev = wyvern_head_step(&mut n, &w, &players, &client, 0);
        assert!(!ev.wants_chain);
    }

    /// `NPC.cs:88909-88933` — the HitEffect plan: the hit half counts
    /// the double-precision dmg/lifeMax*50 loop; the death half is the
    /// flat 10 dusts (the gore count is the wiring's Next(1,4) draw).
    #[test]
    fn the_hit_dust_plan_counts_the_csharp_loop() {
        // 100 dmg / 4000 * 50 = 1.25 → 2 iterations (0, 1 < 1.25).
        assert_eq!(wyvern_hit_dust_plan(4000, 100.0, 4000), WyvernHitDust::Hit { count: 2 });
        // 80/4000*50 = 1.0 → 1 iteration (0 < 1, 1 !< 1).
        assert_eq!(wyvern_hit_dust_plan(1, 80.0, 4000), WyvernHitDust::Hit { count: 1 });
        // The death half: life <= 0.
        assert_eq!(
            wyvern_hit_dust_plan(0, 100.0, 4000),
            WyvernHitDust::Death { dust: 10, gores: 0 }
        );
    }

    /// The RNG witness (module docs): the head step, the follower step
    /// and the assembly draw NOTHING — a seeded `UnifiedRandom`'s full
    /// state is bit-identical across 120 ticks of a full chain, so no
    /// `UpdateNpcs` stream can shift when the wiring lands.
    #[test]
    fn the_wyvern_machine_draws_no_rng() {
        let w = air();
        let rng = UnifiedRandom::new(12345);
        let witness = rng.clone();
        let mut npcs = vec![head(1600.0, 400.0)];
        let mut id = 1u32;
        assemble_wyvern_chain(&mut npcs, 0, &mut id);
        let players = [cand(2400.0, 400.0)];
        for tick in 0..120u64 {
            let _ = wyvern_head_step(&mut npcs[0], &w, &players, &ctx(), tick);
            for i in 1..npcs.len() {
                let (leader, rest) = npcs.split_at_mut(i);
                wyvern_follower_step(&mut rest[0], Some(&leader[i - 1]));
            }
            let _ = wyvern_hit_dust_plan(npcs[0].hp, 80.0, 4000);
        }
        assert_eq!(rng, witness, "no Next/NextFloat anywhere in the machine");
    }

    /// The protocol round-trip: the six kinds encode/decode through the
    /// 213-218 port band (the vanilla 87-92 collide with the
    /// Scutlix/saucer ids), and the bestiary folds the segments onto the
    /// head's single entry (net 87, the Sky tag's credit).
    #[test]
    fn the_wyvern_kinds_round_trip_and_fold_their_bestiary() {
        let pairs = [
            (NpcKind::WyvernHead, 213u16),
            (NpcKind::WyvernLegs, 214),
            (NpcKind::WyvernBody, 215),
            (NpcKind::WyvernBody2, 216),
            (NpcKind::WyvernBody3, 217),
            (NpcKind::WyvernTail, 218),
        ];
        for (k, id) in pairs {
            assert_eq!(k.to_u16(), id, "{k:?} protocol id");
            assert_eq!(NpcKind::from_u16(id), Some(k));
            let e = crate::bestiary::entry_for_npc(k, crate::npc::SlimeVariant::Blue);
            assert_eq!(e.net_id, 87, "{k:?} folds onto the head's entry");
            assert_eq!(e.credit_id, "WyvernHead");
        }
        // The banner chain: every kind's net id 87 → banner 86 → item
        // 1700 (the Wyvern Banner, `BannerSystem.cs:638-644`/`:611-612`).
        assert_eq!(tr_core::bestiary::npc_to_banner(87), 86);
        assert_eq!(tr_core::bestiary::banner_to_item(86), 1700);
        // The debuff immunity: On Fire! (31) for all six
        // (`NPCID.cs:6600-6641`).
        for (k, _) in pairs {
            let b = crate::npc_buffs::NpcBuffs::for_kind(k);
            assert!(b.buff_immune[31], "{k:?} On Fire! immunity");
        }
    }
}
