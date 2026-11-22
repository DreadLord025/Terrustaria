//! The bug net (S1-P-i36): `Player.ItemCheck`'s net arm — the swung-item
//! catch rectangle, the catchable-NPC list, the critter→item conversion, the
//! golden/lavaproof variants' tile cut, and the captured-critter release.
//!
//! C# chain (all citations 1.4.5.8):
//! * `Player.cs:43648-43686` — the melee-hitbox block admits the three nets
//!   (1991 Bug Net, 3183 Golden Bug Net, 4821 Fireproof Bug Net) even though
//!   their `damage` is −1; `ItemCheck_CatchCritters` runs for the local
//!   player (`:43654-43657`), the golden/fireproof pair additionally cuts
//!   tiles with `cutExtraTiles: true` (`:43658-43661`), and the
//!   `damage > 0` gate (`:43662`) keeps the nets OUT of `MeleeHitNPCs`.
//! * `Player.ItemCheck_GetMeleeHitbox` (`Player.cs:45099-45190`) over the
//!   useStyle-1 `itemLocation` thirds (`Player.cs:50715-50852`) — the catch
//!   radius is the swung item's frame rectangle, NOT a cursor circle.
//! * `Player.ItemCheck_CatchCritters` (`Player.cs:44712-44740`) — per-NPC
//!   verdicts in slot order; the plain net burns on lava bait
//!   (`ItemID.Sets.IsLavaBait`, `ItemID.cs:1306`), the 583/584/585
//!   firefly family refuses `ai[2] > 1` (a released, still-glowing critter).
//! * `NPC.CatchNPC` (`NPC.cs:79891-79927`) — the conversion: client `70`,
//!   mystic frog 687 teleport, statue poof, else `Item.NewItem` of
//!   `NPC.catchItem` at the player's centre under
//!   `Item.DefaultAssignNewItemsToPlayer` (`Item.cs:49634-49637`,
//!   `:49753-49765` → `ReserveForLocalPlayer`).
//! * The `catchItem` rows: `NPC.SetDefaults` (per-type cites at
//!   [`catch_item`]); the butterfly 356 re-derives it from `ai[2]` every
//!   AI tick (`NPC.cs:45104-45107`).
//! * `Main.npcCatchable` (`Main.cs:9326-9418`) — the RELEASE whitelist
//!   (distinct from `catchItem > 0`: the bunny variants 303/337/540, the
//!   walker goldfish 230 and the mystic frog 687 carry a catch item but
//!   are not release-catchable).
//! * `Player.ItemCheck_ReleaseCritter` (`Player.cs:44057-44074`) +
//!   `NPC.ReleaseNPC` (`NPC.cs:80106-80167`) + `NPC.CanReleaseNPCs`
//!   (`NPC.cs:80070-80104`) — the makeNPC items' LMB use; the consumable
//!   stack drains on the release tick (`Player.cs:43280-43322`,
//!   `itemTime == itemTimeMax`).
//! * `NPC.SubAI_HandleTemporaryCatchableNPCPlayerInvulnerability`
//!   (`NPC.cs:90534-90556`) + the `SetDefaults` arm (`NPC.cs:17851-17855`):
//!   a wild catchable NPC is `friendly` for its first 90 ticks; a released
//!   or statue-spawned one never is.
//!
//! RNG order (the `UpdatePlayers` swap, `Main.cs:17754` — `ItemCheck` runs
//! under it): `ItemCheck_GetMeleeHitboxParticles` (`Player.cs:45048-45097`,
//! only the 1450 Drill Containment Unit / 3779 Zenith arms — neither is a
//! net) and `ItemCheck_EmitUseVisuals` (`Player.cs:44754-45046`) consume
//! ZERO draws for the nets (no type arm matches; the frostBurn/magmaStone/
//! meleeEnchant tails all gate on `sItem.melee`, false for the nets), then
//! per caught NPC in slot order the `Hurt` dodge rolls (only when a dodge
//! accessory is armed — `Player.cs:38214-38236`) and the `Item.NewItem`
//! draws (`Item.cs:49730` `Prefix(0)` — critter items are not prefixable,
//! `CanHavePrefixes` draws nothing — then the velocity pair `:49749-49750`:
//! X = `Next(-30,31)*0.1`, Y = `Next(-40,-15)*0.1`; the port's
//! `spawn_item_drop_px` seam keeps its pinned velocity/phase-triplet draw
//! order, the recorded deviation), then the golden/fireproof tile cut's
//! `KillTile` draws (the port's `break_tile` seam, its own pinned order).
//! The release's 148 penguin variant roll (`NPC.cs:80133`, `Next(2)`) draws
//! on the same stream. The statue-poof/frog `Utils.PoofOfSmoke` draws
//! (`Utils.cs:1078-1093`) are skipped with the standing gore/dust gap.
//!
//! Deviations (docs/ANALYSIS.md "The bug net (S1-P-i36)"): the gore/dust
//! `Utils.PoofOfSmoke` visuals are skipped (the standing gore/dust gap, the
//! statue-poof and frog arms lose their draws with it); the mystic frog's
//! `AI_AttemptToFindTeleportSpot` is unported (kind 687 has no roster row);
//! the `WrongBugNet` immunity counter collapses onto the port's single
//! damage lane (the M4-HAZARD-1 admission); `gravDir` is the normal 1f;
//! the catch item's drop box is the port's universal 14×14 (`ItemDrop`)
//! centred on the spawn point instead of the per-texture `WorldItem`
//! box; the catcher reservation (`Item.DefaultAssignNewItemsToPlayer`)
//! has no host-side `22 ItemOwner` sender yet; the release reach gate is
//! the port's circular `IsInTileInteractionRange` stand-in.

use glam::Vec2;
use tr_core::{TILE_PX, UnifiedRandom};

use crate::app::App;
use crate::npc::Npc;
use crate::registry;

/// The three nets' `Item.SetDefaults` rows (`Item.cs:21305-21315` (1991),
/// `:29541-29553` (3183), `:39480-39492` (4821)) plus the measured
/// `Item_<id>.xnb` frame sizes — `Item.GetDrawHitbox` (`Item.cs:49595-49632`)
/// returns the whole texture for the nets (no `Main.itemAnimations` row is
/// registered for them, `Main.cs:3634-3680`), so the frame IS the sheet.
/// Measured with the repo's LZX harness (`tools/atlas_sim.js::sheetDims`):
/// Item_1991 36×40, Item_3183 44×48, Item_4821 48×50.
pub(crate) struct NetRow {
    pub kind: u16,
    /// `Item.useTime` == `Item.useAnimation` for all three (25/18/21).
    pub use_ticks: i32,
    /// `Item.scale` — 1f / 1.15f / 0.85f. The nets are not `melee`, so
    /// `GetAdjustedItemScale` (`Player.cs:42566-42574`) applies no glove.
    pub scale: f32,
    pub frame_w: i32,
    pub frame_h: i32,
    /// `Player.cs:43658` — only the golden/fireproof pair cuts tiles
    /// (`cutExtraTiles: true`).
    pub cuts_tiles: bool,
}

pub(crate) const NET_ROWS: [NetRow; 3] = [
    NetRow { kind: 1991, use_ticks: 25, scale: 1.0, frame_w: 36, frame_h: 40, cuts_tiles: false },
    NetRow { kind: 3183, use_ticks: 18, scale: 1.15, frame_w: 44, frame_h: 48, cuts_tiles: true },
    NetRow { kind: 4821, use_ticks: 21, scale: 0.85, frame_w: 48, frame_h: 50, cuts_tiles: true },
];

pub(crate) fn net_row(kind: u16) -> Option<&'static NetRow> {
    NET_ROWS.iter().find(|r| r.kind == kind)
}

pub(crate) fn is_bug_net(kind: u16) -> bool {
    net_row(kind).is_some()
}

/// `ItemID.Sets.IsLavaBait` (`ItemID.cs:1306`): the three underworld
/// critters' items — the plain net burns on them (`Player.cs:44721-44727`).
pub(crate) fn is_lava_bait(item: i32) -> bool {
    matches!(item, 4845 | 4847 | 4849)
}

/// `NPC.catchItem` — the `SetDefaults` rows, verbatim. 0 = not catchable
/// (the `SetDefaults` reset, `NPC.cs:8446`). The butterfly 356 row reads
/// `ai[2]` (the colour style): `NPC.cs:45104-45107` re-derives
/// `(short)(1994f + ai[2] - 1f)` every AI tick, so the catch converts the
/// LIVE style, not the spawn default.
pub(crate) fn catch_item(npc_type: i32, ai2: f32) -> i32 {
    match npc_type {
        // `NPC.cs:9336-9347` — the bunny family (46 + the statue/variant
        // forms 303/337/540) all convert to the Bunny item 2019.
        46 | 303 | 337 | 540 => 2019,
        55 => 261,   // Goldfish (`NPC.cs:9460-9473`)
        230 => 261,  // the walker goldfish (`NPC.cs:11977`)
        74 => 2015,  // Bird (`NPC.cs:9760`)
        297 => 2016, // Blue Jay (`NPC.cs:9764`)
        298 => 2017, // Cardinal (`NPC.cs:9768`)
        299 => 2018, // Squirrel (`NPC.cs:12874`)
        300 => 2003, // Bunny (the `NPC.cs:12887` row)
        148 | 149 => 2205, // Penguin pair (`NPC.cs:10871`/`:10883`)
        355 => 1992, // Firefly (`NPC.cs:13529`)
        // Butterfly (`NPC.cs:13543` + the per-tick `:45106` re-derive).
        356 => 1994 + ai2 as i32 - 1,
        357 => 2002, // Worm (`NPC.cs:13556`)
        358 => 2004, // Lightning Bug (`NPC.cs:13571`)
        359 => 2006, // Snail (`NPC.cs:13585`)
        360 => 2007, // (`NPC.cs:13599`)
        361 => 2121, // (`NPC.cs:13611`)
        362 | 363 => 2122, // (`NPC.cs:13623`/`:13635`)
        364 | 365 => 2123, // (`NPC.cs:13647`/`:13659`)
        // Scorpion pair (`NPC.cs:13671`): `2156 + type - 366`.
        366 | 367 => 2156 + npc_type - 366,
        374 | 375 => 2673, // (`NPC.cs:13773`/`:13792`)
        377 => 2740,       // (`NPC.cs:13822`)
        // The gold critter row 442..448 (`NPC.cs:14258-14343`).
        442..=448 => 2889 + npc_type - 442,
        // Grubby/Sluggy/Buggy 484..487 (`NPC.cs:14587`): `3191 + type - 484`.
        484..=487 => 3191 + npc_type - 484,
        538 => 3563, // (`NPC.cs:15883`)
        539 => 3564, // (`NPC.cs:15895`)
        // The 583/584/585 firefly family (`NPC.cs:16583-16607`).
        583 => 4068,
        584 => 4069,
        585 => 4070,
        592 | 593 => 4274, // (`NPC.cs:16691`/`:16704`)
        // Dragonflies 595..601 (`NPC.cs:16720-16738`): `type - 595 + 4334`.
        595..=601 => npc_type - 595 + 4334,
        602 | 603 => 4359, // (`NPC.cs:16749`/`:16761`)
        // 604/605 (`NPC.cs:16779`): `type - 604 + 4361`.
        604 | 605 => npc_type - 604 + 4361,
        606 => 4363, // (`NPC.cs:16792`)
        607 => 4373, // (`NPC.cs:16807`)
        608 | 609 => 4374, // (`NPC.cs:16819`/`:16831`)
        610 => 4375,       // (`NPC.cs:16844`)
        611 => 4395,       // Owl (`NPC.cs:16857`)
        // 612/613 (`NPC.cs:16875`): `type - 612 + 4418`.
        612 | 613 => npc_type - 612 + 4418,
        614 => 1338, // Explosive Bunny (`NPC.cs:16890`)
        // 616/617 (`NPC.cs:16916`): `type - 616 + 4464`.
        616 | 617 => npc_type - 616 + 4464,
        626 => 4480, // (`NPC.cs:17066`)
        627 => 4482, // the gold form's overwrite (`NPC.cs:17069`)
        // 639..645 (`NPC.cs:17236`): `4831 + (type - 639)`.
        639..=645 => 4831 + npc_type - 639,
        // 646..652 (`NPC.cs:17248`): `4838 + (type - 646)`.
        646..=652 => 4838 + npc_type - 646,
        653 => 4845, // Hell Butterfly (`NPC.cs:17263`) — lava bait
        654 => 4847, // (`NPC.cs:17278`) — lava bait
        655 => 4849, // (`NPC.cs:17293`) — lava bait
        661 => 4961, // (`NPC.cs:17364`)
        669 => 5132, // (`NPC.cs:17479`)
        671 => 5212, // (`NPC.cs:17507`)
        672 => 5300, // (`NPC.cs:17521`)
        673 => 5311, // (`NPC.cs:17535`)
        674 => 5312, // (`NPC.cs:17549`)
        675 => 5313, // (`NPC.cs:17563`)
        677 => 5350, // (`NPC.cs:17591`)
        687 => 2121, // Mystic Frog (`NPC.cs:17651`)
        688 => 5511, // (`NPC.cs:17665`)
        _ => 0,
    }
}

/// `Main.npcCatchable` (`Main.cs:9326-9418`) — the release whitelist.
/// NOTE the asymmetry with [`catch_item`]: 303/337/540/230/687 carry a
/// catch item but are NOT in this set (the C# lists every id explicitly;
/// the 442..=448 loop is `:9415-9418`).
pub(crate) fn npc_catchable(npc_type: i32) -> bool {
    matches!(
        npc_type,
        46 | 55
            | 74
            | 148
            | 149
            | 297
            | 298
            | 299
            | 300
            | 355
            | 356
            | 357
            | 358
            | 359
            | 360
            | 361
            | 362
            | 363
            | 364
            | 365
            | 366
            | 367
            | 374
            | 377
            | 539
            | 538
            | 671
            | 672
            | 673
            | 674
            | 675
            | 484
            | 485
            | 486
            | 487
            | 583
            | 584
            | 585
            | 592
            | 593
            | 595
            | 596
            | 597
            | 598
            | 599
            | 600
            | 601
            | 604
            | 605
            | 602
            | 603
            | 606
            | 607
            | 608
            | 609
            | 610
            | 611
            | 612
            | 613
            | 614
            | 616
            | 617
            | 626
            | 627
            | 639..=655
            | 661
            | 669
            | 677
            | 688
            | 442..=448
    )
}

/// XNA `Rectangle.Intersects` — half-open on both axes.
fn intersects(ax: i32, ay: i32, aw: i32, ah: i32, bx: i32, by: i32, bw: i32, bh: i32) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

/// `Player.ItemCheck_GetMeleeHitbox` (`Player.cs:45099-45190`) for a
/// useStyle-1 net, over the generic `itemLocation` thirds
/// (`Player.cs:50715-50852` — the nets are neither `Item.claw`
/// (`Main.cs:6793-6795`: only 1827) nor `UsesBetterMeleeItemLocation`
/// (`ItemID.cs:348`: only 426)). `anim`/`max` are the integer
/// `itemAnimation`/`itemAnimationMax` tick counts; `px`/`py` the player's
/// `base.position`; `dir` ±1; `gravDir` folds to 1 (no flipped gravity in
/// the port). Returns `(dontAttack, x, y, w, h)` — `dontAttack` is only
/// ever set by the useStyle-3 arm (`:45168-45172`), never for the nets.
pub(crate) fn net_swing_rect(
    row: &NetRow,
    scale: f32,
    anim: i32,
    max: i32,
    px: f32,
    py: f32,
    dir: i32,
) -> (bool, i32, i32, i32, i32) {
    let fw = row.frame_w as f32;
    let fh = row.frame_h as f32;
    let d = dir as f32;
    let d_anim = anim as f64;
    let d_max = max as f64;
    // The itemLocation thirds. The C# cascades run on the FRAME size; the
    // net frames (36/44/48 × 40/48/50) reach the `> 32` bands, and 4821's
    // 48-wide frame reaches the start-third `>= 48` band (`:50805-50808`).
    let (loc_x, loc_y) = if d_anim < d_max * 0.333 {
        // Swing END (`:50715-50744`): num 10 / >32:14 / >=52:24 / >=64:28 /
        // >=92:38; Y = position.Y + 24.
        let mut num = if fw > 32.0 { 14.0 } else { 10.0 };
        if fw >= 52.0 {
            num = 24.0;
        }
        if fw >= 64.0 {
            num = 28.0;
        }
        if fw >= 92.0 {
            num = 38.0;
        }
        (px + 10.0 + (fw * 0.5 - num) * d, py + 24.0)
    } else if d_anim < d_max * 0.666 {
        // Middle third (`:50746-50796`): num2 10 / >32:18 / >=52:24 /
        // >=64:28 / >=92:38; Y num2 10 / >32:8 / >52:12 / >64:14.
        let mut num = if fw > 32.0 { 18.0 } else { 10.0 };
        if fw >= 52.0 {
            num = 24.0;
        }
        if fw >= 64.0 {
            num = 28.0;
        }
        if fw >= 92.0 {
            num = 38.0;
        }
        let num_y = if fh > 64.0 {
            14.0
        } else if fh > 52.0 {
            12.0
        } else if fh > 32.0 {
            8.0
        } else {
            10.0
        };
        (px + 10.0 + (fw * 0.5 - num) * d, py + num_y)
    } else {
        // Swing START (`:50798-50852`): num3 6 / >32:14 / >=48:18 /
        // >=52:24 / >=64:28 / >=92:38, SUBTRACTED; Y num3 10 / >32:10 /
        // >52:12 / >64:14.
        let mut num = if fw > 32.0 { 14.0 } else { 6.0 };
        if fw >= 48.0 {
            num = 18.0;
        }
        if fw >= 52.0 {
            num = 24.0;
        }
        if fw >= 64.0 {
            num = 28.0;
        }
        if fw >= 92.0 {
            num = 38.0;
        }
        let num_y = if fh > 64.0 {
            14.0
        } else if fh > 52.0 {
            12.0
        } else {
            10.0
        };
        (px + 10.0 - (fw * 0.5 - num) * d, py + num_y)
    };
    // `:45102`/`:45126`: the rect anchors at the TRUNCATED itemLocation,
    // sized by the frame; `:45128-45130` scales by GetAdjustedItemScale
    // (the instance scale — the prefix's size multiplier rides it).
    let mut x = loc_x as i32;
    let mut y = loc_y as i32;
    let mut w = (row.frame_w as f32 * scale) as i32;
    let mut h = (row.frame_h as f32 * scale) as i32;
    // `:45131-45138`: the direction flip and the gravDir 1 lift.
    if dir == -1 {
        x -= w;
    }
    y -= h;
    // `:45139-45160`: the useStyle-1 thirds over the SCALED box. The C#
    // multiplies in double (`(double)Width * 1.4`).
    if d_anim < d_max * 0.333 {
        if dir == -1 {
            x -= (w as f64 * 1.4 - w as f64) as i32;
        }
        w = (w as f64 * 1.4) as i32;
        y += (h as f64 * 0.5) as i32;
        h = (h as f64 * 1.1) as i32;
    } else if !(d_anim < d_max * 0.666) {
        if dir == 1 {
            x -= (w as f64 * 1.2) as i32;
        }
        w *= 2;
        y -= (h as f64 * 1.4 - h as f64) as i32;
        h = (h as f64 * 1.4) as i32;
    }
    (false, x, y, w, h)
}

/// One NPC's verdict inside `ItemCheck_CatchCritters`
/// (`Player.cs:44712-44740`), in the C#'s branch order. The caller has
/// already filtered `active && catchItem > 0 && rect.Intersects`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CatchVerdict {
    /// The plain net on lava bait: `Hurt(ByNPC, 1, dir)` then buff 24/300
    /// when the hit lands and the player lives (`Player.cs:44721-44727`).
    /// `hit_dir` is `(npc.Center.X < player.Center.X) ? 1 : -1`.
    LavaHurt { hit_dir: i32 },
    /// `NPC.CatchNPC(i)` (`Player.cs:44732`/`:44737`).
    Catch,
    /// The 583/584/585 family with `ai[2] > 1` — a released, still-glowing
    /// critter refuses the net (`Player.cs:44728-44734`).
    Skip,
}

pub(crate) fn catch_verdict(
    net_kind: u16,
    catch_item_id: i32,
    npc_type: i32,
    ai2: f32,
    npc_cx: f32,
    player_cx: f32,
) -> CatchVerdict {
    // `:44714` — the golden/fireproof flag.
    let lavaproof = net_kind == 3183 || net_kind == 4821;
    if !lavaproof && is_lava_bait(catch_item_id) {
        CatchVerdict::LavaHurt {
            hit_dir: if npc_cx < player_cx { 1 } else { -1 },
        }
    } else if npc_type == 585 || npc_type == 583 || npc_type == 584 {
        // `:44730` — `ai[2] <= 1f` catches, else nothing.
        if ai2 <= 1.0 {
            CatchVerdict::Catch
        } else {
            CatchVerdict::Skip
        }
    } else {
        CatchVerdict::Catch
    }
}

/// `NPC.CatchNPC`'s branch decision (`NPC.cs:79891-79927`), free of the
/// world so the order is unit-testable. `net_mode`: 0 SP, 1 client, 2 host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CatchOutcome {
    /// `!active || catchItem <= 0` (`NPC.cs:79893-79896`).
    Nothing,
    /// `netMode == 1`: deactivate + `70` (`NPC.cs:79901-79905`) — the
    /// client spawns no item; the host's `CatchNPC(i, who)` converts.
    ClientSend,
    /// Type 687: `TryTeleportingCaughtMysticFrog` (`NPC.cs:79906-79909`,
    /// `:79864-79889`) — teleport on a found spot, else the statue-style
    /// poof. The port has no kind 687 (roster gap), so this arm is coded
    /// for parity and unreachable today.
    MysticFrog,
    /// `SpawnedFromStatue`: poof + deactivate, NO item
    /// (`NPC.cs:79910-79917`).
    StatuePoof,
    /// The conversion: `Item.NewItem(catchItem)` at the player's centre
    /// under `DefaultAssignNewItemsToPlayer(who)` (`NPC.cs:79918-79926`).
    Convert,
}

pub(crate) fn catch_outcome(
    net_mode: u8,
    active: bool,
    catch_item_id: i32,
    npc_type: i32,
    spawned_from_statue: bool,
) -> CatchOutcome {
    if !active || catch_item_id <= 0 {
        return CatchOutcome::Nothing;
    }
    if net_mode == 1 {
        return CatchOutcome::ClientSend;
    }
    if npc_type == 687 {
        return CatchOutcome::MysticFrog;
    }
    if spawned_from_statue {
        return CatchOutcome::StatuePoof;
    }
    CatchOutcome::Convert
}

/// `NPC.CanReleaseNPCs` (`NPC.cs:80070-80104`): the 0.7 (client) / 0.75
/// (SP/host) population fraction, the per-player share
/// `(int)(maxNPCs * num / activePlayers)`, and the two strict `<` gates.
/// `Main.maxNPCs = 200` (`InitData.cs:9`).
pub(crate) fn can_release_npcs(
    net_mode: u8,
    live_npcs: i32,
    owned_by_who: i32,
    active_players: i32,
) -> bool {
    let mut num = 0.7f32;
    if net_mode != 1 {
        num += 0.05;
    }
    // `:80098` — the integer truncation of the per-player share.
    let per_player = (200.0 * num / active_players.max(1) as f32) as i32;
    (live_npcs as f32) < 200.0 * num && owned_by_who < per_player
}

/// `NPC.ReleaseNPC`'s spawn-type roll (`NPC.cs:80115-80164`): the switch
/// over the released type. Returns the type to spawn (the 148 penguin
/// variant draws ONE `Next(2)` — `NPC.cs:80133`) and the post-spawn writes
/// the caller applies. Free of the world; the RNG comes in for the 148 arm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ReleasePlan {
    /// The `NewNPC` type after the switch (`-1` = the gate refused).
    pub spawn_type: i32,
    /// `ai[2]` write: the 583/584/585 family sets 2 (`NPC.cs:80122`), the
    /// butterfly 356 sets the item's `placeStyle` (`:80129`).
    pub ai2: Option<f32>,
    /// The 583/584/585 family runs `TargetClosest()` then `ai[3] = 0`
    /// (`NPC.cs:80123-80124`).
    pub target_closest: bool,
    /// The 614 explosive bunny's direction override (`NPC.cs:80140-80156`):
    /// `Some(dir)` writes direction + spriteDirection.
    pub direction: Option<i32>,
    /// 614 with `Style > 2`: `SpawnedFromStatue` + `CanBeReplacedByOtherNPCs`
    /// (`NPC.cs:80141-80146`).
    pub statue_form: bool,
}

pub(crate) fn release_plan(
    npc_type: i32,
    style: i32,
    player_dir: i32,
    rng: &mut UnifiedRandom,
) -> ReleasePlan {
    let mut plan = ReleasePlan {
        spawn_type: npc_type,
        ai2: None,
        target_closest: false,
        direction: None,
        statue_form: false,
    };
    match npc_type {
        583 | 584 | 585 => {
            // `NPC.cs:80117-80126`.
            plan.ai2 = Some(2.0);
            plan.target_closest = true;
        }
        356 => {
            // `NPC.cs:80127-80131` — the butterfly keeps the item's colour.
            plan.ai2 = Some(style as f32);
        }
        148 => {
            // `NPC.cs:80132-80135` — `Type + Main.rand.Next(2)`.
            plan.spawn_type = npc_type + rng.next_range(0, 2);
        }
        614 => {
            // `NPC.cs:80136-80159`.
            let mut style = style;
            let mut dir = player_dir;
            if style > 2 {
                style -= 2;
                plan.statue_form = true;
            }
            if style == 1 {
                dir = 1;
            }
            if style == 2 {
                dir = -1;
            }
            plan.direction = Some(dir);
        }
        _ => {} // `NPC.cs:80160-80163` — the default arm: spawn + owner only.
    }
    plan
}

/// `Player.ItemCheck_ReleaseCritter`'s position decision
/// (`Player.cs:44057-44074`): the explosive bunny 614 releases at the
/// player's `(Center.X, Bottom.Y)` with NO reach/solid gate; every other
/// makeNPC item needs the tile-interaction reach AND a non-solid cursor
/// cell, and releases at the truncated mouse world point.
/// `in_reach` is the caller's `IsInTileInteractionRange` verdict (the
/// port's circular reach stand-in — `editing.rs`'s documented gate).
pub(crate) fn release_position(
    make_npc: i32,
    in_reach: bool,
    cursor_solid: bool,
    player_cx: f32,
    player_bottom: f32,
    mouse: Vec2,
) -> Option<(i32, i32)> {
    if make_npc == 614 {
        return Some((player_cx as i32, player_bottom as i32));
    }
    if in_reach && !cursor_solid {
        return Some((mouse.x as i32, mouse.y as i32));
    }
    None
}

// ---------------------------------------------------------------------------
// The app wiring — the `ItemCheck` net arm inside `update_editing`.
// ---------------------------------------------------------------------------

/// The swing start + per-tick catch pass for the three nets. Called from
/// `update_editing` after the HUD gates, BEFORE the cursor-reach gate —
/// the C# catch arm is player-relative (`Player.cs:43654`), not
/// cursor-gated. Returns `true` when the held item is a net (the caller
/// skips the mining/melee arms — the C# nets never mine, and
/// `Player.cs:43662`'s `damage > 0` gate keeps them out of
/// `MeleeHitNPCs`).
///
/// Order per tick (the C# `ItemCheck` block, `Player.cs:43648-43686`):
/// the catch pass runs on EVERY `itemAnimation > 0` tick (no `itemTime`
/// gate — `:43654`), then the golden/fireproof tile cut. The swing start
/// rides the port's autoReuse cadence (all three nets are `autoReuse`
/// with `useTime == useAnimation`).
pub(crate) fn update_bug_net(app: &mut App) -> bool {
    let held = app.inventory[app.selected];
    let Some(row) = net_row(held.kind) else {
        return false;
    };
    if held.count == 0 {
        return true;
    }
    // The swing start (`Player.cs:42733`'s `controlUseItem &&
    // releaseUseItem && itemAnimation == 0` + the autoReuse restart
    // `:42819-42822` — all three nets are autoReuse, so no release edge
    // is needed).
    if app.mine_down
        && !app.inventory_open
        && app.swing <= 0.0
        && app.attack_cd <= 0.0
        && app.reuse_delay <= 0.0
    {
        let secs = crate::item::use_animation_secs_of(held.kind, held.prefix);
        app.swing_max = secs;
        app.swing = secs;
        crate::world_sim::editing::start_swing_extras(app, held.kind);
        app.attack_cd = secs;
    }
    if app.swing <= 0.0 {
        return true;
    }
    // The integer tick counts the thirds compare (`Player.cs:45141`).
    let anim = (app.swing * 60.0).round() as i32;
    let max = (app.swing_max * 60.0).round() as i32;
    let scale = crate::prefix::prefixed_stats(held.kind, held.prefix)
        .map_or(row.scale, |s| s.scale);
    let Some(p) = app.player.as_ref() else {
        return true;
    };
    let (pcx, _pcy) = p.center();
    let dir = p.facing;
    let (px, py) = (p.x, p.y);
    let (_, rx, ry, rw, rh) = net_swing_rect(row, scale, anim, max, px, py, dir);
    // `ItemCheck_CatchCritters` (`Player.cs:44715-44739`): the slot-order
    // scan. The verdicts are collected first (the player borrow above is
    // immutable; the applies below mutate).
    let mut actions: Vec<(usize, CatchVerdict)> = Vec::new();
    for (i, n) in app.npcs.iter().enumerate() {
        if n.hp <= 0 {
            continue;
        }
        let ty = n.kind.cs_id() as i32;
        let ci = catch_item(ty, n.ai[2]);
        if ci <= 0 {
            continue;
        }
        let (nw, nh) = n.size();
        if !intersects(rx, ry, rw, rh, n.x as i32, n.y as i32, nw as i32, nh as i32) {
            continue;
        }
        let (ncx, _) = n.center();
        actions.push((i, catch_verdict(held.kind, ci, ty, n.ai[2], ncx, pcx)));
    }
    // Apply in slot order. Removals collect indices; the item spawns ride
    // the `UpdatePlayers` stream (`Main.cs:17754`).
    let mut removed: Vec<usize> = Vec::new();
    for (i, verdict) in actions {
        match verdict {
            CatchVerdict::Skip => {}
            CatchVerdict::LavaHurt { hit_dir } => {
                // `Player.cs:44723-44726`: `Hurt(ByNPC(i), 1, hitDir,
                // cooldownCounter: WrongBugNet) > 0 && !dead` → buff 24
                // (On Fire!) 300 ticks. The port's single damage lane
                // stands in for the WrongBugNet counter (ANALYSIS);
                // WrongBugNet is NOT in `Sets.Retaliate`
                // (`ImmunityCooldownID.cs:49`), so the retaliate lane
                // stays off — the lava lane's precedent
                // (`player.rs`'s `hurt_inner(..., false)`). The dodge
                // rolls ride the `UpdatePlayers` stream like every other
                // hazard hurt. `hit_dir` signs the knockback side: the
                // C# `hitDirection` pushes AWAY from the NPC, so the
                // `from_x` anchor sits on the NPC's side.
                let from_x = if hit_dir > 0 {
                    px - 1.0
                } else {
                    px + crate::player::WIDTH + 1.0
                };
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                let landed = if let Some(p) = app.player.as_mut() {
                    p.hurt_inner(1, from_x, true, rng, true, false, None, false)
                } else {
                    false
                };
                if landed {
                    if let Some(p) = app.player.as_mut() {
                        if !p.dead {
                            p.buffs.add_buff(24, 300);
                        }
                    }
                }
            }
            CatchVerdict::Catch => {
                // `NPC.CatchNPC(i)` (`Player.cs:44732`/`:44737`) — the
                // local player is the catcher (`who = Main.myPlayer`,
                // `NPC.cs:79897-79900`).
                if catch_npc(app, i, app.local_id as u8) {
                    removed.push(i);
                }
            }
        }
    }
    // Remove back-to-front so the indices stay valid (the C# deactivates
    // in place; the port's Vec compaction is the documented stand-in —
    // the net cadence's uid-keyed gone-diff sends the despawn `23`s).
    removed.sort_unstable();
    removed.dedup();
    for i in removed.into_iter().rev() {
        app.npcs.remove(i);
    }
    // `Player.cs:43658-43661`: the golden/fireproof pair cuts tiles with
    // `cutExtraTiles: true` on every swing tick, AFTER the catch pass.
    if row.cuts_tiles {
        cut_tiles(app, rx, ry, rw, rh);
    }
    true
}

/// `NPC.CatchNPC(i, who)` (`NPC.cs:79891-79927`) — the conversion itself,
/// shared by the local swing pass and the host's `70` handler
/// (`MessageBuffer.cs:3102-3110`). Returns `true` when the NPC
/// deactivates (the caller's Vec compaction / the host's slot removal).
/// The item spawn rides the `UpdatePlayers` stream on the local path;
/// the `70` handler runs under the host's `UpdateServerOrClient` tick —
/// the C# `GetData` runs under the same per-message stream swap
/// (`Main.cs`'s `SwapRandom` around `MessageBuffer.GetData`); the port
/// keeps the one `UpdatePlayers` seam for both (ANALYSIS).
pub(crate) fn catch_npc(app: &mut App, i: usize, who: u8) -> bool {
    let Some(n) = app.npcs.get(i) else {
        return false;
    };
    let ty = n.kind.cs_id() as i32;
    let ci = catch_item(ty, n.ai[2]);
    let net_mode = app_net_mode(app);
    let outcome = catch_outcome(net_mode, n.hp > 0, ci, ty, n.spawned_from_statue);
    match outcome {
        CatchOutcome::Nothing => false,
        CatchOutcome::ClientSend => {
            // `NPC.cs:79901-79905`: deactivate locally + `70`.
            app.send_catch_npc(i);
            true
        }
        CatchOutcome::MysticFrog | CatchOutcome::StatuePoof => {
            // `NPC.cs:79906-79917`: the frog teleports when a spot is
            // found (`AI_AttemptToFindTeleportSpot` is unported — kind
            // 687 has no roster row), else both arms poof + deactivate
            // with NO item. The `Utils.PoofOfSmoke` visuals are the
            // standing gore/dust gap (ANALYSIS); the `23` despawn rides
            // the net cadence's gone-diff, the `106` smoke broadcast has
            // no port message.
            true
        }
        CatchOutcome::Convert => {
            // `NPC.cs:79918-79926`: `Item.NewItem` of `catchItem` at
            // `(int)player[who].Center.X/Y` under
            // `DefaultAssignNewItemsToPlayer(who)` (`Item.cs:49753-49765`
            // → `ReserveForLocalPlayer`). The port's `Item.NewItem` seam
            // keeps its pinned draw order (the velocity/phase triplet,
            // ANALYSIS); critter items are not prefixable, so the
            // `Prefix(0)` arm draws nothing on either side.
            let (cx, cy) = if who == app.local_id as u8 {
                app.player
                    .as_ref()
                    .map(|p| p.center())
                    .unwrap_or((0.0, 0.0))
            } else {
                // The `70` handler's remote catcher: `Main.player[who]`
                // is the net-synced peer (`PlayerNet.x/y` + the 20×42
                // hitbox).
                app.remote_players
                    .get(&(who as u32))
                    .map(|p| {
                        (
                            p.x + crate::player::WIDTH * 0.5,
                            p.y + crate::player::HEIGHT * 0.5,
                        )
                    })
                    .unwrap_or((0.0, 0.0))
            };
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            // The C# `worldItem.Center = center` (`Item.cs:49736`) anchors
            // the drop's CENTRE on the truncated player centre; the port's
            // seam takes the top-left of its 14×14 box (`ItemDrop::SIZE`),
            // so the half-size offset stands in for the per-texture
            // `WorldItem.width/height` (ANALYSIS).
            let half = crate::item::SIZE * 0.5;
            crate::world_sim::items::spawn_item_drop_px(
                &mut app.items,
                rng,
                ci as u16,
                1,
                Vec2::new(cx as i32 as f32 - half, cy as i32 as f32 - half),
            );
            // The host's `21 ItemDrop` sync rides the item cadence's
            // state diff (`netglue::net_tick`); the catcher reservation
            // (`22 ItemOwner`) has no host-side sender yet — the drop is
            // grabbable by anyone after the port's universal pickup
            // delay (ANALYSIS).
            true
        }
    }
}

/// `Main.netMode` as the port's role enum: 0 SP, 1 client, 2 host.
fn app_net_mode(app: &App) -> u8 {
    match app.net {
        crate::app::NetRole::Host(_) => 2,
        crate::app::NetRole::Client(_) => 1,
        crate::app::NetRole::Off => 0,
    }
}

/// `Player.ItemCheck_CutTiles` (`Player.cs:44646-44710`) for the nets:
/// `shouldIgnore` is `TileCutIgnore.None` (the nets are not the Staff of
/// Regrowth 213/5295, and `dontHurtNature` — the Guide to Plant Fiber
/// Cordage 5309/5323 grant, `Player.cs:12829-12832` — has no ported
/// granter, so the flag is always false today; ANALYSIS). The cell walk
/// is `X / 16 .. (X + W) / 16 + 1` clamped to the world
/// (`Utils.ClampWithinWorld`, `Utils.cs:428-435`), the cut gate is
/// `Main.tileCut[type] || (cutExtraTiles && bonusCutTiles[type])`
/// (`TileID.cs:566`: the bonus set is {233}) and `WorldGen.CanCutTile`
/// (`WorldGen.cs:86828-86839`). The 1786 dart arm never fires for the
/// nets. Each cut cell rides the port's full `break_tile` kill (the C#
/// `WorldGen.KillTile(i, j)` — drops, sounds, the `17` net send).
fn cut_tiles(app: &mut App, rx: i32, ry: i32, rw: i32, rh: i32) {
    let min_x = (rx / 16).clamp(0, app.world.width);
    let max_x = ((rx + rw) / 16 + 1).clamp(0, app.world.width);
    let min_y = (ry / 16).clamp(0, app.world.height);
    let max_y = ((ry + rh) / 16 + 1).clamp(0, app.world.height);
    let mut cells = Vec::new();
    for i in min_x..max_x {
        for j in min_y..max_y {
            let Some(t) = app.world.tile(i, j) else { continue };
            if !t.active() {
                continue;
            }
            let cuttable = tr_core::tileobject::tile_cut(t.kind) || t.kind == 233;
            if !cuttable {
                continue;
            }
            if !tr_core::tileobject::can_cut_tile(&app.world, i, j) {
                continue;
            }
            cells.push((i, j));
        }
    }
    for (i, j) in cells {
        crate::world_sim::editing::break_tile(app, i, j);
    }
}

/// The makeNPC release arm — `Player.ItemCheck`'s `sItem.makeNPC > 0`
/// block (`Player.cs:43614-43625`) → `ItemCheck_ReleaseCritter`
/// (`:44057-44074`). Called from `update_editing` after the HUD gates,
/// BEFORE the cursor-reach gate (the C# starts the swing regardless of
/// reach; only the release itself is reach-gated, `:44064`). Returns
/// `true` when the held item releases critters AND LMB is down (the use
/// is consumed — the C# `PlaceTileOnAltUse` items, `ItemID.cs:412`,
/// release on LMB and place their cage/anchor on RMB; the RMB placement
/// keeps riding the generic place path).
///
/// The use gate is `CanUseItem`'s `makeNPC > 0 && !CanReleaseNPCs`
/// refusal (`Player.cs:52116-52119`); the release itself fires at
/// `ItemTimeIsZero && itemAnimation > 0 && controlUseItem &&
/// altFunctionUse == 0` (`:43621` — the port's `!place_down` stands in
/// for the alt-use suppression). The consumable stack drains on the
/// release tick (`Player.cs:43280-43322` — `itemTime == itemTimeMax`
/// right after `ApplyItemTime`).
pub(crate) fn update_critter_release(app: &mut App, world_pos: Vec2) -> bool {
    let held = app.inventory[app.selected];
    let make_npc = registry::make_npc(held.kind) as i32;
    if make_npc <= 0 || held.count == 0 {
        return false;
    }
    if !app.mine_down || app.inventory_open || app.place_down {
        return false;
    }
    // `NPC.CanReleaseNPCs` census: the live pool + this player's releases.
    let who = app.local_id as u8;
    let live = app.npcs.iter().filter(|n| n.hp > 0).count() as i32;
    let owned = app
        .npcs
        .iter()
        .filter(|n| n.hp > 0 && n.release_owner == who)
        .count() as i32;
    let active_players = 1 + app.remote_players.len() as i32;
    let net_mode = app_net_mode(app);
    if !can_release_npcs(net_mode, live, owned, active_players) {
        // The C# refuses the whole use (`CanUseItem` false) — no swing.
        return true;
    }
    if app.swing <= 0.0 && app.attack_cd <= 0.0 && app.reuse_delay <= 0.0 {
        let secs = crate::item::use_animation_secs_of(held.kind, held.prefix);
        app.swing_max = secs;
        app.swing = secs;
        crate::world_sim::editing::start_swing_extras(app, held.kind);
        app.attack_cd = secs;
    }
    if app.swing <= 0.0 || app.item_time > 0.0 {
        return true;
    }
    // `ItemCheck_ReleaseCritter`'s position decision. The reach verdict
    // is the port's circular `IsInTileInteractionRange` stand-in (the
    // `update_editing` gate, `editing.rs`'s documented deviation) over
    // the cursor CELL centre; `tileBoost` is 0 for every critter item.
    let tx = (world_pos.x / TILE_PX).floor() as i32;
    let ty = (world_pos.y / TILE_PX).floor() as i32;
    let in_reach = app.player.as_ref().is_some_and(|p| {
        let (pcx, pcy) = p.center();
        let dx = tx as f32 * TILE_PX + 8.0 - pcx;
        let dy = ty as f32 * TILE_PX + 8.0 - pcy;
        let reach = (p.tile_range_x() + 1.0) * TILE_PX;
        dx * dx + dy * dy <= reach * reach
    });
    let cursor_solid = solid_tile(&app.world, tx, ty);
    let (pcx, pcy) = app
        .player
        .as_ref()
        .map(|p| p.center())
        .unwrap_or((world_pos.x, world_pos.y));
    let bottom = app
        .player
        .as_ref()
        .map(|p| p.y + p.height)
        .unwrap_or(pcy);
    let Some((x, y)) = release_position(make_npc, in_reach, cursor_solid, pcx, bottom, world_pos)
    else {
        return true;
    };
    // `ApplyItemTime` (`Player.cs:44061`/`:44070`) BEFORE the release.
    let use_secs = crate::prefix::prefixed_stats(held.kind, held.prefix)
        .map_or(0.0, |s| s.use_time as f32 / 60.0);
    app.item_time = use_secs;
    if net_mode == 1 {
        // `NPC.cs:80109-80112`: the client only sends `71`.
        app.send_release_npc(x, y, make_npc, registry::place_style(held.kind));
    } else {
        release_npc(app, x, y, make_npc, registry::place_style(held.kind), who);
    }
    // The consumable drain (`Player.cs:43315-43322`): the stack--, and
    // the emptied slot's `itemTime = itemAnimation` tail rides the
    // port's swing counter (the slot airs out).
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
        app.item_time = app.item_time.max(app.swing);
    }
    true
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70133`): an active,
/// fully solid tile — not a solid-top, not a half brick, no slope and
/// not inactive. The out-of-world read is `false` (the C# catch arm) —
/// the same body as `grapple.rs`/`npc.rs`'s private copies (the house
/// style keeps one copy per module until a shared seam lands).
fn solid_tile(world: &tr_core::World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && tr_core::tiledata::TILE_SOLID.has(t.kind)
        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `NPC.ReleaseNPC`'s host/SP body (`NPC.cs:80113-80165`): the catchable
/// gate, the switch's spawn + writes, `releaseOwner`. The port's roster
/// gap: a type with no `NpcKind` spawns nothing (the
/// `spawn_shimmer_release` precedent) — the use accounting still ran.
/// The kind resolves through the bestiary NET id (the C# `NPC.type`
/// space), not the port's internal protocol ids.
pub(crate) fn release_npc(app: &mut App, x: i32, y: i32, ty: i32, style: i32, who: u8) {
    // `NPC.cs:80113` — `Type >= 0 && Type < NPCID.Count &&
    // (npcCatchable[Type] || who == Main.myPlayer)`: the local player
    // bypasses the whitelist in SP/host-local use; a REMOTE release
    // (the `71` handler) must pass it. `NPCID.Count = 697`
    // (`NPCID.cs:12581`).
    if ty < 0 || ty >= 697 {
        return;
    }
    if !npc_catchable(ty) && who != app.local_id as u8 {
        return;
    }
    let live = app.npcs.iter().filter(|n| n.hp > 0).count() as i32;
    let owned = app
        .npcs
        .iter()
        .filter(|n| n.hp > 0 && n.release_owner == who)
        .count() as i32;
    let active_players = 1 + app.remote_players.len() as i32;
    let net_mode = app_net_mode(app);
    if !can_release_npcs(net_mode, live, owned, active_players) {
        return;
    }
    // `NPC.cs:80140` reads `Main.player[who].direction` — the local
    // player on the SP/host-local path, the net-synced peer on `71`.
    let player_dir = if who == app.local_id as u8 {
        app.player.as_ref().map_or(1, |p| p.facing)
    } else {
        app.remote_players
            .get(&(who as u32))
            .map_or(1, |p| p.facing)
    };
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    let plan = release_plan(ty, style, player_dir, rng);
    let Some((kind, slime)) = crate::npc::kind_from_net_id(plan.spawn_type) else {
        // The roster gap: a type with no `NpcKind` spawns nothing (the
        // C# spawns; the port has no model — ANALYSIS). The W4-c4
        // style-16 wave closed the fish holes: 592/607/688 (the
        // GoldGoldfish/Pupfish/Pufferfish) now resolve through their
        // bestiary entries; the remaining gap is the non-fish critters
        // (the birds/insects/rock critters of the d38/d40/d42 lanes).
        // The roster gap (the ported catchable kinds today: the
        // Goldfish 55 + the W4-c8 crawlers 357/374/448/484-487/606 —
        // the c4/c7 waves' kinds land with their rosters) — the C#
        // spawns; the port has no model. ANALYSIS.
        return;
    };
    // `NewNPC`'s `Bottom = (X, Y)` (`NPC.cs:80770`, `Entity.cs:128-131`):
    // the position is the BOTTOM-centre anchor.
    let (w, h) = kind.size();
    let mut npc = Npc::new(kind, x as f32 - w * 0.5, y as f32 - h, 1, 0.0);
    npc.slime = slime;
    npc.release_owner = who;
    // `NewNPC`'s `timeLeft = (int)(activeTime * 1.25)` (`NPC.cs:80772`,
    // `activeTime = 750` `NPC.cs:95186`).
    npc.despawn_timer = (750.0 * 1.25) as i32;
    if let Some(ai2) = plan.ai2 {
        npc.ai[2] = ai2;
    }
    if plan.target_closest {
        // `NPC.cs:80123-80124`: TargetClosest() then ai[3] = 0. The
        // 583/584/585 family's aiStyle is 112 (`NPC.cs:16587`); the
        // ticker is the port's 0 stand-in (the zombie convention).
        let players = app
            .player
            .as_ref()
            .map(|p| vec![crate::npc_target::session_candidate_for(p, kind)])
            .unwrap_or_default();
        crate::npc_target::target_closest(&mut npc, &app.world, &players, true, 112, 0);
        npc.ai[3] = 0.0;
    }
    if let Some(dir) = plan.direction {
        npc.facing = dir;
        npc.sprite_dir = dir;
    }
    if plan.statue_form {
        npc.spawned_from_statue = true;
        // `CanBeReplacedByOtherNPCs` has no port field (the wiring.rs
        // statue note) — the loot gate rides spawned_from_statue.
    }
    npc.net_update = true;
    app.npcs.push(npc);
}

/// `NPC.SubAI_HandleTemporaryCatchableNPCPlayerInvulnerability`
/// (`NPC.cs:90534-90556`), called from `UpdateNPC` (`NPC.cs:90424`): the
/// 90-tick wild-critter friendly window, zeroed for released/statue
/// spawns. The port runs it as a per-tick pass over the catchable kinds.
pub(crate) fn tick_catchable_immunity(n: &mut Npc) {
    let ty = n.kind.cs_id() as i32;
    if !npc_catchable(ty) {
        return;
    }
    if n.release_owner != 255 || n.spawned_from_statue {
        n.catchable_immunity = 0;
    }
    let was = n.friendly;
    if n.catchable_immunity > 0 {
        n.catchable_immunity -= 1;
        n.friendly = true;
    } else {
        n.friendly = false;
    }
    if was != n.friendly {
        n.net_update = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- The net rows vs the generated item registry ----

    #[test]
    fn s1_p_i36_the_net_rows_match_the_item_registry() {
        // Item.cs:21305-21315 (1991), :29541-29553 (3183), :39480-39492
        // (4821): useTime == useAnimation, autoReuse, useTurn, useStyle 1,
        // damage −1 (the explicit type OR admits them, Player.cs:43648).
        for row in NET_ROWS.iter() {
            let d = registry::get(row.kind).expect("the net row exists");
            assert_eq!(d.use_time as i32, row.use_ticks, "kind {}", row.kind);
            assert_eq!(d.use_animation as i32, row.use_ticks, "kind {}", row.kind);
            assert!((d.scale - row.scale).abs() < 1e-6, "kind {}", row.kind);
            assert!(d.auto_reuse, "kind {}", row.kind);
            assert!(d.use_turn, "kind {}", row.kind);
            assert_eq!(d.use_style, 1, "kind {}", row.kind);
            assert_eq!(d.damage, -1, "kind {}", row.kind);
            assert!(is_bug_net(row.kind));
        }
        assert!(!is_bug_net(1992)); // the Firefly item is not a net
        assert!(!is_bug_net(0));
    }

    // ---- The catchItem conversion table (NPC.cs SetDefaults rows) ----

    #[test]
    fn s1_p_i36_the_catch_item_rows_match_set_defaults() {
        // The base rows, one per cited SetDefaults arm.
        assert_eq!(catch_item(46, 0.0), 2019); // NPC.cs:9346
        assert_eq!(catch_item(303, 0.0), 2019); // the bunny variants share it
        assert_eq!(catch_item(337, 0.0), 2019);
        assert_eq!(catch_item(540, 0.0), 2019);
        assert_eq!(catch_item(55, 0.0), 261); // NPC.cs:9472
        assert_eq!(catch_item(230, 0.0), 261); // NPC.cs:11977
        assert_eq!(catch_item(74, 0.0), 2015); // NPC.cs:9760
        assert_eq!(catch_item(297, 0.0), 2016);
        assert_eq!(catch_item(298, 0.0), 2017);
        assert_eq!(catch_item(299, 0.0), 2018); // NPC.cs:12874
        assert_eq!(catch_item(300, 0.0), 2003); // NPC.cs:12887
        assert_eq!(catch_item(148, 0.0), 2205); // NPC.cs:10871
        assert_eq!(catch_item(149, 0.0), 2205);
        assert_eq!(catch_item(355, 0.0), 1992); // NPC.cs:13529
        assert_eq!(catch_item(357, 0.0), 2002); // NPC.cs:13556
        assert_eq!(catch_item(358, 0.0), 2004);
        assert_eq!(catch_item(359, 0.0), 2006);
        assert_eq!(catch_item(360, 0.0), 2007);
        assert_eq!(catch_item(361, 0.0), 2121);
        assert_eq!(catch_item(362, 0.0), 2122);
        assert_eq!(catch_item(363, 0.0), 2122);
        assert_eq!(catch_item(364, 0.0), 2123);
        assert_eq!(catch_item(365, 0.0), 2123);
        assert_eq!(catch_item(366, 0.0), 2156); // NPC.cs:13671
        assert_eq!(catch_item(367, 0.0), 2157);
        assert_eq!(catch_item(374, 0.0), 2673);
        assert_eq!(catch_item(375, 0.0), 2673);
        assert_eq!(catch_item(377, 0.0), 2740);
        assert_eq!(catch_item(538, 0.0), 3563);
        assert_eq!(catch_item(539, 0.0), 3564);
        assert_eq!(catch_item(583, 0.0), 4068); // NPC.cs:16596
        assert_eq!(catch_item(584, 0.0), 4069);
        assert_eq!(catch_item(585, 0.0), 4070);
        assert_eq!(catch_item(592, 0.0), 4274);
        assert_eq!(catch_item(593, 0.0), 4274);
        assert_eq!(catch_item(602, 0.0), 4359);
        assert_eq!(catch_item(603, 0.0), 4359);
        assert_eq!(catch_item(606, 0.0), 4363);
        assert_eq!(catch_item(607, 0.0), 4373);
        assert_eq!(catch_item(608, 0.0), 4374);
        assert_eq!(catch_item(609, 0.0), 4374);
        assert_eq!(catch_item(610, 0.0), 4375);
        assert_eq!(catch_item(611, 0.0), 4395);
        assert_eq!(catch_item(614, 0.0), 1338); // NPC.cs:16890
        assert_eq!(catch_item(626, 0.0), 4480); // NPC.cs:17066
        assert_eq!(catch_item(627, 0.0), 4482); // the gold overwrite :17069
        assert_eq!(catch_item(653, 0.0), 4845); // NPC.cs:17263
        assert_eq!(catch_item(654, 0.0), 4847);
        assert_eq!(catch_item(655, 0.0), 4849);
        assert_eq!(catch_item(661, 0.0), 4961);
        assert_eq!(catch_item(669, 0.0), 5132);
        assert_eq!(catch_item(671, 0.0), 5212);
        assert_eq!(catch_item(672, 0.0), 5300);
        assert_eq!(catch_item(673, 0.0), 5311);
        assert_eq!(catch_item(674, 0.0), 5312);
        assert_eq!(catch_item(675, 0.0), 5313);
        assert_eq!(catch_item(677, 0.0), 5350);
        assert_eq!(catch_item(687, 0.0), 2121); // NPC.cs:17651
        assert_eq!(catch_item(688, 0.0), 5511);
        // The range rows.
        for t in 442..=448 {
            assert_eq!(catch_item(t, 0.0), 2889 + t - 442); // NPC.cs:14258-14343
        }
        for t in 484..=487 {
            assert_eq!(catch_item(t, 0.0), 3191 + t - 484); // NPC.cs:14587
        }
        for t in 595..=601 {
            assert_eq!(catch_item(t, 0.0), t - 595 + 4334); // NPC.cs:16737
        }
        assert_eq!(catch_item(604, 0.0), 4361); // NPC.cs:16779
        assert_eq!(catch_item(605, 0.0), 4362);
        assert_eq!(catch_item(612, 0.0), 4418); // NPC.cs:16875
        assert_eq!(catch_item(613, 0.0), 4419);
        assert_eq!(catch_item(616, 0.0), 4464); // NPC.cs:16916
        assert_eq!(catch_item(617, 0.0), 4465);
        for t in 639..=645 {
            assert_eq!(catch_item(t, 0.0), 4831 + t - 639); // NPC.cs:17236
        }
        for t in 646..=652 {
            assert_eq!(catch_item(t, 0.0), 4838 + t - 646); // NPC.cs:17248
        }
        // The butterfly's live-style re-derive (NPC.cs:45106).
        assert_eq!(catch_item(356, 1.0), 1994);
        assert_eq!(catch_item(356, 8.0), 2001);
        // A non-catchable kind converts to nothing.
        assert_eq!(catch_item(1, 0.0), 0);
        assert_eq!(catch_item(22, 0.0), 0);
    }

    #[test]
    fn s1_p_i36_the_release_whitelist_is_not_the_catch_list() {
        // Main.cs:9330-9418 — the catchItem carriers OUTSIDE npcCatchable.
        for t in [303, 337, 540, 230, 687] {
            assert!(catch_item(t, 0.0) > 0, "{t} carries a catch item");
            assert!(!npc_catchable(t), "{t} is NOT release-catchable");
        }
        // The set's own members (spot rows + the two loops).
        for t in [46, 55, 74, 148, 149, 356, 583, 584, 585, 614, 653, 654, 655, 688] {
            assert!(npc_catchable(t), "{t} is release-catchable");
        }
        for t in 442..=448 {
            assert!(npc_catchable(t)); // Main.cs:9415-9418
        }
        for t in 639..=655 {
            assert!(npc_catchable(t)); // Main.cs:9394-9410
        }
        assert!(!npc_catchable(22));
        assert!(!npc_catchable(1));
    }

    /// W4-c4 (the fish CATCH census): the nine style-16 swimmers against
    /// the two C# catch surfaces. `NPC.SetDefaults`' `catchItem` rows:
    /// ONLY the GoldGoldfish 592 (`NPC.cs:16691` → 4274), the Pupfish 607
    /// (`:16807` → 4373) and the Pufferfish 688 (`:17665` → 5511) carry
    /// one; the Dolphin 615 (`:16892-16904`) and the Orca 692
    /// (`:17709-17722`) write NO catchItem row (the `:8446` reset's 0),
    /// and neither is in `Main.npcCatchable` (`Main.cs:9326-9418` — the
    /// list jumps 614 → 616, and 692 never appears). The five enemies
    /// (57/157/241/465) carry none either. The fishing side never spawns
    /// them: `FishingCheck_RollEnemySpawns` (`Projectile.cs:20804-20835`)
    /// rolls only 682/620/621/586/587/618, and the item-drop populator
    /// (`GameContentFishDropPopulator.cs`) has no 4274/4373/5511 row —
    /// the three critters are NET-caught, not fished. WRITTEN, NOT RUN.
    #[test]
    fn w4_c4_the_fish_catch_census_matches_the_two_csharp_surfaces() {
        // The three critters: catchItem + the release whitelist.
        for (t, item) in [(592, 4274), (607, 4373), (688, 5511)] {
            assert_eq!(catch_item(t, 0.0), item, "{t} catchItem");
            assert!(npc_catchable(t), "{t} is release-catchable");
        }
        // The Dolphin + the Orca: NO catch row, NOT release-catchable.
        for t in [615, 692] {
            assert_eq!(catch_item(t, 0.0), 0, "{t} has no catchItem row");
            assert!(!npc_catchable(t), "{t} is NOT in Main.npcCatchable");
        }
        // The five enemies: no catch row either.
        for t in [57, 157, 241, 465] {
            assert_eq!(catch_item(t, 0.0), 0, "{t} has no catchItem row");
            assert!(!npc_catchable(t), "{t} is NOT release-catchable");
        }
        // The release round-trip: the three critters' kinds resolve
        // through the bestiary net ids (the W4-c4 entries), so
        // `release_npc`'s roster gap no longer eats them; the release
        // plan is the default arm (spawn + owner only, `NPC.cs:80160`).
        let mut rng = UnifiedRandom::new(3);
        for t in [592, 607, 688] {
            let plan = release_plan(t, 0, 1, &mut rng);
            assert_eq!(plan.spawn_type, t);
            assert_eq!(plan.ai2, None);
            assert!(!plan.target_closest);
            assert!(
                crate::npc::kind_from_net_id(t).is_some(),
                "{t} resolves to a ported kind (the release spawns)"
            );
        }
        // The fishing side: the item-drop table carries NO critter row
        // (4274/4373/5511 appear nowhere in `vanilla_drop_rules` — the
        // populator's Ocean/Desert arms drop fish ITEMS, not the
        // critter-NPC converters), and the enemy roll's pools are
        // 682/620/621/586/587/618 (`roll_enemy_spawns`, the
        // `Projectile.cs:20819-20833` table) — never 615/688/692.
        let rules = tr_core::fishing::vanilla_drop_rules();
        for rule in &rules.rules {
            for &item in rule.items {
                assert!(
                    item != 4274 && item != 4373 && item != 5511,
                    "the fishing item table has no critter-NPC row ({item})"
                );
            }
        }
    }

    #[test]
    fn s1_p_i36_the_lava_bait_set_is_the_three_underworld_items() {
        // ItemID.cs:1306 — CreateBoolSet(4849, 4845, 4847).
        assert!(is_lava_bait(4845));
        assert!(is_lava_bait(4847));
        assert!(is_lava_bait(4849));
        assert!(!is_lava_bait(2015));
        assert!(!is_lava_bait(4068));
    }

    // ---- The catch verdicts (Player.cs:44712-44740) ----

    #[test]
    fn s1_p_i36_the_plain_net_burns_on_lava_bait() {
        // The hell butterfly 653 → item 4845: the plain net takes the
        // Hurt arm, the golden/fireproof nets catch it.
        let v = catch_verdict(1991, 4845, 653, 0.0, 100.0, 200.0);
        assert_eq!(v, CatchVerdict::LavaHurt { hit_dir: 1 }); // npc left of player
        let v = catch_verdict(1991, 4845, 653, 0.0, 300.0, 200.0);
        assert_eq!(v, CatchVerdict::LavaHurt { hit_dir: -1 }); // npc right
        assert_eq!(catch_verdict(3183, 4845, 653, 0.0, 100.0, 200.0), CatchVerdict::Catch);
        assert_eq!(catch_verdict(4821, 4849, 655, 0.0, 100.0, 200.0), CatchVerdict::Catch);
    }

    #[test]
    fn s1_p_i36_the_firefly_family_refuses_a_glowing_release() {
        // Player.cs:44728-44734 — 583/584/585 with ai[2] > 1 skip.
        for t in [583, 584, 585] {
            assert_eq!(catch_verdict(1991, 4068, t, 0.0, 0.0, 0.0), CatchVerdict::Catch);
            assert_eq!(catch_verdict(1991, 4068, t, 1.0, 0.0, 0.0), CatchVerdict::Catch);
            assert_eq!(catch_verdict(1991, 4068, t, 2.0, 0.0, 0.0), CatchVerdict::Skip);
        }
        // Every other catchable kind catches regardless of ai[2].
        assert_eq!(catch_verdict(1991, 2015, 74, 2.0, 0.0, 0.0), CatchVerdict::Catch);
    }

    #[test]
    fn s1_p_i36_the_catch_outcome_order_matches_catch_npc() {
        // NPC.cs:79891-79927 — the branch order: inactive → client →
        // frog → statue → convert.
        assert_eq!(catch_outcome(0, false, 2015, 74, false), CatchOutcome::Nothing);
        assert_eq!(catch_outcome(0, true, 0, 74, false), CatchOutcome::Nothing);
        assert_eq!(catch_outcome(1, true, 2015, 74, false), CatchOutcome::ClientSend);
        // The client arm wins over the statue/frog arms (netMode first).
        assert_eq!(catch_outcome(1, true, 2121, 687, true), CatchOutcome::ClientSend);
        assert_eq!(catch_outcome(0, true, 2121, 687, false), CatchOutcome::MysticFrog);
        assert_eq!(catch_outcome(2, true, 2121, 687, false), CatchOutcome::MysticFrog);
        assert_eq!(catch_outcome(0, true, 2019, 46, true), CatchOutcome::StatuePoof);
        assert_eq!(catch_outcome(0, true, 2015, 74, false), CatchOutcome::Convert);
        assert_eq!(catch_outcome(2, true, 261, 55, false), CatchOutcome::Convert);
    }

    // ---- The swing rectangle (Player.cs:45099-45190 + 50715-50852) ----

    #[test]
    fn s1_p_i36_the_plain_net_rect_walks_the_thirds() {
        // The Bug Net: frame 36×40, scale 1. Player at (1000, 2000),
        // facing right, anim max 25.
        let row = net_row(1991).unwrap();
        // Swing START (anim 25 >= 25*0.666): itemLocation.X =
        // 1000 + 10 - (18 - 14) = 1006 (>32 → num3 14), Y = 2000 + 10.
        // Box 36×40 at (1006, 2010-40=1970); the start third: dir 1 →
        // X -= (int)(36*1.2)=43 → 963; W *= 2 → 72; Y -= (int)(40*0.4)=16
        // → 1954; H = (int)(40*1.4)=56.
        let (dont, x, y, w, h) = net_swing_rect(row, 1.0, 25, 25, 1000.0, 2000.0, 1);
        assert!(!dont);
        assert_eq!((x, y, w, h), (963, 1954, 72, 56));
        // Middle third (anim 12): loc X = 1010 + (18-18) = 1010,
        // Y = 2000 + 8 (>32 → 8). Box (1010, 2008-40=1968, 36, 40) — no adjust.
        let (_, x, y, w, h) = net_swing_rect(row, 1.0, 12, 25, 1000.0, 2000.0, 1);
        assert_eq!((x, y, w, h), (1010, 1968, 36, 40));
        // Swing END (anim 5 < 8.325): loc X = 1010 + (18-14) = 1014,
        // Y = 2024. Box (1014, 1984, 36, 40); end third: W=(int)50.4=50,
        // Y += 20 → 2004, H = 44.
        let (_, x, y, w, h) = net_swing_rect(row, 1.0, 5, 25, 1000.0, 2000.0, 1);
        assert_eq!((x, y, w, h), (1014, 2004, 50, 44));
    }

    #[test]
    fn s1_p_i36_the_left_facing_flips_the_rect() {
        let row = net_row(1991).unwrap();
        // Facing left, middle third: loc X = 1010 + (18-18)*(-1) = 1010;
        // the direction flip subtracts the width: 1010 - 36 = 974.
        let (_, x, y, w, h) = net_swing_rect(row, 1.0, 12, 25, 1000.0, 2000.0, -1);
        assert_eq!((x, y, w, h), (974, 1968, 36, 40));
        // End third, left: loc X = 1010 + (18-14)*(-1) = 1006; flip:
        // 1006-36 = 970; the end adjust X -= (int)(36*1.4-36)=14 → 956;
        // W = 50, Y = 2024-40+20 = 2004, H = 44.
        let (_, x, y, w, h) = net_swing_rect(row, 1.0, 5, 25, 1000.0, 2000.0, -1);
        assert_eq!((x, y, w, h), (956, 2004, 50, 44));
    }

    #[test]
    fn s1_p_i36_the_variant_scales_resize_the_rect() {
        // Golden net: frame 44×48, scale 1.15 → W=(int)50.6=50,
        // H=(int)55.2=55. Middle third loc X = 1010 + (22-18) = 1014,
        // Y = 2000+8; box (1014, 2008-55=1953, 50, 55).
        let row = net_row(3183).unwrap();
        let (_, x, y, w, h) = net_swing_rect(row, 1.15, 9, 18, 1000.0, 2000.0, 1);
        assert_eq!((x, y, w, h), (1014, 1953, 50, 55));
        // Fireproof net: frame 48×50, scale 0.85 → W=(int)40.8=40,
        // H=(int)42.5=42. The START third's num3 hits the >=48 band (18):
        // loc X = 1010 - (24-18) = 1004, Y = 2010; box (1004, 1968, 40, 42);
        // start: X -= (int)(40*1.2)=48 → 956; W=80; Y -= (int)(42*0.4)=16
        // → 1952; H=(int)58.8=58.
        let row = net_row(4821).unwrap();
        let (_, x, y, w, h) = net_swing_rect(row, 0.85, 21, 21, 1000.0, 2000.0, 1);
        assert_eq!((x, y, w, h), (956, 1952, 80, 58));
    }

    // ---- CanReleaseNPCs (NPC.cs:80070-80104) ----

    #[test]
    fn s1_p_i36_the_release_budget_gates() {
        // SP: num 0.75, one player → per-player (int)(200*0.75)=150.
        assert!(can_release_npcs(0, 0, 0, 1));
        assert!(can_release_npcs(0, 149, 149, 1)); // both strict <
        assert!(!can_release_npcs(0, 150, 0, 1)); // 150 < 150 false
        assert!(!can_release_npcs(0, 0, 150, 1));
        // Client: num 0.7 → the population cap 140, per-player 140.
        assert!(can_release_npcs(1, 139, 139, 1));
        assert!(!can_release_npcs(1, 140, 0, 1));
        // Two players: per-player (int)(200*0.75/2)=75.
        assert!(can_release_npcs(0, 10, 74, 2));
        assert!(!can_release_npcs(0, 10, 75, 2));
    }

    // ---- ReleaseNPC's switch (NPC.cs:80115-80164) ----

    #[test]
    fn s1_p_i36_the_firefly_release_glows_and_targets() {
        let mut rng = UnifiedRandom::new(1);
        for t in [583, 584, 585] {
            let plan = release_plan(t, 0, 1, &mut rng);
            assert_eq!(plan.spawn_type, t);
            assert_eq!(plan.ai2, Some(2.0)); // NPC.cs:80122
            assert!(plan.target_closest); // NPC.cs:80123
        }
    }

    #[test]
    fn s1_p_i36_the_butterfly_release_keeps_its_colour() {
        let mut rng = UnifiedRandom::new(1);
        let plan = release_plan(356, 5, 1, &mut rng);
        assert_eq!(plan.ai2, Some(5.0)); // NPC.cs:80129 — the placeStyle
        assert!(!plan.target_closest);
        // And the released style converts back to the same item
        // (NPC.cs:45106): 1994 + 5 - 1 = 1998.
        assert_eq!(catch_item(356, 5.0), 1998);
    }

    #[test]
    fn s1_p_i36_the_penguin_release_draws_one_next2() {
        // NPC.cs:80133 — `Type + Main.rand.Next(2)`: exactly one draw.
        let mut rng = UnifiedRandom::new(7);
        let witness = rng.clone();
        let plan = release_plan(148, 0, 1, &mut rng);
        assert!(plan.spawn_type == 148 || plan.spawn_type == 149);
        // The witness replay: one Next(2) consumed, nothing else.
        let mut replay = witness;
        let roll = replay.next_range(0, 2);
        assert_eq!(plan.spawn_type, 148 + roll);
        assert_eq!(rng, replay, "exactly one Next(2) draw");
    }

    #[test]
    fn s1_p_i36_the_explosive_bunny_release_faces_and_statues() {
        let mut rng = UnifiedRandom::new(1);
        // Style 0: the player's direction.
        let plan = release_plan(614, 0, -1, &mut rng);
        assert_eq!(plan.direction, Some(-1));
        assert!(!plan.statue_form);
        // Style 1/2 force the direction (NPC.cs:80147-80154).
        assert_eq!(release_plan(614, 1, -1, &mut rng).direction, Some(1));
        assert_eq!(release_plan(614, 2, 1, &mut rng).direction, Some(-1));
        // Style > 2: the statue form, style -= 2 (NPC.cs:80141-80146).
        let plan = release_plan(614, 3, 1, &mut rng);
        assert!(plan.statue_form);
        assert_eq!(plan.direction, Some(1)); // 3-2=1 → dir 1
        let plan = release_plan(614, 4, 1, &mut rng);
        assert!(plan.statue_form);
        assert_eq!(plan.direction, Some(-1)); // 4-2=2 → dir -1
    }

    /// W4-c8 sweep — the nine aiStyle-66 kinds in the catch chain: the
    /// catchItem rows (already pinned in the i36 matrix — re-pinned here
    /// per kind against the SetDefaults citations), the release
    /// whitelist (375 the ONE non-member — the digger has a catchItem
    /// but is NOT in `Main.npcCatchable`, `Main.cs:9326-9418` has no
    /// 375 row), the verdict (none is lava bait or the 583-family —
    /// every net catches), the release plan (the default arm — spawn +
    /// owner only, NO draws) and the cage-item round trip (the item's
    /// `makeNPC` row spawns the SAME kind back — the itemdata rows).
    /// WRITTEN, NOT RUN (code-only mode).
    #[test]
    fn w4_c8_the_nine_crawlers_ride_the_catch_chain() {
        use crate::npc::{NpcKind, kind_from_net_id};
        // (vanilla type, catchItem, NpcKind, release-catchable) —
        // NPC.cs:13556 (357→2002), :13773/:13792 (374/375→2673),
        // :14343 (448→2895), :14587 (484-487→3191+type-484),
        // :16792 (606→4363).
        let rows: [(i32, i32, NpcKind, bool); 9] = [
            (357, 2002, NpcKind::Worm, true),
            (374, 2673, NpcKind::TruffleWorm, true),
            (375, 2673, NpcKind::TruffleWormDigger, false),
            (448, 2895, NpcKind::GoldWorm, true),
            (484, 3191, NpcKind::EnchantedNightcrawler, true),
            (485, 3192, NpcKind::Grubby, true),
            (486, 3193, NpcKind::Sluggy, true),
            (487, 3194, NpcKind::Buggy, true),
            (606, 4363, NpcKind::Maggot, true),
        ];
        let mut rng = UnifiedRandom::new(3);
        for (ty, item, kind, catchable) in rows {
            // The catch conversion row.
            assert_eq!(catch_item(ty, 0.0), item, "{ty} catchItem");
            // The release whitelist (Main.cs:9341/:9352/:9361-9364/
            // :9381 + the 442..=448 loop :9415-9418; NO 375 row).
            assert_eq!(npc_catchable(ty), catchable, "{ty} npcCatchable");
            // The verdict: all three nets catch (no lava bait, no
            // 583-family gate — Player.cs:44714-44738).
            for net in [1991u16, 3183, 4821] {
                assert_eq!(
                    catch_verdict(net, item, ty, 0.0, 100.0, 200.0),
                    CatchVerdict::Catch,
                    "{ty} net {net}"
                );
            }
            // The CatchNPC branch: SP converts (the statue poof is the
            // SpawnedFromStatue arm, not a kind gate).
            assert_eq!(catch_outcome(0, true, item, ty, false), CatchOutcome::Convert);
            assert_eq!(catch_outcome(0, true, item, ty, true), CatchOutcome::StatuePoof);
            // The release plan: the default arm (NPC.cs:80160-80163) —
            // spawn + owner only, no ai2/target/direction/statue, and
            // NO RNG draw.
            let witness = rng.clone();
            let plan = release_plan(ty, 0, 1, &mut rng);
            assert_eq!(plan.spawn_type, ty, "{ty} releases as itself");
            assert_eq!(plan.ai2, None);
            assert!(!plan.target_closest);
            assert_eq!(plan.direction, None);
            assert!(!plan.statue_form);
            assert_eq!(rng, witness, "{ty} the default arm draws nothing");
            // The cage item's makeNPC row spawns the SAME kind back
            // (itemdata.rs: 2002→357, 2673→374, 2895→448, 3191-3194→
            // 484-487, 4363→606). NOTE 2673 releases the WALKER 374 —
            // the digger 375 is a Transform-only form (no item row).
            let make = crate::registry::make_npc(item as u16) as i32;
            let expect_type = if ty == 375 { 374 } else { ty };
            assert_eq!(make, expect_type, "item {item} makeNPC");
            let (k, _) = kind_from_net_id(make).expect("the released kind is ported");
            if ty != 375 {
                assert_eq!(k, kind, "{ty} round-trips through its cage item");
            }
        }
    }

    #[test]
    fn s1_p_i36_the_release_position_gates() {
        // Player.cs:44059-44063 — the 614 arm ignores reach/solid.
        let p = release_position(614, false, true, 100.0, 2042.0, Vec2::new(500.0, 500.0));
        assert_eq!(p, Some((100, 2042)));
        // The generic arm: reach + non-solid, truncated mouse.
        let p = release_position(583, true, false, 100.0, 2042.0, Vec2::new(500.9, 500.9));
        assert_eq!(p, Some((500, 500)));
        assert_eq!(release_position(583, false, false, 100.0, 2042.0, Vec2::ZERO), None);
        assert_eq!(release_position(583, true, true, 100.0, 2042.0, Vec2::ZERO), None);
    }

    // ---- The temporary catchable immunity (NPC.cs:90534-90556) ----

    #[test]
    fn s1_p_i36_the_wild_critter_friendliness_lasts_90_ticks() {
        // A wild goldfish (55, npcCatchable) starts friendly with the
        // 90-tick counter (NPC.cs:17851-17855); each tick decrements and
        // keeps friendly true until the counter empties.
        let mut n = Npc::new(crate::npc::NpcKind::Goldfish, 0.0, 0.0, 1, 0.0);
        assert_eq!(n.catchable_immunity, 90);
        assert!(n.friendly);
        for _ in 0..90 {
            tick_catchable_immunity(&mut n);
            assert!(n.friendly);
        }
        assert_eq!(n.catchable_immunity, 0);
        tick_catchable_immunity(&mut n);
        assert!(!n.friendly, "the 91st tick flips friendly off");
        assert!(n.net_update, "the flip arms netUpdate (NPC.cs:90552-90554)");
    }

    #[test]
    fn s1_p_i36_a_released_critter_is_never_friendly() {
        // NPC.cs:90538-90541 — releaseOwner != 255 zeroes the counter.
        let mut n = Npc::new(crate::npc::NpcKind::Goldfish, 0.0, 0.0, 1, 0.0);
        n.release_owner = 0;
        tick_catchable_immunity(&mut n);
        assert_eq!(n.catchable_immunity, 0);
        assert!(!n.friendly);
        // The statue form too.
        let mut s = Npc::new(crate::npc::NpcKind::Goldfish, 0.0, 0.0, 1, 0.0);
        s.spawned_from_statue = true;
        tick_catchable_immunity(&mut s);
        assert!(!s.friendly);
        // A non-catchable kind is untouched.
        let mut z = Npc::new(crate::npc::NpcKind::Zombie, 0.0, 0.0, 1, 0.0);
        let was = z.friendly;
        tick_catchable_immunity(&mut z);
        assert_eq!(z.friendly, was);
    }
}
