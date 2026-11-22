//! `NPC.GetChat` (`NPC.cs:93766-94800`): the town NPCs' chatter — the
//! dialog the C#'s `NPCInteractions` shows when the player talks to a town
//! NPC.
//!
//! The C#'s chain is one `if/else` ladder per NPC type over the `Lang.dialog`
//! lines, gated by the world state (`Main.dayTime`/`time`/`bloodMoon`/...), the
//! player state and the seventeen "a town NPC of this type exists" flags the
//! function scans out of the NPC list. The port keeps the same order and the
//! same rolls; the draws come from the caller's `Main.rand` stream.

use crate::npc::{Npc, NpcKind};

/// The world and player state the chatter reads.
///
/// The C# reads them off `Main`/`Main.player[Main.myPlayer]`/`NPC` statics;
/// the port passes them in so the chain stays a pure function of the state.
pub struct ChatContext<'a> {
    /// The live NPC list — the seventeen presence flags (`NPC.cs:93788-93864`).
    pub npcs: &'a [Npc],
    /// `Main.rand.Next(n)` — the caller's stream.
    pub rng: &'a mut dyn FnMut(i32) -> i32,
    pub day_time: bool,
    /// `Main.time` — the clock in ticks.
    pub time: f64,
    pub blood_moon: bool,
    pub eclipse: bool,
    pub slime_rain: bool,
    pub hard_mode: bool,
    /// `Main.moonPhase` (0..7).
    pub moon_phase: u8,
    pub remix_world: bool,
    pub raining: bool,
    pub storming: bool,
    /// `Main.IsItAHappyWindyDay`.
    pub windy_day: bool,
    /// `Player.ZoneGraveyard`.
    pub zone_graveyard: bool,
    /// `BirthdayParty.PartyIsUp`.
    pub party_is_up: bool,
    /// `WorldGen.crimson`.
    pub crimson: bool,
    /// `NPC.downedBoss1` (the King Slime).
    pub downed_boss1: bool,
    /// `NPC.downedBoss2` (the Eye of Cthulhu).
    pub downed_boss2: bool,
    /// `NPC.downedBoss3` (the Skeletron).
    pub downed_boss3: bool,
    /// `NPC.downedMoonlord`.
    pub downed_moonlord: bool,
    /// `DD2Event.DownedInvasionT1`.
    pub dd2_downed_t1: bool,
    /// `DD2Event.DownedInvasionT2`.
    pub dd2_downed_t2: bool,
    /// `DD2Event.Ongoing`.
    pub dd2_ongoing: bool,
    /// `LanternNight.LanternsUp`.
    pub lanterns_up: bool,
    pub player_life: i32,
    /// `Player.statLifeMax2` — the life maximum.
    pub player_life_max: i32,
    /// `Player.statLifeMax` — the base life maximum (the Merchant's first arm).
    pub player_base_life_max: i32,
    pub player_defense: i32,
    /// `Player.Male`.
    pub player_male: bool,
    /// `Player.head` — the head slot's item id.
    pub player_head: i16,
    /// `Player.numberOfDeathsPVE`.
    pub player_deaths_pve: i32,
    /// `Player.NormalizedLuck`.
    pub player_luck: f32,
    /// `Player.HasItem(5275)` — the cola the Dryad's line asks about.
    pub player_has_cola: bool,
    /// `Main.anglerQuestFinished`.
    pub angler_quest_finished: bool,
    /// `Main.LocalPlayer.anglerQuestsFinished`.
    pub angler_quests_finished: i32,
    /// `Main.LocalPlayer.golferScoreAccumulated`.
    pub golfer_score: i32,
}

impl ChatContext<'_> {
    /// `Main.rand.Next(n)`.
    fn next(&mut self, n: i32) -> i32 {
        (self.rng)(n)
    }

    /// `Main.rand.Next(min, max)` — the exclusive upper bound.
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        min + self.next(max - min)
    }
}

/// The seventeen presence flags (`NPC.cs:93788-93864`): which town NPCs
/// exist. The C# sets them by the NPC **type** — 17/18/19/20/37/38/124/107/54/
/// 160/178/229/209/208/353/22/441 for `flag`..`flag17`.
///
/// The types 178 (the legacy "Steampunker") and 209 (the legacy "Cyborg")
/// are pre-1.4 ids no NPC carries any more, so `flag11` and `flag13` are
/// always false.
#[derive(Clone, Copy, Default)]
struct Flags {
    /// 17 — the Merchant.
    merchant: bool,
    /// 18 — the Nurse.
    nurse: bool,
    /// 19 — the Arms Dealer.
    arms_dealer: bool,
    /// 20 — the Dryad.
    dryad: bool,
    /// 37 — the Old Man.
    old_man: bool,
    /// 38 — the Demolitionist.
    demolitionist: bool,
    /// 124 — the Mechanic.
    mechanic: bool,
    /// 107 — the Goblin Tinkerer.
    tinkerer: bool,
    /// 54 — the Clothier.
    clothier: bool,
    /// 160 — the Party Girl.
    party_girl: bool,
    /// 178 — the legacy Steampunker (never present).
    steampunker: bool,
    /// 229 — the Zoologist.
    zoologist: bool,
    /// 209 — the legacy Cyborg (never present).
    cyborg: bool,
    /// 208 — the Stylist.
    stylist: bool,
    /// 353 — the Witch Doctor.
    witch_doctor: bool,
    /// 22 — the Guide.
    guide: bool,
    /// 441 — the Tax Collector.
    tax_collector: bool,
}

impl Flags {
    fn scan(npcs: &[Npc]) -> Self {
        let mut f = Self::default();
        for npc in npcs {
            // The C#'s `Main.npc[i].active` — the port's liveness is the hp.
            if npc.hp <= 0 {
                continue;
            }
            match npc.kind.cs_id() {
                17 => f.merchant = true,
                18 => f.nurse = true,
                19 => f.arms_dealer = true,
                20 => f.dryad = true,
                37 => f.old_man = true,
                38 => f.demolitionist = true,
                124 => f.mechanic = true,
                107 => f.tinkerer = true,
                54 => f.clothier = true,
                160 => f.party_girl = true,
                178 => f.steampunker = true,
                229 => f.zoologist = true,
                209 => f.cyborg = true,
                208 => f.stylist = true,
                353 => f.witch_doctor = true,
                22 => f.guide = true,
                441 => f.tax_collector = true,
                _ => {}
            }
        }
        f
    }
}

/// `NPC.GetChat` (`NPC.cs:93766-94800`): the NPC's line.
pub fn get_chat(npc: &Npc, ctx: &mut ChatContext) -> String {
    let flags = Flags::scan(ctx.npcs);
    // `NPC.homeless` — the port's `Npc` keeps the home as `Option`, so
    // "homeless" is "no home assigned" (`NPC.cs:6551-6553`).
    let homeless = npc.home.is_none();
    match npc.kind {
        NpcKind::Merchant => merchant(ctx, &flags),
        NpcKind::Nurse => nurse(ctx, &flags),
        NpcKind::ArmsDealer => arms_dealer(ctx, &flags),
        NpcKind::Dryad => dryad(ctx, &flags),
        NpcKind::Demolitionist => demolitionist(ctx, &flags),
        NpcKind::Clothier => clothier(ctx, &flags),
        NpcKind::GoblinTinkerer => goblin_tinkerer(ctx, &flags, homeless),
        NpcKind::BoundWizard => crate::lang::dialog(137),
        NpcKind::Wizard => wizard(ctx, &flags, homeless),
        NpcKind::Mechanic => mechanic(ctx, &flags, homeless),
        NpcKind::Guide => guide(ctx, &flags),
        NpcKind::PartyGirl => party_girl(ctx, &flags),
        NpcKind::DyeTrader => dye_trader(ctx, &flags),
        NpcKind::Painter => painter(ctx, &flags),
        NpcKind::WitchDoctor => witch_doctor(ctx, &flags),
        NpcKind::TravellingMerchant => travelling_merchant(ctx, &flags),
        NpcKind::Angler => angler(ctx, &flags),
        NpcKind::SleepingAngler => {
            let n = ctx.next_range(353, 356);
            crate::lang::dialog(n)
        }
        NpcKind::TaxCollector => tax_collector(ctx, &flags),
        NpcKind::Golfer => golfer(ctx),
        NpcKind::Zoologist => zoologist(ctx),
        NpcKind::Princess => princess(ctx),
        _ => String::new(),
    }
}

/// `NPC.HasSpecialEventText` (`NPC.cs:94860-94887`): the event texts — the
/// graveyard, the party, the rain, the windy day and the storm each roll
/// 1/3 and pick a random line from `<Category>SpecialText.<Event>`.
fn special_event_text(ctx: &mut ChatContext, category: &str) -> Option<String> {
    let events: [(bool, &str); 5] = [
        (ctx.zone_graveyard, "Graveyard"),
        (ctx.party_is_up, "Party"),
        (ctx.raining && !ctx.storming, "Rain"),
        (ctx.windy_day, "Windy"),
        (ctx.storming, "Storm"),
    ];
    for (active, event) in events {
        if !active {
            continue;
        }
        if ctx.next(3) != 0 {
            continue;
        }
        let key = format!("{category}SpecialText.{event}");
        let size = crate::lang::category_size(&key);
        if size == 0 {
            continue;
        }
        let index = ctx.next(size as i32) as usize;
        return Some(crate::lang::random_from_category(&key, index));
    }
    None
}

/// A random line from a chatter category (`Language.SelectRandom` over
/// `Lang.CreateDialogFilter(prefix)`).
fn chatter(ctx: &mut ChatContext, category: &str) -> String {
    let size = crate::lang::category_size(category);
    if size == 0 {
        return String::new();
    }
    let index = ctx.next(size as i32) as usize;
    crate::lang::random_from_category(category, index)
}

/// 17 — the Merchant (`NPC.cs:93866-93898`).
fn merchant(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "Merchant") {
        return text;
    }
    if !ctx.downed_boss1 && ctx.next(3) == 0 {
        if ctx.player_base_life_max < 200 {
            return crate::lang::dialog(1);
        }
        if ctx.player_defense > 10 {
            return crate::lang::dialog(3);
        }
        return crate::lang::dialog(2);
    }
    if ctx.day_time {
        if ctx.time < 16200.0 {
            let n = ctx.next(3);
            return crate::lang::dialog(4 + n);
        }
        if ctx.time > 37800.0 {
            let n = ctx.next(3);
            return crate::lang::dialog(7 + n);
        }
        let n = ctx.next(3);
        return crate::lang::dialog(10 + n);
    }
    if ctx.blood_moon {
        if f.nurse && f.mechanic && ctx.next(3) == 0 {
            return crate::lang::dialog(13);
        }
        let n = ctx.next(4);
        return crate::lang::dialog(14 + n);
    }
    if ctx.time < 9720.0 {
        return if ctx.next(2) != 0 {
            crate::lang::dialog(19)
        } else {
            crate::lang::dialog(18)
        };
    }
    if ctx.time > 22680.0 {
        return if ctx.next(2) != 0 {
            crate::lang::dialog(21)
        } else {
            crate::lang::dialog(20)
        };
    }
    let n = ctx.next(3);
    crate::lang::dialog(22 + n)
}

/// 18 — the Nurse (`NPC.cs:93898-93988`).
fn nurse(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "Nurse") {
        return text;
    }
    if ctx.blood_moon {
        if (ctx.player_life as f64) < ctx.player_life_max as f64 * 0.66 {
            let n = ctx.next(3);
            return crate::lang::dialog(25 + n);
        }
        let n = ctx.next(4);
        return crate::lang::dialog(28 + n);
    }
    if ctx.next(5) == 0 && ctx.player_deaths_pve > 1 {
        // `:93913-93927` — the death-count band.
        let band = if ctx.player_deaths_pve >= 100 {
            "High"
        } else if ctx.player_deaths_pve >= 25 {
            "Medium"
        } else {
            "Low"
        };
        return crate::lang::text(&format!("NurseSpecialText.DeathCount{band}"));
    }
    if ctx.next(3) == 0 && !ctx.downed_boss3 {
        return crate::lang::dialog(32);
    }
    if f.demolitionist && ctx.next(4) == 0 {
        return crate::lang::dialog(33);
    }
    if f.arms_dealer && ctx.next(4) == 0 {
        return crate::lang::dialog(34);
    }
    if f.guide && ctx.next(4) == 0 {
        return crate::lang::dialog(35);
    }
    if (ctx.player_life as f64) < ctx.player_life_max as f64 * 0.33 {
        let n = ctx.next(5);
        return crate::lang::dialog(36 + n);
    }
    if (ctx.player_life as f64) < ctx.player_life_max as f64 * 0.66 {
        let n = ctx.next(7);
        return crate::lang::dialog(41 + n);
    }
    // `:93974-93985` — the four fixed lines or a random chatter line.
    let size = crate::lang::category_size("NurseChatter");
    let n = ctx.next(4 + size as i32);
    if n >= 4 {
        return crate::lang::random_from_category("NurseChatter", (n - 4) as usize);
    }
    crate::lang::dialog(48 + n)
}

/// 19 — the Arms Dealer (`NPC.cs:93989-94037`).
fn arms_dealer(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "ArmsDealer") {
        return text;
    }
    if f.guide && ctx.downed_boss3 && !ctx.hard_mode {
        return crate::lang::dialog(58);
    }
    if f.nurse && ctx.next(5) == 0 {
        return crate::lang::dialog(59);
    }
    if f.nurse && ctx.next(5) == 0 {
        return crate::lang::dialog(60);
    }
    if f.dryad && ctx.next(5) == 0 {
        return crate::lang::dialog(61);
    }
    if f.demolitionist && ctx.next(5) == 0 {
        return crate::lang::dialog(62);
    }
    if f.demolitionist && ctx.next(5) == 0 {
        return crate::lang::dialog(63);
    }
    if ctx.blood_moon {
        return if ctx.next(2) != 0 {
            crate::lang::dialog(65)
        } else {
            crate::lang::dialog(64)
        };
    }
    let n = ctx.next(3);
    crate::lang::dialog(66 + n)
}

/// 20 — the Dryad (`NPC.cs:94038-94080`).
fn dryad(ctx: &mut ChatContext, f: &Flags) -> String {
    let mut result = if let Some(text) = special_event_text(ctx, "Dryad") {
        text
    } else if ctx.dd2_downed_t1 && ctx.next(6) == 0 {
        if ctx.dd2_downed_t2 {
            crate::lang::text("DryadSpecialText.AfterDD2Tier2")
        } else {
            crate::lang::text("DryadSpecialText.AfterDD2Tier1")
        }
    } else if !ctx.downed_boss2 && ctx.next(3) == 0 {
        if ctx.crimson {
            crate::lang::dialog(332)
        } else {
            crate::lang::dialog(69)
        }
    } else if f.arms_dealer && ctx.next(6) == 0 {
        crate::lang::dialog(70)
    } else if f.merchant && ctx.next(6) == 0 {
        crate::lang::dialog(71)
    } else if f.old_man && ctx.next(6) == 0 {
        crate::lang::dialog(72)
    } else if f.party_girl && ctx.next(6) == 0 {
        crate::lang::dialog(238)
    } else if ctx.blood_moon {
        let n = ctx.next(4);
        crate::lang::dialog(73 + n)
    } else {
        let n = ctx.next(5);
        if n == 0 && ctx.crimson {
            crate::lang::dialog(333)
        } else {
            crate::lang::dialog(77 + n)
        }
    };
    // `:94076-94079` — the cola line overrides the whole ladder.
    if ctx.player_has_cola {
        result = crate::lang::text("StardewTalk.PlayerHasColaButIsNotHoldingIt");
    }
    result
}

/// 38 — the Demolitionist (`NPC.cs:94081-94105`).
fn demolitionist(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "Demolitionist") {
        return text;
    }
    if !ctx.downed_boss2 && ctx.next(3) == 0 {
        return crate::lang::dialog(93);
    }
    if ctx.blood_moon {
        let n = ctx.next(3);
        return crate::lang::dialog(94 + n);
    }
    if (ctx.dd2_downed_t1 || ctx.dd2_ongoing) && ctx.next(5) == 0 {
        return crate::lang::text("DemolitionistSpecialText.AfterDD2Start");
    }
    if f.arms_dealer && ctx.next(5) == 0 {
        return crate::lang::dialog(97);
    }
    if f.arms_dealer && ctx.next(5) == 0 {
        return crate::lang::dialog(98);
    }
    if f.nurse && ctx.next(4) == 0 {
        return crate::lang::dialog(99);
    }
    if f.dryad && ctx.next(4) == 0 {
        return crate::lang::dialog(100);
    }
    if !ctx.day_time {
        let n = ctx.next(4);
        return crate::lang::dialog(101 + n);
    }
    let n = ctx.next(5);
    crate::lang::dialog(105 + n)
}

/// 54 — the Clothier (`NPC.cs:94106-94122`).
fn clothier(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "Clothier") {
        return text;
    }
    if !f.mechanic && ctx.next(2) == 0 {
        return crate::lang::dialog(110);
    }
    if f.party_girl && ctx.next(6) == 0 {
        return crate::lang::dialog(237);
    }
    if ctx.blood_moon {
        return crate::lang::dialog(111);
    }
    if f.nurse && ctx.next(4) == 0 {
        return crate::lang::dialog(112);
    }
    if ctx.player_head == 24 {
        return crate::lang::dialog(113);
    }
    let n = ctx.next(6);
    crate::lang::dialog(114 + n)
}

/// 107 — the Goblin Tinkerer (`NPC.cs:94123-94146`).
fn goblin_tinkerer(ctx: &mut ChatContext, f: &Flags, homeless: bool) -> String {
    if homeless {
        let n = ctx.next(5);
        return crate::lang::dialog(121 + n);
    }
    if let Some(text) = special_event_text(ctx, "GoblinTinkerer") {
        return text;
    }
    if f.mechanic && ctx.next(5) == 0 {
        return crate::lang::dialog(126);
    }
    if f.witch_doctor && ctx.next(5) == 0 {
        return crate::lang::dialog(309);
    }
    let size = crate::lang::category_size("GoblinTinkererChatter");
    let n = ctx.next(size as i32 + 5);
    if n >= 5 {
        return crate::lang::random_from_category("GoblinTinkererChatter", (n - 5) as usize);
    }
    if !ctx.day_time {
        crate::lang::dialog(127 + n)
    } else {
        crate::lang::dialog(132 + n)
    }
}

/// 108 — the Wizard (`NPC.cs:94152-94183`).
fn wizard(ctx: &mut ChatContext, f: &Flags, homeless: bool) -> String {
    if homeless {
        let n = ctx.next(3);
        if n == 0 {
            return crate::lang::dialog(138);
        }
        if n == 1 && !ctx.player_male {
            return crate::lang::dialog(139);
        }
        return crate::lang::dialog(140 + n - 1);
    }
    if let Some(text) = wizard_luck_text(ctx) {
        return text;
    }
    if let Some(text) = special_event_text(ctx, "Wizard") {
        return text;
    }
    if ctx.dd2_downed_t1 && ctx.next(6) == 0 {
        return crate::lang::text("WizardSpecialText.AfterDD2Tier1");
    }
    if ctx.player_male && f.guide && ctx.next(6) == 0 {
        return crate::lang::dialog(142);
    }
    if ctx.player_male && f.demolitionist && ctx.next(6) == 0 {
        return crate::lang::dialog(143);
    }
    if ctx.player_male && f.tinkerer && ctx.next(6) == 0 {
        return crate::lang::dialog(144);
    }
    if !ctx.player_male && f.nurse && ctx.next(6) == 0 {
        return crate::lang::dialog(145);
    }
    if !ctx.player_male && f.mechanic && ctx.next(6) == 0 {
        return crate::lang::dialog(146);
    }
    if !ctx.player_male && f.dryad && ctx.next(6) == 0 {
        return crate::lang::dialog(147);
    }
    if ctx.day_time {
        let n = ctx.next(5);
        return crate::lang::dialog(151 + n);
    }
    let n = ctx.next(3);
    crate::lang::dialog(148 + n)
}

/// `NPC.HasLuckTextForWizard` (`NPC.cs:94843-94858`): the luck band line.
fn wizard_luck_text(ctx: &mut ChatContext) -> Option<String> {
    if ctx.next(3) != 0 {
        return None;
    }
    let luck = ctx.player_luck;
    if luck == 0.0 {
        return None;
    }
    let category = if luck < -0.6 {
        "WizardSpecialText.LuckIsCursed"
    } else if luck < -0.4 {
        "WizardSpecialText.LuckIsTerrible"
    } else if luck < -0.2 {
        "WizardSpecialText.LuckIsPoor"
    } else if luck < 0.0 {
        "WizardSpecialText.LuckIsBad"
    } else if luck < 0.25 {
        "WizardSpecialText.LuckIsGood"
    } else if luck < 0.5 {
        "WizardSpecialText.LuckIsGreat"
    } else if luck < 0.75 {
        "WizardSpecialText.LuckIsAmazing"
    } else {
        "WizardSpecialText.LuckIsGodly"
    };
    let size = crate::lang::category_size(category);
    if size == 0 {
        return None;
    }
    let index = ctx.next(size as i32) as usize;
    Some(crate::lang::random_from_category(category, index))
}

/// 124 — the Mechanic (`NPC.cs:94185-94201`).
fn mechanic(ctx: &mut ChatContext, f: &Flags, homeless: bool) -> String {
    if homeless {
        let n = ctx.next(4);
        return crate::lang::dialog(157 + n);
    }
    if let Some(text) = special_event_text(ctx, "Mechanic") {
        return text;
    }
    if ctx.blood_moon {
        let n = ctx.next(4);
        return crate::lang::dialog(161 + n);
    }
    if f.tinkerer && ctx.next(6) == 0 {
        return crate::lang::dialog(165);
    }
    if f.arms_dealer && ctx.next(6) == 0 {
        return crate::lang::dialog(166);
    }
    let size = crate::lang::category_size("MechanicChatter");
    let n = ctx.next(size as i32 + 3);
    if n >= 3 {
        return crate::lang::random_from_category("MechanicChatter", (n - 3) as usize);
    }
    crate::lang::dialog(167 + n)
}

/// 22 — the Guide (`NPC.cs:94202-94218`).
fn guide(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "Guide") {
        return text;
    }
    if ctx.blood_moon {
        let n = ctx.next(3);
        return crate::lang::dialog(170 + n);
    }
    if ctx.lanterns_up && !ctx.downed_moonlord {
        return crate::lang::text("GuideSpecialText.Lantern1");
    }
    if ctx.lanterns_up && ctx.downed_moonlord {
        return crate::lang::text("GuideSpecialText.Lantern2");
    }
    if ctx.eclipse {
        return crate::lang::text("GuideSpecialText.Eclipse");
    }
    if ctx.slime_rain {
        return crate::lang::text("GuideSpecialText.SlimeRain");
    }
    if !ctx.day_time {
        return crate::lang::dialog(173);
    }
    if ctx.hard_mode && f.tax_collector && ctx.next(8) == 0 {
        return crate::lang::text("GuideChatter.Chatter_1");
    }
    if ctx.hard_mode && ctx.next(8) == 0 {
        return crate::lang::text("GuideChatter.Chatter_2");
    }
    let n = ctx.next(3);
    crate::lang::dialog(174 + n)
}

/// 208 — the Party Girl (`NPC.cs:94219-94232`).
fn party_girl(ctx: &mut ChatContext, f: &Flags) -> String {
    let num8 = ctx.next(6);
    if let Some(text) = special_event_text(ctx, "Truffle") {
        return text;
    }
    if f.dryad && ctx.next(6) == 0 {
        return crate::lang::dialog(232);
    }
    if f.clothier && ctx.next(6) == 0 {
        return crate::lang::dialog(236);
    }
    match num8 {
        0 => crate::lang::dialog(231),
        1 => crate::lang::dialog(233),
        2 => crate::lang::dialog(234),
        3 => crate::lang::dialog(235),
        4 => crate::lang::dialog(240),
        _ => crate::lang::dialog(241),
    }
}

/// 207 — the Dye Trader (`NPC.cs:94233-94244`).
fn dye_trader(ctx: &mut ChatContext, f: &Flags) -> String {
    let num10 = ctx.next(3);
    if let Some(text) = special_event_text(ctx, "DyeTrader") {
        return text;
    }
    if f.zoologist && ctx.next(6) == 0 {
        return crate::lang::dialog(260);
    }
    crate::lang::dialog(257 + num10)
}

/// 227 — the Painter (`NPC.cs:94245-94256`).
fn painter(ctx: &mut ChatContext, f: &Flags) -> String {
    let num13 = ctx.next(5);
    if let Some(text) = special_event_text(ctx, "Painter") {
        return text;
    }
    if ctx.hard_mode && ctx.next(7) == 0 {
        return crate::lang::dialog(250);
    }
    if f.stylist && ctx.next(6) == 0 {
        return crate::lang::dialog(251);
    }
    match num13 {
        0 => crate::lang::dialog(252),
        1 => crate::lang::dialog(253),
        2 => crate::lang::dialog(254),
        3 => crate::lang::dialog(255),
        _ => crate::lang::dialog(256),
    }
}

/// 228 — the Witch Doctor (`NPC.cs:94257-94268`).
fn witch_doctor(ctx: &mut ChatContext, f: &Flags) -> String {
    let num14 = ctx.next(4);
    if let Some(text) = special_event_text(ctx, "WitchDoctor") {
        return text;
    }
    if f.nurse && ctx.next(4) == 0 {
        return crate::lang::dialog(263);
    }
    match num14 {
        0 => crate::lang::dialog(261),
        1 => crate::lang::dialog(262),
        2 => crate::lang::dialog(264),
        _ => crate::lang::text("WitchDoctorSpecialText.AfterDD2Tier1"),
    }
}

/// 368 — the Travelling Merchant (`NPC.cs:94269-94275`).
fn travelling_merchant(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "TravellingMerchant") {
        return text;
    }
    if f.guide && ctx.next(5) == 0 {
        return crate::lang::dialog(319);
    }
    if f.merchant && ctx.next(5) == 0 {
        return crate::lang::dialog(320);
    }
    if !f.clothier || ctx.next(5) != 0 {
        let n = ctx.next_range(322, 331);
        return crate::lang::dialog(n);
    }
    crate::lang::dialog(321)
}

/// 369 — the Angler (`NPC.cs:94276-94290`).
fn angler(ctx: &mut ChatContext, f: &Flags) -> String {
    if ctx.next(5) == 0 && ctx.angler_quests_finished > 1 {
        return chatter(ctx, "AnglerQuestChatter");
    }
    if let Some(text) = special_event_text(ctx, "Angler") {
        return text;
    }
    if ctx.blood_moon {
        if ctx.next(3) == 0 {
            return chatter(ctx, "AnglerSpecialText.BloodMoonFishing");
        }
        if ctx.angler_quest_finished {
            let n = ctx.next_range(350, 353);
            return crate::lang::dialog(n);
        }
        let n = ctx.next_range(348, 350);
        return crate::lang::dialog(n);
    }
    if !ctx.angler_quest_finished {
        let n = ctx.next_range(334, 338);
        return crate::lang::dialog(n);
    }
    if !f.dryad || ctx.next(5) != 0 {
        return chatter(ctx, "AnglerChatter");
    }
    crate::lang::dialog(347)
}

/// 441 — the Tax Collector (`NPC.cs:94291-94298`).
fn tax_collector(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "TaxCollector") {
        return text;
    }
    if ctx.next(6) == 0 && f.dryad {
        return crate::lang::dialog(371);
    }
    if ctx.next(6) == 0 && f.arms_dealer {
        return crate::lang::dialog(372);
    }
    if ctx.next(6) == 0 && f.demolitionist {
        return crate::lang::dialog(373);
    }
    if !(ctx.next(6) == 0 && f.merchant) {
        let n = ctx.next_range(364, 371);
        return crate::lang::dialog(n);
    }
    crate::lang::dialog(374)
}

/// 588 — the Golfer (`NPC.cs:94299-94310`).
fn golfer(ctx: &mut ChatContext) -> String {
    if ctx.next(3) != 0 {
        if let Some(text) = special_event_text(ctx, "Golfer") {
            return text;
        }
    }
    if ctx.next(3) != 0 {
        return chatter(ctx, "GolferChatter");
    }
    // `:94306-94310` — the score band picks the quest chatter category.
    let category = if ctx.golfer_score >= 2000 {
        "GolferQuestsChatterMaster"
    } else if ctx.golfer_score >= 1000 {
        "GolferQuestsChatterJourneyman"
    } else if ctx.golfer_score < 500 {
        "GolferQuestsChatterBeginner"
    } else {
        "GolferQuestsChatterApprentice"
    };
    chatter(ctx, category)
}

/// 633 — the Zoologist (`NPC.cs:94311-94316`).
fn zoologist(ctx: &mut ChatContext) -> String {
    // `NPC.ShouldBestiaryGirlBeLycantrope` (`NPC.cs:93753-93764`) — the
    // shared derivation the happiness report's `Transformed` suffix reads
    // (`happiness.rs`, `ShopHelper.cs:191-194`).
    let lycantrope = crate::happiness::should_bestiary_girl_be_lycantrope(
        ctx.blood_moon,
        ctx.day_time,
        ctx.moon_phase,
    );
    if lycantrope {
        return chatter(ctx, "BestiaryGirlLycantropeChatter");
    }
    if special_event_text(ctx, "BestiaryGirl").is_none() {
        return chatter(ctx, "BestiaryGirlChatter");
    }
    special_event_text(ctx, "BestiaryGirl").unwrap_or_default()
}

/// 663 — the Princess (`NPC.cs:94317-94321`).
fn princess(ctx: &mut ChatContext) -> String {
    match special_event_text(ctx, "Princess") {
        Some(text) => text,
        None => chatter(ctx, "PrincessChatter"),
    }
}

/// `Lang.GetSlimeType` (`Lang.cs:375-406`): the town-slime kind → the
/// chatter category's colour word — the 8-word switch the `Lang.SlimeChat`
/// filter composes (`Slime{colour}Chatter.Chatter`, `Lang.cs:408-411`).
/// 670 Blue (also the switch's default), 678 Green, 679 Old, 680 Purple,
/// 681 Rainbow, 682 Red, 683 Yellow, 684 Copper. W4-g10 lane 3 FINAL: the
/// per-KIND mapping over the landed town-slime kinds (the Lead's
/// colour-per-kind deliverable).
///
/// Keep-one (d120): `task/W4-d120-town-chatter` ports the same switch
/// type-keyed (`slime_chat_colour(ty: u16)`, its `npc_chat.rs`) AND the
/// `GetChat` arms that consume it (`NPC.cs:94816-94819`, the
/// `is_town_slime`-guarded `Slime{colour}Chatter.Chatter` pick) plus the
/// `dialog_count`/`select_dialog` localization machinery. d120's arms are
/// keyed on the VANILLA type (`npc.kind.cs_id()`), so they go LIVE the
/// moment these kinds merge — zero d120 edits (its retire note, research/143
/// item 2). At the merge this kind-keyed fn and d120's type-keyed fn dedup
/// (identical words; `kind.cs_id()` bridges them); d120 owns the dispatch.
pub(crate) fn slime_chat_colour(kind: NpcKind) -> &'static str {
    match kind {
        NpcKind::TownSlimeGreen => "Green",
        NpcKind::TownSlimeOld => "Old",
        NpcKind::TownSlimePurple => "Purple",
        NpcKind::TownSlimeRainbow => "Rainbow",
        NpcKind::TownSlimeRed => "Red",
        NpcKind::TownSlimeYellow => "Yellow",
        NpcKind::TownSlimeCopper => "Copper",
        // 670 (TownSlimeBlue) and the switch's default arm.
        _ => "Blue",
    }
}

// ---- the chat dialog's minimal renderer (task wire-npc-chat) -----------

/// The chat dialog's screen box — `NPCChatPanel.Draw`'s rectangle
/// (`NPCChatPanel.cs:65-66`): `new Rectangle(Main.screenWidth / 2 - 250,
/// 100, 500, 30)`, grown per text/interaction row by [`chat_box_height`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChatRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// `NPCChatPanel.cs:65-66` — the base box: `x = Main.screenWidth / 2 - 250`,
/// `y = 100`, `w = 500`, `h = 30` (the screen height plays no part in the
/// C# layout — the panel hangs 100 px under the top edge).
pub fn chat_box_rect(screen_w: i32, _screen_h: i32) -> ChatRect {
    ChatRect {
        x: screen_w / 2 - 250,
        y: 100,
        w: 500,
        h: 30,
    }
}

/// `NPCChatPanel.cs:67-68` — the height additions: `30 * AmountOfLines`
/// plus `30 * _neededInteractionLines + Math.Max(0, 2 *
/// (_neededInteractionLines - 1))` (`AllowedInteractionsPerLine = 4`,
/// `:30`, and `_neededInteractionLines = ceil(count / 4)`, `:174`; the
/// port renders text only, so its callers pass 0).
pub fn chat_box_height(text_lines: usize, interaction_lines: usize) -> i32 {
    let il = interaction_lines as i32;
    30 + 30 * text_lines as i32 + 30 * il + 2 * (il - 1).max(0)
}

/// The panel's inner text width — the `+ (20f, 20f)` text origin
/// (`NPCChatPanel.cs:217`) inside the 500-wide box leaves 460 px, the same
/// number the IME overflow guard uses (`:246`).
pub const CHAT_TEXT_WRAP_WIDTH: i32 = 460;

/// The word wrap the panel's `TextDisplayCache` performs over the drawn
/// rows (`NPCChatPanel.cs:26`'s cache, drawn line-by-line at `:221-235`):
/// greedy, breaking on spaces, one word per row minimum. The C# measures
/// the words with the `MouseText` font; the port has no metrics yet, so it
/// wraps by characters at the same width — a recorded approximation.
pub fn format_chat_lines(line: &str, wrap_width: i32) -> Vec<String> {
    let limit = wrap_width.max(1) as usize;
    let mut rows: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in line.split(' ') {
        if cur.is_empty() {
            // A single word longer than the row still occupies one row.
            cur.push_str(word);
        } else if cur.chars().count() + 1 + word.chars().count() <= limit {
            cur.push(' ');
            cur.push_str(word);
        } else {
            rows.push(std::mem::take(&mut cur));
            cur.push_str(word);
        }
    }
    rows.push(cur);
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::Npc;

    /// A context with every field at its default, so a test states only
    /// what it drives.
    fn context<'a>(
        npcs: &'a [Npc],
        rng: &'a mut dyn FnMut(i32) -> i32,
    ) -> ChatContext<'a> {
        ChatContext {
            npcs,
            rng,
            day_time: true,
            time: 0.0,
            blood_moon: false,
            eclipse: false,
            slime_rain: false,
            hard_mode: false,
            moon_phase: 0,
            remix_world: false,
            raining: false,
            storming: false,
            windy_day: false,
            zone_graveyard: false,
            party_is_up: false,
            crimson: false,
            downed_boss1: false,
            downed_boss2: false,
            downed_boss3: false,
            downed_moonlord: false,
            dd2_downed_t1: false,
            dd2_downed_t2: false,
            dd2_ongoing: false,
            lanterns_up: false,
            player_life: 100,
            player_life_max: 100,
            player_base_life_max: 100,
            player_defense: 0,
            player_male: true,
            player_head: 0,
            player_deaths_pve: 0,
            player_luck: 0.0,
            player_has_cola: false,
            angler_quest_finished: false,
            angler_quests_finished: 0,
            golfer_score: 0,
        }
    }

    /// A draw stream that hands out the queued values in order (the last one
    /// repeats).
    fn draws(values: &'static [i32]) -> impl FnMut(i32) -> i32 {
        let mut i = 0usize;
        move |_n| {
            let v = values[i.min(values.len() - 1)];
            i += 1;
            v
        }
    }

    fn npc(kind: NpcKind) -> Npc {
        Npc::new(kind, 100.0, 100.0, 100, 0.0)
    }

    /// The language resources — the `LegacyDialog` rows live in the shipped
    /// `decompiled/` json (the S1-L-12d note). The init can reload the
    /// process-global manager (a sibling test may have left another culture
    /// active), so it takes the same guard the language readers hold —
    /// otherwise a parallel reader sees bare keys mid-reload.
    fn init_lang() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
    }

    /// S1-E-10a — the Merchant's first arm (`NPC.cs:93870-93876`): the
    /// King Slime gate, the 1/3 roll and the life-max band.
    #[test]
    fn the_merchant_answers_the_c_sharp_line() {
        init_lang();
        let merchant = npc(NpcKind::Merchant);
        let npcs = [merchant];
        let mut rng = draws(&[0]);
        let mut ctx = context(&npcs, &mut rng);
        let chat = get_chat(&npcs[0], &mut ctx);
        assert_eq!(
            chat,
            "I hope a scrawny kid like you isn't all that is standing between us and Cthulhu's Eye."
        );
    }

    /// S1-E-10a — the presence flags (`NPC.cs:93788-93864`): the Arms
    /// Dealer's guide line needs the Guide to live next door.
    #[test]
    fn the_presence_flags_pick_the_neighbour_line() {
        init_lang();
        let arms = npc(NpcKind::ArmsDealer);
        let guide = npc(NpcKind::Guide);
        let npcs = [arms, guide];
        let mut rng = draws(&[0]);
        let mut ctx = context(&npcs, &mut rng);
        ctx.downed_boss3 = true;
        let chat = get_chat(&npcs[0], &mut ctx);
        assert_eq!(
            chat,
            "I heard there is a doll that looks very similar to {Guide} somewhere in the underworld.  I'd like to put a few rounds in it."
        );
    }

    /// S1-E-10a — the Guide's night line (`NPC.cs:94207`).
    #[test]
    fn the_guide_answers_the_night_line() {
        init_lang();
        let guide = npc(NpcKind::Guide);
        let npcs = [guide];
        let mut rng = draws(&[0]);
        let mut ctx = context(&npcs, &mut rng);
        ctx.day_time = false;
        let chat = get_chat(&npcs[0], &mut ctx);
        assert_eq!(
            chat,
            "You should stay indoors at night. It is very dangerous to be wandering around in the dark."
        );
    }

    /// S1-E-10a — a non-town NPC has no line (`NPC.GetChat`'s ladder ends
    /// with an empty result).
    #[test]
    fn a_non_town_npc_has_no_line() {
        let slime = npc(NpcKind::Slime);
        let npcs = [slime];
        let mut rng = draws(&[0]);
        let mut ctx = context(&npcs, &mut rng);
        assert_eq!(get_chat(&npcs[0], &mut ctx), "");
    }

    // ---- the live wiring (task wire-npc-chat) --------------------------

    /// `Main.cs:43081-43082` — the right-click talk: a town NPC under the
    /// `TileReachCheckSettings.Simple.GetWorldRegion` box answers `GetChat`,
    /// the app latches `talkNPC = i` and stores `Main.npcChatText`.
    #[test]
    fn the_talk_action_returns_the_guides_chat_for_a_town_npc() {
        init_lang();
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        let p = app.player.as_ref().unwrap();
        let mut guide = npc(NpcKind::Guide);
        // One tile right of the player — well inside the ±5-tile world
        // region (`Main.cs:43047`'s `worldRegion.Intersects(npc box)`).
        guide.x = p.x + 16.0;
        guide.y = p.y;
        app.npcs.push(guide);
        app.place_down = true; // the fresh right press (`Main.mouseRight && Main.npcChatRelease`)
        let text = crate::world_sim::npcs::talk_to_npc(&mut app, false);
        let text = text.expect("a town NPC under the region opens the chat");
        assert!(!text.is_empty(), "the Guide answers a line");
        assert_eq!(app.talk_npc, Some(0), "SetTalkNPC(i), `:43081`");
        assert_eq!(app.npc_chat_text, text, "Main.npcChatText = GetChat(), `:43082`");
    }

    /// `Main.cs:43047` — the gate is `nPC.townNPC || type in {...}`: a zombie
    /// (no townNPC flag) is never talkable and the click falls through.
    #[test]
    fn the_talk_action_returns_none_for_a_non_town_npc() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        let p = app.player.as_ref().unwrap();
        let mut zombie = npc(NpcKind::Zombie);
        zombie.x = p.x + 16.0;
        zombie.y = p.y;
        app.npcs.push(zombie);
        app.place_down = true;
        let text = crate::world_sim::npcs::talk_to_npc(&mut app, false);
        assert!(text.is_none(), "a zombie is not talkable");
        assert_eq!(app.talk_npc, None);
        assert!(app.npc_chat_text.is_empty());
    }

    /// `NPCChatPanel.cs:65-68` — the panel's box: x = `screenWidth / 2 - 250`,
    /// y = 100, w = 500, h = 30 + 30·textLines + 30·interactionLines +
    /// max(0, 2·(interactionLines − 1)); the text origin is `+ (20, 20)`
    /// (`:217`) and the text area is the 460 px the IME guard uses (`:246`).
    #[test]
    fn the_chat_box_rect_matches_the_csharp_geometry() {
        let r = crate::npc_chat::chat_box_rect(1920, 1080);
        assert_eq!((r.x, r.y, r.w, r.h), (710, 100, 500, 30), "the base box");
        assert_eq!(crate::npc_chat::chat_box_rect(800, 600).x, 150, "half of 800 minus 250");
        assert_eq!(crate::npc_chat::chat_box_height(3, 0), 120, "30 + 30·3");
        assert_eq!(crate::npc_chat::chat_box_height(2, 1), 120, "30 + 60 + 30 + 0");
        assert_eq!(
            crate::npc_chat::chat_box_height(1, 2),
            122,
            "30 + 30 + 60 + 2·(2−1)"
        );
        // The wrap: a line longer than the text area breaks (the C# wraps at
        // the font-measured 460 px — `NPCChatPanel.cs:246`'s number; the port
        // wraps by characters at the same width), a short one stays whole.
        let long_line = "You should stay indoors at night. ".repeat(16); // > 460 chars
        let lines =
            crate::npc_chat::format_chat_lines(&long_line, crate::npc_chat::CHAT_TEXT_WRAP_WIDTH);
        assert!(lines.len() > 1, "a 460+-char line wraps");
        for ln in &lines {
            assert!(
                ln.chars().count() <= crate::npc_chat::CHAT_TEXT_WRAP_WIDTH as usize,
                "no row exceeds the text area"
            );
        }
        assert_eq!(
            crate::npc_chat::format_chat_lines("short line", 460),
            vec!["short line".to_string()]
        );
    }

    /// W4-g10 lane 3 FINAL — `Lang.GetSlimeType` (`Lang.cs:375-406`): the
    /// colour-per-kind mapping over the LANDED town-slime kinds. Each kind
    /// answers its C# switch word, the composed `Slime{colour}Chatter`
    /// category ships its chatter rows, and the kind's `cs_id` (the vanilla
    /// type d120's type-keyed `GetChat` arm reads) matches the switch case.
    /// WRITTEN NOT RUN.
    #[test]
    fn w4_g10_the_town_slime_kinds_answer_their_colour_words() {
        init_lang();
        // (kind, vanilla type, colour word) — the C# switch's eight cases.
        let table: [(NpcKind, u16, &str); 8] = [
            (NpcKind::TownSlimeBlue, 670, "Blue"),
            (NpcKind::TownSlimeGreen, 678, "Green"),
            (NpcKind::TownSlimeOld, 679, "Old"),
            (NpcKind::TownSlimePurple, 680, "Purple"),
            (NpcKind::TownSlimeRainbow, 681, "Rainbow"),
            (NpcKind::TownSlimeRed, 682, "Red"),
            (NpcKind::TownSlimeYellow, 683, "Yellow"),
            (NpcKind::TownSlimeCopper, 684, "Copper"),
        ];
        for (kind, ty, colour) in table {
            // The colour word the kind answers.
            assert_eq!(slime_chat_colour(kind), colour, "{kind:?} colour");
            // The kind's vanilla type is the switch case (d120's arm keys
            // on `cs_id`, so the type-keyed and kind-keyed maps agree).
            assert_eq!(kind.cs_id(), ty, "{kind:?} vanilla type");
            assert_eq!(kind.net_id(), ty as i32, "{kind:?} net id");
            // The dispatch guard (`NPCID.Sets.IsTownSlime`, NPCID.cs:10119).
            assert!(crate::npc_ai_town::is_town_slime(ty as i32), "{kind:?} in the set");
            // The composed category ships its three chatter rows (the
            // shipped en-US Town.json carries Chatter_1..3 for every
            // colour, plus a NoHome row the housing path reads).
            let category = format!("Slime{colour}Chatter");
            for i in 1..=3 {
                assert!(
                    crate::lang::exists(&format!("{category}.Chatter_{i}")),
                    "{category}.Chatter_{i} ships"
                );
            }
            // The colour's OWN first line resolves (not a shared pool).
            let first = crate::lang::text(&format!("{category}.Chatter_1"));
            assert!(!first.is_empty(), "{category}.Chatter_1 resolves");
            assert_ne!(first, format!("{category}.Chatter_1"), "not the bare key");
        }
        // The default arm: a non-town-slime kind answers Blue (the switch's
        // `result = "Blue"` initializer, Lang.cs:377).
        assert_eq!(slime_chat_colour(NpcKind::Slime), "Blue", "the wild slime default");
        assert_eq!(slime_chat_colour(NpcKind::Guide), "Blue", "the switch default");
        // The eight first lines are pairwise distinct (each colour owns its
        // lines — the witness that the mapping is not a constant).
        let mut firsts: Vec<String> = table
            .iter()
            .map(|(_, _, c)| crate::lang::text(&format!("Slime{c}Chatter.Chatter_1")))
            .collect();
        let n = firsts.len();
        firsts.sort();
        firsts.dedup();
        assert_eq!(firsts.len(), n, "every colour's first line is distinct");
    }
}
