//! The chat — `Main.drawingPlayerChat`/`Main.chatText` plus the
//! `RemadeChatMonitor` log and the single-player command processor
//! (`Chat/ChatCommandProcessor.cs`, `Chat/Commands/*`).
//!
//! The port is single-player (`netMode == 0`): `CreateOutgoingMessage` +
//! `ProcessIncomingMessage` fold into one `send` pass, and the broadcast
//! becomes a monitor append (`ChatHelper.DisplayMessage`, `:81-99` — the
//! name tag prefix rides the author byte).

/// `ChatColors.cs` — the palette the ported messages use.
pub const COLOR_BOSS_OR_EVENT: [u8; 3] = [175, 75, 255];
pub const COLOR_NPC_TRAVEL: [u8; 3] = [50, 125, 255];
pub const COLOR_DEATH: [u8; 3] = [255, 25, 25];
pub const COLOR_SERVER_MESSAGE: [u8; 3] = [255, 240, 20];
pub const COLOR_COMMAND: [u8; 3] = [250, 250, 0];
pub const COLOR_ERROR: [u8; 3] = [250, 0, 0];

/// `RemadeChatMonitor.cs:13` — the log keeps 500 lines.
const MESSAGE_CAP: usize = 500;

/// One `ChatMessageContainer` — the parsed text and its flat color (the
/// port's lines carry no `[c:...]` snippet markup yet).
#[derive(Clone, Debug, PartialEq)]
pub struct ChatLine {
    pub text: String,
    pub color: [u8; 3],
}

/// The chat state: the monitor log plus the open-input pair
/// (`Main.drawingPlayerChat`/`Main.chatText`).
#[derive(Default)]
pub struct ChatMonitor {
    /// Newest first (`_messages.Insert(0, ...)`, `RemadeChatMonitor.cs:44`).
    pub messages: Vec<ChatLine>,
    /// `Main.drawingPlayerChat`.
    pub open: bool,
    /// `Main.chatText`.
    pub text: String,
    /// `Main.textBlinkerCount` — frames since the cursor last toggled
    /// (advanced by the chat draw, `Main.cs:40928-40944`).
    pub blink_timer: u32,
    /// `Main.textBlinkerState`: `1` draws the `|` cursor.
    pub blink_on: bool,
}

impl ChatMonitor {
    /// `RemadeChatMonitor.AddNewMessage` (`:40-52`): insert at the head,
    /// trim past the 500-line cap.
    pub fn new_text(&mut self, text: impl Into<String>, color: [u8; 3]) {
        self.messages.insert(0, ChatLine {
            text: text.into(),
            color,
        });
        while self.messages.len() > MESSAGE_CAP {
            self.messages.pop();
        }
    }

    /// `DoUpdate_HandleChat`'s send arm (`Main.cs:18355-18364`): the
    /// single-player `CreateOutgoingMessage` + `ProcessIncomingMessage`
    /// fold. A leading `/` routes the command table; an unknown command
    /// falls back to Say (`ChatCommandProcessor.cs:100-110`'s
    /// `_defaultCommand` arm).
    pub fn send(
        &mut self,
        raw: &str,
        player_name: &str,
        rng: &mut u64,
        emotes: &mut crate::emotes::EmoteSystem,
    ) {
        self.send_with(raw, player_name, rng, emotes, None);
    }

    /// [`ChatMonitor::send`] with the chat-side player state the
    /// `DeathCommand` family reads (the death counters and the dead flag
    /// the outgoing gates check).
    pub fn send_with(
        &mut self,
        raw: &str,
        player_name: &str,
        rng: &mut u64,
        emotes: &mut crate::emotes::EmoteSystem,
        player: Option<&crate::player::Player>,
    ) {
        let Some(command) = raw.strip_prefix('/') else {
            self.say(raw, player_name);
            return;
        };
        let (name, rest) = match command.split_once(' ') {
            Some((n, r)) => (n, r.trim_start()),
            None => (command, ""),
        };
        // `ChatCommandProcessor.AddCommand` registers every
        // `ChatCommand.<Name>` localization key (and its `_`-suffixed
        // aliases, `:20-41`) as the prefix; the match is case-insensitive
        // (`LocalizedText.EqualsCommand`, `Localization/LocalizedText.cs:176`).
        match name.to_ascii_lowercase().as_str() {
            "say" => self.say(rest, player_name),
            "p" | "party" => {
                // `PartyChatCommand` (`:11-20`): no team -> the no-team
                // error; the port is single-player (team 0), so the error
                // arm always applies.
                self.new_text(crate::lang::text("LegacyMultiplayer.10"), COLOR_SERVER_MESSAGE);
            }
            "roll" => {
                // `RollCommand.ProcessIncomingMessage` (`:14-18`):
                // `*{name} rolls a {1..100}` in the ServerMessage color —
                // `Lang.mp[9]` = "rolls a" (`Legacy.json:839`).
                let num = crate::app::next_rand(rng) % 100 + 1;
                self.new_text(format!("*{player_name} rolls a {num}"), COLOR_SERVER_MESSAGE);
            }
            "rps" => {
                // `RockPaperScissorsCommand.ProcessOutgoingMessage` (`:10-30`):
                // dead players are silenced (`:12-16`), the pick is
                // `NextFromList(RPSRock, RPSPaper, RPSScissors)` (`:23`)
                // shown as the 360-tick emote bubble.
                if player.is_some_and(|p| p.dead) {
                    return;
                }
                let pick = [37i32, 38, 36][(crate::app::next_rand(rng) % 3) as usize];
                emotes.new_bubble(pick);
            }
            "me" | "emote" => {
                // `EmoteCommand.ProcessIncomingMessage` (`:12-19`): the
                // third-person line in the hardcoded (200,100,0).
                self.new_text(format!("*{player_name} {rest}"), [200, 100, 0]);
            }
            "e" | "emoji" => {
                // `EmojiCommand.ProcessOutgoingMessage` (`:44-72`): the int
                // parse first, then the name table; a dead player is
                // silenced (`:46-49`), a bad argument just consumes.
                if player.is_some_and(|p| p.dead) {
                    return;
                }
                if let Some(id) = crate::emotes::resolve(rest.trim()) {
                    emotes.new_bubble(id);
                }
            }
            "help" => {
                // `HelpCommand.ComposeMessage` (`:39-64`): the
                // `ChatCommandDescription.*` list in the ServerMessage
                // color. One line per command, newest first down the log.
                for key in [
                    "ChatCommandDescription.Party",
                    "ChatCommandDescription.Roll",
                    "ChatCommandDescription.RPS",
                    "ChatCommandDescription.Emote",
                    "ChatCommandDescription.Emoji",
                    "ChatCommandDescription.Playing",
                    "ChatCommandDescription.Death",
                    "ChatCommandDescription.PVPDeath",
                    "ChatCommandDescription.AllDeath",
                    "ChatCommandDescription.AllPVPDeath",
                    "ChatCommandDescription.BossDamage",
                    "ChatCommandDescription.Help",
                ] {
                    self.new_text(crate::lang::text(key), COLOR_SERVER_MESSAGE);
                }
            }
            "playing" | "players" => {
                // `ListPlayersCommand` (`:8-14`): the online roster joined
                // with ", " — the port is always the one local player.
                self.new_text(player_name, COLOR_SERVER_MESSAGE);
            }
            "death" => {
                // `DeathCommand` (`:11-18`): the singular/plural keys by the
                // counter, in the Death color.
                let count = player.map_or(0, |p| p.deaths_pve);
                let key = if count == 1 {
                    "LegacyMultiplayer.25"
                } else {
                    "LegacyMultiplayer.23"
                };
                self.new_text(
                    &crate::lang::text_args(key, &[player_name.to_string(), count.to_string()]),
                    COLOR_DEATH,
                );
            }
            "pvpdeath" => {
                // `PVPDeathCommand` (`:11-18`): the PVP counter, same keys.
                let count = player.map_or(0, |p| p.deaths_pvp);
                let key = if count == 1 {
                    "LegacyMultiplayer.26"
                } else {
                    "LegacyMultiplayer.24"
                };
                self.new_text(
                    &crate::lang::text_args(key, &[player_name.to_string(), count.to_string()]),
                    COLOR_DEATH,
                );
            }
            "alldeath" | "allpvpdeath" => {
                // `AllDeathCommand`/`AllPVPDeathCommand` (`:11-20`): the
                // active players ordered by the counter descending — the
                // port has the one local player.
                let pvp = name.eq_ignore_ascii_case("allpvpdeath");
                let count = player.map_or(0, |p| if pvp {
                    p.deaths_pvp
                } else {
                    p.deaths_pve
                });
                let key = if count == 1 {
                    if pvp {
                        "LegacyMultiplayer.26"
                    } else {
                        "LegacyMultiplayer.25"
                    }
                } else if pvp {
                    "LegacyMultiplayer.24"
                } else {
                    "LegacyMultiplayer.23"
                };
                self.new_text(
                    &crate::lang::text_args(key, &[player_name.to_string(), count.to_string()]),
                    COLOR_DEATH,
                );
            }
            "bossdamage" => {
                // `BossDamageCommand` (`:11-24`): the recent boss trackers
                // — no tracker exists in the port yet (the boss damage
                // trackers land with the boss UI).
                self.new_text(
                    crate::lang::text("ChatCommandDescription.BossDamage"),
                    COLOR_COMMAND,
                );
            }
            _ => {
                // The unknown-command fallback: Say consumes the raw text
                // (`:100-110`).
                self.say(raw, player_name);
            }
        }
    }
    /// `SayChatCommand.ProcessIncomingMessage` (`:14-20`): the message shows
    /// as `<name-tag> {text}` in the player's chat color — classic
    /// difficulty is `Color.White` (`Player.ChatColor()`, `:34745`).
    fn say(&mut self, text: &str, player_name: &str) {
        self.new_text(format!("{player_name} {text}"), [255, 255, 255]);
    }
}

/// `Player.KillMe`'s death broadcast (`NetworkText.FromLiteral($"{name}
/// {text}")` shape) — the Death color (`ChatColors.cs`).
pub fn broadcast_death(chat: &mut ChatMonitor, player_name: &str, text: &str) {
    chat.new_text(format!("{player_name} {text}"), COLOR_DEATH);
}

/// The colored system line (`ChatHelper.DisplayMessage`'s flat form) — the
/// banner tallies and the other server announcements.
pub fn broadcast_death_text(chat: &mut ChatMonitor, text: &str, color: [u8; 3]) {
    chat.new_text(text, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_says_with_the_name_tag() {
        // `SayChatCommand` + `DisplayMessage`'s `{name-tag} {text}` prefix.
        let mut chat = ChatMonitor::default();
        chat.send("hello world", "Belsin", &mut 1, &mut crate::emotes::EmoteSystem::default());
        assert_eq!(chat.messages[0].text, "Belsin hello world");
        assert_eq!(chat.messages[0].color, [255, 255, 255]);
    }

    #[test]
    fn the_commands_route_and_the_unknown_falls_back_to_say() {
        let mut chat = ChatMonitor::default();
        chat.send("/roll", "Belsin", &mut 7, &mut crate::emotes::EmoteSystem::default());
        assert!(chat.messages[0].text.starts_with("*Belsin rolls a "));
        let num: u32 = chat.messages[0].text.rsplit(' ').next().unwrap().parse().unwrap();
        assert!((1..=100).contains(&num));
        assert_eq!(chat.messages[0].color, COLOR_SERVER_MESSAGE);
        // The roster command lists the local player.
        chat.send("/players", "Belsin", &mut 7, &mut crate::emotes::EmoteSystem::default());
        assert_eq!(chat.messages[0].text, "Belsin");
        // An unknown command says the raw text (the `_defaultCommand` arm).
        chat.send("/frobnicate now", "Belsin", &mut 7, &mut crate::emotes::EmoteSystem::default());
        assert_eq!(chat.messages[0].text, "Belsin /frobnicate now");
        // `/say` rest routes as a plain say.
        chat.send("/say hi there", "Belsin", &mut 7, &mut crate::emotes::EmoteSystem::default());
        assert_eq!(chat.messages[0].text, "Belsin hi there");
    }

    #[test]
    fn the_log_caps_at_500_newest_first() {
        let mut chat = ChatMonitor::default();
        for i in 0..600 {
            chat.new_text(format!("line {i}"), COLOR_DEATH);
        }
        assert_eq!(chat.messages.len(), 500);
        assert_eq!(chat.messages[0].text, "line 599");
        assert_eq!(chat.messages[499].text, "line 100");
    }

    #[test]
    fn the_death_commands_read_the_counters() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut chat = ChatMonitor::default();
        let mut p = crate::player::Player::spawn(&tr_core::World::new("t".into(), 64, 800));
        p.deaths_pve = 3;
        p.deaths_pvp = 1;
        let emotes = &mut crate::emotes::EmoteSystem::default();
        chat.send_with("/death", "Belsin", &mut 7, emotes, Some(&p));
        // `LegacyMultiplayer.23` = "*{0} was slain {1} times."
        assert_eq!(chat.messages[0].text, "*Belsin was slain 3 times.");
        assert_eq!(chat.messages[0].color, COLOR_DEATH);
        // The PVP counter: `LegacyMultiplayer.24` = "*{0} was slain by other
        // players {1} times."
        chat.send_with("/pvpdeath", "Belsin", &mut 7, emotes, Some(&p));
        assert_eq!(
            chat.messages[0].text,
            "*Belsin was slain by other players 1 time."
        );
        // One death switches to the singular key (`:25`/`:26`).
        p.deaths_pve = 1;
        p.deaths_pvp = 1;
        chat.send_with("/death", "Belsin", &mut 7, emotes, Some(&p));
        assert_eq!(chat.messages[0].text, "*Belsin was slain 1 time.");
        chat.send_with("/pvpdeath", "Belsin", &mut 7, emotes, Some(&p));
        assert_eq!(
            chat.messages[0].text,
            "*Belsin was slain by other players 1 time."
        );
    }

    #[test]
    fn the_party_command_errors_without_a_team() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        // `PartyChatCommand.SendNoTeamError` (`LegacyMultiplayer.10`).
        let mut chat = ChatMonitor::default();
        chat.send_with(
            "/party hi",
            "Belsin",
            &mut 7,
            &mut crate::emotes::EmoteSystem::default(),
            None,
        );
        assert_eq!(chat.messages[0].text, "You are not in a team!");
        assert_eq!(chat.messages[0].color, COLOR_SERVER_MESSAGE);
    }

    #[test]
    fn the_death_broadcast_uses_the_death_color() {
        let mut chat = ChatMonitor::default();
        broadcast_death(&mut chat, "Belsin", "was slain...");
        assert_eq!(chat.messages[0].text, "Belsin was slain...");
        assert_eq!(chat.messages[0].color, COLOR_DEATH);
    }
}
