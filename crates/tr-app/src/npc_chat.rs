//! `NPC.GetChat` (`NPC.cs:93766-94821`): the town NPCs' chatter — the
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
    /// `NPC.downedBoss1` (the Eye of Cthulhu).
    pub downed_boss1: bool,
    /// `NPC.downedBoss2` (the Eater of Worlds / Brain of Cthulhu).
    pub downed_boss2: bool,
    /// `NPC.downedBoss3` (the Skeletron).
    pub downed_boss3: bool,
    /// `NPC.downedMoonlord`.
    pub downed_moonlord: bool,
    /// `DD2Event.DownedInvasionT1`.
    pub dd2_downed_t1: bool,
    /// `DD2Event.DownedInvasionT2`.
    pub dd2_downed_t2: bool,
    /// `DD2Event.DownedInvasionT3` — the Bartender's tier line
    /// (`Lang.cs:335`).
    pub dd2_downed_t3: bool,
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

/// `Player.NormalizedLuck` (`Player.cs:4107-4122`): the raw `Player.luck`
/// divided by its sign's cap (`luckMaximumCap = 1f`, `luckMinimumCap =
/// -0.7f`, `:3088-3090`), clamped to [-1, 1]. The wizard's luck-band gate
/// (`NPC.cs:94848`) reads THIS value, not the raw unclamped luck.
pub(crate) fn normalized_luck(luck: f32) -> f32 {
    let value = if luck > 0.0 {
        luck / 1.0
    } else if luck < 0.0 {
        0.0 - luck / -0.7
    } else {
        0.0
    };
    value.clamp(-1.0, 1.0)
}

/// The seventeen presence flags (`NPC.cs:93788-93864`): which town NPCs
/// exist. The C# sets them by the NPC **type** — 17/18/19/20/37/38/124/
/// 107/54/160/178/229/209/208/353/22/441 for `flag`..`flag17`.
///
/// W4-d120: the Truffle (160), Steampunker (178) and Cyborg (209) kinds
/// are ported (the claim doc); the Old Man (37) and Pirate (229) ride the
/// d34/d37 lanes — their flags light up when those kinds merge (the scan
/// keys on `cs_id`, the vanilla type every lane keeps).
#[derive(Clone, Copy, Default)]
struct Flags {
    /// 17 — the Merchant (`flag`).
    merchant: bool,
    /// 18 — the Nurse (`flag2`).
    nurse: bool,
    /// 19 — the Arms Dealer (`flag3`).
    arms_dealer: bool,
    /// 20 — the Dryad (`flag4`).
    dryad: bool,
    /// 37 — the Old Man (`flag5`).
    old_man: bool,
    /// 38 — the Demolitionist (`flag6`).
    demolitionist: bool,
    /// 124 — the Mechanic (`flag7`).
    mechanic: bool,
    /// 107 — the Goblin Tinkerer (`flag8`).
    tinkerer: bool,
    /// 54 — the Clothier (`flag9`).
    clothier: bool,
    /// 160 — the Truffle (`flag10`; W4-d120 re-pin — the field was
    /// misnamed `party_girl`, the scan id was always the C#'s 160).
    truffle: bool,
    /// 178 — the Steampunker (`flag11`).
    steampunker: bool,
    /// 229 — the Pirate (`flag12`; was misnamed `zoologist`).
    pirate: bool,
    /// 209 — the Cyborg (`flag13`).
    cyborg: bool,
    /// 208 — the Party Girl (`flag14`; was misnamed `stylist`).
    party_girl: bool,
    /// 353 — the Stylist (`flag15`; was misnamed `witch_doctor`).
    stylist: bool,
    /// 22 — the Guide (`flag16`).
    guide: bool,
    /// 441 — the Tax Collector (`flag17`).
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
                160 => f.truffle = true,
                178 => f.steampunker = true,
                229 => f.pirate = true,
                209 => f.cyborg = true,
                208 => f.party_girl = true,
                353 => f.stylist = true,
                22 => f.guide = true,
                441 => f.tax_collector = true,
                _ => {}
            }
        }
        f
    }
}

/// `NPC.GetChat` (`NPC.cs:93766-94820`): the NPC's line. The pending
/// conditional dialogue (`nextDialogue`, `:93768-93771`) is consumed by
/// the caller (`talk_to_npc` — the cake gift needs the App); this ladder
/// is the `nextDialogue == null` body.
pub fn get_chat(npc: &Npc, ctx: &mut ChatContext) -> String {
    let flags = Flags::scan(ctx.npcs);
    // `NPC.homeless` — the port's `Npc` keeps the home as `Option`, so
    // "homeless" is "no home assigned" (`NPC.cs:6551-6553`).
    let homeless = npc.home.is_none();
    let ty = npc.kind.cs_id();
    match npc.kind {
        NpcKind::Merchant => merchant(ctx, &flags),
        NpcKind::Nurse => nurse(ctx, &flags),
        NpcKind::ArmsDealer => arms_dealer(ctx, &flags),
        NpcKind::Dryad => dryad(ctx, &flags),
        NpcKind::BoundGoblin => crate::lang::dialog(120), // `:94072-94075` (105)
        NpcKind::Demolitionist => demolitionist(ctx, &flags),
        NpcKind::Clothier => clothier(ctx, &flags),
        NpcKind::GoblinTinkerer => goblin_tinkerer(ctx, &flags, homeless),
        NpcKind::BoundWizard => crate::lang::dialog(137),
        NpcKind::Wizard => wizard(ctx, &flags, homeless),
        NpcKind::BoundMechanic => crate::lang::dialog(156), // `:94215-94218` (123)
        NpcKind::Mechanic => mechanic(ctx, &flags, homeless),
        NpcKind::Guide => guide(ctx, &flags),
        NpcKind::PartyGirl => party_girl(ctx, &flags),
        NpcKind::DyeTrader => dye_trader(ctx, &flags),
        NpcKind::Painter => painter(ctx, &flags),
        NpcKind::WitchDoctor => witch_doctor(ctx, &flags),
        NpcKind::WebbedStylist => {
            // `:94610-94613` (354) — `Lang.dialog(Main.rand.Next(315, 319))`.
            let n = ctx.next_range(315, 319);
            crate::lang::dialog(n)
        }
        NpcKind::Stylist => stylist(ctx, &flags),
        NpcKind::TravellingMerchant => travelling_merchant(ctx, &flags),
        NpcKind::Angler => angler(ctx, &flags),
        NpcKind::SleepingAngler => {
            let n = ctx.next_range(353, 356);
            crate::lang::dialog(n)
        }
        NpcKind::TaxCollector => tax_collector(ctx, &flags),
        NpcKind::DD2Bartender => bartender(ctx, ty),
        NpcKind::SleepingGolfer => {
            // `:94788-94791` (589) — the first-meeting pair.
            if ctx.next(2) != 0 {
                crate::lang::text("GolferSpecialText.FirstMeeting2")
            } else {
                crate::lang::text("GolferSpecialText.FirstMeeting1")
            }
        }
        NpcKind::Golfer => golfer(ctx),
        NpcKind::Zoologist => zoologist(ctx),
        NpcKind::Princess => princess(ctx),
        // W4-d120 — the claimed chatter kinds (the claim doc
        // `tools/_d120_claim.md`).
        NpcKind::SantaClaus => santa(ctx),
        NpcKind::Truffle => truffle(ctx, &flags),
        // W4-d120 — the sibling lanes' kinds (OldMan 37 = d34, Pirate 229
        // = d37, SkeletonMerchant 453 = d40): the arms key on the VANILLA
        // type so they compile before the kinds merge and go live the
        // moment they do (`cs_id` keeps the vanilla ids on every lane).
        _ => match ty {
            37 => old_man(ctx),
            _ => String::new(),
        },
    }
}

/// `NPC.HasSpecialEventText` (`NPC.cs:94860-94884`): the event texts — the
/// graveyard, the party, the rain, the windy day and the storm each roll
/// 1/3 and pick a random line from `<Category>SpecialText.<Event>` via
/// `GetSpecialEventTextIfNotEmpty` (`:94886-94895`) — a
/// `Language.SelectRandom` whose EMPTY answer declines the event.
///
/// The draw census (W4-d120): the 1/3 roll only fires while the event is
/// active (the C# `&&` short-circuit); a passed roll ALWAYS pays the
/// `Next(count)` category draw — even for an empty category
/// (`UnifiedRandom.Next(0)` still advances the stream, `:140`). The
/// party arm rolls `Next(3)` FIRST, then gates on
/// `NPCID.Sets.HasNoPartyText` (`:94867` — the Tax Collector 441 and the
/// Skeleton Merchant 453, `NPCID.cs:10461`): a no-party NPC still pays
/// the roll, but never the category draw.
fn special_event_text(ctx: &mut ChatContext, category: &str, npc_type: u16) -> Option<String> {
    let no_party_text = matches!(npc_type, 441 | 453);
    // The C#'s five sequential `if`s (`:94863-94882`): a returning event
    // skips every later roll, so the gates evaluate one event at a time.
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
        // `:94867` — the party's HasNoPartyText gate sits AFTER the roll.
        if event == "Party" && no_party_text {
            continue;
        }
        let prefix = format!("{category}SpecialText.{event}");
        let count = crate::lang::dialog_count(&prefix);
        let index = ctx.next(count as i32);
        let text = crate::lang::select_dialog(&prefix, index);
        if !text.is_empty() {
            return Some(text);
        }
    }
    None
}

/// A random line from a chatter category (`Language.SelectRandom` over
/// `Lang.CreateDialogFilter(prefix + ".")`, `Lang.cs:422-425`): the count
/// draw always fires (even at 0), the pick is the REVERSE index
/// (`LanguageManager.cs:403-409`).
fn chatter(ctx: &mut ChatContext, category: &str) -> String {
    let prefix = format!("{category}.");
    let count = crate::lang::dialog_count(&prefix);
    let index = ctx.next(count as i32);
    crate::lang::select_dialog(&prefix, index)
}

/// The mixed `FindAll` arm the Nurse/Tinkerer/Mechanic/Stylist ladders
/// share (`NPC.cs:93976-93984`): `Next(fixed + array.Length)` — indices
/// `>= fixed` answer `array[index - fixed]` (the FORWARD FindAll row),
/// the low indices the caller's fixed lines. The count draw always fires.
fn mixed_chatter(ctx: &mut ChatContext, category: &str, fixed: i32) -> (i32, Option<String>) {
    let prefix = format!("{category}.");
    let count = crate::lang::dialog_count(&prefix);
    let n = ctx.next(fixed + count as i32);
    if n >= fixed {
        let text = crate::lang::dialog_row(&prefix, (n - fixed) as usize);
        (n, Some(text))
    } else {
        (n, None)
    }
}

/// 17 — the Merchant (`NPC.cs:93867-93896`).
fn merchant(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "Merchant", 17) {
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

/// 18 — the Nurse (`NPC.cs:93897-93986`).
fn nurse(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "Nurse", 18) {
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
    // `:93974-93985` — the four fixed lines or a random chatter line
    // (`FindAll` FORWARD row, `Next(4 + array.Length)`).
    let (n, row) = mixed_chatter(ctx, "NurseChatter", 4);
    if let Some(text) = row {
        return text;
    }
    crate::lang::dialog(48 + n)
}

/// 19 — the Arms Dealer (`NPC.cs:93987-93995`).
fn arms_dealer(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "ArmsDealer", 19) {
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

/// 20 — the Dryad (`NPC.cs:93996-94016`).
fn dryad(ctx: &mut ChatContext, f: &Flags) -> String {
    let mut result = if let Some(text) = special_event_text(ctx, "Dryad", 20) {
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
    } else if f.truffle && ctx.next(6) == 0 {
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

/// 38 — the Demolitionist (`NPC.cs:94038-94059`).
fn demolitionist(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "Demolitionist", 38) {
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

/// 54 — the Clothier (`NPC.cs:94060-94071`).
fn clothier(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "Clothier", 54) {
        return text;
    }
    if !f.mechanic && ctx.next(2) == 0 {
        return crate::lang::dialog(110);
    }
    if f.truffle && ctx.next(6) == 0 {
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

/// 107 — the Goblin Tinkerer (`NPC.cs:94076-94121`).
fn goblin_tinkerer(ctx: &mut ChatContext, f: &Flags, homeless: bool) -> String {
    if homeless {
        let n = ctx.next(5);
        return crate::lang::dialog(121 + n);
    }
    if let Some(text) = special_event_text(ctx, "GoblinTinkerer", 107) {
        return text;
    }
    if f.mechanic && ctx.next(5) == 0 {
        return crate::lang::dialog(126);
    }
    if f.stylist && ctx.next(5) == 0 {
        return crate::lang::dialog(309);
    }
    // `:94101-94120` — five fixed lines (day/night split) or a chatter row.
    let (n, row) = mixed_chatter(ctx, "GoblinTinkererChatter", 5);
    if let Some(text) = row {
        return text;
    }
    if !ctx.day_time {
        crate::lang::dialog(127 + n)
    } else {
        crate::lang::dialog(132 + n)
    }
}

/// 108 — the Wizard (`NPC.cs:94126-94214`).
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
    if let Some(text) = special_event_text(ctx, "Wizard", 108) {
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
    // `:94854` — the filter has NO trailing dot: the rows are
    // `LuckIsCursed1/2` etc. `Language.SelectRandom(...).Value`: the count
    // draw fires even for an empty category; an empty pick answers ""
    // (the C#'s `LocalizedText.Empty.Value`), which the wizard returns.
    let count = crate::lang::dialog_count(category);
    let index = ctx.next(count as i32);
    Some(crate::lang::select_dialog(category, index))
}

/// 124 — the Mechanic (`NPC.cs:94219-94264`).
fn mechanic(ctx: &mut ChatContext, f: &Flags, homeless: bool) -> String {
    if homeless {
        let n = ctx.next(4);
        return crate::lang::dialog(157 + n);
    }
    if let Some(text) = special_event_text(ctx, "Mechanic", 124) {
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
    // `:94253-94263` — three fixed lines or a chatter row.
    let (n, row) = mixed_chatter(ctx, "MechanicChatter", 3);
    if let Some(text) = row {
        return text;
    }
    crate::lang::dialog(167 + n)
}

/// 22 — the Guide (`NPC.cs:94265-94278`).
fn guide(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "Guide", 22) {
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

/// 208 — the Party Girl (`NPC.cs:94410-94456`). W4-d120 re-pin: the old
/// port carried the TRUFFLE (160, `:94302-94341`) ladder here — the real
/// 208 arm rolls `Next(7)` UP FRONT (before the event text), then the
/// DD2/male/Stylist gates, then the seven fixed lines.
fn party_girl(ctx: &mut ChatContext, f: &Flags) -> String {
    let num11 = ctx.next(7); // `:94412` — the draw precedes every gate
    if let Some(text) = special_event_text(ctx, "PartyGirl", 208) {
        return text;
    }
    if ctx.dd2_downed_t1 && ctx.next(5) == 0 {
        return crate::lang::text("PartyGirlSpecialText.AfterDD2Tier1");
    }
    if ctx.player_male && ctx.next(5) == 0 {
        return crate::lang::dialog(268);
    }
    if f.stylist && ctx.next(5) == 0 {
        return crate::lang::dialog(310);
    }
    match num11 {
        0 => crate::lang::dialog(265),
        1 => crate::lang::dialog(266),
        2 => crate::lang::dialog(267),
        3 => crate::lang::dialog(269),
        4 => crate::lang::dialog(270),
        5 => crate::lang::dialog(271),
        _ => crate::lang::dialog(272),
    }
}

/// 207 — the Dye Trader (`NPC.cs:94383-94409`).
fn dye_trader(ctx: &mut ChatContext, f: &Flags) -> String {
    let num10 = ctx.next(3); // `:94385` — the draw precedes the gates
    if let Some(text) = special_event_text(ctx, "DyeTrader", 207) {
        return text;
    }
    if f.pirate && ctx.next(6) == 0 {
        return crate::lang::dialog(260);
    }
    crate::lang::dialog(257 + num10)
}

/// 227 — the Painter (`NPC.cs:94502-94538`).
fn painter(ctx: &mut ChatContext, f: &Flags) -> String {
    let num13 = ctx.next(5); // `:94504` — the draw precedes the gates
    if let Some(text) = special_event_text(ctx, "Painter", 227) {
        return text;
    }
    if ctx.hard_mode && ctx.next(7) == 0 {
        return crate::lang::dialog(250);
    }
    if f.party_girl && ctx.next(6) == 0 {
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

/// 228 — the Witch Doctor (`NPC.cs:94539-94568`).
fn witch_doctor(ctx: &mut ChatContext, f: &Flags) -> String {
    let num14 = ctx.next(4); // `:94541` — the draw precedes the gates
    if let Some(text) = special_event_text(ctx, "WitchDoctor", 228) {
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

/// 353 — the Stylist (`NPC.cs:94614-94749`).
fn stylist(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "Stylist", 353) {
        return text;
    }
    if ctx.blood_moon {
        let n = ctx.next(3);
        return crate::lang::dialog(304 + n);
    }
    if ctx.day_time && ctx.time < 16200.0 && ctx.next(3) == 0 {
        return crate::lang::dialog(311);
    }
    if ctx.player_male && ctx.next(3) == 0 {
        let n = ctx.next(2);
        return if n == 0 {
            crate::lang::dialog(293)
        } else {
            crate::lang::dialog(300)
        };
    }
    if !ctx.player_male && ctx.next(3) == 0 {
        let n = ctx.next(3);
        return match n {
            0 => crate::lang::dialog(291),
            1 => crate::lang::dialog(292),
            _ => crate::lang::dialog(312),
        };
    }
    // `:94666` — `(flag14 & flag7 & flag2 & flag3) && Next(5) == 0`: the
    // flag conjunction sits LEFT of the short-circuit `&&`, so a missing
    // neighbour skips the draw (only the Tax Collector's `(roll) & flag`
    // arms draw unconditionally).
    if f.party_girl && f.mechanic && f.nurse && f.arms_dealer && ctx.next(5) == 0 {
        return crate::lang::dialog(307);
    }
    if f.party_girl && ctx.next(5) == 0 {
        return crate::lang::dialog(302);
    }
    if f.demolitionist && ctx.next(5) == 0 {
        return crate::lang::dialog(303);
    }
    if f.cyborg && ctx.next(5) == 0 {
        return crate::lang::dialog(308);
    }
    // `:94682-94748` — the moon-phase fixed band: phases 0-2 carry five
    // fixed lines, 3-7 four; then the StylistChatter rows.
    let fixed = if ctx.moon_phase < 3 { 5 } else { 4 };
    let (n, row) = mixed_chatter(ctx, "StylistChatter", fixed);
    if let Some(text) = row {
        return text;
    }
    if ctx.moon_phase < 3 {
        match n {
            0 => crate::lang::dialog(287),
            1 => crate::lang::dialog(288),
            2 => crate::lang::dialog(289),
            3 => crate::lang::dialog(290),
            _ => crate::lang::dialog(294),
        }
    } else if ctx.moon_phase < 6 {
        crate::lang::dialog(295 + n)
    } else {
        match n {
            0 => crate::lang::dialog(299),
            1 => crate::lang::dialog(301),
            2 => crate::lang::dialog(313),
            _ => crate::lang::dialog(314),
        }
    }
}

/// 368 — the Travelling Merchant (`NPC.cs:94750-94753`).
fn travelling_merchant(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "TravellingMerchant", 368) {
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

/// 369 — the Angler (`NPC.cs:94758-94761`).
fn angler(ctx: &mut ChatContext, f: &Flags) -> String {
    if ctx.next(5) == 0 && ctx.angler_quests_finished > 1 {
        return chatter(ctx, "AnglerQuestChatter");
    }
    if let Some(text) = special_event_text(ctx, "Angler", 369) {
        return text;
    }
    if ctx.blood_moon {
        if ctx.next(3) == 0 {
            // `:94760` — the filter is "AnglerSpecialText.BloodMoonFishing"
            // WITHOUT a trailing dot: the rows are `BloodMoonFishing1/2`.
            let prefix = "AnglerSpecialText.BloodMoonFishing";
            let count = crate::lang::dialog_count(prefix);
            let index = ctx.next(count as i32);
            return crate::lang::select_dialog(prefix, index);
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

/// 441 — the Tax Collector (`NPC.cs:94776-94779`). Each `(roll == 0) &
/// flag` uses the NON-short-circuit `&` — the `Next(6)` draw fires even
/// when its flag is false. The ternary chain still evaluates the
/// conditions one at a time: an early true arm skips the later rolls.
fn tax_collector(ctx: &mut ChatContext, f: &Flags) -> String {
    if let Some(text) = special_event_text(ctx, "TaxCollector", 441) {
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

/// 550 — the Tavernkeep (`NPC.cs:94784-94787` → `Lang.BartenderChat`,
/// `Lang.cs:331-338`): the 1/5 DD2-tier line, else the BartenderChatter.
fn bartender(ctx: &mut ChatContext, ty: u16) -> String {
    if let Some(text) = special_event_text(ctx, "Bartender", ty) {
        return text;
    }
    if ctx.next(5) == 0 {
        let key = if ctx.dd2_downed_t3 {
            "BartenderSpecialText.AfterDD2Tier3"
        } else if ctx.dd2_downed_t2 {
            "BartenderSpecialText.AfterDD2Tier2"
        } else if !ctx.dd2_downed_t1 {
            "BartenderSpecialText.BeforeDD2Tier1"
        } else {
            "BartenderSpecialText.AfterDD2Tier1"
        };
        return crate::lang::text(key);
    }
    chatter(ctx, "BartenderChatter")
}

/// 588 — the Golfer (`NPC.cs:94792-94795`).
fn golfer(ctx: &mut ChatContext) -> String {
    if ctx.next(3) != 0 {
        if let Some(text) = special_event_text(ctx, "Golfer", 588) {
            return text;
        }
    }
    if ctx.next(3) != 0 {
        return chatter(ctx, "GolferChatter");
    }
    // `:94794` — the score band picks the quest chatter category.
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

/// 633 — the Zoologist (`NPC.cs:94796-94799`): the lycantrope check is
/// PURE (`:93753-93764`, no draws) and short-circuits the event text —
/// `HasSpecialEventText` fires at most ONCE per chat.
fn zoologist(ctx: &mut ChatContext) -> String {
    // `NPC.ShouldBestiaryGirlBeLycantrope` (`NPC.cs:93753-93764`).
    let lycantrope = if !ctx.blood_moon || ctx.day_time {
        ctx.moon_phase == 0 && !ctx.day_time
    } else {
        true
    };
    if !lycantrope {
        if let Some(text) = special_event_text(ctx, "BestiaryGirl", 633) {
            return text;
        }
    }
    // `Lang.BestiaryGirlChat` (`Lang.cs:345-353`) re-checks the (pure)
    // lycantrope flag to pick the category.
    if lycantrope {
        chatter(ctx, "BestiaryGirlLycantropeChatter")
    } else {
        chatter(ctx, "BestiaryGirlChatter")
    }
}

/// 663 — the Princess (`NPC.cs:94800-94803`).
fn princess(ctx: &mut ChatContext) -> String {
    match special_event_text(ctx, "Princess", 663) {
        Some(text) => text,
        None => chatter(ctx, "PrincessChatter"),
    }
}

/// 37 — the Old Man (`NPC.cs:94017-94037`). NO event text — the arm has
/// no `HasSpecialEventText` call. Night OR remix answers the stat band
/// (`statLifeMax < 300 || statDefense < 10` → the 85-88 quartet, else
/// 89-92); the plain day the 82-84 trio. The kind rides the d34 lane;
/// this arm keys on the vanilla type until it merges.
fn old_man(ctx: &mut ChatContext) -> String {
    if !ctx.day_time || ctx.remix_world {
        let n = ctx.next(4);
        if ctx.player_base_life_max < 300 || ctx.player_defense < 10 {
            return crate::lang::dialog(85 + n);
        }
        return crate::lang::dialog(89 + n);
    }
    let n = ctx.next(3);
    crate::lang::dialog(82 + n)
}

/// 142 — Santa (`NPC.cs:94279-94301`): `Next(3)` UP FRONT (`:94280`),
/// then the event text, then the three fixed lines 224/225/226.
fn santa(ctx: &mut ChatContext) -> String {
    let num7 = ctx.next(3); // `:94280` — the draw precedes the event text
    if let Some(text) = special_event_text(ctx, "Santa", 142) {
        return text;
    }
    crate::lang::dialog(224 + num7)
}

/// 160 — the Truffle (`NPC.cs:94302-94341`): `Next(6)` UP FRONT
/// (`:94304`), the event text, the Dryad/Clothier neighbour gates, then
/// the six fixed lines 231/233/234/235/240/241 (232/236 are the
/// neighbour lines, NOT switch arms).
fn truffle(ctx: &mut ChatContext, f: &Flags) -> String {
    let num8 = ctx.next(6); // `:94304` — the draw precedes every gate
    if let Some(text) = special_event_text(ctx, "Truffle", 160) {
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
            dd2_downed_t3: false,
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

    // ---- W4-d120: the chatter re-pin witnesses --------------------------
    //
    // Each test drives the queued draw values AND records the requested
    // maxes — the C#'s draw census (`Main.rand.Next(n)` order) is behaviour,
    // so a wrong gate order shows up as a wrong max sequence even when the
    // returned line happens to match.

    /// A draw stream that hands out the queued values (last repeats) and
    /// records every requested max into `log`.
    fn census<'a>(
        values: &'static [i32],
        log: &'a mut Vec<i32>,
    ) -> impl FnMut(i32) -> i32 + 'a {
        let mut i = 0usize;
        move |n| {
            log.push(n);
            let v = values[i.min(values.len() - 1)];
            i += 1;
            v
        }
    }

    /// The owned-values census: for the tests whose draw targets depend on
    /// the live dialog-filter count (the `{Bartender}`/`{WorldName}` rows
    /// only count while their substitutions resolve), so the expected maxes
    /// are computed, not hardcoded.
    fn census_vec(values: Vec<i32>, log: &mut Vec<i32>) -> impl FnMut(i32) -> i32 + '_ {
        let mut i = 0usize;
        move |n| {
            log.push(n);
            let v = values[i.min(values.len() - 1)];
            i += 1;
            v
        }
    }

    /// `NPC.cs:94863-94882` + `:94886-94895` — the event census: the 1/3
    /// roll fires only while the event is active, a passed roll ALWAYS pays
    /// the `Next(count)` category draw, and the pick is the REVERSE index
    /// (`LanguageManager.cs:403-409`: index 0 answers the LAST row). The
    /// Guide's party category is one row (`GuideSpecialText.Party`).
    #[test]
    fn the_event_text_pays_the_count_draw_and_picks_in_reverse() {
        init_lang();
        let guide = npc(NpcKind::Guide);
        let npcs = [guide];
        let mut log = Vec::new();
        let mut rng = census(&[0, 0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.party_is_up = true;
        let chat = get_chat(&npcs[0], &mut ctx);
        assert_eq!(
            chat,
            "Never been to a party before? You might want to check around with the others. People sometimes bring out special party favors."
        );
        assert_eq!(log, vec![3, 1], "the 1/3 roll, then Next(count=1)");
        // An EMPTY category still pays its Next(0) (`UnifiedRandom.cs:140`
        // draws Sample() even at max 0) and declines the event.
        let mut log = Vec::new();
        let mut rng = census(&[0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.zone_graveyard = true;
        assert!(special_event_text(&mut ctx, "NoSuchCategory", 22).is_none());
        assert_eq!(log, vec![3, 0], "the roll passes, Next(0) still fires");
    }

    /// `NPC.cs:94867` — the party's `HasNoPartyText` gate (`NPCID.cs:10461`:
    /// 441/453) sits AFTER the 1/3 roll: the Tax Collector pays the roll but
    /// never the category draw. `:94778` — his `(roll == 0) & flag` arms use
    /// the NON-short-circuit `&`, so all four `Next(6)` draws fire even with
    /// every neighbour absent.
    #[test]
    fn the_tax_collector_pays_the_party_roll_but_never_its_category_draw() {
        init_lang();
        let tax = npc(NpcKind::TaxCollector);
        let npcs = [tax];
        let mut log = Vec::new();
        let mut rng = census(&[0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.party_is_up = true;
        let chat = get_chat(&npcs[0], &mut ctx);
        assert_eq!(chat, crate::lang::dialog(364), "the Next(364, 371) band");
        assert_eq!(
            log,
            vec![3, 6, 6, 6, 6, 7],
            "party roll (no category draw), four unconditional Next(6), Next(7)"
        );
    }

    /// `NPC.cs:94871/:94879` — the rain arm gates on `raining &&
    /// !IsItStorming`; a storm answers the Storm category instead. Both
    /// picks are the reverse index over the two-row categories.
    #[test]
    fn the_weather_arms_split_rain_from_storm() {
        init_lang();
        let guide = npc(NpcKind::Guide);
        let npcs = [guide];
        let mut log = Vec::new();
        let mut rng = census(&[0, 0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.raining = true;
        assert_eq!(
            get_chat(&npcs[0], &mut ctx),
            "Don't be alarmed, it is just water. I hear certain species of fish can fly when it rains!",
            "reverse index 0 = Rain2"
        );
        assert_eq!(log, vec![3, 2]);
        let mut log = Vec::new();
        let mut rng = census(&[0, 0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.raining = true;
        ctx.storming = true;
        assert_eq!(
            get_chat(&npcs[0], &mut ctx),
            "Lightning is no joke, very powerful fiends come out in this weather.",
            "the storm skips the rain arm — Storm2"
        );
        assert_eq!(log, vec![3, 2], "one roll only: the rain gate is off");
    }

    /// `NPC.cs:94410-94456` — the REAL Party Girl (208) ladder: `Next(7)`
    /// fires UP FRONT (before the event text), then the DD2/male/Stylist
    /// gates, then the seven fixed lines (265/266/267/269/270/271/272 —
    /// 268 is the male-gate line, NOT a switch arm).
    #[test]
    fn the_party_girl_rolls_seven_up_front() {
        init_lang();
        let pg = npc(NpcKind::PartyGirl);
        let npcs = [pg];
        let mut log = Vec::new();
        let mut rng = census(&[3], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.player_male = false; // isolate the up-front draw (the male gate rolls Next(5))
        assert_eq!(
            get_chat(&npcs[0], &mut ctx),
            "Put up a disco ball and then I'll show you how to party.",
            "num11 = 3 → dialog 269"
        );
        assert_eq!(log, vec![7], "no gates passed: only the up-front draw");
        // The party event still answers — but AFTER the Next(7) was paid.
        let mut log = Vec::new();
        let mut rng = census(&[0, 0, 0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.party_is_up = true;
        assert_eq!(
            get_chat(&npcs[0], &mut ctx),
            "At last, my time has come!",
            "reverse index 0 over Party_1/Party_2 = Party_2"
        );
        assert_eq!(log, vec![7, 3, 2], "Next(7) precedes the event census");
    }

    /// `NPC.cs:94796-94799` — the lycantrope check is PURE and short-circuits
    /// the event text (`||`): a blood-moon night Zoologist pays NO event
    /// rolls, only the lycantrope category's `Next(2)` (reverse pick).
    #[test]
    fn the_lycantrope_zoologist_skips_the_event_census() {
        init_lang();
        let zoo = npc(NpcKind::Zoologist);
        let npcs = [zoo];
        let mut log = Vec::new();
        let mut rng = census(&[0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.blood_moon = true;
        ctx.day_time = false;
        ctx.party_is_up = true;
        assert_eq!(
            get_chat(&npcs[0], &mut ctx),
            "Rawgh! Roo! Danger. Me. Wraaaath!",
            "reverse index 0 = the LAST lycantrope row"
        );
        assert_eq!(log, vec![2], "no 1/3 rolls: the event text never ran");
    }

    /// `NPC.cs:93976-93984` — the Nurse's mixed arm: `Next(4 + FindAll.len)`
    /// with the FORWARD row for indices >= 4 (NurseChatter has one row),
    /// the low indices the four fixed lines 48..51.
    #[test]
    fn the_nurse_mixed_arm_walks_findall_forward() {
        init_lang();
        let nurse = npc(NpcKind::Nurse);
        let npcs = [nurse];
        // The row carries `{Bartender}` — it only counts while that
        // substitution resolves, so the census computes the count live.
        let count = crate::lang::dialog_count("NurseChatter.") as i32;
        let mut log = Vec::new();
        let mut rng = census_vec(vec![1, 1, 4], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        let chat = get_chat(&npcs[0], &mut ctx);
        if count > 0 {
            assert_eq!(chat, crate::lang::dialog_row("NurseChatter.", 0));
        }
        assert_eq!(log, vec![5, 3, 4 + count], "Next(5) deaths, Next(3), Next(4+len)");
        let mut log = Vec::new();
        let mut rng = census_vec(vec![1, 1, 0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        assert_eq!(
            get_chat(&npcs[0], &mut ctx),
            "Turn your head and cough.",
            "index 0 → dialog 48"
        );
        assert_eq!(log, vec![5, 3, 4 + count]);
    }

    /// `NPC.cs:94682-94748` — the Stylist's moon-phase band: phases 0-2
    /// carry five fixed lines (287/288/289/290/294), 3-7 four; the mixed
    /// draw is `Next(fixed + StylistChatter.len)`. The flag conjunctions
    /// (`:94666`) sit LEFT of `&&`, so absent neighbours skip the draws.
    #[test]
    fn the_stylist_moon_band_sizes_the_mixed_draw() {
        init_lang();
        let stylist = npc(NpcKind::Stylist);
        let npcs = [stylist];
        // The chatter row carries `{Bartender}` — count it live.
        let count = crate::lang::dialog_count("StylistChatter.") as i32;
        let mut log = Vec::new();
        let mut rng = census_vec(vec![1, 1, 5], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        let chat = get_chat(&npcs[0], &mut ctx);
        if count > 0 {
            assert_eq!(chat, crate::lang::dialog_row("StylistChatter.", 0));
        } else {
            assert_eq!(chat, "Sit down for a second and I'll have you steppin' razor.");
        }
        assert_eq!(log, vec![3, 3, 5 + count], "morning + male rolls, then Next(5+len)");
        let mut log = Vec::new();
        let mut rng = census_vec(vec![1, 1, 0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.moon_phase = 3;
        assert_eq!(
            get_chat(&npcs[0], &mut ctx),
            "Either you have style, or you get styled.",
            "phase 3: four fixed lines, index 0 → dialog 295"
        );
        assert_eq!(log, vec![3, 3, 4 + count], "the fixed band shrinks to 4");
    }

    /// `NPC.cs:94784-94787` → `Lang.BartenderChat` (`Lang.cs:331-338`): the
    /// 1/5 tier line reads T3 first, then T2, then the !T1 default.
    #[test]
    fn the_bartender_tier_line_reads_t3_first() {
        init_lang();
        let bar = npc(NpcKind::DD2Bartender);
        let npcs = [bar];
        let mut log = Vec::new();
        let mut rng = census(&[0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.dd2_downed_t3 = true;
        assert_eq!(
            get_chat(&npcs[0], &mut ctx),
            "You really kept the full force of the Old One's Army at bay? Maybe you should visit Etheria some time."
        );
        assert_eq!(log, vec![5]);
        let mut log = Vec::new();
        let mut rng = census(&[0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        assert_eq!(
            get_chat(&npcs[0], &mut ctx),
            "We should really do something about the Old One's Army. Ask me about the Eternia Crystal if you want to learn more.",
            "no tiers downed → BeforeDD2Tier1"
        );
        assert_eq!(log, vec![5]);
    }

    /// `NPC.cs:94760` — the Angler's blood-moon fishing filter carries NO
    /// trailing dot (`AnglerSpecialText.BloodMoonFishing` matches
    /// `BloodMoonFishing1/2`); the pick is the reverse index.
    #[test]
    fn the_angler_blood_moon_filter_has_no_trailing_dot() {
        init_lang();
        let angler = npc(NpcKind::Angler);
        let npcs = [angler];
        let mut log = Vec::new();
        let mut rng = census(&[1, 0, 0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.blood_moon = true;
        assert_eq!(
            get_chat(&npcs[0], &mut ctx),
            "You should try fishing when the water is all bloody! It's to die for!",
            "reverse index 0 = BloodMoonFishing2"
        );
        assert_eq!(log, vec![5, 3, 2], "quest roll, 1/3 roll, Next(count=2)");
    }

    /// `NPC.cs:94843-94858` — the wizard's luck gate: the 1/3 roll, the
    /// NormalizedLuck band, then the dot-less filter's reverse pick. The
    /// cursed rows carry `{WorldName}` in row 2 — count them live.
    #[test]
    fn the_wizard_luck_band_picks_the_cursed_rows_in_reverse() {
        init_lang();
        let mut wizard = npc(NpcKind::Wizard);
        wizard.home = Some((10, 10)); // housed — the homeless arm is separate
        let npcs = [wizard];
        let count = crate::lang::dialog_count("WizardSpecialText.LuckIsCursed") as i32;
        assert!(count >= 1, "LuckIsCursed1 carries no variables");
        let mut log = Vec::new();
        let mut rng = census_vec(vec![0, 0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.player_luck = -1.0; // NormalizedLuck -1 → LuckIsCursed
        assert_eq!(
            get_chat(&npcs[0], &mut ctx),
            crate::lang::select_dialog("WizardSpecialText.LuckIsCursed", 0),
            "reverse index 0 = the LAST matching cursed row"
        );
        assert_eq!(log, vec![3, count]);
    }

    /// `Player.NormalizedLuck` (`Player.cs:4107-4122`): the ±cap ratio
    /// (`luckMaximumCap = 1f`, `luckMinimumCap = -0.7f`) clamped to [-1, 1].
    #[test]
    fn normalized_luck_divides_by_the_sign_cap_and_clamps() {
        assert_eq!(normalized_luck(0.0), 0.0);
        assert_eq!(normalized_luck(0.5), 0.5);
        assert_eq!(normalized_luck(1.0), 1.0);
        assert_eq!(normalized_luck(2.0), 1.0, "the +1 cap clamps");
        assert!((normalized_luck(-0.35) - -0.5).abs() < 1e-6, "0 - (-0.35 / -0.7)");
        assert!((normalized_luck(-0.7) - -1.0).abs() < 1e-6, "the -0.7 cap");
        assert_eq!(normalized_luck(-1.4), -1.0, "the -1 clamp");
    }

    /// `ConditionalDialogue.cs:51-66` — the FreeCakeDialogue talk arm: the
    /// latch clears, the cake (3750) rides `QuickSpawnItem` → the inventory
    /// slot walk (useStyle 1 → the hotbar forward pass), and the text's
    /// `Next(1, 4)` draw fires AFTER the gift. The draw rides the bare
    /// `Main.rand` stream — the flow app's world-seed-0 global.
    #[test]
    fn the_party_girl_gifts_the_cake_then_draws_the_text() {
        init_lang();
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        let p = app.player.as_ref().unwrap();
        let mut pg = npc(NpcKind::PartyGirl);
        pg.x = p.x + 16.0;
        pg.y = p.y;
        app.npcs.push(pg);
        app.world_events.free_cake = true;
        app.place_down = true;
        let text = crate::world_sim::npcs::talk_to_npc(&mut app, false)
            .expect("the cake dialogue answers");
        let n = tr_core::UnifiedRandom::new(0).next_range(1, 4);
        assert_eq!(
            text,
            crate::lang::text(&format!("PartyGirlSpecialText.Cake{n}")),
            "the Next(1, 4) text draw rides the seed-0 global stream"
        );
        assert!(!app.world_events.free_cake, "`:60` clears the latch");
        assert_eq!(app.inventory[0].kind, 3750, "the hotbar forward pass");
        assert_eq!(app.inventory[0].count, 1);
        assert_eq!(app.npc_chat_text, text, "`Main.cs:43082`");
        // The same NPC re-clicked falls through (`Main.cs:43077`).
        assert!(crate::world_sim::npcs::talk_to_npc(&mut app, false).is_none());
    }

    /// `Player.cs:7193-7199` — `GetOrDropItem`: a full inventory overflows
    /// to `Item.RequestNewItem` at the player centre (the bobber-catch
    /// shape); the gift still clears the latch and answers the text.
    #[test]
    fn a_full_inventory_overflows_the_cake_to_a_world_drop() {
        init_lang();
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        let (cx, cy) = app.player.as_ref().unwrap().center();
        for slot in app.inventory.iter_mut() {
            *slot = crate::item::ItemSlot {
                kind: 2, // Dirt Block — a full 9999 stack never merges
                name: crate::registry::name(2),
                count: 9999,
                prefix: 0,
            };
        }
        let mut pg = npc(NpcKind::PartyGirl);
        pg.x = cx + 16.0;
        pg.y = cy;
        app.npcs.push(pg);
        app.world_events.free_cake = true;
        app.place_down = true;
        let text = crate::world_sim::npcs::talk_to_npc(&mut app, false)
            .expect("the overflow still answers the cake text");
        // The overflow drop pays the velocity/phase triplet first (the cake
        // is not prefixable — `roll_prefix` draws nothing), THEN the text's
        // Next(1, 4) — all on the same seed-0 global stream.
        let mut expect = tr_core::UnifiedRandom::new(0);
        expect.next_float();
        expect.next_float();
        expect.next_float();
        let n = expect.next_range(1, 4);
        assert_eq!(
            text,
            crate::lang::text(&format!("PartyGirlSpecialText.Cake{n}"))
        );
        assert!(!app.world_events.free_cake);
        assert_eq!(app.items.len(), 1, "the overflow drop");
        assert_eq!(app.items[0].kind, 3750);
        assert_eq!(app.items[0].count, 1);
        assert_eq!(app.items[0].prefix, 0, "the gift carries no prefix roll");
        // The port's pinned spawn shape: top-left = centre + (1, 0) — the
        // same offset every `spawn_item_drop_px` site uses (the C# centres
        // the 14x14 box on the player centre; the 1-px x-shift is the
        // recorded port convention, `world_sim/items.rs`).
        assert_eq!(app.items[0].pos, glam::Vec2::new(cx + 1.0, cy));
    }

    // ---- W4-d120: the claimed kinds' arms (batch 1: 37/142/160) --------

    /// `NPC.cs:94017-94037` — the Old Man: NO event text, the night/remix
    /// stat band (`statLifeMax < 300 || statDefense < 10` → 85-88, else
    /// 89-92), the plain day's 82-84 trio. The kind rides the d34 lane —
    /// the arm is driven directly until it merges.
    #[test]
    fn the_old_man_stat_band_splits_night_from_day() {
        init_lang();
        let guide = npc(NpcKind::Guide); // any live shell — the arm is type-keyed
        let npcs = [guide];
        let mut log = Vec::new();
        let mut rng = census(&[2], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.day_time = false; // night: the stat band
        assert_eq!(old_man(&mut ctx), crate::lang::dialog(87), "weak band 85+2");
        assert_eq!(log, vec![4], "one Next(4), no event roll");
        // The strong band: life >= 300 AND defense >= 10.
        let mut log = Vec::new();
        let mut rng = census(&[0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.day_time = false;
        ctx.player_base_life_max = 300;
        ctx.player_defense = 10;
        assert_eq!(old_man(&mut ctx), crate::lang::dialog(89), "strong band 89+0");
        // The plain day: the trio.
        let mut log = Vec::new();
        let mut rng = census(&[1], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        assert_eq!(old_man(&mut ctx), crate::lang::dialog(83), "day 82+1");
        assert_eq!(log, vec![3]);
        // Remix answers the night band even by day (`:94019`).
        let mut log = Vec::new();
        let mut rng = census(&[3], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.remix_world = true;
        assert_eq!(old_man(&mut ctx), crate::lang::dialog(88));
        assert_eq!(log, vec![4]);
    }

    /// `NPC.cs:94279-94301` — Santa: `Next(3)` UP FRONT (before the event
    /// text), then the three fixed lines 224/225/226.
    #[test]
    fn the_santa_rolls_three_up_front() {
        init_lang();
        let santa = npc(NpcKind::SantaClaus);
        let npcs = [santa];
        let mut log = Vec::new();
        let mut rng = census(&[1], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        assert_eq!(
            get_chat(&npcs[0], &mut ctx),
            "Care to bake me some cookies?",
            "index 1 answers dialog 225"
        );
        assert_eq!(log, vec![3], "the up-front draw only");
    }

    /// `NPC.cs:94302-94341` — the Truffle: `Next(6)` UP FRONT, the event
    /// text, the Dryad/Clothier gates (each `Next(6)`), then the six fixed
    /// lines 231/233/234/235/240/241 — 232/236 are the neighbour lines.
    #[test]
    fn the_truffle_rolls_six_up_front_and_gates_the_neighbours() {
        init_lang();
        let truffle = npc(NpcKind::Truffle);
        let npcs = [truffle];
        let mut log = Vec::new();
        let mut rng = census(&[4], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        assert_eq!(get_chat(&npcs[0], &mut ctx), crate::lang::dialog(240), "arm 4");
        assert_eq!(log, vec![6], "no neighbours — no gate draws");
        // With a Dryad next door: the gate's Next(6) fires; a 0 answers 232.
        let dryad = npc(NpcKind::Dryad);
        let npcs = [npc(NpcKind::Truffle), dryad];
        let mut log = Vec::new();
        let mut rng = census(&[0, 0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        assert_eq!(get_chat(&npcs[0], &mut ctx), crate::lang::dialog(232));
        assert_eq!(log, vec![6, 6], "the up-front draw + the Dryad gate");
        // A passed event roll short-circuits the gates but NOT the up-front
        // draw (`:94304` precedes `HasSpecialEventText`).
        let npcs = [npc(NpcKind::Truffle)];
        let mut log = Vec::new();
        let mut rng = census(&[5, 0, 0], &mut log);
        let mut ctx = context(&npcs, &mut rng);
        ctx.party_is_up = true;
        let text = get_chat(&npcs[0], &mut ctx);
        assert_eq!(
            text,
            crate::lang::select_dialog("TruffleSpecialText.Party", 0),
            "the event text wins over the fixed arm 5"
        );
        assert_eq!(log[0], 6, "the up-front draw precedes the event roll");
        assert_eq!(log[1], 3, "the 1/3 event roll");
    }
}
