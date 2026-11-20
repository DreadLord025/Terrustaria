//! The missing world-event schedulers (S1-F-14): `LanternNight`
//! (`GameContent/Events/LanternNight.cs`), `BirthdayParty`
//! (`GameContent/Events/BirthdayParty.cs`) and `CultistRitual`'s delay
//! machine (`GameContent/Events/CultistRitual.cs`). The Sandstorm event
//! already rides `tr_core::scene::Sandstorm` (the R-11-5 walk). The state
//! lives on the App's [`WorldEvents`]; the walks wire at the C#'s sites —
//! `UpdateTime` (:64963-64966), `CheckNight` (:65225-65226), `CheckMorning`
//! (:65374-65375) — with the draws on the UpdateTime stream (S1-A-10's
//! contract entry: the stream = `UpdateWorld_Time`'s `Main.rand`).
//!
//! Deviations: the party's `OrderBy(Main.rand.Next())` celebrating-NPC pick
//! indexes the port's compacted `Vec<Npc>` (the C# indexes the stable
//! `Main.npc` slots — recorded); `NPC.freeCake = true`'s companion consumer
//! (the party girl's cake NPC) is unported; the party's `BroadcastText`
//! rides the port's death-text broadcast with the C# colour
//! (`new Color(255, 0, 160)`).

use crate::chat::{self, ChatMonitor};
use crate::npc::{Npc, NpcKind};

/// The three schedulers' state, zeroed per world (`WorldClear`,
/// `Main.cs:52245-52247`'s family).
#[derive(Debug, Default)]
pub(crate) struct WorldEvents {
    // --- LanternNight (`LanternNight.cs`) ---
    /// `LanternNight.ManualLanterns`.
    pub manual_lanterns: bool,
    /// `LanternNight.GenuineLanterns`.
    pub genuine_lanterns: bool,
    /// `LanternNight.NextNightIsLanternNight`.
    pub next_night_is_lantern_night: bool,
    /// `LanternNight.LanternNightsOnCooldown`.
    pub lantern_nights_on_cooldown: i32,
    /// `LanternNight._wasLanternNight`.
    was_lantern_night: bool,
    // --- BirthdayParty (`BirthdayParty.cs`) ---
    /// `BirthdayParty.ManualParty`.
    pub manual_party: bool,
    /// `BirthdayParty.GenuineParty`.
    pub genuine_party: bool,
    /// `BirthdayParty.PartyDaysOnCooldown`.
    pub party_days_on_cooldown: i32,
    /// `BirthdayParty.CelebratingNPCs` — the compacted vec's indices (the
    /// C#'s stable `Main.npc` slots; deviation recorded).
    pub celebrating_npcs: Vec<usize>,
    /// `BirthdayParty._wasCelebrating`.
    was_celebrating: bool,
    // --- CultistRitual (`CultistRitual.cs`) ---
    /// `CultistRitual.delay` (`CultistRitual.delayStart` = 86400).
    pub cultist_delay: i32,
    /// `CultistRitual.recheck`.
    pub cultist_recheck: i32,
}

/// `LanternNight.LanternsUp` (`LanternNight.cs:20-26`).
pub(crate) fn lanterns_up(ev: &WorldEvents) -> bool {
    ev.genuine_lanterns || ev.manual_lanterns
}

/// `BirthdayParty.PartyIsUp` (`BirthdayParty.cs:25-32`).
pub(crate) fn party_is_up(ev: &WorldEvents) -> bool {
    ev.genuine_party || ev.manual_party
}

/// The gate snapshot `LanternNight.LanternsCanStart`'s C# body reads
/// (`LanternNight.cs:49-56`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct LanternGates {
    /// `WorldGen.spawnMeteor`.
    pub spawn_meteor: bool,
    /// `Main.bloodMoon`.
    pub blood_moon: bool,
    /// `Main.pumpkinMoon`.
    pub pumpkin_moon: bool,
    /// `Main.snowMoon`.
    pub snow_moon: bool,
    /// `Main.invasionType`.
    pub invasion_type: i32,
    /// `NPC.MoonLordCountdown`.
    pub moon_lord_countdown: i32,
    /// `LanternNight.BossIsActive`'s scan (`:59-71`): any active
    /// `npc.boss`-flagged NPC or the Eater's worm segments 13..15.
    pub boss_active: bool,
}

impl LanternGates {
    /// The gates from the port's world + NPC walk (`BossIsActive`'s scan —
    /// the port's `NPC.boss`-flagged kinds are `revenge.rs`'s `BOSS_KINDS`).
    pub fn from_world(app_world: &tr_core::World, npcs: &[Npc]) -> Self {
        Self {
            spawn_meteor: app_world.spawn_meteor,
            blood_moon: app_world.blood_moon,
            pumpkin_moon: app_world.pumpkin_moon,
            snow_moon: app_world.snow_moon,
            invasion_type: app_world.invasion_type,
            moon_lord_countdown: app_world.moon_lord_countdown,
            // `NPCID.Sets`' boss flag (the port's `BOSS_KINDS` walk) plus
            // the Eater's worm segments 13..15 (`LanternNight.cs:59-71`'s
            // `n.type >= 13 && n.type <= 15`).
            boss_active: npcs.iter().any(|n| {
                n.hp > 0
                    && (crate::revenge::BOSS_KINDS.contains(&n.kind)
                        || matches!(
                            n.kind,
                            NpcKind::EaterOfWorldsHead
                                | NpcKind::EaterOfWorldsBody
                                | NpcKind::EaterOfWorldsTail
                        ))
            }),
        }
    }
}

/// `LanternNight.LanternsCanStart` (`LanternNight.cs:49-56`).
pub(crate) fn lanterns_can_start(g: &LanternGates) -> bool {
    !g.spawn_meteor
        && !g.blood_moon
        && !g.pumpkin_moon
        && !g.snow_moon
        && g.invasion_type == 0
        && g.moon_lord_countdown == 0
        && !g.boss_active
}

/// `LanternNight.CheckNight` → `NaturalAttempt` (`LanternNight.cs:110-133`):
/// the cooldown's decrement, the post-Moon-Lord `Next(14) == 0` roll, the
/// `NextNightIsLanternNight` override, and the `Next(5, 11)` cooldown on a
/// hit. `rng` mirrors `Main.rand`'s `Next(min, max)` on the UpdateTime
/// stream.
pub(crate) fn check_night_lanterns(
    ev: &mut WorldEvents,
    gates: &LanternGates,
    downed_moon_lord: bool,
    rng: &mut dyn FnMut(i32, i32) -> i32,
) {
    if !lanterns_can_start(gates) {
        return;
    }
    let mut hit = false;
    if ev.lantern_nights_on_cooldown > 0 {
        ev.lantern_nights_on_cooldown -= 1;
    }
    if ev.lantern_nights_on_cooldown == 0 && downed_moon_lord && rng(0, 14) == 0 {
        hit = true;
    }
    if !hit && ev.next_night_is_lantern_night {
        ev.next_night_is_lantern_night = false;
        hit = true;
    }
    if hit {
        ev.genuine_lanterns = true;
        ev.lantern_nights_on_cooldown = rng(5, 11);
    }
}

/// `LanternNight.CheckMorning` (`LanternNight.cs:29-40`): the genuine and
/// manual lanterns clear at dawn.
pub(crate) fn check_morning_lanterns(ev: &mut WorldEvents) {
    if ev.genuine_lanterns {
        ev.genuine_lanterns = false;
    }
    if ev.manual_lanterns {
        ev.manual_lanterns = false;
    }
}

/// `BirthdayParty.CheckNight` (`BirthdayParty.cs:37-51`): the genuine party
/// clears at dusk (its celebrating roster with it) and the manual one
/// expires, with the "everyone has gone home" broadcast — `Lang.misc[99]`
/// keyed, `new Color(255, 0, 160)`.
pub(crate) fn check_night_party(ev: &mut WorldEvents, chat: &mut ChatMonitor) {
    let mut cleared = false;
    if ev.genuine_party {
        cleared = true;
        ev.genuine_party = false;
        ev.celebrating_npcs.clear();
    }
    if ev.manual_party {
        cleared = true;
        ev.manual_party = false;
    }
    if cleared {
        chat::broadcast_death_text(chat, "Everyone has gone home!", [255, 0, 160]);
    }
}

/// `BirthdayParty.CheckMorning` → `NaturalAttempt` (`BirthdayParty.cs:69-117`):
/// the party girl must be present (`AnyNPCs(208)`), the cooldown's
/// decrement, the `Next(10) == 0` roll, the >= 5 partyable NPCs gather, and
/// the `Next(5, 11)` cooldown + the 1/2/3-celebrating roll on a hit.
/// `partyable` folds `CanNPCParty`'s gates (`:55-64`) over the port's
/// compacted vec — the C#'s `OrderBy(Main.rand.Next())` pick indexes the
/// port's vec slots (deviation recorded).
///
/// Returns the `CheckForAchievement()` call the C# makes at `:142` — the
/// party just came up, so THROW_A_PARTY (25) credits (S1-E-18f).
pub(crate) fn check_morning_party(
    ev: &mut WorldEvents,
    party_girl_present: bool,
    partyable: &[bool],
    rng: &mut dyn FnMut(i32, i32) -> i32,
) -> bool {
    if !party_girl_present {
        return false;
    }
    if ev.party_days_on_cooldown > 0 {
        ev.party_days_on_cooldown -= 1;
        return false;
    }
    if rng(0, 10) != 0 {
        return false;
    }
    let candidates: Vec<usize> = (0..partyable.len()).filter(|&i| partyable[i]).collect();
    if candidates.len() < 5 {
        return false;
    }
    ev.genuine_party = true;
    ev.party_days_on_cooldown = rng(5, 11);
    ev.celebrating_npcs.clear();
    // `Main.rand.Next(5) == 0 && list.Count > 12` → 3 celebrants, else the
    // `Next(3) == 0` → 2 arm — both draws consumed in the C#'s order.
    let mut num = 1;
    if rng(0, 5) == 0 && candidates.len() > 12 {
        num = 3;
    } else if rng(0, 3) == 0 {
        num = 2;
    }
    // The `OrderBy(Main.rand.Next())` pick: one `Next()` per candidate, the
    // stable sort taking the first `num` (the port mirrors it as a
    // shuffle-by-key over the compacted indices).
    let mut order: Vec<(usize, i32)> =
        candidates.iter().map(|&i| (i, rng(0, i32::MAX))).collect();
    order.sort_by_key(|&(_, k)| k);
    for &(i, _) in order.iter().take(num) {
        ev.celebrating_npcs.push(i);
    }
    // `BirthdayParty.cs:142` — `CheckForAchievement` on the fresh party.
    true
}

/// `LanternNight.UpdateTime` (`LanternNight.cs:150-170`): the genuine
/// lanterns clear when they cannot persist (the `LanternsCanPersist` gate
/// — `!dayTime && LanternsCanStart()`), and the lanterns-up edge flips
/// (`SkyManager`'s activate/deactivate — the port's sky layer owns it).
pub(crate) fn update_time_lanterns(
    ev: &mut WorldEvents,
    can_persist: bool,
) -> Option<bool> {
    if ev.genuine_lanterns && !can_persist {
        ev.genuine_lanterns = false;
    }
    if ev.was_lantern_night != lanterns_up(ev) {
        ev.was_lantern_night = lanterns_up(ev);
        Some(ev.was_lantern_night)
    } else {
        None
    }
}

/// `BirthdayParty.UpdateTime` (`BirthdayParty.cs`'s per-tick walk — the
/// `_wasCelebrating` edge bookkeeping; the dusk clear rides CheckNight).
pub(crate) fn update_time_party(ev: &mut WorldEvents) {
    if ev.was_celebrating != party_is_up(ev) {
        ev.was_celebrating = party_is_up(ev);
    }
}

/// `CultistRitual.UpdateTime` (`CultistRitual.cs:23-45`): the delay/recheck
/// decay and the `delay == 0 && recheck == 0` window — the window opens the
/// ritual attempt (`recheck = 600 * 6` under `AnyDanger`, else 600). The
/// caller drives `TrySpawning`'s floor/LOS gates and the spawn.
pub(crate) fn update_time_cultist(
    ev: &mut WorldEvents,
    day_rate: i32,
    any_danger: bool,
) -> bool {
    ev.cultist_delay -= day_rate;
    if ev.cultist_delay < 0 {
        ev.cultist_delay = 0;
    }
    ev.cultist_recheck -= day_rate;
    if ev.cultist_recheck < 0 {
        ev.cultist_recheck = 0;
    }
    if ev.cultist_delay == 0 && ev.cultist_recheck == 0 {
        ev.cultist_recheck = 600 * if any_danger { 6 } else { 1 };
        return true;
    }
    false
}

/// `NPC.SetEventFlagCleared` + `OnGameEventClearedForTheFirstTime`
/// (`NPC.cs:78759-78795`): the FIRST clear of a progression flag arms the
/// next lantern night. The C#'s switch arms `NextNightIsLanternNight` on
/// the default row and on cases 10/16/17/18; only cases 4 (Pumpking),
/// 21 (Evergreen) and 22 (Santa-NK1) skip it — none of those three kinds
/// is ported, so every ported first-clear arms the flag. `was_down` is
/// the flag's pre-kill value (the C# `eventFlag != flag` edge).
///
/// The case-10 credits roll and the case-16/17/18 mech-trio Plantera-bulb
/// pass (`WorldGen.GeneratePlanteraBulbOnAllMechsDefeated`) are unported
/// (the credits event and the bulb pass are later waves).
pub(crate) fn on_game_event_cleared_first_time(ev: &mut WorldEvents, was_down: bool) {
    if !was_down {
        ev.next_night_is_lantern_night = true;
    }
}

/// `CultistRitual.CultistSlain` (`CultistRitual.cs:48-51`): `- 3600`.
pub(crate) fn cultist_slain(ev: &mut WorldEvents) {
    ev.cultist_delay -= 3600;
}

/// `CultistRitual.TabletDestroyed` (`CultistRitual.cs:54-57`):
/// `delay = respawnDelay` (43200).
pub(crate) fn tablet_destroyed(ev: &mut WorldEvents) {
    ev.cultist_delay = 43200;
}

/// `WorldClear`'s family (`Main.cs:52245-52247`'s per-event clears —
/// `LanternNight.WorldClear`/`BirthdayParty.WorldClear`; the ritual's
/// `delay`/`recheck` ride the world's load reset, `WorldFile.cs:163`).
pub(crate) fn world_clear(ev: &mut WorldEvents) {
    ev.manual_lanterns = false;
    ev.genuine_lanterns = false;
    ev.lantern_nights_on_cooldown = 0;
    ev.was_lantern_night = false;
    ev.manual_party = false;
    ev.genuine_party = false;
    ev.party_days_on_cooldown = 0;
    ev.celebrating_npcs.clear();
    ev.was_celebrating = false;
    // The ritual's `delay`/`recheck` are NOT in the WorldClear family — they
    // ride the world-file load (`WorldFile.cs:163`'s `_tempCultistDelay`
    // restore), so this fn leaves them alone (the C#'s own scope).
} 


#[cfg(test)]
mod tests {
    use super::*;

    /// S1-F-14 - the machine walks: the lantern night's gated roll order,
    /// the party's NaturalAttempt sequence and the cultist delay's decay.
    #[test]
    fn s1_f_14_the_lantern_night_machine_walks_the_csharp_order() {
        let mut ev = WorldEvents::default();
        let gates = LanternGates {
            spawn_meteor: false,
            blood_moon: false,
            pumpkin_moon: false,
            snow_moon: false,
            invasion_type: 0,
            moon_lord_countdown: 0,
            boss_active: false,
        };
        // The Moon Lord unbeaten: the roll never runs.
        let mut draws = 0;
        let mut rng = |_lo: i32, _hi: i32| { draws += 1; 0 };
        check_night_lanterns(&mut ev, &gates, false, &mut rng);
        assert_eq!(draws, 0);
        // The Moon Lord beaten, a 1/14 hit: GenuineLanterns + the cooldown.
        let mut seq = [0i32, 7i32].into_iter();
        check_night_lanterns(&mut ev, &gates, true, &mut |lo, hi| seq.next().unwrap_or(lo));
        assert!(ev.genuine_lanterns);
        assert_eq!(ev.lantern_nights_on_cooldown, 7);
        // The boss gate blocks the whole attempt (no cooldown decrement).
        let mut ev2 = WorldEvents::default();
        ev2.lantern_nights_on_cooldown = 2;
        let gated = LanternGates { boss_active: true, ..gates };
        let mut called = 0;
        let mut rng2 = |_lo: i32, _hi: i32| { called += 1; 0 };
        check_night_lanterns(&mut ev2, &gated, true, &mut rng2);
        assert_eq!(ev2.lantern_nights_on_cooldown, 2);
        // The override fires without a draw.
        let mut ev3 = WorldEvents::default();
        ev3.next_night_is_lantern_night = true;
        let mut consumed = 0;
        check_night_lanterns(&mut ev3, &gates, false, &mut |_lo, _hi| { consumed += 1; 1 });
        assert!(ev3.genuine_lanterns);
        assert_eq!(consumed, 1, "the hit's Next(5, 11) still draws (`:130-133`)");
        check_morning_lanterns(&mut ev3);
        assert!(!ev3.genuine_lanterns);
        ev3.manual_lanterns = true;
        check_morning_lanterns(&mut ev3);
        assert!(!ev3.manual_lanterns);
    }

    #[test]
    fn w4_c39_the_persistence_gate_drops_genuine_lanterns_on_a_midnight_boss() {
        // `LanternNight.cs:150-154` + `:42-48`: `UpdateTime` clears the
        // GENUINE lanterns when `LanternsCanPersist` fails — night AND
        // `LanternsCanStart`. A boss coming up mid-night drops them; the
        // MANUAL lanterns persist regardless (the clear is genuine-only).
        let mut ev = WorldEvents::default();
        ev.genuine_lanterns = true;
        let calm = LanternGates {
            spawn_meteor: false,
            blood_moon: false,
            pumpkin_moon: false,
            snow_moon: false,
            invasion_type: 0,
            moon_lord_countdown: 0,
            boss_active: false,
        };
        // Night (`!dayTime` = true) + calm: persists; was_lantern_night
        // starts false, so the first call reports the up-edge.
        let night = true;
        assert_eq!(
            update_time_lanterns(&mut ev, night && lanterns_can_start(&calm)),
            Some(true),
            "the up-edge fires once"
        );
        assert_eq!(update_time_lanterns(&mut ev, true), None, "no re-edge");
        // The boss gate fails mid-night: the genuine lanterns drop.
        let boss = LanternGates { boss_active: true, ..calm };
        assert_eq!(
            update_time_lanterns(&mut ev, night && lanterns_can_start(&boss)),
            Some(false),
            "the down-edge fires"
        );
        assert!(!ev.genuine_lanterns);
        // The manual lanterns ride through the same failed gate.
        let mut manual = WorldEvents::default();
        manual.manual_lanterns = true;
        assert_eq!(update_time_lanterns(&mut manual, false), Some(true));
        assert_eq!(update_time_lanterns(&mut manual, false), None);
        assert!(manual.manual_lanterns, "the clear is genuine-only");
        // Daytime fails the persist gate on its own (`!dayTime` half).
        let mut day = WorldEvents::default();
        day.genuine_lanterns = true;
        assert_eq!(update_time_lanterns(&mut day, false), Some(false));
        assert!(!day.genuine_lanterns);
    }

    #[test]
    fn s1_f_14_the_party_machine_follows_natural_attempt() {
        let mut ev = WorldEvents::default();
        let mut chat = ChatMonitor::default();
        let partyable = [true; 5];
        let mut draws = 0;
        let mut rng = |_lo: i32, _hi: i32| { draws += 1; 0 };
        check_morning_party(&mut ev, false, &partyable, &mut rng);
        assert_eq!(draws, 0);
        let mut seq = [0i32, 6i32, 1i32, 1i32].into_iter();
        // S1-E-18f — the return is `:142`'s `CheckForAchievement` call.
        assert!(
            check_morning_party(&mut ev, true, &partyable, &mut |lo, hi| seq.next().unwrap_or(lo)),
            "a fresh party credits THROW_A_PARTY"
        );
        assert!(ev.genuine_party);
        assert_eq!(ev.party_days_on_cooldown, 6);
        assert_eq!(ev.celebrating_npcs.len(), 1);
        check_night_party(&mut ev, &mut chat);
        assert!(!ev.genuine_party);
        assert!(ev.celebrating_npcs.is_empty());
        let mut ev2 = WorldEvents::default();
        let partyable12 = [true; 13];
        let mut seq2 = [0i32, 6i32, 0i32].into_iter();
        check_morning_party(&mut ev2, true, &partyable12, &mut |lo, hi| seq2.next().unwrap_or(lo));
        assert_eq!(ev2.celebrating_npcs.len(), 3);
    }

    #[test]
    fn s1_f_14_the_cultist_delay_machine_walks_the_csharp_shape() {
        let mut ev = WorldEvents::default();
        tablet_destroyed(&mut ev);
        assert_eq!(ev.cultist_delay, 43200);
        update_time_cultist(&mut ev, 43200, false);
        assert_eq!(ev.cultist_delay, 0);
        assert_eq!(ev.cultist_recheck, 600);
        ev.cultist_delay = 86400;
        cultist_slain(&mut ev);
        assert_eq!(ev.cultist_delay, 82800);
        ev.cultist_delay = 0;
        ev.cultist_recheck = 0;
        update_time_cultist(&mut ev, 1, true);
        assert_eq!(ev.cultist_recheck, 3600);
    }

    #[test]
    fn s1_f_14_world_clear_zeroes_the_state() {
        let mut ev = WorldEvents {
            manual_lanterns: true,
            genuine_lanterns: true,
            next_night_is_lantern_night: true,
            lantern_nights_on_cooldown: 5,
            was_lantern_night: true,
            manual_party: true,
            genuine_party: true,
            party_days_on_cooldown: 5,
            celebrating_npcs: vec![1, 2, 3],
            was_celebrating: true,
            cultist_delay: 86400,
            cultist_recheck: 600,
        };
        world_clear(&mut ev);
        assert!(!ev.manual_lanterns && !ev.genuine_lanterns);
        assert_eq!(ev.lantern_nights_on_cooldown, 0);
        assert!(!ev.manual_party && !ev.genuine_party);
        assert!(ev.celebrating_npcs.is_empty());
        // The ritual's delay survives the clear (the world-file restore owns
        // it — `WorldFile.cs:163`).
        assert_eq!(ev.cultist_delay, 86400);
        assert_eq!(ev.cultist_recheck, 600);
    }
}
