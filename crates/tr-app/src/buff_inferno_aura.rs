//! W4-f12 gap 2 — the Inferno (116) aura's NPC damage ring
//! (`Player.cs:10212-10264`, the `ApplyDamageToNPC` core `:20432-20461`):
//! while the buff rides, every hostile NPC within 200 px of the player's
//! centre gets Hellfire (323) for 120 ticks; on the 60-tick cadence
//! (`infernoCounter % 60 == 0`) the ring also strikes for 20 damage.
//!
//! The flag / light / cadence halves ride `buff_effects.rs` (wave B) and
//! `player_lights.rs`; this module is the roster scan the wiring deferred.
//! The PvP twin (`:10239-10263`) is single-player dead code (`hostile`
//! defaults false, no second player) — the `!hostile` early-out is kept.
//!
//! RNG census: the scan itself draws NOTHING. The strike's combat-text
//! placement (`CombatText.NewText`, `NPC.cs:81250-81264`) draws on the
//! caller's `UpdatePlayers` stream BEFORE the life write; the `HitSound`
//! pitch (`NPC.cs:81544-81553`) and the `HitEffect` probe roll
//! (`NPC.cs:81488`) draw after — the caller drains the returned strike
//! requests onto the audio / probe seams in that order.
//!
//! Code-only worker: witnesses WRITTEN, NOT RUN (no cargo/tests/graphify).

use crate::npc::{Npc, NpcKind};
use crate::player::Player;

/// One landed aura strike — the caller drains it onto the app seams (the
/// hit sound + the Destroyer probe launch need `&mut App`, which the scan's
/// `&mut [Npc]` borrow cannot hand over — the request-queue convention).
pub struct InfernoStrike {
    /// The NPC slot struck (the probe-launch / sound reads).
    pub idx: usize,
}

/// `Player.CanNPCBeHitByPlayerOrPlayerProjectile` (`Player.cs:20513-20529`)
/// with `projectile = null`: the critter refusal. `dontHurtCritters` (the
/// Guide to Critter Companions 4767/5323) blocks `CountsAsCritter` kinds
/// UNLESS the critter has turned hostile (`IsCritterThatIsHostileToPlayers`,
/// `NPC.cs:19810-19817`: `CritterThatCanTurnOnPlayers` && aiStyle 3).
///
/// The ported `CountsAsCritter` member is the Goldfish (55, `NPCID.cs:
/// 10460`); the rest of the 100-id set has no `NpcKind` yet and never
/// reaches the scan. The Goldfish is NOT in `CritterThatCanTurnOnPlayers`
/// (`NPCID.cs:10436`), so the hostile-turn exemption never fires for it.
fn can_npc_be_hit(n: &Npc, dont_hurt_critters: bool) -> bool {
    if !dont_hurt_critters {
        return true;
    }
    if n.kind != NpcKind::Goldfish {
        return true;
    }
    // `IsCritterThatIsHostileToPlayers` — the Goldfish cannot turn, so the
    // book's refusal stands.
    false
}

/// The Inferno aura ring (`Player.cs:10224-10238`): one pass over the NPC
/// roster, in slot order. Returns the landed-strike requests the caller
/// drains (hit sound + probe launch). `damage_tick` is the
/// `infernoCounter % 60 == 0` gate the wave-B arm already computed.
///
/// The `whoAmI != Main.myPlayer` early `continue` (`:10220-10223`) folds to
/// false in the single-player port — the scan always runs for the local
/// player. The `!this.hostile` PvP gate (`:10239-10242`) returns before the
/// player ring; the port has one player, so the twin is unreachable.
pub fn inferno_aura_scan(
    p: &Player,
    npcs: &mut [Npc],
    damage_tick: bool,
    metrics: &tr_core::scene::SceneMetrics,
    rng: &mut tr_core::UnifiedRandom,
    texts: &mut Vec<crate::combat_text::CombatText>,
    font: Option<&crate::assets::UiFont>,
) -> Vec<InfernoStrike> {
    let mut strikes = Vec::new();
    if !p.inferno {
        return strikes;
    }
    let (pcx, pcy) = p.center();
    let radius = crate::buff_effects::INFERNO_RADIUS;
    let debuff = crate::buff_effects::INFERNO_DEBUFF_ID;
    let debuff_time = crate::buff_effects::INFERNO_DEBUFF_TIME;
    let base_damage = crate::buff_effects::INFERNO_DAMAGE;
    for idx in 0..npcs.len() {
        // `nPC.active && !nPC.friendly && nPC.damage > 0 && !nPC.dontTakeDamage
        // && !nPC.buffImmune[num3] && CanNPCBeHit... && Distance <= num4`
        // (`:10227`). The port's `active` is `hp > 0`; `damage` is the
        // contact-damage seam (`damage_override` wins, `npc.rs:5532`).
        let n = &npcs[idx];
        let damage = n.damage_override.unwrap_or_else(|| n.kind.contact_damage());
        if n.hp <= 0
            || n.friendly
            || damage <= 0
            || n.dont_take_damage
            || n.buffs.buff_immune[debuff as usize]
            || !can_npc_be_hit(n, p.dont_hurt_critters)
        {
            continue;
        }
        let (nw, nh) = n.size();
        let (ncx, ncy) = (n.x + nw * 0.5, n.y + nh * 0.5);
        let dx = pcx - ncx;
        let dy = pcy - ncy;
        if (dx * dx + dy * dy).sqrt() > radius {
            continue;
        }
        // `if (nPC.FindBuffIndex(num3) == -1) nPC.AddBuff(num3, 120)`
        // (`:10229-10232`).
        if npcs[idx].buffs.find_buff_index(debuff).is_none() {
            npcs[idx].buffs.add_buff(debuff, debuff_time);
        }
        if !damage_tick {
            continue;
        }
        // `this.ApplyDamageToNPC(nPC, damage, 0f, 0, crit: false, null, 2348)`
        // (`:10235`) — the `:20437-20461` core, RNG-exact:
        //   1. the banner buff scales the damage (`:20439-20442`),
        //   2. `OnHit` (`:20443`) — the on-hit accessory families are
        //      unported player flags (recorded in ANALYSIS), no draws here,
        //   3. `checkArmorPenetration(GetArmorPenetration(melee: false), 0f)`
        //      (`:20444`) — the ichor/broken-armor/Betsy bonuses join the
        //      player's flat pen,
        //   4. `StrikeNPC` (`:20447`) — the combat text BEFORE the life
        //      write (`NPC.cs:81250-81264`), then the defence line + pool.
        let mut dmg = base_damage;
        // The banner lookup keys on the NPC's net id — the slime variants
        // carry distinct ids (`entry_for_npc(kind, variant)`, the bestiary
        // convention), so the scan reads the live variant, not `net_id()`'s
        // Blue default.
        let net_id = crate::bestiary::entry_for_npc(n.kind, n.slime).net_id;
        if let Some((dealt, _)) = crate::banners::banner_buff_for(metrics, net_id) {
            dmg = (dmg as f32 * dealt) as i32;
        }
        let n_defense = npcs[idx].defense();
        let ichor = npcs[idx].buffs.has_buff(69);
        let broken_armor = npcs[idx].buffs.has_buff(36);
        let betsys_curse = npcs[idx].buffs.has_buff(203);
        dmg += tr_core::damage::check_armor_penetration(
            p.get_armor_penetration(false),
            0.0,
            n_defense,
            ichor,
            broken_armor,
            betsys_curse,
        );
        // `StrikeNPC_Inner`'s defence/crit line (`NPC.cs:81239`) — no
        // DamageVar here (the aura passes a flat 20, `:10219`).
        let amount = tr_core::damage::strike_npc_damage(dmg, n_defense, false);
        // The combat text (`NPC.cs:81250-81264`): `lifeMax > 1` spawns the
        // hostile-coloured number at the hitbox, BEFORE the strike. The
        // placement draws ride the caller's `UpdatePlayers` stream (the C#
        // `CombatText.NewText` runs inside `StrikeNPC`, under the player
        // update's `Main.rand` swap). The font measure matches the melee
        // strike's (`world_sim/editing.rs`): the combat font when loaded,
        // the 8px/char stand-in otherwise.
        let (nx, ny) = (npcs[idx].x, npcs[idx].y);
        if npcs[idx].max_hp > 1 {
            let text = amount.to_string();
            let size = match font {
                Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
                None => [text.chars().count() as f32 * 8.0, 10.0],
            };
            let location = crate::combat_text::Rect { x: nx, y: ny, width: nw, height: nh };
            let color = crate::combat_text::damage_text_color(false, false, false);
            crate::combat_text::spawn(
                texts, location, color, text, false, false, size, 1.0, rng,
            );
        }
        // The strike itself: the `realLife` pool arm + the interaction flag
        // + the knockback (`Npc::strike_projectile` — the aura is not a
        // melee swing, so no `hurt_cd` gate; knockback 0, direction 0,
        // crit false, owner the local player).
        let me = 0u8; // `Main.myPlayer` folds to 0 in the single-player port
        let expert = p.buffs.expert_mode;
        let landed = Npc::strike_projectile(
            npcs, idx, amount, pcx, 0.0, false, Some(me), expert,
        );
        if landed {
            strikes.push(InfernoStrike { idx });
        }
    }
    // `if (!this.hostile) continue;` (`:10239-10242`) — the PvP player ring
    // (`:10243-10263`) needs a second hostile player; the single-player port
    // never reaches it (recorded, not ported).
    strikes
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::{UnifiedRandom, World};

    fn player() -> Player {
        Player::spawn(&World::new("t".into(), 64, 800))
    }

    fn zombie_at(x: f32, y: f32) -> Npc {
        Npc::new(NpcKind::Zombie, x, y, 1, 0.0)
    }

    fn metrics() -> tr_core::scene::SceneMetrics {
        tr_core::scene::SceneMetrics::new()
    }

    /// `Player.cs:10227-10232`: a hostile NPC in the ring gets Hellfire 323
    /// for 120 ticks even on a non-damage tick; the strike waits for the
    /// 60-tick cadence.
    #[test]
    fn w4_f12_inferno_spreads_hellfire_without_the_damage_tick() {
        let mut p = player();
        p.inferno = true;
        let (cx, cy) = p.center();
        let mut npcs = vec![zombie_at(cx - 10.0, cy - 10.0)];
        let mut texts = Vec::new();
        let mut rng = UnifiedRandom::new(0);
        let strikes = inferno_aura_scan(
            &p, &mut npcs, false, &metrics(), &mut rng, &mut texts, None,
        );
        assert!(strikes.is_empty(), "no strike off the cadence");
        assert!(npcs[0].buffs.has_buff(323), "Hellfire spreads (`:10231`)");
        assert_eq!(npcs[0].buffs.buff_time_of(323), 120);
        assert_eq!(npcs[0].hp, npcs[0].max_hp, "no damage off the cadence");
    }

    /// `Player.cs:10233-10236` + `NPC.cs:81239`: on the cadence the ring
    /// strikes for 20 − defence/2 (the zombie's 6 → 17), the combat text
    /// spawns, and the life drops.
    #[test]
    fn w4_f12_inferno_strikes_on_the_cadence() {
        let mut p = player();
        p.inferno = true;
        let (cx, cy) = p.center();
        let mut npcs = vec![zombie_at(cx - 10.0, cy - 10.0)];
        let hp0 = npcs[0].hp;
        let mut texts = Vec::new();
        let mut rng = UnifiedRandom::new(0);
        let strikes = inferno_aura_scan(
            &p, &mut npcs, true, &metrics(), &mut rng, &mut texts, None,
        );
        assert_eq!(strikes.len(), 1, "one landed strike");
        assert_eq!(npcs[0].hp, hp0 - 17, "20 − 6/2 = 17 (`NPC.cs:81239`)");
        assert_eq!(texts.len(), 1, "the combat text spawned (`:81263`)");
        assert_eq!(texts[0].text, "17");
    }

    /// `Player.cs:10227`: the radius gate — an NPC beyond 200 px is skipped
    /// (no debuff, no strike).
    #[test]
    fn w4_f12_inferno_respects_the_radius() {
        let mut p = player();
        p.inferno = true;
        let (cx, cy) = p.center();
        let mut npcs = vec![zombie_at(cx + 400.0, cy)];
        let mut texts = Vec::new();
        let mut rng = UnifiedRandom::new(0);
        let strikes = inferno_aura_scan(
            &p, &mut npcs, true, &metrics(), &mut rng, &mut texts, None,
        );
        assert!(strikes.is_empty());
        assert!(!npcs[0].buffs.has_buff(323), "out of the ring");
    }

    /// `Player.cs:10227`: friendly / zero-damage / dontTakeDamage / immune
    /// NPCs are all refused.
    #[test]
    fn w4_f12_inferno_refuses_the_gated_npcs() {
        let mut p = player();
        p.inferno = true;
        let (cx, cy) = p.center();
        let mut friendly = zombie_at(cx, cy);
        friendly.friendly = true;
        let mut immune = zombie_at(cx, cy);
        immune.buffs.buff_immune[323] = true;
        let mut shielded = zombie_at(cx, cy);
        shielded.dont_take_damage = true;
        let mut goldfish = Npc::new(NpcKind::Goldfish, cx, cy, 1, 0.0); // damage 0
        let mut npcs = vec![friendly, immune, shielded, goldfish];
        let mut texts = Vec::new();
        let mut rng = UnifiedRandom::new(0);
        let strikes = inferno_aura_scan(
            &p, &mut npcs, true, &metrics(), &mut rng, &mut texts, None,
        );
        assert!(strikes.is_empty(), "every gate refuses");
        assert!(npcs.iter().all(|n| !n.buffs.has_buff(323)));
    }

    /// `Player.cs:20515` + `:12825-12828`: the critter book's refusal — a
    /// Goldfish (damage forced > 0 so it reaches the critter gate) is
    /// skipped while `dontHurtCritters` holds.
    #[test]
    fn w4_f12_inferno_critter_book_refuses_the_goldfish() {
        let mut p = player();
        p.inferno = true;
        p.dont_hurt_critters = true;
        let (cx, cy) = p.center();
        let mut goldfish = Npc::new(NpcKind::Goldfish, cx, cy, 1, 0.0);
        goldfish.damage_override = Some(10);
        let mut npcs = vec![goldfish];
        let mut texts = Vec::new();
        let mut rng = UnifiedRandom::new(0);
        let strikes = inferno_aura_scan(
            &p, &mut npcs, true, &metrics(), &mut rng, &mut texts, None,
        );
        assert!(strikes.is_empty(), "the book refuses the critter");
        // Without the book the same goldfish is hittable.
        p.dont_hurt_critters = false;
        let strikes = inferno_aura_scan(
            &p, &mut npcs, true, &metrics(), &mut rng, &mut texts, None,
        );
        assert_eq!(strikes.len(), 1, "no book -> the critter is hittable");
    }

    /// `Player.cs:20444` + `NPC.cs:81148-81160`: the ichor debuff's +15
    /// penetration joins the aura's strike (the zombie's 6 defence caps the
    /// pen at 6/2 = 3, so 20 + 3 − 3 = 20).
    #[test]
    fn w4_f12_inferno_ichor_penetration_joins_the_strike() {
        let mut p = player();
        p.inferno = true;
        let (cx, cy) = p.center();
        let mut npcs = vec![zombie_at(cx - 10.0, cy - 10.0)];
        npcs[0].buffs.add_buff(69, 600); // Ichor
        let hp0 = npcs[0].hp;
        let mut texts = Vec::new();
        let mut rng = UnifiedRandom::new(0);
        inferno_aura_scan(&p, &mut npcs, true, &metrics(), &mut rng, &mut texts, None);
        // pen = min(15, 6)/2 = 3; dmg = 20 + 3 = 23; strike = 23 − 3 = 20.
        assert_eq!(npcs[0].hp, hp0 - 20, "the ichor pen lifts the strike");
    }

    /// The scan draws nothing off the cadence; on the cadence the combat
    /// text's two placement draws ride the caller's stream (the C#
    /// `CombatText.NewText` order, `CombatText.cs:88-89`).
    #[test]
    fn w4_f12_inferno_rng_census() {
        let mut p = player();
        p.inferno = true;
        let (cx, cy) = p.center();
        let mut npcs = vec![zombie_at(cx - 10.0, cy - 10.0)];
        let mut texts = Vec::new();
        let mut rng = UnifiedRandom::new(0);
        let pristine = UnifiedRandom::new(0);
        inferno_aura_scan(&p, &mut npcs, false, &metrics(), &mut rng, &mut texts, None);
        assert_eq!(rng, pristine, "the debuff-only pass draws nothing");
        inferno_aura_scan(&p, &mut npcs, true, &metrics(), &mut rng, &mut texts, None);
        assert_ne!(rng, pristine, "the strike's combat text draws");
    }
}
