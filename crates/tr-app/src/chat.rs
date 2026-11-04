//! The chat — `Main.drawingPlayerChat`/`Main.chatText` plus the
//! `RemadeChatMonitor` log and the single-player command processor
//! (`Chat/ChatCommandProcessor.cs`, `Chat/Commands/*`).
//!
//! The port is single-player (`netMode == 0`): `CreateOutgoingMessage` +
//! `ProcessIncomingMessage` fold into one `send` pass, and the broadcast
//! becomes a monitor append (`ChatHelper.DisplayMessage`, `:81-99` — the
//! name tag prefix rides the author byte).

/// `ChatColors.cs` — the palette the ported messages use.
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
        let Some(command) = raw.strip_prefix('/') else {
            self.say(raw, player_name);
            return;
        };
        let (name, rest) = match command.split_once(' ') {
            Some((n, r)) => (n, r.trim_start()),
            None => (command, ""),
        };
        // The localized command keys match case-insensitively
        // (`ParseCommandPrefix`'s `StartsWithIgnoreCase` shape); the port's
        // table is the ASCII subset.
        match name.to_ascii_lowercase().as_str() {
            "say" | "party" => self.say(rest, player_name),
            "roll" => {
                // `RollCommand.ProcessIncomingMessage` (`:14-18`):
                // `*{name} rolls a {1..100}` in the ServerMessage color —
                // `Lang.mp[9]` = "rolls a" (`Legacy.json:839`).
                let num = crate::app::next_rand(rng) % 100 + 1;
                self.new_text(format!("*{player_name} rolls a {num}"), COLOR_SERVER_MESSAGE);
            }
            "help" => {
                // `HelpCommand.ProcessIncomingMessage`: the command-alias
                // list in the ServerMessage color. The ported subset.
                for line in [
                    "/say <text>",
                    "/party <text>",
                    "/roll",
                    "/help",
                    "/players",
                ] {
                    self.new_text(line, COLOR_SERVER_MESSAGE);
                }
            }
            "players" => {
                // `ListPlayersCommand`: the online roster — the port is
                // always the one local player.
                self.new_text(format!("{player_name}"), COLOR_SERVER_MESSAGE);
            }
            "emoji" => {
                // `EmojiCommand.ProcessOutgoingMessage` (`:44-72`): the int
                // parse first, then the name table; a dead player is
                // silenced (`:46-49`), a bad argument just consumes.
                if let Some(id) = crate::emotes::resolve(rest.trim()) {
                    emotes.new_bubble(id);
                }
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
        chat.send("/help", "Belsin", &mut 7, &mut crate::emotes::EmoteSystem::default());
        // The help lines insert head-first: the newest ("/players") sits at
        // [0], "/say <text>" ends up at [4].
        assert_eq!(chat.messages[0].text, "/players");
        assert_eq!(chat.messages[4].text, "/say <text>");
        chat.send("/players", "Belsin", &mut 7, &mut crate::emotes::EmoteSystem::default());
        assert_eq!(chat.messages[0].text, "Belsin");
        // An unknown command says the raw text (the `_defaultCommand` arm).
        chat.send("/frobnicate now", "Belsin", &mut 7, &mut crate::emotes::EmoteSystem::default());
        assert_eq!(chat.messages[0].text, "Belsin /frobnicate now");
        // `/say` rest routes as a plain say.
        chat.send("/party hi there", "Belsin", &mut 7, &mut crate::emotes::EmoteSystem::default());
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
    fn the_death_broadcast_uses_the_death_color() {
        let mut chat = ChatMonitor::default();
        broadcast_death(&mut chat, "Belsin", "was slain...");
        assert_eq!(chat.messages[0].text, "Belsin was slain...");
        assert_eq!(chat.messages[0].color, COLOR_DEATH);
    }
}
