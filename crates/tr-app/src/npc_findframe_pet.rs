//! W4-g10 lane 3 — `NPC.FindFrame`'s humanoid-group arm for the TOWN PETS
//! (637/638/656/670/678-684): the case list carries them (`NPC.cs:68622-
//! 68633`), so they ride the shared town body (`:68634-69655`) with the
//! pet-specific overrides — the slime `spriteDirection` flip (`:68657-
//! 68660`), the state-20/21/22 idle poses (`:68680-68917`), the cat's sit
//! row 19 (`:68980-68983`), the dog's idle 0..7 cycle (`:69491-69508`),
//! the dog/bunny/slime walk stride 12 + start-row overrides (`:69532-
//! 69571`) and the airborne slime hop rows / dog air row 8 (`:69601-
//! 69654`).
//!
//! Shape: the `npc_findframe_b.rs` precedent — a pure fn over the NPC's
//! own fields (the pets ARE in `NpcKind`, so no `FrameState` seam is
//! needed); `npc.rs::find_frame`'s pet arm calls [`town_pet_find_frame`].
//! No RNG is touched by the live pet states: the only `Main.rand` draws
//! in the group arm sit in the state-11 Tax-Collector dust (`:68944-
//! 68963`, type 229 only), the state-6 party dust (`:69000-69008`, the
//! party gate excludes pets, `:55289`), the state-3/4 talk emotes
//! (`:69262-69273`, `CanTalk` is false for pets, `NPC.cs:6633-6643`) and
//! the state-16/17 gossip emotes (`:69381-69481`, the gossip scan skips
//! pets, `:55258-55288`'s `!is_town_pet` gate). The emote spawns
//! themselves are the port's documented unmodelled seam (the frame rows
//! still advance exactly).
//!
//! Row convention: the port tracks frame ROW indices, not pixels — the
//! C#'s `num` (the sheet height / `Main.npcFrameCount[type]`) cancels out
//! of every comparison. `num58 = npcFrameCount - AttackFrameCount` and
//! the pets' `AttackFrameCount` rows are all 0 (`NPCID.cs:10465`), so
//! `num58` IS the full row count (28/28/27/14, `Main.cs:67147-67152`).
//! `num57 = ExtraFramesCount` (`NPCID.cs:10464`: 637 = 18, 638 = 11,
//! 656 = 20, slimes = 6) bounds the walk wrap.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-g10 town pets"):
//! the disabled profile-texture block (`NPC.cs:68637`, `if (false &&
//! …)`), the `EmoteBubble` spawns (the port has no emote system), and
//! the group arm's state branches that are UNREACHABLE for pets — they
//! are dropped so those states fall through to the idle/walk tail
//! exactly as the C#'s own gates make them do:
//! - state 6 party talk (`:68986-69015`): the AI gate excludes pets
//!   (`:55289`'s `!IsTownPet`);
//! - state 7/19 player talk (`:69016-69056`): the branch itself carries
//!   `!IsTownPet` — a pet in state 7 (reachable via `:55339`) falls
//!   through to idle/walk, which the trimmed chain reproduces;
//! - state 11 grumble (`:68937-68976`): type 229 only (`:55333`);
//! - state 18 Bartender talk (`:69079-69112`): type 550 only (`:55308`);
//! - states 10/13/15/12/14/24 attacks (`:69113-69189`): the pets'
//!   `AttackType`/`AttackTime` rows are -1 (`NPCID.cs:10471-10473`) —
//!   they never attack; 13 is the Nurse's heal (type 18, `:55422`);
//! - states 3/4/16/17 talk/gossip (`:69195-69488`): `CanTalk` is false
//!   for pets (`NPC.cs:6633-6643`), so the C# falls through too;
//! - state 25 shimmer (`:69146-69149`) and state 1001 sleep
//!   (`:69190-69194`): the shimmer/sleep systems are unported (the town
//!   machine early-returns on 25 without writing it).
//! The state-2 small-talk blink (`:68918-68936`) and the state-23
//! ShowItem pose (`GameContent/Animations/Actions.cs:269`) are carried
//! verbatim although both are unreachable today (the `:55327` pet gate
//! / the unported animation system) — they are cheap and keep the
//! pose-reset shape complete.

use crate::npc::{Npc, NpcKind};

/// `Main.npcFrameCount[type]` for the pets (`Main.cs:67147`/`:67149`/
/// `:67151-67152`) — also `num58`, the pets' `AttackFrameCount` being 0.
fn pet_frame_count(kind: NpcKind) -> u32 {
    match kind {
        NpcKind::TownCat | NpcKind::TownDog => 28,
        NpcKind::TownBunny => 27,
        _ => 14, // the town slimes
    }
}

/// `NPCID.Sets.IsTownSlime` (`NPCID.cs:10119`) over the ported kinds.
fn is_slime(kind: NpcKind) -> bool {
    matches!(
        kind,
        NpcKind::TownSlimeBlue
            | NpcKind::TownSlimeGreen
            | NpcKind::TownSlimeOld
            | NpcKind::TownSlimePurple
            | NpcKind::TownSlimeRainbow
            | NpcKind::TownSlimeRed
            | NpcKind::TownSlimeYellow
    )
}

/// The `(uint)(num60 - 1) > 1u && (uint)(num60 - 4) > 1u && row != 0`
/// reset gate shared by the state arms (`NPC.cs:68667`/`:68991`/
/// `:69020`/`:69061`/`:69083`/`:69200`/`:69284`): the unsigned wraps make
/// it "row is NOT one of {num58-1, num58-2, num58-3, num58-4, 0}".
fn pose_reset_needed(num58: u32, row: u32) -> bool {
    let num60 = num58.wrapping_sub(row);
    num60.wrapping_sub(1) > 1 && num60.wrapping_sub(4) > 1 && row != 0
}

/// One `FindFrame` tick for a town pet — the humanoid-group arm
/// (`NPC.cs:68634-69655`) with the pet overrides. The caller's
/// `find_frame` match routes only the ten pet kinds here.
pub fn town_pet_find_frame(npc: &mut Npc) {
    let kind = npc.kind;
    let net_id = kind.net_id();
    let slime = is_slime(kind);
    let num58 = pet_frame_count(kind);
    let num57 = kind.extra_frames_count();

    if npc.vy == 0.0 {
        // `:68649-68660` — the grounded flip; the town slimes flip AGAIN
        // (their sheets face the opposite way).
        if npc.facing == 1 {
            npc.sprite_dir = 1;
        }
        if npc.facing == -1 {
            npc.sprite_dir = -1;
        }
        if slime {
            npc.sprite_dir *= -1;
        }
        if npc.ai[0] == 23.0 {
            // `:68662-68679` — the ShowItem pose (the animation system's
            // state; carried verbatim, unreachable until it lands).
            npc.frame_counter += 1.0;
            let num59 = npc.frame;
            if pose_reset_needed(num58, num59) {
                npc.frame = 0;
                npc.frame_counter = 0.0;
            }
            let mut num61 = if npc.frame_counter >= 6.0 { num58 - 4 } else { num58 - 5 };
            if npc.ai[1] < 6.0 {
                num61 = num58 - 5;
            }
            npc.frame = num61;
        } else if (20.0..=22.0).contains(&npc.ai[0]) {
            // `:68680-68917` — the pet idle poses. Each per-kind block
            // reads the CURRENT row (the C# shares `num62` across the
            // blocks; only one block matches per kind, so the port reads
            // `npc.frame` per block — identical).
            let state = npc.ai[0] as i32;
            if state == 20 {
                if slime {
                    // `:68686-68709` — the slime nap: rows 8..13, the
                    // 12-tick stride, the >30 timer loops 8..13, else 0.
                    let mut row = npc.frame;
                    if npc.ai[1] > 30.0 && !(8..=13).contains(&row) {
                        row = 8;
                    }
                    if row > 0 {
                        npc.frame_counter += 1.0;
                    }
                    if npc.frame_counter >= 12.0 {
                        npc.frame_counter = 0.0;
                        row += 1;
                        if row > 13 && npc.ai[1] > 30.0 {
                            row = 8;
                        }
                        if row > 13 {
                            row = 0;
                        }
                    }
                    npc.frame = row;
                }
                if net_id == 656 {
                    // `:68710-68733` — the bunny's ear twitch: rows
                    // 7..9, the 4-tick stride, >8 loops to 8 while the
                    // timer runs, >9 snaps to 0.
                    let mut row = npc.frame;
                    if npc.ai[1] > 30.0 && !(7..=9).contains(&row) {
                        row = 7;
                    }
                    if row > 0 {
                        npc.frame_counter += 1.0;
                    }
                    if npc.frame_counter > 4.0 {
                        npc.frame_counter = 0.0;
                        row += 1;
                        if row > 8 && npc.ai[1] > 30.0 {
                            row = 8;
                        }
                        if row > 9 {
                            row = 0;
                        }
                    }
                    npc.frame = row;
                }
                if net_id == 637 {
                    // `:68734-68757` — the cat's wash: rows 10..16, the
                    // 4-tick stride, >13 loops to 13 while the timer
                    // runs, >16 snaps to 0.
                    let mut row = npc.frame;
                    if npc.ai[1] > 30.0 && !(10..=16).contains(&row) {
                        row = 10;
                    }
                    if row > 0 {
                        npc.frame_counter += 1.0;
                    }
                    if npc.frame_counter > 4.0 {
                        npc.frame_counter = 0.0;
                        row += 1;
                        if row > 13 && npc.ai[1] > 30.0 {
                            row = 13;
                        }
                        if row > 16 {
                            row = 0;
                        }
                    }
                    npc.frame = row;
                }
                if net_id == 638 {
                    // `:68762-68782` — the dog's scratch: rows 23..27,
                    // the 4-tick stride, >26 loops to 24 while the timer
                    // runs, >27 snaps to 0.
                    let mut row = npc.frame;
                    if npc.ai[1] > 30.0 && !(23..=27).contains(&row) {
                        row = 23;
                    }
                    if row > 0 {
                        npc.frame_counter += 1.0;
                    }
                    if npc.frame_counter > 4.0 {
                        npc.frame_counter = 0.0;
                        row += 1;
                        if row > 26 && npc.ai[1] > 30.0 {
                            row = 24;
                        }
                        if row > 27 {
                            row = 0;
                        }
                    }
                    npc.frame = row;
                }
            } else if state == 21 {
                if net_id == 656 {
                    // `:68785-68808` — the bunny's second pose: rows
                    // 10..16, >13 loops to 13, >16 snaps to 0.
                    let mut row = npc.frame;
                    if npc.ai[1] > 30.0 && !(10..=16).contains(&row) {
                        row = 10;
                    }
                    if row > 0 {
                        npc.frame_counter += 1.0;
                    }
                    if npc.frame_counter > 4.0 {
                        npc.frame_counter = 0.0;
                        row += 1;
                        if row > 13 && npc.ai[1] > 30.0 {
                            row = 13;
                        }
                        if row > 16 {
                            row = 0;
                        }
                    }
                    npc.frame = row;
                }
                if net_id == 637 {
                    // `:68809-68832` — the cat's stretch: rows 17..21,
                    // >19 loops to 19, >21 snaps to 0.
                    let mut row = npc.frame;
                    if npc.ai[1] > 30.0 && !(17..=21).contains(&row) {
                        row = 17;
                    }
                    if row > 0 {
                        npc.frame_counter += 1.0;
                    }
                    if npc.frame_counter > 4.0 {
                        npc.frame_counter = 0.0;
                        row += 1;
                        if row > 19 && npc.ai[1] > 30.0 {
                            row = 19;
                        }
                        if row > 21 {
                            row = 0;
                        }
                    }
                    npc.frame = row;
                }
                if net_id == 638 {
                    // `:68837-68857` — the dog's second pose: rows
                    // 17..22, >21 loops to 18, >22 snaps to 0.
                    let mut row = npc.frame;
                    if npc.ai[1] > 30.0 && !(17..=22).contains(&row) {
                        row = 17;
                    }
                    if row > 0 {
                        npc.frame_counter += 1.0;
                    }
                    if npc.frame_counter > 4.0 {
                        npc.frame_counter = 0.0;
                        row += 1;
                        if row > 21 && npc.ai[1] > 30.0 {
                            row = 18;
                        }
                        if row > 22 {
                            row = 0;
                        }
                    }
                    npc.frame = row;
                }
            } else {
                // state 22 — `:68859-68914`.
                if net_id == 656 {
                    // `:68860-68884` — the bunny's third pose: rows
                    // 17..26 (the `>= num63` bound is the 27-row strip),
                    // the timer gate is >40 here, >20 loops to 19.
                    let num63 = num58;
                    let mut row = npc.frame;
                    if npc.ai[1] > 40.0 && (row < 17 || row >= num63) {
                        row = 17;
                    }
                    if row > 0 {
                        npc.frame_counter += 1.0;
                    }
                    if npc.frame_counter > 4.0 {
                        npc.frame_counter = 0.0;
                        row += 1;
                        if row > 20 && npc.ai[1] > 40.0 {
                            row = 19;
                        }
                        if row >= num63 {
                            row = 0;
                        }
                    }
                    npc.frame = row;
                }
                if net_id == 637 {
                    // `:68889-68913` — the cat's third pose: rows
                    // 17..27, >27 wraps to 22 (timer) / 20 (no timer),
                    // the no-timer row 22 snaps to 0, the timer's 20/21
                    // jump to 22.
                    let mut row = npc.frame;
                    if npc.ai[1] > 30.0 && !(17..=27).contains(&row) {
                        row = 17;
                    }
                    if row > 0 {
                        npc.frame_counter += 1.0;
                    }
                    if npc.frame_counter > 4.0 {
                        npc.frame_counter = 0.0;
                        row += 1;
                        if row > 27 {
                            row = if npc.ai[1] > 30.0 { 22 } else { 20 };
                        } else if npc.ai[1] <= 30.0 && row == 22 {
                            row = 0;
                        } else if npc.ai[1] > 30.0 && row > 19 && row < 22 {
                            row = 22;
                        }
                    }
                    npc.frame = row;
                }
                // The dog and the slimes have NO state-22 block — the
                // row stands (`:68916` writes num62 unchanged).
            }
        } else if npc.ai[0] == 2.0 {
            // `:68918-68936` — the small-talk blink (the pet gate at
            // `:55327` keeps pets out of state 2; carried verbatim).
            npc.frame_counter += 1.0;
            if npc.frame == num58 - 1 && npc.frame_counter >= 5.0 {
                npc.frame = 0;
                npc.frame_counter = 0.0;
            } else if npc.frame == 0 && npc.frame_counter >= 40.0 {
                npc.frame = num58 - 1;
                npc.frame_counter = 0.0;
            } else if npc.frame != 0 && npc.frame != num58 - 1 {
                npc.frame = 0;
                npc.frame_counter = 0.0;
            }
        } else if npc.ai[0] == 5.0 {
            // `:68977-68985` — the sit: row num58-3, the cat's own row 19.
            npc.frame = if net_id == 637 { 19 } else { num58 - 3 };
            npc.frame_counter = 0.0;
        } else if npc.ai[0] == 9.0 {
            // `:69057-69078` — the interact pause (the door/workbench
            // look, `:55401-55421` — pets DO reach it).
            npc.frame_counter += 1.0;
            let row = npc.frame;
            if pose_reset_needed(num58, row) {
                npc.frame = 0;
                npc.frame_counter = 0.0;
            }
            let mut num73 = if npc.frame_counter >= 16.0 {
                num58 - 4
            } else if npc.frame_counter >= 10.0 {
                num58 - 5
            } else {
                0
            };
            if npc.ai[1] < 16.0 {
                num73 = num58 - 5;
            }
            if npc.ai[1] < 10.0 {
                num73 = 0;
            }
            npc.frame = num73;
        } else if npc.vx == 0.0 {
            // `:69489-69519` — the idle: the dog cycles rows 0..7 on the
            // 4-tick stride (a row above 7 snaps to 0 first); the rest
            // snap to row 0. (The 140/489 row-1 quirk is not a pet.)
            if net_id == 638 {
                let mut num107 = npc.frame;
                if num107 > 7 {
                    num107 = 0;
                }
                npc.frame_counter += 1.0;
                if npc.frame_counter > 4.0 {
                    npc.frame_counter = 0.0;
                    num107 += 1;
                    if num107 > 7 {
                        num107 = 0;
                    }
                }
                npc.frame = num107;
            } else {
                npc.frame = 0;
                npc.frame_counter = 0.0;
            }
        } else {
            // `:69521-69585` — the walk. The dog/bunny/slime stride is
            // 12 (`:69532-69543`), the cat keeps 6; the accumulator is
            // `|vx| * 2 + 1` (`:69552-69553`, the 489 branch is not a
            // pet). The start row: dog 9, bunny/slime 1, cat 2
            // (`:69559-69571`); the wrap stops before the ExtraFrames
            // rows (`:69581-69584`).
            let num108 = if net_id == 638 || net_id == 656 || slime { 12.0 } else { 6.0 };
            npc.frame_counter += npc.vx.abs() * 2.0;
            npc.frame_counter += 1.0;
            let num109 = if net_id == 638 {
                9
            } else if net_id == 656 || slime {
                1
            } else {
                2
            };
            if npc.frame < num109 {
                npc.frame = num109;
            }
            if npc.frame_counter > num108 {
                npc.frame += 1;
                npc.frame_counter = 0.0;
            }
            if npc.frame >= num58 - num57 {
                npc.frame = num109;
            }
        }
        return;
    }
    // `:69588-69655` — the airborne tail. (The 462 branch is not a pet.)
    if slime {
        // `:69601-69641` — the slime hop: the rise rows 2..3, the fall
        // rows 3..6, the 4-tick stride (the counter seeds at -1 so the
        // first in-band tick lands on the band's floor row).
        npc.sprite_dir = -npc.facing;
        let mut num110 = npc.frame;
        if npc.vy < 0.0 {
            if !(2..=3).contains(&num110) {
                num110 = 2;
                npc.frame_counter = -1.0;
            }
            npc.frame_counter += 1.0;
            if npc.frame_counter >= 4.0 {
                npc.frame_counter = 0.0;
                num110 += 1;
                if num110 >= 3 {
                    num110 = 3;
                }
            }
            npc.frame = num110;
        } else if npc.vy > 0.0 {
            if !(3..=6).contains(&num110) {
                num110 = 3;
                npc.frame_counter = -1.0;
            }
            npc.frame_counter += 1.0;
            if npc.frame_counter >= 4.0 {
                npc.frame_counter = 0.0;
                num110 += 1;
                if num110 >= 6 {
                    num110 = 6;
                }
            }
            npc.frame = num110;
        }
    } else {
        // `:69642-69646` — the cat/bunny air row 1.
        npc.frame_counter = 0.0;
        npc.frame = 1;
    }
    // `:69647-69650`'s row-0 pin list carries no pet.
    if net_id == 638 {
        // `:69651-69654` — the dog's air row 8.
        npc.frame = 8;
    }
}
