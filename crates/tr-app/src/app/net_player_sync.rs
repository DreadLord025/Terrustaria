//! W4-c28 — the PLAYER-STATE `SendData` cadence (the send path).
//!
//! The C# client fans its own state out over three send paths, all ported
//! here as pure decision functions in the wave conventions of
//! `net_npc_sync.rs` (wave B) / `net_proj_sync.rs` (wave C):
//!
//! | path | C# | cadence |
//! | --- | --- | --- |
//! | input change | `Player.TrySyncingInput` (`Player.cs:6852-6916`, called `:6732`/`:25288`) | the 12-field compare vs `Main.clientPlayer`; one `13` per changed tick |
//! | state change | `Main.TrySyncingMyPlayer` (`Main.cs:17545-17703`, called `:17192` at netMode 1) | per-tick clone-compare: `5` per slot, `33` close flush, `40`, `142`, `36`, `16`/`42` (60-tick throttles), `50`+`13`, `99`/`115`, `13` (direction/shield/snapstone), `138` |
//! | heartbeat | `Main.UpdateClient` (`Main.cs:64516-64553`) | `netPlayCounter++` wrap `>3600`; `%420` → one `13`; `%900` → `36`,`16`,`40` in that order |
//!
//! The clone bookkeeping: `13`'s writer copies the control fields into
//! `Main.clientPlayer` (`NetMessage.cs:510-512` → `Player.OnControlsSynced`
//! `Player.cs:54435-54459`); `TrySyncingMyPlayer`'s tail copies the whole
//! synced set (`Player.clientClone`, `Player.cs:54461-54506`). The port keeps
//! one [`ClientClone`] mirroring both.
//!
//! Event sites ported as helpers: `Hurt`'s `13`(knockback)+`16` pair
//! (`Player.cs:38369-38377`), `HealEffect`'s `35` (`:4519-4528`), the
//! stealth-edge `84`s (`:25744-25876`, `:44134-44141`, `:44469-44476`) and
//! `TrySwitchingLoadout`'s `147` (`:5661-5674`).
//!
//! The `36 SyncPlayerZone` / `18 SetTime` BODIES are W4-c27's lane
//! (`net.rs`'s `PlayerZone`/`SetTime` + apply arms); this module carries the
//! send DECISIONS only — the intents name the fields the c27 bodies will
//! carry, so the two lanes merge without a wire change here.

#![allow(dead_code)] // the event-site helpers stay test-exercised until their callers land

use crate::net;

/// One send intent, in the C#'s emission order. The caller maps each to the
/// existing `net.rs` message by its role; `13` rides `net::PlayerState`,
/// `16` `PlayerHp`, `42` `PlayerMana`, `50` `PlayerBuffs`, `5` `Inventory`,
/// `33` `PlayerChest`, `35`/`40`/`84`/`147`/`36` the ids' bodies (36 = c27).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PlayerSend {
    /// `13 PlayerControls` (`NetMessage.cs:446-513`).
    Controls,
    /// `16 PlayerLifeMana` (`NetMessage.cs:519-523`).
    Life { hp: i16, max_hp: i16 },
    /// `42 PlayerMana` (`NetMessage.cs:961-965`).
    Mana { mana: i16, max_mana: i16 },
    /// `35 PlayerHeal` (`NetMessage.cs:929-931`).
    Heal { amount: i16 },
    /// `36 SyncPlayerZone` (`NetMessage.cs:933-944`) — the c27 body's fields.
    Zone {
        zone1: u8,
        zone2: u8,
        zone3: u8,
        zone4: u8,
        zone5: u8,
        town_npcs: u8,
    },
    /// `40 TalkNPC` (`NetMessage.cs:952-955`): `(short)talkNPC`.
    TalkNpc { npc: i16 },
    /// `50 PlayerBuffs` (`NetMessage.cs:996-1008`): the live ids, 0-terminated.
    Buffs { ids: Vec<u16> },
    /// `84 PlayerStealth` (`NetMessage.cs:1197-1203`): the float.
    Stealth { stealth: f32 },
    /// `147 SyncLoadout` (`NetMessage.cs:1624-1628`).
    Loadout { loadout: u8, hide_accessory: u16 },
    /// `5 PlayerInventory` (`NetMessage.cs:147-197`): one changed slot.
    Inventory { slot: i16 },
    /// `33 SyncPlayerChest`'s close-transition arm (`Main.cs:17575-17593`):
    /// `chest` is the LIVE (negative) index the writer carries; `name`
    /// present = the `editedChestName` flush (the `number2 == 1f` arm, text
    /// from the clone's chest).
    ChestClose { chest: i16, name: Option<String> },
    /// `99 MinionRestTarget` / `115 MinionAttackTarget` (`Main.cs:17677-17684`)
    /// — unported bodies (M6-NET-5); the decision is recorded, no wire yet.
    MinionTarget { attack: bool },
    /// `138 ClientSyncedInventory` (`Main.cs:17697-17700`) — a bodyless ping.
    InventorySynced,
}

/// The 12 fields `TrySyncingInput` compares (`Player.cs:6858-6905`) plus the
/// `cloneSyncedControls` extras the `13` writer copies (`:54441-54459`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ControlState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub use_item: bool,
    pub selected_item: i32,
    pub auto_reuse_all: bool,
    pub down_hold: bool,
    pub dash: bool,
    pub operating_entity: bool,
    pub last_use_success: bool,
    // The writer-copied extras (`Player.cs:54442`/`:54456-54458`).
    pub direction: i32,
    pub shield_raised: bool,
    pub snap_stone_light: bool,
}

/// `Player.TrySyncingInput` (`Player.cs:6852-6916`): netMode 1 only, the
/// 12-field compare vs the clone; ONE `13` when any differs. The compare
/// order is the source's (it only sets a flag, so order is behaviour-free).
pub(crate) fn try_syncing_input(now: &ControlState, clone: &ControlState, net_mode: u8) -> bool {
    if net_mode != 1 {
        return false; // `:6854`
    }
    now.up != clone.up
        || now.down != clone.down
        || now.left != clone.left
        || now.right != clone.right
        || now.jump != clone.jump
        || now.use_item != clone.use_item
        || now.selected_item != clone.selected_item
        || now.auto_reuse_all != clone.auto_reuse_all
        || now.down_hold != clone.down_hold
        || now.dash != clone.dash
        || now.operating_entity != clone.operating_entity
        || now.last_use_success != clone.last_use_success
}

/// `Main.UpdateClient`'s counter block (`Main.cs:64522-64536`): the ++/wrap
/// then the two modulo gates. Returns `(send_13, send_36_16_40)` — the trio
/// fires in the order 36, 16, 40 (`:64533-64535`).
pub(crate) fn update_client_beat(counter: &mut u32) -> (bool, bool) {
    *counter += 1; // `:64522`
    if *counter > 3600 {
        *counter = 0; // `:64523-64526` — NOTE: `>`, not `% == 0` (the server's)
    }
    (*counter % 420 == 0, *counter % 900 == 0)
}

/// The `netLife`/`netMana` throttle pair (`Main.cs:17636-17661`): a change
/// arms the flag; while the 60-tick timer runs it only decrements; at zero a
/// flagged stat sends ONE report and re-arms the timer to 60.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct StatThrottle {
    /// `Player.netLife` / `netMana` (`Player.cs:960-962`).
    pub pending: bool,
    /// `Player.netLifeTime` / `netManaTime` (`Player.cs:956-958`).
    pub time: i32,
}

impl StatThrottle {
    /// One tick of the `Main.cs:17636-17648` block. `changed` is the
    /// stat-vs-clone compare; returns whether the report fires THIS tick.
    pub(crate) fn step(&mut self, changed: bool) -> bool {
        if changed {
            self.pending = true; // `:17638`/`:17651`
        }
        if self.time > 0 {
            self.time -= 1; // `:17642`/`:17655`
            false
        } else if self.pending {
            self.pending = false;
            self.time = 60; // `:17646`/`:17659`
            true
        } else {
            false
        }
    }
}

/// The zone compare (`Main.cs:17607-17635`): five BitsBytes + `townNPCs`,
/// any difference sends `36`.
pub(crate) fn zone_changed(
    now: (u8, u8, u8, u8, u8, u8),
    clone: (u8, u8, u8, u8, u8, u8),
) -> bool {
    now != clone
}

/// The port's `tr_core::scene::Zones` snapshot packed into the C# zone
/// BitsBytes (`Player.cs:3427-3821`'s property slots): zone1 bit0 Dungeon,
/// bit1 Corrupt, bit2 Hallow, bit3 Meteor, bit4 Jungle, bit5 Snow,
/// bit6 Crimson; zone2 bit5 Desert, bit6 Glowshroom; zone4 bit5 Temple.
/// The unported zones (meteor/beach/graveyard/shimmer/…) ride 0 — the
/// scene-metrics lane fills them (recorded in ANALYSIS).
pub(crate) fn zone_bytes(z: &tr_core::scene::Zones, shimmer: bool) -> (u8, u8, u8, u8, u8, u8) {
    let mut z1 = 0u8;
    for (bit, on) in [
        (0, z.dungeon),
        (1, z.corrupt),
        (2, z.hallow),
        (4, z.jungle),
        (5, z.snow),
        (6, z.crimson),
    ] {
        if on {
            z1 |= 1 << bit;
        }
    }
    let mut z2 = 0u8;
    if z.desert {
        z2 |= 1 << 5;
    }
    if z.glowshroom {
        z2 |= 1 << 6;
    }
    let mut z4 = 0u8;
    if z.temple {
        z4 |= 1 << 5;
    }
    let z5 = u8::from(shimmer); // zone5 bit0 = ZoneShimmer (`Player.cs:3811`)
    (z1, z2, 0, z4, z5, 0)
}

/// `Player.Hurt`'s client tail (`Player.cs:38369-38377`): netMode 1, local,
/// not quiet — the knockback-gated `13` FIRST (`!noKnockback &&
/// hitDirection != 0 && (!mount.Active || !mount.Cart)`), then `16`. The
/// caller (`player_sync_cadence`) collapses the three-clause gate into the
/// `no_knockback`/`hit_direction` pair from the `pending_hurt_net` queue.
/// (`SendPlayerHurt`'s `102` is unported — M6-NET-5.)
pub(crate) fn hurt_sends(
    net_mode: u8,
    quiet: bool,
    no_knockback: bool,
    hit_direction: i32,
    hp: i32,
    max_hp: i32,
    out: &mut Vec<PlayerSend>,
) {
    if net_mode != 1 || quiet {
        return;
    }
    if !no_knockback && hit_direction != 0 {
        out.push(PlayerSend::Controls); // `:38373`
    }
    out.push(PlayerSend::Life {
        hp: hp.clamp(0, i16::MAX as i32) as i16,
        max_hp: max_hp.clamp(0, i16::MAX as i32) as i16,
    }); // `:38375`
}

/// `Player.HealEffect` (`Player.cs:4519-4528`): broadcast + netMode 1 + local
/// → one `35` with the heal amount.
pub(crate) fn heal_effect(
    amount: i32,
    broadcast: bool,
    net_mode: u8,
    out: &mut Vec<PlayerSend>,
) {
    if broadcast && net_mode == 1 {
        out.push(PlayerSend::Heal {
            amount: amount.clamp(0, i16::MAX as i32) as i16,
        });
    }
}

/// The stealth-edge `84` sites: the C# sends only on the 0/1 REACH edges
/// (`stealth == 0 && num != stealth`, `Player.cs:25841-25844`'s vortex pair;
/// the shroomite/psychoscope fade-outs `:25750-25755`/`:25797-25802` send at
/// the `<= 0` clamp), plus the 3106 stealth-reset `1f` writes
/// (`:44134-44141`, `:44469-44476`). Returns whether an edge fired.
pub(crate) fn stealth_edge(
    before: f32,
    after: f32,
    net_mode: u8,
    out: &mut Vec<PlayerSend>,
) -> bool {
    let edge = (after == 0.0 && before != 0.0) || (after == 1.0 && before != 1.0);
    if edge && net_mode == 1 {
        out.push(PlayerSend::Stealth { stealth: after });
    }
    edge
}

/// `Player.TrySwitchingLoadout`'s send (`Player.cs:5674`): one `147` with the
/// new index + the visibility mask. The swap gate itself is the caller's.
pub(crate) fn loadout_switch(
    loadout: u8,
    hide_accessory: u16,
    net_mode: u8,
    out: &mut Vec<PlayerSend>,
) {
    if net_mode == 1 {
        out.push(PlayerSend::Loadout {
            loadout,
            hide_accessory,
        });
    }
}

/// `Main.TrySyncingMyPlayer`'s close-transition `33` (`Main.cs:17575-17593`):
/// the clone's chest was open (`>= 0`), the live one is closed (`< 0`). The
/// writer's `number` is the LIVE chest (`Main.player[myPlayer].chest`,
/// negative — so `NetMessage.cs:884-889` writes coords 0/0); the
/// `editedChestName` flag picks the named arm, whose text reads
/// `Main.chest[player.chest]` — the CLONE's chest (`:17581`), `None` when
/// that array slot is null (`:17585-17587`). The flag clears on the named
/// arm only. Returns the intent when the edge fires.
pub(crate) fn chest_close_flush(
    live_chest: i32,
    clone_chest: i32,
    edited_name: bool,
    name: impl FnOnce() -> Option<String>,
) -> Option<PlayerSend> {
    if !(live_chest < 0 && clone_chest >= 0) {
        return None;
    }
    Some(PlayerSend::ChestClose {
        chest: live_chest as i16,
        name: if edited_name { name() } else { None },
    })
}

/// `Item.IsNetStateDifferent`'s field set as the clone carries it
/// (`Player.clientCloneItem`, `Player.cs:54517-54523`: type/stack/prefix/
/// favorited). The port's chest/inventory slots compare on the same four.
pub(crate) fn item_net_state(slot: &crate::item::ItemSlot, favorited: bool) -> (u16, u32, u16, bool) {
    (slot.kind, slot.count, slot.prefix, favorited)
}

/// The `50` body's live-id list (`NetMessage.cs:996-1008`): every
/// `buffType > 0` in slot order (the writer does NOT gate on time).
pub(crate) fn buff_ids(buffs: &crate::buffs::BuffSlots) -> Vec<u16> {
    buffs
        .buff_type
        .iter()
        .filter(|t| **t > 0)
        .map(|t| *t as u16)
        .collect()
}

/// The `Main.cs:17663-17676` buff compare: ANY slot's `buffType` differs →
/// `50` then `13`, in that order (times are not compared).
pub(crate) fn buff_list_changed(
    now: &crate::buffs::BuffSlots,
    clone: &[i32; crate::buffs::MAX_BUFFS],
    out: &mut Vec<PlayerSend>,
) {
    if now.buff_type.iter().zip(clone.iter()).any(|(a, b)| a != b) {
        out.push(PlayerSend::Buffs { ids: buff_ids(now) });
        out.push(PlayerSend::Controls);
    }
}

/// The full `TrySyncingMyPlayer` decision pass (`Main.cs:17545-17703`), minus
/// the arrays the caller diffs (the `5`s) — this carries the scalar tail in
/// the source's order: 33-close, 40, 36, 16, 42, 50+13, 99, 115,
/// 13-direction, 13-shield, 13-snapstone, 138. The caller runs the `5` array
/// diffs FIRST (`:17547-17574`) and passes `synced_any_inventory`. The
/// `142` (voidLensChest/piggyBankProjTracker, `:17598-17601`) and `122`
/// (tileEntityAnchor, `:17602-17605`) arms have no port state — recorded,
/// not emitted (M6-NET-5).
#[derive(Clone, Debug)]
pub(crate) struct ClientClone {
    pub controls: ControlState,
    pub zones: (u8, u8, u8, u8, u8, u8),
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub life: StatThrottle,
    pub mana_throttle: StatThrottle,
    pub buff_type: [i32; crate::buffs::MAX_BUFFS],
    pub chest: i32,
    pub talk_npc: i32,
    pub minion_rest: (f32, f32),
    pub minion_attack: i32,
}

impl Default for ClientClone {
    /// `Main.clientPlayer = new Player()` (`Main.cs:67158`): the clone starts
    /// at the C# ctor's values — `statLife = statLifeMax = 100`
    /// (`Player.cs:1924-1928`), `statMana = 0` / `statManaMax = 20`
    /// (`:1930` + the ctor), `direction = 1` (`Entity.cs:20`), `chest = -1`
    /// (`Player.cs:2876`), `talkNPC = -1` (the ctor), `MinionAttackTargetNPC
    /// = -1` (`:3008`) — NOT an all-zero struct: a zero-hp live player must
    /// compare DIFFERENT on the first tick (the C# sends its 16 then).
    fn default() -> Self {
        Self {
            controls: ControlState {
                direction: 1,
                ..Default::default()
            },
            zones: (0, 0, 0, 0, 0, 0),
            hp: 100,
            max_hp: 100,
            mana: 0,
            max_mana: 20,
            life: StatThrottle::default(),
            mana_throttle: StatThrottle::default(),
            buff_type: [0; crate::buffs::MAX_BUFFS],
            chest: -1,
            talk_npc: -1,
            minion_rest: (0.0, 0.0),
            minion_attack: -1,
        }
    }
}

/// The scalar tail of `TrySyncingMyPlayer` in emission order. `now_*` are the
/// live values; the clone updates in place (the `clientClone` tail,
/// `Main.cs:17702`). `chest_name` resolves the edited-name flush's source.
#[allow(clippy::too_many_arguments)]
pub(crate) fn try_syncing_my_player_tail(
    clone: &mut ClientClone,
    zones: (u8, u8, u8, u8, u8, u8),
    hp: i32,
    max_hp: i32,
    mana: i32,
    max_mana: i32,
    buffs: &crate::buffs::BuffSlots,
    live_chest: i32,
    edited_chest_name: bool,
    chest_name: impl FnOnce() -> Option<String>,
    talk_npc: i32,
    minion_rest: (f32, f32),
    minion_attack: i32,
    controls: &ControlState,
    synced_any_inventory: bool,
    out: &mut Vec<PlayerSend>,
) {
    // `:17575-17593` — the close-transition 33 (the flag clears on the send).
    if let Some(send) = chest_close_flush(live_chest, clone.chest, edited_chest_name, chest_name) {
        out.push(send);
    }
    // `:17594-17597` — the talk-NPC change.
    if talk_npc != clone.talk_npc {
        out.push(PlayerSend::TalkNpc {
            npc: talk_npc.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        });
    }
    // `:17607-17635` — the zone compare → 36.
    if zone_changed(zones, clone.zones) {
        let (zone1, zone2, zone3, zone4, zone5, town_npcs) = zones;
        out.push(PlayerSend::Zone {
            zone1,
            zone2,
            zone3,
            zone4,
            zone5,
            town_npcs,
        });
    }
    // `:17636-17661` — the two 60-tick stat throttles.
    if clone.life.step(hp != clone.hp || max_hp != clone.max_hp) {
        out.push(PlayerSend::Life {
            hp: hp.clamp(0, i16::MAX as i32) as i16,
            max_hp: max_hp.clamp(0, i16::MAX as i32) as i16,
        });
    }
    if clone
        .mana_throttle
        .step(mana != clone.mana || max_mana != clone.max_mana)
    {
        out.push(PlayerSend::Mana {
            mana: mana.clamp(0, i16::MAX as i32) as i16,
            max_mana: max_mana.clamp(0, i16::MAX as i32) as i16,
        });
    }
    // `:17663-17676` — the buff list → 50 then 13.
    buff_list_changed(buffs, &clone.buff_type, out);
    // `:17677-17684` — the minion targets (bodies unported, decisions kept).
    if minion_rest != clone.minion_rest {
        out.push(PlayerSend::MinionTarget { attack: false });
    }
    if minion_attack != clone.minion_attack {
        out.push(PlayerSend::MinionTarget { attack: true });
    }
    // `:17685-17696` — direction / shieldRaised / accSnappingStoneLightUp,
    // each its OWN 13 (three separate sends when all three changed).
    if controls.direction != clone.controls.direction {
        out.push(PlayerSend::Controls);
    }
    if controls.shield_raised != clone.controls.shield_raised {
        out.push(PlayerSend::Controls);
    }
    if controls.snap_stone_light != clone.controls.snap_stone_light {
        out.push(PlayerSend::Controls);
    }
    // `:17697-17700` — the inventory-synced ping.
    if synced_any_inventory {
        out.push(PlayerSend::InventorySynced);
    }
    // `:17702` — the clientClone tail: the scalar set the next tick compares.
    clone.controls = *controls;
    clone.zones = zones;
    clone.hp = hp;
    clone.max_hp = max_hp;
    clone.mana = mana;
    clone.max_mana = max_mana;
    clone.buff_type = buffs.buff_type;
    clone.chest = live_chest;
    clone.talk_npc = talk_npc;
    clone.minion_rest = minion_rest;
    clone.minion_attack = minion_attack;
}

/// The `13` control-bit packing for `net::PlayerState.flags_a`/`flags_d`
/// (`NetMessage.cs:450-490`): the bits the port's control state reaches.
/// Bits with no port field ride false (the recorded two-client deviation).
pub(crate) fn apply_control_bits(ps: &mut net::PlayerState, c: &ControlState) {
    ps.flags_a.set(0, c.up);
    ps.flags_a.set(1, c.down);
    ps.flags_a.set(2, c.left);
    ps.flags_a.set(3, c.right);
    ps.flags_a.set(4, c.jump);
    ps.flags_a.set(5, c.use_item);
    ps.flags_a.set(6, c.direction == 1);
    ps.flags_a.set(7, c.dash);
    ps.flags_d.set(1, c.auto_reuse_all);
    ps.flags_d.set(2, c.down_hold);
    ps.flags_d.set(3, c.operating_entity);
    ps.flags_d.set(6, c.last_use_success);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctl() -> ControlState {
        ControlState {
            direction: 1,
            ..Default::default()
        }
    }

    // --- TrySyncingInput (Player.cs:6852-6916) ---

    #[test]
    fn the_input_compare_sends_on_any_of_the_twelve_fields() {
        let base = ctl();
        assert!(!try_syncing_input(&base, &base, 1), "unchanged sends nothing");
        assert!(!try_syncing_input(&base, &base, 0), "SP never sends");
        assert!(!try_syncing_input(&base, &base, 2), "the host never sends");
        let mut now = base;
        now.jump = true;
        assert!(try_syncing_input(&now, &base, 1));
        now = base;
        now.selected_item = 3;
        assert!(try_syncing_input(&now, &base, 1));
        now = base;
        now.last_use_success = true;
        assert!(try_syncing_input(&now, &base, 1));
        // The writer-copied extras are NOT in the 12-field compare — a
        // direction-only change rides the TrySyncingMyPlayer tail instead.
        now = base;
        now.direction = -1;
        assert!(!try_syncing_input(&now, &base, 1));
    }

    // --- UpdateClient's counter (Main.cs:64522-64536) ---

    #[test]
    fn the_client_beat_fires_13_every_420_and_the_trio_every_900() {
        let mut c = 0u32;
        let mut beats13 = Vec::new();
        let mut trios = Vec::new();
        for tick in 1..=3601u32 {
            let (b13, trio) = update_client_beat(&mut c);
            if b13 {
                beats13.push(tick);
            }
            if trio {
                trios.push(tick);
            }
        }
        assert_eq!(beats13, vec![420, 840, 1260, 1680, 2100, 2520, 2940, 3360]);
        assert_eq!(trios, vec![900, 1800, 2700, 3600]);
        // The wrap is `> 3600` (:64523): tick 3601 resets to 0, no send.
        assert_eq!(c, 0);
        let (b13, trio) = update_client_beat(&mut c);
        assert!(!b13 && !trio && c == 1);
    }

    // --- the netLife/netMana throttle (Main.cs:17636-17661) ---

    #[test]
    fn the_stat_throttle_sends_once_then_holds_for_60_ticks() {
        let mut t = StatThrottle::default();
        assert!(t.step(true), "the first change sends at once (time 0)");
        assert_eq!(t.time, 60);
        // Continuous change: the flag re-arms every tick but the timer holds.
        for _ in 0..59 {
            assert!(!t.step(true), "the 60-tick window suppresses");
        }
        assert_eq!(t.time, 1);
        assert!(t.step(true), "the 61st tick sends the parked change");
        // No change: the timer drains silently, then nothing.
        let mut t2 = StatThrottle::default();
        assert!(t2.step(true));
        for _ in 0..120 {
            assert!(!t2.step(false), "no change, no send");
        }
        assert_eq!(t2.time, 0);
        assert!(!t2.pending);
    }

    // --- Hurt / HealEffect (Player.cs:38369-38377, :4519-4528) ---

    #[test]
    fn hurt_sends_the_knockback_13_then_the_16() {
        let mut out = Vec::new();
        hurt_sends(1, false, false, 1, 80, 100, &mut out);
        assert_eq!(
            out,
            vec![
                PlayerSend::Controls,
                PlayerSend::Life { hp: 80, max_hp: 100 }
            ]
        );
        out.clear();
        hurt_sends(1, false, true, 1, 80, 100, &mut out);
        assert_eq!(out, vec![PlayerSend::Life { hp: 80, max_hp: 100 }]);
        out.clear();
        hurt_sends(1, false, false, 0, 80, 100, &mut out);
        assert_eq!(out.len(), 1, "no hit direction, no 13");
        out.clear();
        hurt_sends(1, true, false, 1, 80, 100, &mut out);
        hurt_sends(0, false, false, 1, 80, 100, &mut out);
        hurt_sends(2, false, false, 1, 80, 100, &mut out);
        assert!(out.is_empty(), "quiet / SP / host send nothing");
    }

    #[test]
    fn heal_effect_sends_35_only_on_the_client_broadcast() {
        let mut out = Vec::new();
        heal_effect(20, true, 1, &mut out);
        assert_eq!(out, vec![PlayerSend::Heal { amount: 20 }]);
        out.clear();
        heal_effect(20, false, 1, &mut out);
        heal_effect(20, true, 0, &mut out);
        assert!(out.is_empty());
    }

    // --- the stealth edges (Player.cs:25841-25844 etc.) ---

    #[test]
    fn the_stealth_send_fires_only_on_the_reach_edges() {
        let mut out = Vec::new();
        assert!(stealth_edge(0.04, 0.0, 1, &mut out), "the fade-out edge");
        assert_eq!(out, vec![PlayerSend::Stealth { stealth: 0.0 }]);
        out.clear();
        assert!(stealth_edge(0.96, 1.0, 1, &mut out), "the fade-in edge");
        assert!(!stealth_edge(0.5, 0.46, 1, &mut out), "mid-fade is silent");
        assert!(!stealth_edge(0.0, 0.0, 1, &mut out), "already 0 is silent");
        assert!(out.len() == 1);
        out.clear();
        stealth_edge(0.5, 0.0, 0, &mut out);
        assert!(out.is_empty(), "SP never sends");
    }

    // --- the zone packing (Player.cs:3427-3821) ---

    #[test]
    fn the_zone_bytes_pack_the_csharp_bit_slots() {
        let z = tr_core::scene::Zones {
            dungeon: true,
            jungle: true,
            snow: true,
            desert: true,
            temple: true,
            ..Default::default()
        };
        let (z1, z2, z3, z4, z5, town) = zone_bytes(&z, true);
        assert_eq!(z1, 0b0101_0001, "dungeon 0 + jungle 4 + snow 5");
        assert_eq!(z2, 1 << 5, "desert bit 5");
        assert_eq!(z3, 0);
        assert_eq!(z4, 1 << 5, "temple bit 5");
        assert_eq!(z5, 1, "shimmer bit 0");
        assert_eq!(town, 0);
        assert!(zone_changed((1, 0, 0, 0, 0, 0), (0, 0, 0, 0, 0, 0)));
        assert!(!zone_changed((1, 2, 3, 4, 5, 6), (1, 2, 3, 4, 5, 6)));
    }

    // --- the close-transition 33 (Main.cs:17575-17593) ---

    #[test]
    fn the_chest_close_flush_rides_the_live_index_and_the_edited_name() {
        // The writer's `number` is the LIVE chest (`Main.cs:17590` passes
        // `Main.player[myPlayer].chest`, negative) — the clone index only
        // gates the edge and feeds the name source.
        let s = chest_close_flush(-1, 7, true, || Some("Loot".into()));
        assert_eq!(
            s,
            Some(PlayerSend::ChestClose {
                chest: -1,
                name: Some("Loot".into())
            })
        );
        let s = chest_close_flush(-1, 7, false, || Some("Loot".into()));
        assert_eq!(s, Some(PlayerSend::ChestClose { chest: -1, name: None }));
        // The null-chest arm (`Main.cs:17585-17587`): edited, but the name
        // source is null — the nameless 33 still fires.
        let s = chest_close_flush(-1, 7, true, || None);
        assert_eq!(s, Some(PlayerSend::ChestClose { chest: -1, name: None }));
        assert_eq!(chest_close_flush(3, 7, true, || None), None, "still open");
        assert_eq!(chest_close_flush(-1, -1, true, || None), None, "was closed");
    }

    // --- the full scalar tail (Main.cs:17545-17703) ---

    #[test]
    fn the_tail_emits_in_the_csharp_order_and_clones_at_the_end() {
        // The clone starts at `new Player()` (100/100 hp, 0/20 mana,
        // direction 1, chest/talk -1): a fresh 100-hp player observes NO
        // life change; the mana 0 → 20 compare fires at time 0.
        let mut clone = ClientClone::default();
        let buffs = crate::buffs::BuffSlots::default();
        let mut out = Vec::new();
        let zones = (1, 0, 0, 0, 0, 0);
        try_syncing_my_player_tail(
            &mut clone,
            zones,
            100,
            100,
            20,
            20,
            &buffs,
            -1,
            false,
            || None,
            -1,
            (0.0, 0.0),
            -1,
            &ctl(),
            false,
            &mut out,
        );
        // Order: 36 (zones 0 → 1), 42 (mana 0 → 20). No 16 (hp matches the
        // ctor), no 13 (direction 1 matches).
        assert!(matches!(out[0], PlayerSend::Zone { zone1: 1, .. }));
        assert!(matches!(out[1], PlayerSend::Mana { mana: 20, .. }));
        assert_eq!(out.len(), 2);
        assert_eq!(clone.zones, zones);
        assert_eq!(clone.mana, 20);
        // Second identical pass: the throttles hold, nothing changed.
        out.clear();
        try_syncing_my_player_tail(
            &mut clone, zones, 100, 100, 20, 20, &buffs, -1, false, || None,
            -1, (0.0, 0.0), -1, &ctl(), false, &mut out,
        );
        assert!(out.is_empty(), "the clone suppresses the unchanged pass");
        // A hurt to 80 hp: the life throttle sat at 0, so the 16 fires at
        // once; the mana throttle is still inside its 60-tick window.
        try_syncing_my_player_tail(
            &mut clone, zones, 80, 100, 20, 20, &buffs, -1, false, || None,
            -1, (0.0, 0.0), -1, &ctl(), false, &mut out,
        );
        assert_eq!(out, vec![PlayerSend::Life { hp: 80, max_hp: 100 }]);
        assert_eq!(clone.hp, 80);
    }

    #[test]
    fn the_clone_defaults_are_the_csharp_player_ctor() {
        // `Main.clientPlayer = new Player()` (`Main.cs:67158`) — a zero-hp
        // dead player MUST compare different on the first tick.
        let c = ClientClone::default();
        assert_eq!((c.hp, c.max_hp, c.mana, c.max_mana), (100, 100, 0, 20));
        assert_eq!(c.controls.direction, 1);
        assert_eq!(c.chest, -1);
        assert_eq!(c.talk_npc, -1);
        assert_eq!(c.minion_attack, -1);
        assert!(c.life.step(false) == false && c.mana_throttle.time == 0);
    }

    #[test]
    fn the_buff_change_sends_50_then_13_and_three_13s_are_three_sends() {
        let mut clone = ClientClone {
            controls: ctl(),
            ..Default::default()
        };
        // Pre-seed the throttles so only the buff/direction arms fire.
        clone.life.time = 60;
        clone.mana_throttle.time = 60;
        let mut buffs = crate::buffs::BuffSlots::default();
        buffs.buff_type[0] = 20;
        buffs.buff_time[0] = 300;
        let mut c = ctl();
        c.direction = -1;
        c.shield_raised = true;
        c.snap_stone_light = true;
        let mut out = Vec::new();
        try_syncing_my_player_tail(
            &mut clone, (0, 0, 0, 0, 0, 0), 0, 0, 0, 0, &buffs, -1, false,
            || None, -1, (0.0, 0.0), -1, &c, true, &mut out,
        );
        // 50, 13 (buff pair), 13 (direction), 13 (shield), 13 (snapstone), 138.
        assert!(matches!(&out[0], PlayerSend::Buffs { ids } if ids == &vec![20]));
        assert!(matches!(out[1], PlayerSend::Controls));
        assert!(matches!(out[2], PlayerSend::Controls));
        assert!(matches!(out[3], PlayerSend::Controls));
        assert!(matches!(out[4], PlayerSend::Controls));
        assert!(matches!(out[5], PlayerSend::InventorySynced));
        assert_eq!(out.len(), 6);
    }

    #[test]
    fn the_control_bits_land_in_the_csharp_flag_slots() {
        let mut ps = net::PlayerState::default();
        let c = ControlState {
            up: true,
            jump: true,
            use_item: true,
            direction: 1,
            auto_reuse_all: true,
            last_use_success: true,
            ..Default::default()
        };
        apply_control_bits(&mut ps, &c);
        assert!(ps.flags_a.get(0) && ps.flags_a.get(4) && ps.flags_a.get(5));
        assert!(ps.flags_a.get(6), "direction == 1");
        assert!(ps.flags_d.get(1) && ps.flags_d.get(6));
        assert!(!ps.flags_a.get(1) && !ps.flags_d.get(2));
    }
}
