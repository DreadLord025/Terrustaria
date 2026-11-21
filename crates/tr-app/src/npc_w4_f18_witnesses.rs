//! W4-f18 first-20 port — the stat witnesses for the twenty kinds 355-380
//! + 430 (the census doc `docs/research/381-npc-kinds-351-697-census.md`'s
//! "first 20 missing" list). WRITTEN, NOT RUN (code-only mode — no compiler
//! or test runner was invoked on this branch; the convention the prior
//! waves used).
//!
//! Each test pins one kind's FULL `SetDefaults` stat row against the C#
//! citation, through the port's tables (`Npc::new` + the `Npc` accessors /
//! `entry_stats`): the vanilla type (`cs_id`), the aiStyle, the hitbox,
//! damage, defense, lifeMax, knockBackResist, value, the hit/death sounds,
//! the protocol-id round-trip and the bestiary resolution. The folded
//! kinds (the Duck2 363 / DuckWhite2 365 / TruffleWormDigger 375 /
//! CultistArcherWhite 380 — Hide=true) resolve their bestiary credit onto
//! the walker/head row, so their `cs_id`/`net_id` ride the FOLDED id (the
//! s1_f_3 invariant); the render key + the draw sheet keep the true vanilla
//! type (`render_key` / `texture_net_id`).
//!
//! The stat values are the `f18_stats.json` census (the exact `SetDefaults`
//! fields, read arm-by-arm from `NPC.cs`); the citations below name each
//! kind's arm.

#![cfg(test)]

use crate::npc::{Npc, NpcKind, SlimeVariant};

/// One kind's full-row pin. `(kind, vanilla_type, ai_style, w, h, dmg, def,
/// life, kb, value, hit_sound, death_sound)` — the sounds are the port's
/// `hit_sound`/`death_sound` string façades (`NPC_Hit_N` / `NPC_Killed_N`).
fn pin(
    kind: NpcKind,
    ty: u16,
    ai: u8,
    w: f32,
    h: f32,
    dmg: i32,
    def: i32,
    life: i32,
    kb: f32,
    value: f32,
    hit: &str,
    death: &str,
) {
    let shell = Npc::new(kind, 0.0, 0.0, 1, 0.0);
    assert_eq!(kind.ai_style(), ai, "{kind:?} aiStyle");
    assert_eq!(kind.size(), (w, h), "{kind:?} size");
    assert_eq!(shell.size(), (w, h), "{kind:?} npc size");
    assert_eq!(kind.contact_damage(), dmg, "{kind:?} damage");
    assert_eq!(shell.defense(), def, "{kind:?} defense");
    assert_eq!(kind.max_hp(), life, "{kind:?} lifeMax");
    assert_eq!(shell.hp, life, "{kind:?} spawn hp");
    assert_eq!(shell.knock_back_resist(), kb, "{kind:?} kb");
    assert_eq!(shell.value(), value, "{kind:?} value");
    assert_eq!(kind.hit_sound(), Some(hit), "{kind:?} hit sound");
    assert_eq!(kind.death_sound(), Some(death), "{kind:?} death sound");
    // The protocol id round-trips (the S1-F-1 bijection).
    assert_eq!(NpcKind::from_u16(kind.to_u16()), Some(kind), "{kind:?} round-trip");
    // The bestiary resolves (the entry_for_npc expect never panics).
    let entry = crate::bestiary::entry_for_npc(kind, SlimeVariant::Blue);
    // The folded kinds' cs_id rides the FOLDED net id (the s1_f_3
    // invariant: cs_id == the entry's net_id when an entry resolves).
    assert_eq!(kind.cs_id() as i32, entry.net_id, "{kind:?} cs_id == entry net");
    let _ = ty; // the vanilla type is pinned per-kind below (the folds differ)
}

// ---- batch 1: the five critters 355-359 (`NPC.cs:13517-13586`) ----

#[test]
fn w4_f18_firefly_355_full_row() {
    // `NPC.cs:13517-13530` — 10x10, aiStyle 64, dmg 0, def 0, lifeMax 5,
    // noGravity, kb 1 (the class default), value 0, NPCHit1/NPCDeath1.
    pin(NpcKind::Firefly, 355, 64, 10.0, 10.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::Firefly.cs_id(), 355);
    assert!(NpcKind::Firefly.no_gravity(), "the strike-time noGravity (kb 1)");
}

#[test]
fn w4_f18_butterfly_356_full_row() {
    // `NPC.cs:13531-13544` — 10x10, aiStyle 65, the zero-stat row, kb 1,
    // value 0, NPCHit1/NPCDeath1, catchItem 1994 + the colour style.
    pin(NpcKind::Butterfly, 356, 65, 10.0, 10.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::Butterfly.cs_id(), 356);
    assert!(NpcKind::Butterfly.no_gravity());
}

#[test]
fn w4_f18_worm_357_full_row() {
    // `NPC.cs:13545-13558` — 10x4, aiStyle 66, the zero-stat row, friendly,
    // kb 1, value 0, NPCHit1/NPCDeath1, catchItem 2002. NO noGravity.
    pin(NpcKind::Worm, 357, 66, 10.0, 4.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::Worm.cs_id(), 357);
    assert!(!NpcKind::Worm.no_gravity(), "the grounded crawler");
}

#[test]
fn w4_f18_lightning_bug_358_full_row() {
    // `NPC.cs:13559-13572` — 12x12, aiStyle 64, the zero-stat row, kb 1,
    // value 0, NPCHit1/NPCDeath1, catchItem 2004, the teal glow.
    pin(NpcKind::LightningBug, 358, 64, 12.0, 12.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::LightningBug.cs_id(), 358);
    assert!(NpcKind::LightningBug.no_gravity());
}

#[test]
fn w4_f18_snail_359_full_row() {
    // `NPC.cs:13573-13586` — 12x12, aiStyle 67, the zero-stat row, kb 1,
    // value 0, NPCHit1/NPCDeath1, catchItem 2006, noGravity.
    pin(NpcKind::Snail, 359, 67, 12.0, 12.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::Snail.cs_id(), 359);
    assert!(NpcKind::Snail.no_gravity());
}

// ---- batch 2: the five critters 360-364 (`NPC.cs:13587-13648`) ----

#[test]
fn w4_f18_glowing_snail_360_full_row() {
    // `NPC.cs:13587-13600` — 14x14, aiStyle 67, the zero-stat row, kb 1,
    // value 0, NPCHit1/NPCDeath1, catchItem 2007, the blue glow.
    pin(NpcKind::GlowingSnail, 360, 67, 14.0, 14.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::GlowingSnail.cs_id(), 360);
    assert!(NpcKind::GlowingSnail.no_gravity());
}

#[test]
fn w4_f18_frog_361_full_row() {
    // `NPC.cs:13601-13612` — 12x10, aiStyle 7 (the town-critter path), the
    // zero-stat row, kb 1, value 0, NPCHit1/NPCDeath1, catchItem 2121.
    pin(NpcKind::Frog, 361, 7, 12.0, 10.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::Frog.cs_id(), 361);
    assert!(!NpcKind::Frog.no_gravity(), "the grounded town critter");
}

#[test]
fn w4_f18_duck_362_full_row() {
    // `NPC.cs:13613-13624` — 22x26, aiStyle 7, the zero-stat row, kb 1,
    // value 0, NPCHit1/NPCDeath1, catchItem 2122.
    pin(NpcKind::Duck, 362, 7, 22.0, 26.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::Duck.cs_id(), 362);
}

#[test]
fn w4_f18_duck2_363_full_row() {
    // `NPC.cs:13625-13636` — 28x22, aiStyle 68, the zero-stat row, kb 1,
    // value 0, NPCHit1/NPCDeath1, catchItem 2122. Hide=true: the bestiary
    // folds onto the Duck 362, so cs_id/net_id ride 362; the render key +
    // the draw sheet keep 363.
    pin(NpcKind::Duck2, 363, 68, 28.0, 22.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::Duck2.cs_id(), 362, "the folded credit");
    assert_eq!(crate::render::entities::texture_net_id(NpcKind::Duck2), 363, "the own draw sheet");
}

#[test]
fn w4_f18_duck_white_364_full_row() {
    // `NPC.cs:13637-13648` — 22x26, aiStyle 7, the zero-stat row, kb 1,
    // value 0, NPCHit1/NPCDeath1, catchItem 2123.
    pin(NpcKind::DuckWhite, 364, 7, 22.0, 26.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::DuckWhite.cs_id(), 364);
}

// ---- batch 3: the five critters 365-375 (`NPC.cs:13649-13795`) ----

#[test]
fn w4_f18_duck_white2_365_full_row() {
    // `NPC.cs:13649-13660` — 28x22, aiStyle 68, the zero-stat row, kb 1,
    // value 0, NPCHit1/NPCDeath1, catchItem 2123. Hide=true: folds onto the
    // DuckWhite 364; the render key + the draw sheet keep 365.
    pin(NpcKind::DuckWhite2, 365, 68, 28.0, 22.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::DuckWhite2.cs_id(), 364, "the folded credit");
    assert_eq!(crate::render::entities::texture_net_id(NpcKind::DuckWhite2), 365);
}

#[test]
fn w4_f18_scorpion_black_366_full_row() {
    // `NPC.cs:13661-13672` (the shared 366/367 arm) — 26x18, aiStyle 7, the
    // zero-stat row, kb 1, value 0, NPCHit1/NPCDeath1, catchItem 2156.
    pin(NpcKind::ScorpionBlack, 366, 7, 26.0, 18.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::ScorpionBlack.cs_id(), 366);
}

#[test]
fn w4_f18_scorpion_367_full_row() {
    // `NPC.cs:13661-13672` — 26x18, aiStyle 7, the zero-stat row, kb 1,
    // value 0, NPCHit1/NPCDeath1, catchItem 2157 (2156 + type - 366).
    pin(NpcKind::Scorpion, 367, 7, 26.0, 18.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::Scorpion.cs_id(), 367);
}

#[test]
fn w4_f18_truffle_worm_374_full_row() {
    // `NPC.cs:13762-13776` — 20x10, aiStyle 66, the zero-stat row, kb 1,
    // value 0, NPCHit1/NPCDeath1, catchItem 2673, rarity 4,
    // dontTakeDamageFromHostiles. NOT friendly (the head default).
    pin(NpcKind::TruffleWorm, 374, 66, 20.0, 10.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::TruffleWorm.cs_id(), 374);
    assert!(!NpcKind::TruffleWorm.no_gravity(), "the grounded crawler");
}

#[test]
fn w4_f18_truffle_worm_digger_375_full_row() {
    // `NPC.cs:13777-13795` — 10x10, aiStyle 6 (the lone worm head), the
    // zero-stat row, kb 0, value 0, NPCHit1/NPCDeath1, catchItem 2673,
    // rarity 4, noGravity/noTileCollide/behindTiles. Hide=true: folds onto
    // the TruffleWorm 374; the render key + the draw sheet keep 375. The
    // strike-time no_gravity reads FALSE (kb 0 — the W3-i11 convention).
    pin(NpcKind::TruffleWormDigger, 375, 6, 10.0, 10.0, 0, 0, 5, 0.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::TruffleWormDigger.cs_id(), 374, "the folded credit");
    assert_eq!(crate::render::entities::texture_net_id(NpcKind::TruffleWormDigger), 375);
    assert!(!NpcKind::TruffleWormDigger.no_gravity(), "kb 0 — the Y-pop never reaches it");
    assert!(NpcKind::TruffleWormDigger.behind_tiles(), "the behindTiles row");
    assert!(NpcKind::TruffleWormDigger.is_worm(), "the AI_006 lone head");
    assert!(!NpcKind::TruffleWormDigger.is_worm_head(), "no chain (the StardustWormHead precedent)");
    assert_eq!(NpcKind::TruffleWormDigger.worm_tunables(), (6.0, 0.15), "the :51730 tunables");
}

// ---- batch 4: the five kinds 377-430 (`NPC.cs:13811-13855` / `:8612`) ----

#[test]
fn w4_f18_grasshopper_377_full_row() {
    // `NPC.cs:13811-13824` — 14x10, aiStyle 1 (the slime hop machine), the
    // zero-stat row, friendly, kb 1, value 0, NPCHit1/NPCDeath1, catchItem
    // 2740.
    pin(NpcKind::Grasshopper, 377, 1, 14.0, 10.0, 0, 0, 5, 1.0, 0.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::Grasshopper.cs_id(), 377);
    assert!(!NpcKind::Grasshopper.no_gravity(), "the grounded hopper");
}

#[test]
fn w4_f18_chattering_teeth_bomb_378_full_row() {
    // `NPC.cs:13825-13838` — 20x26, aiStyle 41, dmg 120, def 30, lifeMax
    // 200, kb 0.5, value 450, NPCHit1/NPCDeath1.
    pin(NpcKind::ChatteringTeethBomb, 378, 41, 20.0, 26.0, 120, 30, 200, 0.5, 450.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::ChatteringTeethBomb.cs_id(), 378);
}

#[test]
fn w4_f18_cultist_archer_blue_379_full_row() {
    // `NPC.cs:13839-13855` — 18x40, aiStyle 3, dmg 45, def 14, lifeMax 210,
    // kb 0.55, value 1000, NPCHit1/NPCDeath1, lavaImmune, chaseable false,
    // netAlways.
    pin(NpcKind::CultistArcherBlue, 379, 3, 18.0, 40.0, 45, 14, 210, 0.55, 1000.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::CultistArcherBlue.cs_id(), 379);
    let shell = Npc::new(NpcKind::CultistArcherBlue, 0.0, 0.0, 1, 0.0);
    assert!(!shell.chaseable, "the chaseable = false row");
    assert_eq!(NpcKind::CultistArcherBlue.ai3_walk_speed(), Some(1.0), "the shooter default");
}

#[test]
fn w4_f18_cultist_archer_white_380_full_row() {
    // `NPC.cs:13839-13855` (the shared 379/380 arm) — 18x40, aiStyle 3,
    // dmg 45, def 14, lifeMax 210, kb 0.55, value 1000, NPCHit1/NPCDeath1,
    // lavaImmune, chaseable false, netAlways. Hide=true: folds onto the
    // CultistArcherBlue 379; the render key + the draw sheet keep 380.
    pin(NpcKind::CultistArcherWhite, 380, 3, 18.0, 40.0, 45, 14, 210, 0.55, 1000.0, "NPC_Hit_1", "NPC_Killed_1");
    assert_eq!(NpcKind::CultistArcherWhite.cs_id(), 379, "the folded credit");
    assert_eq!(crate::render::entities::texture_net_id(NpcKind::CultistArcherWhite), 380);
    let shell = Npc::new(NpcKind::CultistArcherWhite, 0.0, 0.0, 1, 0.0);
    assert!(!shell.chaseable);
}

#[test]
fn w4_f18_armed_zombie_430_full_row() {
    // `NPC.cs:8612-8624` — 18x40, aiStyle 3 (the charge family), dmg 14,
    // def 6, lifeMax 45, kb 0.5, value 60, NPCHit1/NPCDeath2.
    pin(NpcKind::ArmedZombie, 430, 3, 18.0, 40.0, 14, 6, 45, 0.5, 60.0, "NPC_Hit_1", "NPC_Killed_2");
    assert_eq!(NpcKind::ArmedZombie.cs_id(), 430);
    assert_eq!(NpcKind::ArmedZombie.ai3_walk_speed(), Some(1.0), "the charge cap");
    // The 430 has NO DebuffImmunitySets row (the null-row else-branch) —
    // not immune to the Confused debuff 31 (the critters above are).
    let buffs = crate::npc_buffs::NpcBuffs::for_kind(NpcKind::ArmedZombie);
    assert!(!buffs.buff_immune[31], "no Confused immunity");
}

// ---- the roster-level invariants the twenty kinds feed ----

#[test]
fn w4_f18_the_twenty_kinds_round_trip_and_resolve() {
    // Every one of the twenty kinds round-trips its protocol id, resolves
    // a bestiary entry, and keeps its FindFrame inside its vanilla strip
    // (the M4-NPC-3t invariant, the per-kind frame counts).
    let kinds = [
        NpcKind::Firefly, NpcKind::Butterfly, NpcKind::Worm, NpcKind::LightningBug, NpcKind::Snail,
        NpcKind::GlowingSnail, NpcKind::Frog, NpcKind::Duck, NpcKind::Duck2, NpcKind::DuckWhite,
        NpcKind::DuckWhite2, NpcKind::ScorpionBlack, NpcKind::Scorpion, NpcKind::TruffleWorm, NpcKind::TruffleWormDigger,
        NpcKind::Grasshopper, NpcKind::ChatteringTeethBomb, NpcKind::CultistArcherBlue, NpcKind::CultistArcherWhite, NpcKind::ArmedZombie,
    ];
    for kind in kinds {
        assert_eq!(NpcKind::from_u16(kind.to_u16()), Some(kind), "{kind:?} round-trip");
        assert!(crate::bestiary::has_entry(kind, SlimeVariant::Blue) || crate::bestiary::try_entry_for_npc(kind, SlimeVariant::Blue).is_some(), "{kind:?} resolves an entry");
        let rows = crate::render::entities::npc_rows(crate::render::entities::texture_net_id(kind));
        let mut n = Npc::new(kind, 0.0, 0.0, 1, 0.0);
        // A few velocity/state samples — the frame must stay in-strip.
        for (vx, vy) in [(0.0, 0.0), (2.5, 0.0), (-2.5, 0.0), (0.0, 3.0), (0.0, -3.0)] {
            n.vx = vx;
            n.vy = vy;
            n.frame = 0;
            n.frame_counter = 0.0;
            for _ in 0..24 {
                n.find_frame();
                assert!(n.frame < rows, "{kind:?} row {} of {rows}", n.frame);
            }
        }
    }
}
