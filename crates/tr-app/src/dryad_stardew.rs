//! The Dryad's Stardew Valley bit (W4-f30, progression 46
//! INTERDIMENSIONAL_RECYCLING).
//!
//! `Main.CanDryadPlayStardewAnimation` (`Main.cs:40449-40457`), the
//! `StardewValleyBit` interaction (`NPCInteractions.cs:79-103`, registered
//! `:517` — LIVE code, not the dead `DryadText_WorldPurityOrStardewValleyBit`
//! wrapper), `Main.DryadText_Do_StardewValleyBit` (`Main.cs:40280-40295`),
//! `NPC.HaveDryadDoStardewAnimation` (`NPC.cs:19184-19229`) and the chat
//! panel's text override (`NPCChatPanel.OverrideChatTextWithShenanigans`,
//! `NPCChatPanel.cs:195-213`).
//!
//! The gift: holding the Joja Cola (5275) while talking to the Dryad (20,
//! not already in the animation state 24) shows the "Purify" button; the
//! click consumes one cola (`Player.ConsumeItem(5275, reverseOrder: true)`,
//! `Player.cs:9269-9308`), sets the Dryad to the animation state 24, spawns
//! the Junimo (projectile 995, aiStyle 192) and fires progression 46.
//!
//! Deferred halves (cited, docs/ANALYSIS.md "The Dryad Joja bit (W4-f30)"):
//! the Junimo's AI_192 fall/opacity animation (`Projectile.cs:41501-41600` —
//! the projectile spawns and the aiStyle-192 dispatch is a no-op until that
//! arm lands), the `NetMessage.SendData(144)` client relay (the port's host
//! path runs `HaveDryadDoStardewAnimation` directly), and the
//! `DryadPurification` world-status button (the purity report is unported).

use crate::app::App;
use crate::npc::{Npc, NpcKind};

/// `Main.CanDryadPlayStardewAnimation` (`Main.cs:40449-40457`): the held
/// item is the Joja Cola (5275), the talk NPC is the Dryad (20) and she is
/// not already in the animation state 24.
pub fn can_dryad_play_stardew_animation(held_kind: u16, talk_npc: Option<&Npc>) -> bool {
    match talk_npc {
        Some(n) => n.kind == NpcKind::Dryad && n.ai[0] != 24.0 && held_kind == 5275,
        None => false,
    }
}

/// `Player.ConsumeItem(5275, reverseOrder: true)` (`Player.cs:9269-9308`):
/// one cola off the LAST matching inventory cell (the C# walks 57 → 0; the
/// port's grid is 0..49, the reverse walk 49..0). A drained cell turns to
/// air (`SetDefaults(0)`). Returns whether one was consumed. The void-bag
/// arm (`includeVoidBag`) is false here — the C# call passes only
/// `reverseOrder`.
fn consume_one_cola(inventory: &mut [crate::item::ItemSlot]) -> bool {
    for slot in inventory.iter_mut().rev() {
        if slot.count > 0 && slot.kind == 5275 {
            slot.count -= 1;
            if slot.count == 0 {
                *slot = crate::item::EMPTY_SLOT;
            }
            return true;
        }
    }
    false
}

/// `Main.DryadText_Do_StardewValleyBit` (`Main.cs:40280-40295`) + the
/// `StardewValleyBit.Interact` head (`NPCInteractions.cs:97-102`: the
/// portrait hop + sound 12). Consumes the cola, sets the Dryad's animation
/// state, spawns the Junimo, prints the thank-you line and fires 46.
/// Returns whether the gift ran (the caller plays the menu sound + the
/// portrait hop on a true).
pub fn dryad_do_stardew_valley_bit(app: &mut App) -> bool {
    // The condition re-check (the button only shows while it holds, but the
    // held item / talk NPC may have changed since the draw).
    let held = app.inventory.get(app.selected).map_or(0, |s| s.kind);
    let ok = can_dryad_play_stardew_animation(held, app.talk_npc.and_then(|i| app.npcs.get(i)));
    if !ok {
        return false;
    }
    // `:40282-40283` — the static latches.
    app.prevent_joja_cola_dialog = true;
    app.reroll_dryad_text = 2;
    // `:40284` — `Main.LocalPlayer.ConsumeItem(5275, reverseOrder: true)`.
    consume_one_cola(&mut app.inventory);
    // `:40285-40292` — the client relays `144`; the host / single-player
    // runs the animation directly (the port's host path).
    have_dryad_do_stardew_animation(app);
    // `:40293` — the thank-you line.
    app.npc_chat_text = crate::lang::text("StardewTalk.PlayerGivesCola");
    // `:40294` — progression 46 (INTERDIMENSIONAL_RECYCLING).
    let _ = app.notify_progression_net(46);
    true
}

/// `NPC.HaveDryadDoStardewAnimation` (`NPC.cs:19184-19229`): the FIRST live
/// Dryad enters the animation state 24 (480 ticks), faces right, and the
/// Junimo (projectile 995) spawns above the ground 100 px to her right.
pub fn have_dryad_do_stardew_animation(app: &mut App) {
    // `:19186-19192` — the first active Dryad (the port's `hp > 0` liveness).
    let Some(idx) = app.npcs.iter().position(|n| n.hp > 0 && n.kind == NpcKind::Dryad) else {
        return;
    };
    let (nx, ny, nw, nh) = {
        let n = &mut app.npcs[idx];
        // `:19193-19200` — the state writes.
        n.local_ai[2] = 480.0;
        n.ai[0] = 24.0;
        n.ai[1] = 480.0;
        n.ai[2] = 0.0;
        n.local_ai[3] = 0.0;
        n.net_update = true;
        n.facing = 1;
        n.sprite_dir = 1;
        let (w, h) = n.size();
        (n.x, n.y, w, h)
    };
    // `:19201-19202` — the scan anchor: `Bottom + (100, 0)` in tiles.
    let bottom_x = nx + nw * 0.5;
    let bottom_y = ny + nh;
    let anchor_x = bottom_x + 100.0;
    let px = (anchor_x / 16.0) as i32;
    let py = (bottom_y / 16.0) as i32;
    // `:19203` — `p.Y = Clamp(p.Y - 10, 10, maxTilesY - 10)`.
    // Saturating max: tiny test worlds (< 20 rows) would otherwise give
    // max < min and panic; production worlds keep the exact bound.
    let max_y = app.world.height.saturating_sub(10).max(10);
    let mut py = (py - 10).clamp(10, max_y);
    let start_y = py;
    let end_y = py + 15;
    // `:19206-19219` — the 15-row ground scan; the first solid row stops it
    // (`flag` true only when that row is the START row).
    let mut flag = false;
    for j in start_y..end_y {
        py = j;
        let solid = app
            .world
            .tile(px, j)
            .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind));
        if solid {
            if j == start_y {
                flag = true;
            }
            break;
        }
    }
    // `:19220-19225` — the spawn point: `p.ToWorldCoordinates(8, 0) + (0, -52)`,
    // or the anchor itself when the ground was the start row.
    let spawn = if flag {
        glam::Vec2::new(anchor_x, bottom_y)
    } else {
        glam::Vec2::new(px as f32 * 16.0 + 8.0, py as f32 * 16.0)
    };
    let spawn = spawn + glam::Vec2::new(0.0, -52.0);
    // `:19226` — `Projectile.NewProjectile(..., 995, 0, 0f, Main.myPlayer)`.
    // The Junimo's 240x104 box centres on the spawn point (the C# X/Y
    // overload, `Projectile.cs:10862`'s `X - width/2, Y - height/2`).
    let owner = app.local_id as u8;
    if let Some(slot) = app.projectiles.spawn_with_data_ai_owned(
        995, owner, spawn.x - 120.0, spawn.y - 52.0, 0.0, 0.0, 0, 0.0, 0.0, 0.0,
    ) {
        // `:9446-9457` — the Junimo row: friendly, penetrate -1,
        // tileCollide false, ignoreWater, timeLeft *= 5.
        let p = &mut app.projectiles.slots_mut()[slot];
        p.friendly = true;
        p.tile_collide = false;
        p.ignore_water = true;
    }
}

/// `NPCChatPanel.OverrideChatTextWithShenanigans` (`NPCChatPanel.cs:195-213`):
/// the Dryad's cola text override + the reroll state machine. Runs per draw
/// while the panel is up. Returns the text to show (the caller writes it back
/// to `Main.npcChatText` on the reroll arm).
///
/// `talk_npc` is the talk index; `held_kind` the selected item; the Dryad's
/// `ai[0]` is read off the NPC list. The `GetChat()` reroll (the `:206` arm)
/// needs the full chat context — the caller supplies the re-rolled line.
pub fn override_dryad_chat_text(
    app: &mut App,
    held_kind: u16,
) -> Option<String> {
    let talk = app.talk_npc?;
    let npc = app.npcs.get(talk)?;
    let can = can_dryad_play_stardew_animation(held_kind, Some(npc));
    let ai0 = npc.ai[0];
    let is_dryad = npc.kind == NpcKind::Dryad;
    let active = npc.hp > 0;
    // `:199-202` — the Dryad entered the animation state: 2 → 1.
    if ai0 == 24.0 && app.reroll_dryad_text == 2 {
        app.reroll_dryad_text = 1;
    }
    // `:203-208` — she left it while the reroll is armed: re-roll to GetChat.
    if can
        && app.reroll_dryad_text == 1
        && ai0 != 24.0
        && active
        && is_dryad
    {
        app.reroll_dryad_text = 0;
        app.prevent_joja_cola_dialog = true;
        // The caller re-rolls `GetChat()` (the full context lives there).
        return Some(String::new()); // the sentinel: "re-roll needed"
    }
    // `:209-212` — the holding-cola reminder.
    if can && !app.prevent_joja_cola_dialog {
        return Some(crate::lang::text("StardewTalk.PlayerHasColaAndIsHoldingIt"));
    }
    None
}

/// The `StardewValleyBit` button's label (`NPCInteractions.cs:92-95`).
pub fn give_cola_button_text() -> String {
    crate::lang::text("StardewTalk.GiveColaButtonText")
}

/// The chat-close reset (`NPC.cs:93772-93773` / `:95303-95304`): the static
/// latches clear when the conversation ends.
pub fn reset_joja_latches(app: &mut App) {
    app.prevent_joja_cola_dialog = false;
    app.reroll_dryad_text = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;

    /// A rig with a live Dryad as the talk NPC and a cola in the hotbar.
    fn rig() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        let mut dryad = Npc::new(NpcKind::Dryad, 100.0, 100.0, 1, 0.0);
        dryad.hp = dryad.max_hp;
        app.npcs.push(dryad);
        app.talk_npc = Some(0);
        app.inventory[0] = crate::item::ItemSlot {
            kind: 5275, name: crate::registry::name(5275), count: 3, prefix: 0,
        };
        app.selected = 0;
        app
    }

    /// `Main.CanDryadPlayStardewAnimation` (`Main.cs:40449-40457`): the cola
    /// held + the Dryad talk NPC + not already animating.
    #[test]
    fn f30_the_stardew_condition_reads_the_cola_and_the_dryad() {
        let app = rig();
        let dryad = app.npcs.get(0);
        assert!(can_dryad_play_stardew_animation(5275, dryad));
        // No cola: no bit.
        assert!(!can_dryad_play_stardew_animation(0, dryad));
        // Already animating (state 24): no bit.
        let mut animating = app.npcs[0].clone();
        animating.ai[0] = 24.0;
        assert!(!can_dryad_play_stardew_animation(5275, Some(&animating)));
        // A non-Dryad: no bit.
        let guide = Npc::new(NpcKind::Guide, 0.0, 0.0, 1, 0.0);
        assert!(!can_dryad_play_stardew_animation(5275, Some(&guide)));
        // No talk NPC: no bit.
        assert!(!can_dryad_play_stardew_animation(5275, None));
    }

    /// `Main.DryadText_Do_StardewValleyBit` (`Main.cs:40280-40295`): the gift
    /// consumes one cola, sets the Dryad to state 24, spawns the Junimo (995),
    /// prints the thank-you line and fires 46 (INTERDIMENSIONAL_RECYCLING).
    /// WRITTEN NOT RUN (code-only).
    #[test]
    fn f30_the_gift_consumes_the_cola_and_fires_46() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = rig();
        assert!(dryad_do_stardew_valley_bit(&mut app));
        // One cola consumed (3 → 2).
        assert_eq!(app.inventory[0].count, 2, "ConsumeItem(5275)");
        // The Dryad entered the animation state 24 (`NPC.cs:19194`).
        assert_eq!(app.npcs[0].ai[0], 24.0);
        assert_eq!(app.npcs[0].ai[1], 480.0);
        assert_eq!(app.npcs[0].local_ai[2], 480.0);
        assert_eq!(app.npcs[0].facing, 1);
        // The Junimo spawned (projectile 995).
        assert!(
            app.projectiles.slots().iter().any(|p| p.active && p.kind == 995),
            "the Junimo spawned"
        );
        // The thank-you line + the latches.
        assert_eq!(app.npc_chat_text, crate::lang::text("StardewTalk.PlayerGivesCola"));
        assert!(app.prevent_joja_cola_dialog);
        assert_eq!(app.reroll_dryad_text, 2);
        // Progression 46 fired.
        let row = app.achievements.list.iter().find(|a| a.name == "INTERDIMENSIONAL_RECYCLING").unwrap();
        assert!(row.completed, "INTERDIMENSIONAL_RECYCLING (46) fired");
        // A second gift refuses (the Dryad is animating now).
        assert!(!dryad_do_stardew_valley_bit(&mut app));
    }

    /// `Player.ConsumeItem(5275, reverseOrder: true)` (`Player.cs:9269-9308`):
    /// the LAST matching cell drains first; a drained cell turns to air.
    #[test]
    fn f30_the_cola_consumes_from_the_last_cell() {
        let mut inv = vec![crate::item::EMPTY_SLOT; 50];
        inv[2] = crate::item::ItemSlot { kind: 5275, name: "", count: 1, prefix: 0 };
        inv[40] = crate::item::ItemSlot { kind: 5275, name: "", count: 2, prefix: 0 };
        assert!(consume_one_cola(&mut inv));
        assert_eq!(inv[40].count, 1, "the last cell drains first");
        assert_eq!(inv[2].count, 1, "the earlier cell untouched");
        assert!(consume_one_cola(&mut inv));
        assert_eq!(inv[40].count, 0, "the last cell aired");
        assert_eq!(inv[40].kind, 0, "SetDefaults(0)");
        assert!(consume_one_cola(&mut inv));
        assert_eq!(inv[2].count, 0, "then the earlier cell");
        assert!(!consume_one_cola(&mut inv), "no cola left");
    }

    /// `NPCChatPanel.OverrideChatTextWithShenanigans` (`NPCChatPanel.cs:195-213`):
    /// the holding-cola reminder prints while the bit is available; the
    /// post-gift reroll state machine walks 2 → 1 → 0.
    #[test]
    fn f30_the_chat_override_and_the_reroll_machine() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = rig();
        // The reminder prints while the cola is held + the bit available.
        let text = override_dryad_chat_text(&mut app, 5275);
        assert_eq!(text, Some(crate::lang::text("StardewTalk.PlayerHasColaAndIsHoldingIt")));
        // The gift latches prevent + reroll 2.
        dryad_do_stardew_valley_bit(&mut app);
        // The Dryad is animating (24): the reroll walks 2 → 1.
        let _ = override_dryad_chat_text(&mut app, 5275);
        assert_eq!(app.reroll_dryad_text, 1);
        // She leaves the state (ai[0] back to 0): the reroll fires (the
        // sentinel) and walks 1 → 0, prevent latches.
        app.npcs[0].ai[0] = 0.0;
        let text = override_dryad_chat_text(&mut app, 5275);
        assert_eq!(text, Some(String::new()), "the reroll sentinel");
        assert_eq!(app.reroll_dryad_text, 0);
        assert!(app.prevent_joja_cola_dialog);
        // The close resets both latches.
        reset_joja_latches(&mut app);
        assert!(!app.prevent_joja_cola_dialog);
        assert_eq!(app.reroll_dryad_text, 0);
    }
}
