//! The banner kill tally + claimable counts — `BannerSystem`
//! (`BannerSystem.cs`): the per-banner kill counts (`killCount[]`), the
//! milestone every `ItemID.Sets.KillsToBanner[bannerItem]` kills
//! (`AddNPCKillBy` `:231-258` → `AddKill` `:271-278` + `AddClaimableBanner`
//! `:260-269`), the claimable-banner pool (`claimableBanners[]` /
//! `newBanner[]` / `AnyNewClaimableBanners`) and the claim request/response
//! chain (`HandleBannerClaimRequest` `:295-324` / `HandleBannerClaimResponse`
//! `:326-339`). The milestone does NOT drop the banner item at the death
//! spot — it increments the claimable count and announces the tally in
//! (250, 250, 0); the player claims the item through the BannerClaimingUI
//! (`RequestBannerClaim` `:280-284`). The placed-banner strength
//! (`ItemID.BannerEffect`'s classic 1.5/0.75 pair) rides `tr_core::bestiary`.
//!
//! W4-e24: the drop-rule gate chain (`NPC.NPCLoot`'s banner arms,
//! `NPC.cs:78830-78848`) — `GetWereThereAnyInteractions` (`:78823-78831`),
//! `ExcludedFromDeathTally` (`:79793-79808`), `CountKillForBannersAndDropThem`
//! (`:79810-79821`)'s `lastInteraction`/`FindClosestPlayer` plr pick and
//! `BannerID` (`:7644-7651`). The f31 index-space reconciliation (the
//! `NPCtoBanner` keying) is already landed — this lane keeps it untouched.

use std::collections::BTreeMap;

use tr_core::bestiary;

/// `Item.CommonMaxStack` (`Item.cs:49966`) — the claimable-banner cap
/// (`AddNPCKillBy`'s `claimableBanners[num] < Item.CommonMaxStack` gate,
/// `BannerSystem.cs:243`). The port's `crate::item::COMMON_MAX_STACK` is the
/// same 9999; re-stated here so the tally module reads it without the item
/// module's inventory seams.
const COMMON_MAX_STACK: u16 = 9999;

/// `BannerSystem.MaxBannerTypes` (`BannerSystem.cs:1453`) — the dense array
/// length the C# allocates; the port keys sparsely but the claim-list walk
/// (`BannerClaimingUI.UpdateAndGetClaimableItemsCount`) bounds at it.
pub const MAX_BANNER_TYPES: i32 = 293;

/// The `BannerSystem` state — the kill counts, the claimable counts and the
/// new-banner flags, all keyed sparsely by banner id (the C#'s dense
/// `killCount[293]` / `claimableBanners[293]` / `newBanner[293]` arrays,
/// `BannerSystem.cs:126-132`).
#[derive(Default)]
pub struct BannerSystem {
    kills: BTreeMap<i32, i32>,
    claimable: BTreeMap<i32, u16>,
    new_banner: BTreeMap<i32, bool>,
    /// `BannerSystem.AnyNewClaimableBanners` (`:132`).
    pub any_new_claimable: bool,
}

impl BannerSystem {
    /// `AddNPCKillBy` (`BannerSystem.cs:231-258`): count the kill, and on the
    /// `KillsToBanner` milestone add a claimable banner (capped at
    /// `Item.CommonMaxStack`) and return the announcement text. The C#
    /// builds a `NetworkText` — `Game.EnemiesDefeatedByAnnouncement`
    /// (`{0} has defeated the {1}th {2}!`) when `plr` is a valid player slot
    /// (`0 <= plr < 255`), else `Game.EnemiesDefeatedAnnouncement`
    /// (`The {0}th {1} has been defeated!`); the port resolves the same keys
    /// through `lang::text_args` with `Lang.GetNPCName(BannerToNPC(banner))`.
    /// The caller broadcasts it in the Command color (250, 250, 0).
    ///
    /// The `Main.netMode == 1` early-return (`:233-236`) is structural in the
    /// port: the death pass never runs on a client (`npcs_phase` gates
    /// `!is_client`, `app/frame/world.rs`), so a client never tallies.
    pub fn add_npc_kill_by(
        &mut self,
        npc_type: i32,
        plr: i32,
        player_name: &str,
    ) -> Option<String> {
        let banner = bestiary::npc_to_banner(npc_type);
        if banner <= 0 {
            return None;
        }
        self.add_kill(banner);
        let needed = bestiary::kills_to_banner_item(bestiary::banner_to_item(banner));
        let count = self.kills(banner);
        if count % needed != 0 {
            return None;
        }
        // `:243-246`: the claimable count caps at CommonMaxStack; the
        // announcement fires regardless (the C# announces even at the cap).
        if self.claimable(banner) < COMMON_MAX_STACK {
            self.add_claimable_banner(banner);
        }
        let net = bestiary::banner_to_npc(banner);
        let npc_name = crate::lang::npc_name(net);
        let text = if (0..255).contains(&plr) {
            crate::lang::text_args(
                "Game.EnemiesDefeatedByAnnouncement",
                &[player_name.to_string(), count.to_string(), npc_name],
            )
        } else {
            crate::lang::text_args(
                "Game.EnemiesDefeatedAnnouncement",
                &[count.to_string(), npc_name],
            )
        };
        Some(text)
    }

    /// `AddKill` (`BannerSystem.cs:271-278`): `killCount[banner]++`. The
    /// `netMode == 2` `WriteKillCountUpdate` broadcast is the banner net
    /// module's half (unported — the port's net glue carries the bestiary
    /// kill count, not the banner tally; ANALYSIS).
    fn add_kill(&mut self, banner: i32) {
        *self.kills.entry(banner).or_insert(0) += 1;
    }

    /// `AddClaimableBanner` (`BannerSystem.cs:260-269`):
    /// `newBanner[banner] |= claimableBanners[banner] == 0` (the FIRST
    /// claimable arms the new-banner shine), `AnyNewClaimableBanners |=
    /// newBanner[banner]`, then `claimableBanners[banner]++`.
    fn add_claimable_banner(&mut self, banner: i32) {
        let first = self.claimable(banner) == 0;
        let new = self.new_banner.entry(banner).or_insert(false);
        *new |= first;
        self.any_new_claimable |= *new;
        *self.claimable.entry(banner).or_insert(0) += 1;
    }

    /// `GetKillCount` (`BannerSystem.cs:134-137`).
    pub fn kills(&self, banner: i32) -> i32 {
        self.kills.get(&banner).copied().unwrap_or(0)
    }

    /// `GetClaimableBannerCounts()[banner]` (`BannerSystem.cs:139-142`).
    pub fn claimable(&self, banner: i32) -> u16 {
        self.claimable.get(&banner).copied().unwrap_or(0)
    }

    /// `IsNewBanner` (`BannerSystem.cs:144-147`).
    pub fn is_new_banner(&self, banner: i32) -> bool {
        self.new_banner.get(&banner).copied().unwrap_or(false)
    }

    /// `SetNewBanner` (`BannerSystem.cs:149-153`).
    pub fn set_new_banner(&mut self, banner: i32) {
        self.new_banner.insert(banner, true);
        self.any_new_claimable = true;
    }

    /// `ClearNewBanner` (`BannerSystem.cs:155-162`): clear the flag, then
    /// refresh `AnyNewClaimableBanners` if it was set.
    pub fn clear_new_banner(&mut self, banner: i32) {
        self.new_banner.insert(banner, false);
        if self.any_new_claimable {
            self.refresh_any_new_claimable();
        }
    }

    /// `RefreshAnyNewClaimableBanners` (`BannerSystem.cs:286-293`): re-OR
    /// every `newBanner` entry into the aggregate flag.
    fn refresh_any_new_claimable(&mut self) {
        self.any_new_claimable = self.new_banner.values().any(|&b| b);
    }

    /// `Clear` (`BannerSystem.cs:164-170`): zero all three arrays + the flag.
    pub fn clear(&mut self) {
        self.kills.clear();
        self.claimable.clear();
        self.new_banner.clear();
        self.any_new_claimable = false;
    }

    /// `HandleBannerClaimRequest` (`BannerSystem.cs:295-324`): the server
    /// half of a claim. `amount < 1` clamps to 1; `num = min(amount,
    /// claimable[banner])` is granted (the count decrements, the new-banner
    /// flag clears at zero), the remainder is refused. Returns
    /// `(granted, refused)` — the two `WriteClaimResponse` payloads the C#
    /// sends (`granted: true` for `num`, `granted: false` for the remainder).
    /// The single-player loopback applies both halves back to back.
    pub fn handle_banner_claim_request(&mut self, banner: i32, amount: u16) -> (u16, u16) {
        let mut amount = amount.max(1);
        let num = amount.min(self.claimable(banner));
        if num > 0 {
            *self.claimable.entry(banner).or_insert(0) -= num;
            amount -= num;
            if self.claimable(banner) == 0 {
                self.new_banner.insert(banner, false);
                if self.any_new_claimable {
                    self.refresh_any_new_claimable();
                }
            }
        }
        (num, amount)
    }

    /// The kill pairs in ascending banner-id order (`BannerSystem.Save`'s
    /// per-banner counts, `BannerSystem.cs:172-186`) — the world-save
    /// section's payload; the port's sparse map where the vanilla writes a
    /// dense array.
    pub fn pairs(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.kills.iter().map(|(&banner, &kills)| (banner, kills))
    }

    /// The claimable pairs in ascending banner-id order
    /// (`BannerSystem.Save`'s `claimableBanners` half, `:180-185`).
    pub fn claimable_pairs(&self) -> impl Iterator<Item = (i32, u16)> + '_ {
        self.claimable
            .iter()
            .map(|(&banner, &count)| (banner, count))
    }

    /// `BannerSystem.Load` (`BannerSystem.cs:188-212`): the recorded counts
    /// replace the tally. The C# keeps only the entries its count array
    /// carries and the port's map only the recorded ones — a recorded 0
    /// stays in the map so a zeroed banner round-trips.
    pub fn load_pairs(&mut self, pairs: impl IntoIterator<Item = (i32, i32)>) {
        for (banner, kills) in pairs {
            self.kills.insert(banner, kills);
        }
    }

    /// `BannerSystem.Load`'s `version >= 289` claimable half (`:203-211`).
    pub fn load_claimable_pairs(&mut self, pairs: impl IntoIterator<Item = (i32, u16)>) {
        for (banner, count) in pairs {
            self.claimable.insert(banner, count);
        }
    }
}

/// `NPC.AnyInteractions` (`NPC.cs:78722-78742`): in single-player
/// (`netMode == 0`) only the local player's (`Main.myPlayer`) interaction
/// flag counts; in multiplayer any of the 255 slots counts. The port's
/// death pass runs on the host / single-player (never a client), so the
/// local-slot read is the `netMode == 0` arm; a host reads its own slot the
/// same way (the C# host's `Main.myPlayer` is its own player slot).
pub fn any_interactions(player_interaction: &[bool; 256], local_slot: usize) -> bool {
    player_interaction.get(local_slot).copied().unwrap_or(false)
}

/// `NPC.GetWereThereAnyInteractions` (`NPC.cs:78823-78831`): the
/// `realLife >= 0` arm reads the CARRIER's `AnyInteractions`, else the
/// NPC's own. The port reads the dead clone's own `player_interaction`
/// array: `strike_pooled` (`npc.rs`, the `ApplyInteraction_Inner` port)
/// writes the interaction onto the CARRIER for a linked segment, so the
/// segment's own array stays empty and the gate correctly refuses it,
/// while the carrier's own array carries the strike and passes. The head
/// is its own carrier in the C# (`realLife = whoAmI`), so its own array is
/// the carrier's — the read agrees without a pool borrow. A no-interaction
/// death (the Golem parts' 9999 self-strike, the despawn kills) leaves the
/// array empty and the gate refuses, exactly like the C#.
pub fn get_were_there_any_interactions(npc: &crate::npc::Npc, local_slot: usize) -> bool {
    any_interactions(&npc.player_interaction, local_slot)
}

/// `NPC.BannerID` (`NPC.cs:7644-7651`): the net id when `netID >= -10`,
/// else the type. The port's bestiary net ids are all `>= -10` (the slime
/// family's -1..-10), so the banner id is the net id; the `type` fallback
/// is the C#'s deeper-negative arm (no ported kind reaches it).
pub fn banner_id(net_id: i32, npc_type: i32) -> i32 {
    if net_id >= -10 {
        net_id
    } else {
        npc_type
    }
}

/// `NPC.ExcludedFromDeathTally` (`NPC.cs:79793-79808`): the negative-net-id
/// arms (`-5` and `-1` excluded), the `NPCID.Sets.ProjectileNPC` set
/// (`NPCID.cs:10448`: 25, 30, 665, 33, 112, 666, 261, 265, 371, 516, 519)
/// and `NPCID.Sets.PositiveNPCTypesExcludedFromDeathTally`
/// (`NPCID.cs:10480`: 121, 384, 478, 479, 410, 472, 378). An excluded NPC
/// never counts toward a banner.
pub fn excluded_from_death_tally(net_id: i32) -> bool {
    if net_id < 0 {
        // `:79795-79802`: netID != -5 → return netID == -1; netID == -5 → true.
        return net_id == -5 || net_id == -1;
    }
    if projectile_npc(net_id) {
        return true;
    }
    positive_excluded_from_death_tally(net_id)
}

/// `NPCID.Sets.ProjectileNPC` (`NPCID.cs:10448`).
fn projectile_npc(net_id: i32) -> bool {
    matches!(net_id, 25 | 30 | 665 | 33 | 112 | 666 | 261 | 265 | 371 | 516 | 519)
}

/// `NPCID.Sets.PositiveNPCTypesExcludedFromDeathTally` (`NPCID.cs:10480`).
fn positive_excluded_from_death_tally(net_id: i32) -> bool {
    matches!(net_id, 121 | 384 | 478 | 479 | 410 | 472 | 378)
}

/// `NPC.IsNPCValidForBestiaryKillCredit` (`NPC.cs:78901-78914`): the
/// Slimer (121) never credits; the EoW segments (13/14/15) credit only
/// their boss head (`this.boss`); every other kind credits. The port reads
/// the live `NPC.boss` field — the EoW head's boss wire is the port's
/// (incomplete) boss table (`npc.rs`'s `boss` field doc), so a directly
/// constructed EoW head with `boss == false` does not credit until that
/// table lands (ANALYSIS "the banner drop-rule chain").
pub fn is_npc_valid_for_bestiary_kill_credit(npc_type: i32, boss: bool) -> bool {
    match npc_type {
        121 => false,
        13 | 14 | 15 => boss,
        _ => true,
    }
}

/// One `Main.player[i]` slot as `FindClosestPlayer` sees it — the
/// `active`/`dead`/`ghost` gates and the centre (`NPC.cs:77456-77492`).
#[derive(Clone, Copy, Debug)]
pub struct ClosestPlayerCandidate {
    pub active: bool,
    pub dead: bool,
    pub ghost: bool,
    pub cx: f32,
    pub cy: f32,
}

/// `NPC.FindClosestPlayer(out float)` (`NPC.cs:77456-77492`): the nearest
/// `active && !dead && !ghost` player by squared centre distance; if none
/// qualifies, the nearest `active` player (the dead/ghost fallback). `-1`
/// when no player is active. The port's single session player rides slot 0.
pub fn find_closest_player(center: (f32, f32), players: &[ClosestPlayerCandidate]) -> i32 {
    let mut best = f32::MAX;
    let mut pick = -1i32;
    for (i, p) in players.iter().enumerate() {
        if p.active && !p.dead && !p.ghost {
            let d = dist_sq(center, (p.cx, p.cy));
            if d < best {
                best = d;
                pick = i as i32;
            }
        }
    }
    if pick < 0 {
        for (i, p) in players.iter().enumerate() {
            if p.active {
                let d = dist_sq(center, (p.cx, p.cy));
                if d < best {
                    best = d;
                    pick = i as i32;
                }
            }
        }
    }
    pick
}

/// `Vector2.DistanceSquared` — the squared centre distance (the C# compares
/// squared distances, taking the sqrt only for the out param).
fn dist_sq(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    dx * dx + dy * dy
}

/// `FakeCursorItem` (`GameContent/FakeCursorItem.cs`): the pending-claim
/// cursor overlay the BannerClaimingUI shows while a claim is in flight.
/// `Add` (`:42-56`) sets/stacks the type; `Remove` (`:63-72`) decrements and
/// clears at zero. The `Item` getter adds `Main.mouseItem.stack` when the
/// cursor is non-air — the port's claim delivery reads the mouse item
/// separately, so this struct carries the fake half only.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeCursorItem {
    item_type: i32,
    stack: i32,
    prefix: i32,
}

impl FakeCursorItem {
    /// `FakeCursorItem.Add(itemType, itemStack, itemPrefix)` (`:42-56`):
    /// a zero stack is a no-op; the same type stacks, a different type
    /// replaces.
    pub fn add(&mut self, item_type: i32, item_stack: i32, item_prefix: i32) {
        if item_stack == 0 {
            return;
        }
        if self.item_type == item_type {
            self.stack += item_stack;
        } else {
            self.stack = item_stack;
        }
        self.item_type = item_type;
        self.prefix = item_prefix;
    }

    /// `FakeCursorItem.Remove(itemType, itemStack)` (`:63-72`): only the
    /// matching type decrements; `stack <= 0` clears the type.
    pub fn remove(&mut self, item_type: i32, item_stack: i32) {
        if item_stack != 0 && self.item_type == item_type {
            self.stack -= item_stack;
            if self.stack <= 0 {
                self.item_type = 0;
            }
        }
    }

    /// `FakeCursorItem.Reset` (`:36-40`).
    pub fn reset(&mut self) {
        self.item_type = 0;
        self.stack = 0;
    }

    pub fn item_type(&self) -> i32 {
        self.item_type
    }

    pub fn stack(&self) -> i32 {
        self.stack
    }

    pub fn prefix(&self) -> i32 {
        self.prefix
    }
}

/// `Player.GetBannerBuffEffect` (`Player.cs:20483-20493`) + the
/// `BannerEffect` classic sample: `Some((1.5, 0.75))` when the NPC's banner
/// stands in the scene window, `None` otherwise. `npc_banner_buff` lives
/// in the `NPCtoBanner` index space — the C# scan writes
/// `NPCBannerBuff[num3]` with the -21-shifted decode
/// (`SceneMetrics.cs:424-440`, the port's `banner_num_from_frame`), the
/// same space this reader's `npc_to_banner` lookup reads.
pub fn banner_buff_for(
    metrics: &tr_core::scene::SceneMetrics,
    net_id: i32,
) -> Option<(f32, f32)> {
    let banner = bestiary::npc_to_banner(net_id);
    if banner <= 0 {
        return None;
    }
    metrics
        .npc_banner_buff
        .get(banner as usize)
        .copied()
        .filter(|b| *b)
        .map(|_| (bestiary::BANNER_DAMAGE_DEALT, bestiary::BANNER_DAMAGE_RECEIVED))
}

/// `BannerSystem.RequestBannerClaim` (`BannerSystem.cs:280-284`): the client
/// half of a claim — `FakeCursorItem.Add(BannerToItem(banner), amount)` arms
/// the pending-claim cursor overlay, then the request goes to the server (the
/// port's single-player loopback applies the server half immediately, see
/// [`claim_banner`]). The `SoundEngine.PlaySound(7)` grab cue rides the
/// caller (`BannerClaimingUI.SendRequestForBanner`, `BannerClaimingUI.cs`).
pub fn request_banner_claim(app: &mut crate::app::App, banner: i32, amount: u16) {
    let item = bestiary::banner_to_item(banner);
    app.fake_cursor.add(item, amount as i32, 0);
    // The loopback: the server grants/refuses, then the client applies the
    // response — both halves back to back (the C# `SendToServerOrLoopback`
    // + `SendToClientOrLoopback` collapse to one call in single-player).
    claim_banner(app, banner, amount);
}

/// The single-player claim loopback — `HandleBannerClaimRequest`
/// (`BannerSystem.cs:295-324`, the server half) followed by
/// `HandleBannerClaimResponse` (`:326-339`, the client half) for both the
/// granted and refused payloads. The granted units fill the mouse item first
/// (`Main.mouseItem.TryAddStack`, only while the inventory is open and the
/// player is alive), the overflow drops at the player centre
/// (`Player.QuickSpawnItem(GetItemSource_InventoryOverflow(), ...)`); the
/// refused units only clear the fake-cursor overlay. Returns the granted
/// count (the caller's sound/UX hook).
///
/// RNG: the overflow drop's velocity/prefix draws ride the `UpdatePlayers`
/// stream (the claim is a player-inventory action, the `Player.ItemCheck`
/// identity — the extractinator's cursor-drop precedent,
/// `world_sim/extract.rs`); the C# `Item.NewItem` under `QuickSpawnItem`
/// reads the active swap, which for a UI claim is the player stage.
pub fn claim_banner(app: &mut crate::app::App, banner: i32, amount: u16) -> u16 {
    let (granted, refused) = app.banners.handle_banner_claim_request(banner, amount);
    let item = bestiary::banner_to_item(banner);
    // `HandleBannerClaimResponse(banner, granted, true)`: remove the granted
    // units from the fake cursor, then deliver them.
    if granted > 0 {
        app.fake_cursor.remove(item, granted as i32);
        deliver_claimed_banner(app, item as u16, granted);
    }
    // `HandleBannerClaimResponse(banner, refused, false)`: the refused units
    // only clear the fake cursor (no delivery).
    if refused > 0 {
        app.fake_cursor.remove(item, refused as i32);
    }
    granted
}

/// `HandleBannerClaimResponse`'s granted delivery (`BannerSystem.cs:330-338`):
/// `(!Main.playerInventory || localPlayer.dead || !mouseItem.TryAddStack(...))
/// && amountLeftToAdd > 0` → `QuickSpawnItem` the remainder. The port fills
/// the mouse item only while the inventory is open and the player is alive;
/// the overflow (or the whole grant when the cursor cannot take it) drops at
/// the player centre through the `Item.NewItem` scatter.
fn deliver_claimed_banner(app: &mut crate::app::App, item: u16, amount: u16) {
    let mut left = amount as u32;
    let cursor_can_take = app.inventory_open
        && app.player.as_ref().is_some_and(|p| !p.dead);
    if cursor_can_take {
        app.mouse_item.try_add_stack(item, &mut left);
    }
    if left > 0 {
        if let Some(p) = app.player.as_ref() {
            let (cx, cy) = p.center();
            crate::world_sim::items::spawn_item_drop_px(
                &mut app.items,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                item,
                left,
                glam::Vec2::new(cx, cy),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The announcement keys resolve through the shipped en-US resources;
    /// the tests hold the lang lock for the whole read (the `lang::test_lock`
    /// convention — a parallel culture switch would race the singleton).
    fn lang_guard() -> std::sync::MutexGuard<'static, ()> {
        let g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        g
    }

    #[test]
    fn the_milestone_lands_every_kills_to_banner() {
        let _g = lang_guard();
        // The blue slime needs 50 kills for the banner milestone; the 50th
        // returns the announcement, the 49th returns nothing.
        let mut sys = BannerSystem::default();
        let mut last = None;
        for _ in 0..49 {
            last = sys.add_npc_kill_by(1, 0, "Belsin");
        }
        assert_eq!(last, None);
        last = sys.add_npc_kill_by(1, 0, "Belsin");
        let text = last.take().expect("the 50th kill lands the milestone");
        // `Game.EnemiesDefeatedByAnnouncement` = "{0} has defeated the {1}th {2}!"
        assert_eq!(text, "Belsin has defeated the 50th Blue Slime!", "{text}");
        assert_eq!(sys.kills(69), 50);
        // The milestone made the banner CLAIMABLE, not a world drop.
        assert_eq!(sys.claimable(69), 1);
        assert!(sys.is_new_banner(69));
        assert!(sys.any_new_claimable);
        // The Pinky needs only 25 (`ItemID.cs:1304`'s override).
        let mut sys = BannerSystem::default();
        for _ in 0..25 {
            last = sys.add_npc_kill_by(-4, 0, "Belsin");
        }
        assert!(last.is_some());
        assert_eq!(sys.claimable(151), 1);
    }

    #[test]
    fn the_invalid_plr_announces_the_unnamed_form() {
        let _g = lang_guard();
        // plr 255 (the unset lastInteraction sentinel) → the plain
        // `Game.EnemiesDefeatedAnnouncement` = "The {0}th {1} has been defeated!"
        let mut sys = BannerSystem::default();
        let mut last = None;
        for _ in 0..50 {
            last = sys.add_npc_kill_by(1, 255, "Belsin");
        }
        assert_eq!(
            last.unwrap(),
            "The 50th Blue Slime has been defeated!"
        );
    }

    #[test]
    fn the_claimable_count_caps_at_common_max_stack() {
        let _g = lang_guard();
        // 50 kills per milestone; CommonMaxStack (9999) milestones saturate
        // the claimable count, but the tally keeps counting + announcing.
        let mut sys = BannerSystem::default();
        for _ in 0..(50 * (COMMON_MAX_STACK as i32 + 2)) {
            sys.add_npc_kill_by(1, 0, "Belsin");
        }
        assert_eq!(sys.claimable(69), COMMON_MAX_STACK, "the cap holds");
        assert_eq!(sys.kills(69), 50 * (COMMON_MAX_STACK as i32 + 2));
    }

    #[test]
    fn the_claim_request_grants_up_to_the_claimable_count() {
        // `HandleBannerClaimRequest`: amount clamps to >= 1, grants
        // min(amount, claimable), refuses the remainder.
        let mut sys = BannerSystem::default();
        sys.load_claimable_pairs([(69, 3)]);
        sys.set_new_banner(69);
        // Request 5 of 3: grant 3, refuse 2; the count zeroes + the flag clears.
        assert_eq!(sys.handle_banner_claim_request(69, 5), (3, 2));
        assert_eq!(sys.claimable(69), 0);
        assert!(!sys.is_new_banner(69));
        assert!(!sys.any_new_claimable);
        // A zero amount clamps to 1; an empty pool grants nothing.
        assert_eq!(sys.handle_banner_claim_request(69, 0), (0, 1));
        // A partial claim keeps the remainder + the flag.
        sys.load_claimable_pairs([(69, 4)]);
        sys.set_new_banner(69);
        assert_eq!(sys.handle_banner_claim_request(69, 2), (2, 0));
        assert_eq!(sys.claimable(69), 2);
        assert!(sys.is_new_banner(69), "a non-zero pool keeps the shine");
    }

    #[test]
    fn the_new_banner_flag_tracks_the_first_claimable() {
        let _g = lang_guard();
        let mut sys = BannerSystem::default();
        // The first milestone arms the shine; the second does not re-arm it
        // (the `|= claimable == 0` gate is false once the count is positive).
        for _ in 0..50 {
            sys.add_npc_kill_by(1, 0, "Belsin");
        }
        assert!(sys.is_new_banner(69));
        sys.clear_new_banner(69);
        assert!(!sys.is_new_banner(69));
        assert!(!sys.any_new_claimable);
        for _ in 0..50 {
            sys.add_npc_kill_by(1, 0, "Belsin");
        }
        // claimable was 1 (non-zero) at the second milestone → no re-shine.
        assert!(!sys.is_new_banner(69));
        assert_eq!(sys.claimable(69), 2);
    }

    #[test]
    fn the_death_tally_exclusions_match_the_csharp_sets() {
        // `ExcludedFromDeathTally` (NPC.cs:79793-79808).
        assert!(excluded_from_death_tally(-1), "the -1 arm");
        assert!(excluded_from_death_tally(-5), "the -5 arm");
        assert!(!excluded_from_death_tally(-3), "the green slime counts");
        assert!(!excluded_from_death_tally(-4), "the pinky counts");
        // ProjectileNPC set (NPCID.cs:10448).
        for id in [25, 30, 665, 33, 112, 666, 261, 265, 371, 516, 519] {
            assert!(excluded_from_death_tally(id), "projectile npc {id}");
        }
        // PositiveNPCTypesExcludedFromDeathTally (NPCID.cs:10480).
        for id in [121, 384, 478, 479, 410, 472, 378] {
            assert!(excluded_from_death_tally(id), "excluded positive {id}");
        }
        // A plain banner-carrying kind is not excluded.
        assert!(!excluded_from_death_tally(1), "the blue slime counts");
        assert!(!excluded_from_death_tally(3), "the zombie counts");
    }

    #[test]
    fn the_bestiary_credit_gate_matches_the_csharp() {
        // `IsNPCValidForBestiaryKillCredit` (NPC.cs:78901-78914).
        assert!(!is_npc_valid_for_bestiary_kill_credit(121, true));
        assert!(is_npc_valid_for_bestiary_kill_credit(13, true));
        assert!(!is_npc_valid_for_bestiary_kill_credit(13, false));
        assert!(!is_npc_valid_for_bestiary_kill_credit(14, false));
        assert!(!is_npc_valid_for_bestiary_kill_credit(15, false));
        assert!(is_npc_valid_for_bestiary_kill_credit(1, false));
    }

    #[test]
    fn the_banner_id_keeps_the_net_id_above_minus_ten() {
        // `BannerID` (NPC.cs:7644-7651).
        assert_eq!(banner_id(1, 1), 1);
        assert_eq!(banner_id(-4, 1), -4);
        assert_eq!(banner_id(-10, 1), -10);
        // Below -10 the type stands in (no ported kind reaches it).
        assert_eq!(banner_id(-11, 81), 81);
    }

    #[test]
    fn find_closest_player_prefers_the_living_then_falls_back() {
        let c = |active, dead, ghost, cx, cy| ClosestPlayerCandidate {
            active,
            dead,
            ghost,
            cx,
            cy,
        };
        // The living player wins over a nearer dead one.
        let players = [c(true, true, false, 0.0, 0.0), c(true, false, false, 100.0, 0.0)];
        assert_eq!(find_closest_player((0.0, 0.0), &players), 1);
        // No living player: the nearest active (dead) one is the fallback.
        let players = [c(true, true, false, 50.0, 0.0), c(true, true, false, 10.0, 0.0)];
        assert_eq!(find_closest_player((0.0, 0.0), &players), 1);
        // No active player at all: -1.
        let players = [c(false, false, false, 0.0, 0.0)];
        assert_eq!(find_closest_player((0.0, 0.0), &players), -1);
        // The ghost is skipped in the living pass.
        let players = [c(true, false, true, 0.0, 0.0), c(true, false, false, 5.0, 0.0)];
        assert_eq!(find_closest_player((0.0, 0.0), &players), 1);
    }

    #[test]
    fn fake_cursor_item_stacks_and_removes() {
        let mut f = FakeCursorItem::default();
        f.add(1683, 2, 0);
        assert_eq!((f.item_type(), f.stack()), (1683, 2));
        f.add(1683, 3, 0);
        assert_eq!(f.stack(), 5, "the same type stacks");
        f.add(1684, 1, 0);
        assert_eq!((f.item_type(), f.stack()), (1684, 1), "a new type replaces");
        f.remove(1684, 1);
        assert_eq!(f.item_type(), 0, "the zero stack clears the type");
        f.remove(9999, 5);
        assert_eq!(f.item_type(), 0, "a mismatched type is a no-op");
    }

    #[test]
    fn the_strength_needs_the_placed_banner() {
        let mut metrics = tr_core::scene::SceneMetrics::new();
        assert_eq!(banner_buff_for(&metrics, 1), None);
        metrics.npc_banner_buff[69] = true;
        assert_eq!(banner_buff_for(&metrics, 1), Some((1.5, 0.75)));
        // A net id without a banner never buffs.
        assert_eq!(banner_buff_for(&metrics, 50), None);
    }

    #[test]
    fn clear_zeroes_the_tally_and_the_pool() {
        let _g = lang_guard();
        let mut sys = BannerSystem::default();
        for _ in 0..50 {
            sys.add_npc_kill_by(1, 0, "Belsin");
        }
        assert_eq!(sys.kills(69), 50);
        assert_eq!(sys.claimable(69), 1);
        assert!(sys.any_new_claimable);
        // `BannerSystem.Clear` (BannerSystem.cs:164-170): all three arrays
        // zero, the aggregate flag down.
        sys.clear();
        assert_eq!(sys.kills(69), 0);
        assert_eq!(sys.claimable(69), 0);
        assert!(!sys.is_new_banner(69));
        assert!(!sys.any_new_claimable);
        assert_eq!(sys.pairs().count(), 0);
        assert_eq!(sys.claimable_pairs().count(), 0);
    }

    #[test]
    fn the_claimable_pairs_round_trip_the_load() {
        // `BannerSystem.Load`'s claimable half restores the pool; the pairs
        // walk ascending banner id (the save's deterministic order).
        let mut sys = BannerSystem::default();
        sys.load_claimable_pairs([(151, 2), (69, 1)]);
        let pairs: Vec<(i32, u16)> = sys.claimable_pairs().collect();
        assert_eq!(pairs, vec![(69, 1), (151, 2)]);
        assert_eq!(sys.claimable(69), 1);
        assert_eq!(sys.claimable(151), 2);
        // The load does NOT arm the new-banner shine (the C# Load only fills
        // claimableBanners; newBanner stays cleared on a world load).
        assert!(!sys.is_new_banner(69));
        assert!(!sys.any_new_claimable);
    }

    #[test]
    fn the_max_banner_types_matches_the_csharp_array() {
        // `BannerSystem.MaxBannerTypes` (BannerSystem.cs:1453) — the dense
        // array length the claim-list walk bounds at.
        assert_eq!(MAX_BANNER_TYPES, 293);
    }

    /// W4-e24 — the single-player claim loopback end to end: a milestone
    /// makes the banner claimable, `request_banner_claim` arms the fake
    /// cursor and delivers the granted units to the mouse item (the
    /// inventory is open, the player alive), the pool decrements and the
    /// fake cursor clears. The overflow path (closed inventory) drops at
    /// the player centre instead.
    #[test]
    fn w4_e24_the_claim_loopback_delivers_to_the_cursor() {
        let _g = lang_guard();
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = crate::test_world_helpers::ground();
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 100.0 * tr_core::TILE_PX;
        player.y = 110.0 * tr_core::TILE_PX;
        app.player = Some(player);
        app.inventory_open = true;
        // 50 Blue Slime kills → the Slime Banner (69) claimable.
        for _ in 0..50 {
            app.banners.add_npc_kill_by(1, 0, "Belsin");
        }
        assert_eq!(app.banners.claimable(69), 1);
        // The claim: the fake cursor arms then clears, the banner lands on
        // the mouse item (1683 = BannerToItem(69)).
        request_banner_claim(&mut app, 69, 1);
        assert_eq!(app.banners.claimable(69), 0);
        assert_eq!(app.fake_cursor.item_type(), 0, "the loopback clears the overlay");
        assert_eq!((app.mouse_item.kind, app.mouse_item.count), (1683, 1));
        assert!(app.items.is_empty(), "no overflow drop — the cursor took it");
    }

    #[test]
    fn w4_e24_the_claim_overflow_drops_at_the_player_centre() {
        let _g = lang_guard();
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = crate::test_world_helpers::ground();
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 100.0 * tr_core::TILE_PX;
        player.y = 110.0 * tr_core::TILE_PX;
        app.player = Some(player);
        // The inventory is CLOSED: the cursor cannot take the grant, so the
        // whole amount drops at the player centre (`QuickSpawnItem`).
        app.inventory_open = false;
        for _ in 0..50 {
            app.banners.add_npc_kill_by(1, 0, "Belsin");
        }
        let granted = claim_banner(&mut app, 69, 1);
        assert_eq!(granted, 1);
        assert_eq!(app.banners.claimable(69), 0);
        assert_eq!(app.mouse_item.count, 0, "the closed inventory takes nothing");
        let drop = app
            .items
            .iter()
            .find(|d| d.kind == 1683)
            .expect("the overflow drops the banner");
        assert_eq!(drop.count, 1);
        // The overflow's scatter rides the `UpdatePlayers` stream (the
        // player-inventory action's swap, the extractinator cursor-drop
        // precedent): the banner 1683 is non-prefixable, so the drop pays
        // exactly the three velocity draws and nothing else.
        use crate::app::RngKey;
        let mut witness = app.rngs.stream(RngKey::UpdatePlayers).clone();
        let _r1 = witness.next_float();
        let _r2 = witness.next_float();
        let _r3 = witness.next_float();
        let mut twin = app.rngs.stream(RngKey::UpdatePlayers).clone();
        crate::world_sim::items::spawn_item_drop_px(
            &mut Vec::new(),
            &mut twin,
            1683,
            1,
            glam::Vec2::new(0.0, 0.0),
        );
        assert_eq!(
            twin, witness,
            "the overflow paid exactly the three velocity draws on UpdatePlayers"
        );
    }
}
