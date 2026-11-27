//! W4-g10 lane 3 FINAL — the PETTING interaction: `PlayerPettingInfo`
//! (`GameContent/PlayerPettingInfo.cs:5-77`) + `NPC.GetPettingInfo`
//! (`NPC.cs:80858-80892`) + `Player.PetAnimal`/`PetMount`/`StopPettingAnimal`/
//! `UpdatePettingAnimal` (`Player.cs:33339-33490`) + the
//! `ItemCheck_ApplyHoldStyle` petting arm (`:50243-50269`).
//!
//! The e10 deferred host: the player-sounds wave recorded "petting has no
//! field" at five sites (`player.rs`'s StopVanityActions folds, the
//! `player_lights.rs:488` Fairy Queen gate, the projectile `flag9`/`flag10`
//! folds). This module lands the STATE + the interaction machine; the
//! folds' activation is the one-line swap at each site (the
//! `pending_*_sound` convention — the Player queues, the caller drains).
//!
//! Census (the Lead's pet-pet-animation / heart-particle / sound question):
//! * **animation**: the ONLY petting pose is the composite BACK arm's
//!   14-tick pat cycle (`miscCounter % 14 / 7`: ThreeQuarters ↔ Full,
//!   `Player.cs:50257-50268`) — rotation `-2π·0.3·direction` (small pets
//!   0.2), or `-2π·0.36·direction` on the Chillemt mounts 62-65
//!   (`:50245-50253`). The pet NPC itself has NO petting FindFrame state
//!   (the town machine's idle poses are independent); the held item hides
//!   for holdStyles 4/5/6 (`CanVisuallyHoldItem` `:50204`/`:50225-50232`).
//! * **heart particle**: the heart is the Pal-projectile EMOTE bubble
//!   (`Main.cs:34270-34275`: `EmoteBubble.DrawTemporaryBubble(0, 60,
//!   localAI[0], proj)` — emote 0 = `EmoteID.EmotionLove`,
//!   `EmoteBubble.cs:189-197`), drawn while `petting.proj == whoAmI` and
//!   `1 < localAI[0] < 60`. The TOWN-PET path draws NO particle at all
//!   (`PetAnimal` `:33358-33399` has no dust/gore/emote call) — the
//!   PetExchange smoke burst is the exchange's, not the petting's.
//! * **sound**: `PetMount` plays `SoundID.PalChilletJoy` for mount types
//!   62-65 (`:33352-33355`, the `(uint)(type - 62) <= 3` gate);
//!   `PetAnimal` on an NPC plays NOTHING. The Pal Digtoise's joy cue
//!   (`Projectile.cs:38517`) is the projectile path's, out of this lane.
//! * **achievement**: `AchievementsHelper.HandleSpecialEvent(player, 21)`
//!   → `PET_THE_PET`/`Do` (`AchievementsHelper.cs:358-360`), local player
//!   only (`:284-287`), on BOTH PetAnimal and PetMount success.
//!
//! RNG census: the whole petting chain draws NOTHING — no `Main.rand` slot
//! in GetPettingInfo/PetAnimal/PetMount/UpdatePettingAnimal/the hold arm.
//! The PalChilletJoy play's `GetRandomPitch` NextFloat rides the audio-only
//! stream (variance 0, the draw still happens — the `play_chillet_sound`
//! contract), never a sim stream.

use tr_core::World;

use crate::npc::{Npc, NpcKind};
use crate::player::Player;

/// `Math.Sign(float)` — the C# returns 0 at exactly 0 (Rust's `signum`
/// returns ±1 there, so the `num == 0 → direction` fallback needs this).
fn math_sign(v: f32) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

/// `PlayerPettingInfo` (`PlayerPettingInfo.cs:5-19`): the petting state —
/// the target slot + type, the offset from the pet's bottom, the small-pet
/// flag and the mount flag. `Default` is the C# field initializer (all
/// zero/false — `isPetting` false, `npc`/`proj` 0; the port uses -1 for the
/// unset slots like the C# constructors `:25`/`:35`/`:46-47`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PettingInfo {
    /// `isPetting` (`:7`).
    pub is_petting: bool,
    /// `npc` (`:9`) — the NPC slot, -1 when the target is a proj/mount.
    pub npc: i32,
    /// `proj` (`:11`) — the projectile slot, -1 on the NPC/mount paths.
    pub proj: i32,
    /// `type` (`:13`) — the target's NPC type / proj type / mount id.
    pub kind: i32,
    /// `mount` (`:15`) — the target is the player's own mount.
    pub mount: bool,
    /// `offsetFromPet` (`:17`) — the player's bottom sits at
    /// `target.Bottom + offset` (the mount path keeps Zero).
    pub offset_from_pet: (f32, f32),
    /// `isPetSmall` (`:19`) — the hold arm's 0.2-vs-0.3 rotation pick.
    pub is_pet_small: bool,
}

impl Default for PettingInfo {
    fn default() -> Self {
        Self {
            is_petting: false,
            npc: -1,
            proj: -1,
            kind: 0,
            mount: false,
            offset_from_pet: (0.0, 0.0),
            is_pet_small: false,
        }
    }
}

impl PettingInfo {
    /// The NPC constructor (`PlayerPettingInfo.cs:21-30`).
    pub fn from_npc(slot: i32, npc_type: i32, offset: (f32, f32), small: bool) -> Self {
        Self {
            is_petting: false,
            npc: slot,
            proj: -1,
            kind: npc_type,
            mount: false,
            offset_from_pet: offset,
            is_pet_small: small,
        }
    }

    /// The projectile constructor (`PlayerPettingInfo.cs:32-41`, W4-d21
    /// fold 2): `isPetting = false`, `proj = whoAmI`, `type = proj.type`,
    /// the offset + small flag ride (the NPC/mount lanes keep their own ctors).
    /// d21's K1 half, spliced S1 at the batch-L merge (the 13-DROP/10-KEEP
    /// compose contract, d21 ANALYSIS "Fold-2 keep-one adjudication"):
    /// the Pal-projectile petting path (foxsparks/digtoise/cattiva) keys
    /// on the projectile slot + type; the `get_petting_info` proj ladder
    /// (`projectile_ai_201_210.rs`, d21's lane) builds this — it passes the
    /// `u16` projectile type, so the cast to `kind` happens here.
    pub fn from_projectile(who_am_i: i32, kind: u16, offset: (f32, f32), small: bool) -> Self {
        Self {
            is_petting: false,
            npc: -1,
            proj: who_am_i,
            kind: kind as i32,
            mount: false,
            offset_from_pet: offset,
            is_pet_small: small,
        }
    }

    /// The mount constructor (`PlayerPettingInfo.cs:43-52`).
    pub fn from_mount(mount_id: i32, small: bool) -> Self {
        Self {
            is_petting: false,
            npc: -1,
            proj: -1,
            kind: mount_id,
            mount: true,
            offset_from_pet: (0.0, 0.0),
            is_pet_small: small,
        }
    }

    /// `TryGetTarget`'s NPC arm (`PlayerPettingInfo.cs:56-63`): the slot's
    /// NPC must be active AND still the same type. The port's liveness is
    /// the hp (the `npc_chat.rs` convention); the type check is the kind's
    /// vanilla id. Returns the target's bottom + the offset (the snap
    /// point) — `None` when the target is gone or swapped.
    pub fn npc_snap_point(&self, npcs: &[Npc]) -> Option<(f32, f32)> {
        if self.npc < 0 {
            return None;
        }
        let n = npcs.get(self.npc as usize)?;
        if n.hp <= 0 || n.kind.net_id() != self.kind {
            return None;
        }
        let (w, h) = n.size();
        Some((
            n.x + w * 0.5 + self.offset_from_pet.0,
            n.y + h + self.offset_from_pet.1,
        ))
    }
}

/// `NPC.GetPettingInfo` (`NPC.cs:80858-80892`): the side pick
/// (`Center.X > player.Center.X` → the player stands LEFT of the pet,
/// offset `-num·num2`), the per-type distance ladder and the small flag.
/// The default arm (num2 36, not small) is unreachable for the eleven
/// town pets — every one carries a case — but kept for the C# shape (the
/// interaction's Condition gates on `IsTownPet`, `NPCInteractions.cs:157`).
pub fn get_petting_info(npc: &Npc, slot: i32, player_center_x: f32) -> PettingInfo {
    let (cx, _) = npc.center();
    let num = if cx > player_center_x { 1 } else { -1 };
    // `:80861-80890` — the switch. 638 (the dog) keeps the default 36/not
    // small; 637 → 28, 656 → 24, 670/678/679/680/681/683 → 26, 682 → 22,
    // 684 → 20, all small.
    let (num2, small) = match npc.kind {
        NpcKind::TownCat => (28, true),
        NpcKind::TownBunny => (24, true),
        NpcKind::TownSlimeBlue
        | NpcKind::TownSlimeGreen
        | NpcKind::TownSlimeOld
        | NpcKind::TownSlimePurple
        | NpcKind::TownSlimeRainbow
        | NpcKind::TownSlimeYellow => (26, true),
        NpcKind::TownSlimeRed => (22, true),
        NpcKind::TownSlimeCopper => (20, true),
        _ => (36, false),
    };
    PettingInfo::from_npc(
        slot,
        npc.kind.net_id(),
        (-(num as f32) * num2 as f32, 0.0),
        small,
    )
}

/// The outcome of `Player.PetAnimal` — the caller drains the achievement
/// (the `pending_*` convention; the Player has no achievement manager).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PetAnimalOutcome {
    /// The snap landed: `PET_THE_PET` fires (`Player.cs:33395-33398`).
    Started,
    /// The repeat click at the same spot stopped the petting (`:33373-33377`).
    Stopped,
    /// A gate refused (no target / no snap / no floor) — nothing changed.
    Refused,
}

impl Player {
    /// `Player.StopPettingAnimal` (`Player.cs:33465-33468`).
    pub fn stop_petting_animal(&mut self) {
        self.petting.is_petting = false;
    }


    /// `Player.PetAnimal` (`Player.cs:33358-33399`): the target gate, the
    /// floor-snapped bottom (`(target.Bottom + offset).Floor()`), the
    /// `CanSnapToPosition` + `SolidTileAllowBottomSlope` gates, the repeat
    /// click's stop, the vanity/hook/mount teardown, the snap + facing +
    /// velocity zero + gravDir 1, then the petting latch. The achievement
    /// rides the outcome (the caller completes `PET_THE_PET`/`Do`).
    ///
    /// S2 (the d21 compose contract): `proj_bottom` is the resolved
    /// PROJECTILE-lane target bottom (`TryGetTarget`'s proj arm,
    /// `PlayerPettingInfo.cs:70-75` — the caller checks slot active + same
    /// type; `None` = no live proj target). The NPC arm keeps
    /// `npc_snap_point`; the mount arm keeps the player's own bottom.
    /// The facing change rides [`Self::change_dir`] (`:33390`), not a
    /// bare assign.
    ///
    /// Deviations (recorded): `CanSnapToPosition` rides the port's
    /// `can_snap_to` (the `Collision.CanHit` reach half is the standing
    /// stand-in, `player.rs:3027`); `RemoveAllGrapplingHooks` rides the
    /// `release_grappling_hooks` request (the pool drain owns the hooks);
    /// `gravDir` is always 1 in the port so the `:33392` write is a no-op.
    pub fn pet_animal(
        &mut self,
        world: &World,
        npcs: &[Npc],
        info: PettingInfo,
        proj_bottom: Option<(f32, f32)>,
    ) -> PetAnimalOutcome {
        // `:33360-33363` — TryGetTarget; the mount arm (target null) keeps
        // the player's own bottom, the NPC/proj arms snap to the target.
        let mut bottom = (self.x + self.width * 0.5, self.y + self.height);
        if !info.mount {
            // The proj lane (S2): the resolved projectile BOTTOM (raw —
            // the offset applies here like the C# `target.Bottom +
            // info.offsetFromPet`); the NPC lane: the roster snap point
            // (offset already folded by `npc_snap_point`). Neither → the
            // C#'s TryGetTarget-false early return.
            let target_bottom = if info.proj >= 0 {
                proj_bottom.map(|b| (b.0 + info.offset_from_pet.0, b.1 + info.offset_from_pet.1))
            } else {
                info.npc_snap_point(npcs)
            };
            let Some(snap) = target_bottom else {
                return PetAnimalOutcome::Refused;
            };
            // `:33367` — `.Floor()` (both components).
            bottom = (snap.0.floor(), snap.1.floor());
            // `:33368` — CanSnapToPosition(offset) + the floor tile.
            let offset = (bottom.0 - (self.x + self.width * 0.5), bottom.1 - (self.y + self.height));
            if !self.can_snap_to(world, offset.0, offset.1)
                || !crate::projectile_ai026_c::solid_tile_allow_bottom_slope(
                    world,
                    bottom.0 as i32 / 16,
                    bottom.1 as i32 / 16,
                )
            {
                return PetAnimalOutcome::Refused;
            }
        }
        // `:33373-33377` — the repeat click at the same bottom stops.
        let own_bottom = (self.x + self.width * 0.5, self.y + self.height);
        if self.petting.is_petting && own_bottom == bottom {
            self.stop_petting_animal();
            return PetAnimalOutcome::Stopped;
        }
        // `:33378-33383` — StopVanityActions + RemoveAllGrapplingHooks +
        // the non-mount petting dismounts an active mount.
        self.stop_petting_animal();
        self.sit_up();
        self.stop_sleeping();
        self.release_grappling_hooks = true;
        self.grappling_active = false;
        if !info.mount && self.mount.is_active() {
            let _ = self.try_dismount(world);
        }
        // `:33384-33392` — the snap, the facing (sign of -offset.X, the
        // player's own direction when 0), velocity zero, gravDir 1.
        self.x = bottom.0 - self.width * 0.5;
        self.y = bottom.1 - self.height;
        let mut num = math_sign(0.0 - info.offset_from_pet.0);
        if num == 0 {
            num = self.facing;
        }
        // `:33390` — ChangeDir (S2/K4: the pulley-2 reposition + the
        // composite/itemRotation mirrors), NOT the bare facing assign.
        self.change_dir(world, num);
        self.vx = 0.0;
        self.vy = 0.0;
        // `:33393-33394` — the latch.
        self.petting = info;
        self.petting.is_petting = true;
        // `:33395-33398` — the local-player achievement (the port is SP).
        self.pending_pet_achievement = true;
        PetAnimalOutcome::Started
    }

    /// `Player.PetMount` (`Player.cs:33339-33356`): already petting → stop;
    /// else latch + the achievement + the Chillemt joy cue (types 62-65,
    /// the `(uint)(type - 62) <= 3` gate). Returns true when the joy cue
    /// should play (the caller drains it — the `pending_chillet_sound`
    /// convention; the cue is `SoundID.PalChilletJoy`, NOT PalChillet).
    pub fn pet_mount(&mut self, info: PettingInfo) -> bool {
        if self.petting.is_petting {
            self.stop_petting_animal();
            return false;
        }
        self.petting = info;
        self.petting.is_petting = true;
        // `:33348-33351` — the local-player achievement (the port is SP).
        self.pending_pet_achievement = true;
        // `:33352-33355` — the unsigned-range gate: 62..=65.
        (self.petting.kind - 62) as u32 <= 3
    }

    /// `Player.UpdatePettingAnimal` (`Player.cs:33470-33490`): the per-tick
    /// stop gates — the target gone/swapped, the talk latch moved off the
    /// petted NPC, the bottom drifted > 2 px from the snap point, any
    /// movement control / pulley / a non-mount petting while mounted / the
    /// facing flipped off the pet side. `talk_npc` is the App's talk latch
    /// (`Option<usize>`, None = -1).
    ///
    /// S3 (the d21 compose contract): `proj_target` is the proj-lane
    /// `TryGetTarget` result (`PlayerPettingInfo.cs:70-75` — slot active +
    /// same type; the caller resolves it off the projectile pool, d21's
    /// `petting_target_bottom` feed): `Some(bottom)` = live proj target,
    /// `None` = gone/swapped. The `target != null` drift gate applies to
    /// the proj lane too (the C# checks any non-null target); the talk
    /// gate is NPC-only (`target is NPC`). The NPC/talk/mount arms stay
    /// g10's.
    pub fn update_petting_animal(
        &mut self,
        npcs: &[Npc],
        talk_npc: Option<usize>,
        proj_target: Option<(f32, f32)>,
    ) {
        if !self.petting.is_petting {
            return;
        }
        // `:33476` — TryGetTarget + the talk gate + the 2 px drift.
        let is_proj = !self.petting.mount && self.petting.proj >= 0;
        let snap = if self.petting.mount {
            // The mount arm's target is null → the drift check is skipped
            // (`target != null` gate), TryGetTarget returns true.
            None
        } else if is_proj {
            // S3 — the proj-lane TryGetTarget arm: gone/swapped stops.
            match proj_target {
                Some(b) => Some((b.0 + self.petting.offset_from_pet.0, b.1 + self.petting.offset_from_pet.1)),
                None => {
                    self.stop_petting_animal();
                    return;
                }
            }
        } else {
            match self.petting.npc_snap_point(npcs) {
                Some(s) => Some(s),
                None => {
                    self.stop_petting_animal();
                    return;
                }
            }
        };
        // The talk gate is NPC-only (`target is NPC`, `:33476`).
        if !self.petting.mount && !is_proj {
            let talk_matches = talk_npc.is_some_and(|t| t as i32 == self.petting.npc);
            if !talk_matches {
                self.stop_petting_animal();
                return;
            }
        }
        // The drift gate applies to ANY non-null target (NPC or proj).
        if let Some(s) = snap {
            let bottom = (self.x + self.width * 0.5, self.y + self.height);
            let d = ((bottom.0 - s.0).powi(2) + (bottom.1 - s.1).powi(2)).sqrt();
            if d > 2.0 {
                self.stop_petting_animal();
                return;
            }
        }
        // `:33481-33489` — the control/facing exit mask.
        let mut num = math_sign(0.0 - self.petting.offset_from_pet.0);
        if self.petting.mount {
            num = self.facing;
        }
        if self.control_left
            || self.control_right
            || self.control_up
            || self.control_down
            || self.control_jump
            || self.pulley
            || (self.mount.is_active() && !self.petting.mount)
            || num != self.facing
        {
            self.stop_petting_animal();
        }
    }
}

/// The `ItemCheck_ApplyHoldStyle` petting arm's back-arm pose
/// (`Player.cs:50243-50269`): the 14-tick pat cycle
/// (`miscCounter % 14 / 7` — 0 → ThreeQuarters, 1 → Full), the rotation
/// `-2π·k·direction` with k = 0.36 on the Chillemt mounts 62-65, else 0.3
/// (0.2 for small pets). Pure so the frame feed and the witness share it.
pub fn petting_back_arm(
    misc_counter: u32,
    mount_active: bool,
    mount_kind: i32,
    is_pet_small: bool,
    direction: i32,
) -> crate::player::CompositeArmState {
    let stretch = if misc_counter % 14 / 7 == 1 {
        crate::player::CompositeStretch::Full
    } else {
        crate::player::CompositeStretch::ThreeQuarters
    };
    let k = if mount_active && matches!(mount_kind, 62 | 63 | 64 | 65) {
        0.36
    } else if is_pet_small {
        0.2
    } else {
        0.3
    };
    crate::player::CompositeArmState {
        enabled: true,
        stretch,
        rotation: std::f32::consts::PI * -2.0 * k * direction as f32,
    }
}

#[cfg(test)]
mod tests {
    //! W4-g10 lane 3 FINAL (code-only): the petting witnesses, WRITTEN NOT
    //! RUN. The chain draws NO RNG (the module census), so the witnesses
    //! pin the state machine + the pose arithmetic, not a stream.
    use super::*;
    use crate::player::PlayerInput;

    /// A flat floor at y = 30, spawn on it.
    fn floor_world() -> World {
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.spawn_x = 20;
        w.spawn_y = 30;
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// A player resting on the floor (the spawn snap puts the feet at
    /// `spawn_y * 16`; the floor top is y = 30 * 16 = 480).
    fn resting_player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.y = 480.0 - p.height;
        p
    }

    /// A cat standing on the floor at tile 24 (feet y = 480).
    fn cat_at(x: f32) -> Npc {
        Npc::new(NpcKind::TownCat, x, 480.0 - 20.0, 1, 0.0)
    }

    /// `NPC.cs:80860-80891` — the side pick + the per-kind ladder.
    #[test]
    fn w4_g10_get_petting_info_picks_the_side_and_the_ladder() {
        let cat = cat_at(24.0 * 16.0);
        // The player LEFT of the cat: num = 1, offset -28 (the player
        // stands 28 px left of the cat's bottom-centre).
        let info = get_petting_info(&cat, 3, 100.0);
        assert_eq!(info.npc, 3);
        assert_eq!(info.kind, 637);
        assert!(info.is_pet_small);
        assert_eq!(info.offset_from_pet, (-28.0, 0.0));
        assert!(!info.is_petting, "the constructor starts un-latched");
        // The player RIGHT of the cat: num = -1, offset +28.
        let info = get_petting_info(&cat, 3, 900.0);
        assert_eq!(info.offset_from_pet, (28.0, 0.0));
        // The ladder: dog 36/not-small (the default arm), bunny 24, the
        // six slimes 26, red 22, copper 20.
        let ladder: [(NpcKind, f32, bool); 11] = [
            (NpcKind::TownCat, 28.0, true),
            (NpcKind::TownDog, 36.0, false),
            (NpcKind::TownBunny, 24.0, true),
            (NpcKind::TownSlimeBlue, 26.0, true),
            (NpcKind::TownSlimeGreen, 26.0, true),
            (NpcKind::TownSlimeOld, 26.0, true),
            (NpcKind::TownSlimePurple, 26.0, true),
            (NpcKind::TownSlimeRainbow, 26.0, true),
            (NpcKind::TownSlimeRed, 22.0, true),
            (NpcKind::TownSlimeYellow, 26.0, true),
            (NpcKind::TownSlimeCopper, 20.0, true),
        ];
        for (kind, dist, small) in ladder {
            let n = Npc::new(kind, 400.0, 460.0, 1, 0.0);
            let info = get_petting_info(&n, 0, 100.0);
            assert_eq!(info.offset_from_pet, (-dist, 0.0), "{kind:?} distance");
            assert_eq!(info.is_pet_small, small, "{kind:?} small flag");
        }
    }

    /// `Player.cs:33358-33399` — the snap: the player's bottom lands on
    /// `(cat.Bottom + offset).Floor()`, the facing flips toward the pet,
    /// the velocity zeroes, the latch sets, the achievement queues.
    #[test]
    fn w4_g10_pet_animal_snaps_to_the_pet_and_latches() {
        let w = floor_world();
        let mut p = resting_player(&w);
        let cat = cat_at(24.0 * 16.0);
        let npcs = [cat];
        let info = get_petting_info(&npcs[0], 0, p.x + p.width * 0.5);
        let out = p.pet_animal(&w, &npcs, info, None);
        assert_eq!(out, PetAnimalOutcome::Started);
        assert!(p.petting.is_petting);
        // The cat's bottom-centre x = 24*16 + 9 = 393; offset -28 → 365.
        let expect_bottom = (24.0 * 16.0 + 9.0 - 28.0, 480.0);
        assert_eq!(p.x + p.width * 0.5, expect_bottom.0);
        assert_eq!(p.y + p.height, expect_bottom.1);
        // The facing: sign(-offset.X) = sign(28) = +1 (toward the pet).
        assert_eq!(p.facing, 1);
        assert_eq!((p.vx, p.vy), (0.0, 0.0));
        assert!(p.take_pet_achievement(), "the PET_THE_PET request queues");
        assert!(!p.take_pet_achievement(), "the queue drains once");
    }

    /// `Player.cs:33373-33377` — the repeat click at the same bottom stops.
    #[test]
    fn w4_g10_the_repeat_click_stops_the_petting() {
        let w = floor_world();
        let mut p = resting_player(&w);
        let npcs = [cat_at(24.0 * 16.0)];
        let info = get_petting_info(&npcs[0], 0, p.x + p.width * 0.5);
        assert_eq!(p.pet_animal(&w, &npcs, info, None), PetAnimalOutcome::Started);
        let _ = p.take_pet_achievement();
        // The same info again: the bottom already sits at the snap point.
        assert_eq!(p.pet_animal(&w, &npcs, info, None), PetAnimalOutcome::Stopped);
        assert!(!p.petting.is_petting);
        assert!(!p.take_pet_achievement(), "no achievement on the stop");
    }

    /// `Player.cs:33360-33363` — a gone/swapped target refuses.
    #[test]
    fn w4_g10_a_gone_target_refuses() {
        let w = floor_world();
        let mut p = resting_player(&w);
        let npcs = [cat_at(24.0 * 16.0)];
        let mut info = get_petting_info(&npcs[0], 0, p.x + p.width * 0.5);
        info.kind = 638; // the slot holds a cat, the info says dog
        assert_eq!(p.pet_animal(&w, &npcs, info, None), PetAnimalOutcome::Refused);
        assert!(!p.petting.is_petting);
        // An empty roster refuses too.
        let info = get_petting_info(&npcs[0], 0, p.x + p.width * 0.5);
        assert_eq!(p.pet_animal(&w, &[], info, None), PetAnimalOutcome::Refused);
    }

    /// `Player.cs:33470-33490` — the per-tick stop gates: the talk latch
    /// moving off the pet, a movement control, the facing flip.
    #[test]
    fn w4_g10_update_petting_stops_on_talk_move_and_controls() {
        let w = floor_world();
        let mut p = resting_player(&w);
        let npcs = [cat_at(24.0 * 16.0)];
        let info = get_petting_info(&npcs[0], 0, p.x + p.width * 0.5);
        assert_eq!(p.pet_animal(&w, &npcs, info, None), PetAnimalOutcome::Started);
        let _ = p.take_pet_achievement();
        // The talk latch on the pet: the petting survives a calm tick.
        p.update_petting_animal(&npcs, Some(0), None);
        assert!(p.petting.is_petting, "calm tick keeps the petting");
        // The talk latch moved off: stop.
        p.update_petting_animal(&npcs, None, None);
        assert!(!p.petting.is_petting, "the talk gate stops it");
        // Re-latch, then a movement control stops it.
        assert_eq!(p.pet_animal(&w, &npcs, info, None), PetAnimalOutcome::Started);
        let _ = p.take_pet_achievement();
        p.control_left = true;
        p.update_petting_animal(&npcs, Some(0), None);
        assert!(!p.petting.is_petting, "controlLeft stops it");
        p.control_left = false;
        // Re-latch, then the target dies: stop.
        assert_eq!(p.pet_animal(&w, &npcs, info, None), PetAnimalOutcome::Started);
        let _ = p.take_pet_achievement();
        let mut dead = npcs;
        dead[0].hp = 0;
        p.update_petting_animal(&dead, Some(0), None);
        assert!(!p.petting.is_petting, "the dead target stops it");
    }

    /// `Player.cs:33476` — the > 2 px drift stops the petting.
    #[test]
    fn w4_g10_the_two_pixel_drift_stops_the_petting() {
        let w = floor_world();
        let mut p = resting_player(&w);
        let npcs = [cat_at(24.0 * 16.0)];
        let info = get_petting_info(&npcs[0], 0, p.x + p.width * 0.5);
        assert_eq!(p.pet_animal(&w, &npcs, info, None), PetAnimalOutcome::Started);
        let _ = p.take_pet_achievement();
        p.x += 3.0; // 3 px off the snap point
        p.update_petting_animal(&npcs, Some(0), None);
        assert!(!p.petting.is_petting, "the drift stops it");
    }

    /// `Player.cs:33339-33356` — PetMount: the toggle, the 62-65 joy gate,
    /// the achievement on the fresh latch only.
    #[test]
    fn w4_g10_pet_mount_toggles_and_gates_the_joy_cue() {
        let w = floor_world();
        let mut p = resting_player(&w);
        // A Chillemt (62): fresh latch → joy + achievement.
        p.pet_mount_click(62);
        assert!(p.petting.is_petting);
        assert!(p.petting.mount);
        assert!(p.take_petting_joy_sound().is_some(), "the joy cue queues");
        assert!(p.take_pet_achievement());
        // The repeat click stops — no joy, no achievement.
        p.pet_mount_click(62);
        assert!(!p.petting.is_petting);
        assert!(p.take_petting_joy_sound().is_none());
        assert!(!p.take_pet_achievement());
        // A non-Chillemt mount (10, the Unicorn): latch + achievement, NO joy.
        p.pet_mount_click(10);
        assert!(p.petting.is_petting);
        assert!(p.take_petting_joy_sound().is_none(), "no joy outside 62-65");
        assert!(p.take_pet_achievement());
        // The gate's edges: 65 joy, 66 none.
        p.stop_petting_animal();
        p.pet_mount_click(65);
        assert!(p.take_petting_joy_sound().is_some(), "65 is inside the gate");
        p.stop_petting_animal();
        p.pet_mount_click(66);
        assert!(p.take_petting_joy_sound().is_none(), "66 is outside the gate");
    }

    /// `Mount.cs:6163-6166` — the dismount stops the MOUNT petting only.
    #[test]
    fn w4_g10_the_dismount_stops_only_the_mount_petting() {
        let w = floor_world();
        let mut p = resting_player(&w);
        p.set_mount(62, false);
        p.pet_mount_click(62);
        assert!(p.petting.is_petting);
        let _ = p.take_pet_achievement();
        let _ = p.take_petting_joy_sound();
        p.dismount(&w, true);
        assert!(!p.petting.is_petting, "the mount petting stops on dismount");
        // An NPC petting while riding survives the dismount (the
        // `petting.mount` gate) — the mount-petting flag is false.
        let npcs = [cat_at(24.0 * 16.0)];
        let info = get_petting_info(&npcs[0], 0, p.x + p.width * 0.5);
        let _ = p.pet_animal(&w, &npcs, info, None);
        let _ = p.take_pet_achievement();
        p.set_mount(10, false);
        p.dismount(&w, true);
        assert!(p.petting.is_petting, "the NPC petting survives the dismount");
    }

    /// `Player.cs:50243-50269` — the pat arm: the 14-tick cycle, the
    /// 0.3/0.2/0.36 rotation picks, the direction mirror.
    #[test]
    fn w4_g10_the_pat_arm_cycles_and_picks_the_rotation() {
        use crate::player::CompositeStretch;
        let pi2 = std::f32::consts::PI * 2.0;
        // miscCounter 0..6 → ThreeQuarters, 7..13 → Full.
        for c in 0..14u32 {
            let arm = petting_back_arm(c, false, -1, false, 1);
            let want = if c % 14 / 7 == 1 { CompositeStretch::Full } else { CompositeStretch::ThreeQuarters };
            assert_eq!(arm.stretch, want, "counter {c}");
            assert!(arm.enabled);
            assert!((arm.rotation - (-pi2 * 0.3)).abs() < 1e-5, "counter {c} rotation");
        }
        // The small pet: 0.2. The Chillemt mount: 0.36 (overrides small).
        let small = petting_back_arm(0, false, -1, true, 1);
        assert!((small.rotation - (-pi2 * 0.2)).abs() < 1e-5);
        let chillemt = petting_back_arm(0, true, 63, true, 1);
        assert!((chillemt.rotation - (-pi2 * 0.36)).abs() < 1e-5);
        // A non-Chillemt mount keeps the small/big pick.
        let unicorn = petting_back_arm(0, true, 10, false, 1);
        assert!((unicorn.rotation - (-pi2 * 0.3)).abs() < 1e-5);
        // The direction mirror: -1 flips the sign.
        let left = petting_back_arm(0, false, -1, false, -1);
        assert!((left.rotation - (pi2 * 0.3)).abs() < 1e-5);
    }

    /// The full tick wires `update_petting_animal` at the C# site
    /// (`Player.cs:26124`): a control press inside `update_with_grapple`
    /// stops the petting on the same tick.
    #[test]
    fn w4_g10_the_tick_runs_the_petting_stop_gates() {
        let w = floor_world();
        let mut p = resting_player(&w);
        let npcs = [cat_at(24.0 * 16.0)];
        let info = get_petting_info(&npcs[0], 0, p.x + p.width * 0.5);
        assert_eq!(p.pet_animal(&w, &npcs, info, None), PetAnimalOutcome::Started);
        let _ = p.take_pet_achievement();
        // A calm tick with the talk latch on the pet keeps it.
        p.update_with_grapple(
            &w,
            PlayerInput::default(),
            &[],
            false, false, false, None,
            &mut tr_core::UnifiedRandom::default(),
            &npcs,
            Some(0),
            true,
        );
        assert!(p.petting.is_petting, "the calm tick keeps the petting");
        // A jump press stops it (controlJump, `:33486`).
        p.update_with_grapple(
            &w,
            PlayerInput { jump: true, ..Default::default() },
            &[],
            false, false, false, None,
            &mut tr_core::UnifiedRandom::default(),
            &npcs,
            Some(0),
            true,
        );
        assert!(!p.petting.is_petting, "the jump press stops the petting");
    }

    // ---- batch-L: the d21 compose-contract splices (S1/S2/S3/S5) --------
    // The 13-DROP/10-KEEP adjudication (d21 ANALYSIS "Fold-2 keep-one"):
    // g10's host keeps the machine; d21's unique proj-lane halves splice
    // in here. d21's own projectile wiring (try_petting, the 204 state-5
    // joy, the proj ladder) lands with batch-D — these witnesses pin the
    // g10-side seats it composes onto. WRITTEN NOT RUN.

    /// S1 — `from_projectile` (`PlayerPettingInfo.cs:32-41`): the ctor
    /// keys the proj slot + type, npc -1, mount false.
    #[test]
    fn batch_l_s1_from_projectile_seeds_the_proj_lane() {
        let info = PettingInfo::from_projectile(7, 1098, (-48.0, 0.0), true);
        assert_eq!(info.proj, 7);
        assert_eq!(info.npc, -1);
        assert_eq!(info.kind, 1098);
        assert!(!info.mount);
        assert!(!info.is_petting);
        assert_eq!(info.offset_from_pet, (-48.0, 0.0));
        assert!(info.is_pet_small);
    }

    /// S2 — `pet_animal`'s proj-lane target input: the resolved bottom +
    /// offset snaps like the NPC arm; a None feed refuses (TryGetTarget
    /// false); the mount arm ignores the feed.
    #[test]
    fn batch_l_s2_pet_animal_snaps_to_the_proj_bottom() {
        let w = floor_world();
        let mut p = resting_player(&w);
        // A digtoise-like proj bottom on the floor at (400, 480), offset -48
        // → 352. (`Player.PetAnimal`'s gate is
        // `SolidTileAllowBottomSlope((int)vector.X / 16, (int)vector.Y / 16)`,
        // `Player.cs:33368` — the tile AT the snapped bottom must be solid,
        // so the target bottom sits on the floor top 30*16, not 470.)
        let info = PettingInfo::from_projectile(0, 1098, (-48.0, 0.0), true);
        assert_eq!(
            p.pet_animal(&w, &[], info, None),
            PetAnimalOutcome::Refused,
            "no live proj target refuses"
        );
        assert_eq!(
            p.pet_animal(&w, &[], info, Some((400.0, 480.0))),
            PetAnimalOutcome::Started
        );
        assert_eq!(p.x + p.width * 0.5, 352.0);
        assert_eq!(p.y + p.height, 480.0);
        assert_eq!(p.facing, 1, "sign(-offset.X) toward the proj");
        assert!(p.petting.is_petting);
        assert_eq!(p.petting.proj, 0);
    }

    /// S2/K4 — `change_dir` (`Player.cs:42091-42116`): the same-dir no-op,
    /// the composite/itemRotation mirrors, the pulley-2 tile-centre
    /// reposition + the SolidCollision refusal.
    #[test]
    fn batch_l_s2_change_dir_mirrors_and_repositions() {
        let w = floor_world();
        let mut p = resting_player(&w);
        p.facing = 1;
        p.composite_back.rotation = 0.5;
        p.composite_front.rotation = -0.25;
        p.item_rotation = 0.75;
        // The same direction: NOTHING flips.
        p.change_dir(&w, 1);
        assert_eq!(p.composite_back.rotation, 0.5);
        assert_eq!(p.item_rotation, 0.75);
        // The flip mirrors all three.
        p.change_dir(&w, -1);
        assert_eq!(p.facing, -1);
        assert_eq!(p.composite_back.rotation, -0.5);
        assert_eq!(p.composite_front.rotation, 0.25);
        assert_eq!(p.item_rotation, -0.75);
        // Pulley-2: the tile-centre snap (`(int)(X + w/2)/16*16 + 8 - w/2`).
        let mut q = resting_player(&w);
        q.facing = 1;
        q.pulley = true;
        q.pulley_dir = 2;
        q.x = 330.0; // centre 340 → tile 21 → 21*16+8-10 = 334
        q.y = 200.0; // airborne, no solids
        q.change_dir(&w, -1);
        assert_eq!(q.pulley_dir, 1);
        assert_eq!(q.x, 334.0);
        assert_eq!(q.facing, -1);
        // The refusal: a solid box at the snap point keeps everything.
        let mut s = floor_world();
        for y in 12..16 {
            for x in 20..23 {
                let i = s.index(x, y);
                s.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
                s.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        let mut r = resting_player(&s);
        r.facing = 1;
        r.pulley = true;
        r.pulley_dir = 2;
        r.x = 330.0;
        r.y = 200.0; // the snap box overlaps the solid patch
        r.change_dir(&s, -1);
        assert_eq!(r.facing, 1, "the SolidCollision refusal keeps the dir");
        assert_eq!(r.pulley_dir, 2);
        assert_eq!(r.x, 330.0);
    }

    /// S3 — `update_petting_animal`'s proj TryGetTarget arm: the live
    /// proj bottom keeps the petting (NO talk gate — `target is NPC`
    /// only), the None feed stops it, the > 2 px drift applies.
    #[test]
    fn batch_l_s3_update_petting_reads_the_proj_target() {
        let w = floor_world();
        let mut p = resting_player(&w);
        let info = PettingInfo::from_projectile(0, 1098, (-48.0, 0.0), true);
        assert_eq!(
            p.pet_animal(&w, &[], info, Some((400.0, 480.0))),
            PetAnimalOutcome::Started
        );
        let _ = p.take_pet_achievement();
        // The live proj target keeps it — talk_npc None is NOT a stop
        // (the talk gate is NPC-only).
        p.update_petting_animal(&[], None, Some((400.0, 480.0)));
        assert!(p.petting.is_petting, "the proj lane has no talk gate");
        // The drift gate applies to the proj target too.
        p.x += 3.0;
        p.update_petting_animal(&[], None, Some((400.0, 480.0)));
        assert!(!p.petting.is_petting, "the 2 px drift stops the proj lane");
        // Re-latch; the gone target stops it.
        assert_eq!(
            p.pet_animal(&w, &[], info, Some((403.0, 480.0))),
            PetAnimalOutcome::Started
        );
        p.update_petting_animal(&[], None, None);
        assert!(!p.petting.is_petting, "TryGetTarget false stops it");
    }

    /// S5 — `math_sign`'s zero-at-zero (the C# `Math.Sign`): the offset-0
    /// facing pick keeps the player's own direction (d21's `signum`
    /// +1-at-zero deviation died with its D7/D8 twins).
    #[test]
    fn batch_l_s5_math_sign_is_zero_at_zero() {
        assert_eq!(math_sign(0.0), 0);
        assert_eq!(math_sign(-0.0), 0);
        assert_eq!(math_sign(0.5), 1);
        assert_eq!(math_sign(-0.5), -1);
        // The offset-0 pet_animal keeps the player's facing.
        let w = floor_world();
        let mut p = resting_player(&w);
        p.facing = -1;
        let info = PettingInfo::from_mount(10, false);
        // The mount arm: offset Zero → num 0 → the own direction stands.
        assert_eq!(p.pet_animal(&w, &[], info, None), PetAnimalOutcome::Started);
        assert_eq!(p.facing, -1, "offset 0 keeps the player's direction");
    }
}
