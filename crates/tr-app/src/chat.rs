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
pub const COLOR_WORLD: [u8; 3] = [50, 255, 130];
pub const COLOR_DEATH: [u8; 3] = [255, 25, 25];
pub const COLOR_SERVER_MESSAGE: [u8; 3] = [255, 240, 20];
pub const COLOR_COMMAND: [u8; 3] = [250, 250, 0];
pub const COLOR_ERROR: [u8; 3] = [250, 0, 0];

/// `RemadeChatMonitor.cs:13` — the log keeps 500 lines.
const MESSAGE_CAP: usize = 500;

/// `ChatMessageContainer.cs:33` — `SetContents` arms `_timeLeft = 600`:
/// ten seconds of corner visibility at the 60 Hz update rate. The vanilla
/// "fade" is this binary visibility gate, not an alpha decay — the line
/// draws at full color until the timer hits zero, then only the open chat
/// shows it (`CanBeShownWhenChatIsClosed`, `ChatMessageContainer.cs:23`).
pub const MESSAGE_SHOW_TIME: u32 = 600;

/// `RemadeChatMonitor.cs:25` — `_showCount = 10`: the draw window, open or
/// closed (the screen-fit `showCount` recompute is `LegacyChatMonitor`'s,
/// not the monitor `Main` installs — `Main.cs:67239`).
pub const SHOW_COUNT: usize = 10;

/// One `ChatMessageContainer` — the parsed text and its flat color (the
/// port's lines carry no `[c:...]` snippet markup yet).
#[derive(Clone, Debug, PartialEq)]
pub struct ChatLine {
    pub text: String,
    pub color: [u8; 3],
    /// `ChatMessageContainer._timeLeft` (`ChatMessageContainer.cs:19`):
    /// the corner-visibility countdown, armed to [`MESSAGE_SHOW_TIME`] on
    /// insertion and decremented once per game update by
    /// [`ChatMonitor::update`].
    pub time_left: u32,
}

impl ChatLine {
    /// `ChatMessageContainer.CanBeShownWhenChatIsClosed`
    /// (`ChatMessageContainer.cs:23`): `_timeLeft > 0`.
    pub fn can_be_shown_when_chat_is_closed(&self) -> bool {
        self.time_left > 0
    }
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
    /// W4-g23 — `Main.NewText`/`NewTextMultiline`'s `PlaySound(12)` tails
    /// (`Main.cs:64635`/`:64641`): one legacy id per append, drained by the
    /// App once per frame (`ChatMonitor` has no `Audio` — the pure monitor
    /// records the cue, the frame loop plays it). The C# plays inline at
    /// the append; the port's drain preserves the per-line count and order.
    pending_cues: Vec<u32>,
}

impl ChatMonitor {
    /// `RemadeChatMonitor.AddNewMessage` (`:40-52`): insert at the head,
    /// trim past the 500-line cap. The container's `SetContents`
    /// (`ChatMessageContainer.cs:27-35`) arms the 600-tick visibility
    /// timer — the new line belongs to the always-visible corner set
    /// immediately.
    pub fn new_text(&mut self, text: impl Into<String>, color: [u8; 3]) {
        self.messages.insert(
            0,
            ChatLine {
                text: text.into(),
                color,
                time_left: MESSAGE_SHOW_TIME,
            },
        );
        while self.messages.len() > MESSAGE_CAP {
            self.messages.pop();
        }
        // `Main.NewText`'s `SoundEngine.PlaySound(12)` tail (`Main.cs:64635`;
        // the multiline twin `:64641` folds into the same append) — W4-g23
        // wires the centralized cue here; the App drains it per frame.
        self.pending_cues
            .push(crate::main_sounds_batch3::new_text_cue());
    }

    /// W4-g23 — drains the `NewText` cue queue: the legacy ids the appends
    /// recorded since the last drain, in append order. The frame loop plays
    /// each (`12` → `Menu_Tick`); one cue per line, like the C# tail.
    pub fn take_pending_cues(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.pending_cues)
    }

    /// W4-g23 — the App-side drain of [`Self::take_pending_cues`]: plays each
    /// queued `NewText` tail through `Audio` (12 → `Menu_Tick`, the
    /// `Main.cs:64635`/`:64641` ids). The frame loop calls this once per
    /// frame; the achievement subscriber calls it inline after its append.
    pub fn drain_new_text_cues(&mut self, audio: &crate::audio::Audio) {
        for id in self.take_pending_cues() {
            audio.play(match id {
                10 => "Menu_Open",
                11 => "Menu_Close",
                _ => "Menu_Tick", // 12
            });
        }
    }

    /// `RemadeChatMonitor.Update` (`:115-129`) → each container's
    /// `ChatMessageContainer.Update` (`ChatMessageContainer.cs:45-52`):
    /// one tick off every live `_timeLeft`, once per game update
    /// (`Main.cs:17282` — after the gated `DoUpdateInWorld`, so it runs
    /// on stepper-blocked frames but not while the game or focus pause
    /// holds). The width-limit re-parse has no port equivalent (the lines
    /// carry flat text, no word wrap yet).
    pub fn update(&mut self) {
        for line in &mut self.messages {
            if line.time_left > 0 {
                line.time_left -= 1;
            }
        }
    }

    /// `RemadeChatMonitor.DrawChat`'s visible window (`:55-97`): the
    /// newest [`SHOW_COUNT`] lines while the input is open; closed, the
    /// walk **breaks** at the first expired container
    /// (`!(drawingPlayerChat | CanBeShownWhenChatIsClosed)`, `:79-82`) —
    /// an old line hides the newer-window tail behind it, exactly like
    /// the source. `_startChatLine` scroll offset and multi-line
    /// containers are not ported (every line is one row).
    pub fn visible_lines(&self, drawing_player_chat: bool) -> impl Iterator<Item = &ChatLine> {
        self.messages
            .iter()
            .take(SHOW_COUNT)
            .take_while(move |line| drawing_player_chat || line.can_be_shown_when_chat_is_closed())
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
        rng: &mut tr_core::UnifiedRandom,
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
        rng: &mut tr_core::UnifiedRandom,
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
                // `Main.rand.Next(1, 101)` (`RollCommand.cs:14`) — the bare
                // `Main.rand` stream the caller feeds (`global`).
                let num = rng.next_range(1, 101);
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
                // `Main.rand.NextFromList<int>(37, 38, 36)` (`RockPaperScissorsCommand.cs:23`).
                let pick = rng.next_from_slice(&[37i32, 38, 36]);
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
pub fn broadcast_death(chat: &mut ChatMonitor, text: &str) {
    // The C# line is the resolved NetworkText — "X was slain..." with the
    // name already inside (the DeathText rows carry `{0}`), so the port
    // broadcasts the final text as-is.
    chat.new_text(text.to_string(), COLOR_DEATH);
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
        chat.send("hello world", "Belsin", &mut tr_core::UnifiedRandom::new(1), &mut crate::emotes::EmoteSystem::default());
        assert_eq!(chat.messages[0].text, "Belsin hello world");
        assert_eq!(chat.messages[0].color, [255, 255, 255]);
    }

    #[test]
    fn the_commands_route_and_the_unknown_falls_back_to_say() {
        let mut chat = ChatMonitor::default();
        chat.send("/roll", "Belsin", &mut tr_core::UnifiedRandom::new(7), &mut crate::emotes::EmoteSystem::default());
        assert!(chat.messages[0].text.starts_with("*Belsin rolls a "));
        let num: u32 = chat.messages[0].text.rsplit(' ').next().unwrap().parse().unwrap();
        assert!((1..=100).contains(&num));
        assert_eq!(chat.messages[0].color, COLOR_SERVER_MESSAGE);
        // The roster command lists the local player.
        chat.send("/players", "Belsin", &mut tr_core::UnifiedRandom::new(7), &mut crate::emotes::EmoteSystem::default());
        assert_eq!(chat.messages[0].text, "Belsin");
        // An unknown command says the raw text (the `_defaultCommand` arm).
        chat.send("/frobnicate now", "Belsin", &mut tr_core::UnifiedRandom::new(7), &mut crate::emotes::EmoteSystem::default());
        assert_eq!(chat.messages[0].text, "Belsin /frobnicate now");
        // `/say` rest routes as a plain say.
        chat.send("/say hi there", "Belsin", &mut tr_core::UnifiedRandom::new(7), &mut crate::emotes::EmoteSystem::default());
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
        chat.send_with("/death", "Belsin", &mut tr_core::UnifiedRandom::new(7), emotes, Some(&p));
        // `LegacyMultiplayer.23` = "*{0} was slain {1} times."
        assert_eq!(chat.messages[0].text, "*Belsin was slain 3 times.");
        assert_eq!(chat.messages[0].color, COLOR_DEATH);
        // The PVP counter: `LegacyMultiplayer.24` = "*{0} was slain by other
        // players {1} times."
        chat.send_with("/pvpdeath", "Belsin", &mut tr_core::UnifiedRandom::new(7), emotes, Some(&p));
        assert_eq!(
            chat.messages[0].text,
            "*Belsin was slain by other players 1 time."
        );
        // One death switches to the singular key (`:25`/`:26`).
        p.deaths_pve = 1;
        p.deaths_pvp = 1;
        chat.send_with("/death", "Belsin", &mut tr_core::UnifiedRandom::new(7), emotes, Some(&p));
        assert_eq!(chat.messages[0].text, "*Belsin was slain 1 time.");
        chat.send_with("/pvpdeath", "Belsin", &mut tr_core::UnifiedRandom::new(7), emotes, Some(&p));
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
            &mut tr_core::UnifiedRandom::new(7),
            &mut crate::emotes::EmoteSystem::default(),
            None,
        );
        assert_eq!(chat.messages[0].text, "You are not in a team!");
        assert_eq!(chat.messages[0].color, COLOR_SERVER_MESSAGE);
    }

    #[test]
    fn the_death_broadcast_uses_the_death_color() {
        let mut chat = ChatMonitor::default();
        broadcast_death(&mut chat, "Belsin was slain...");
        assert_eq!(chat.messages[0].text, "Belsin was slain...");
        assert_eq!(chat.messages[0].color, COLOR_DEATH);
    }

    /// The corner-HUD regression: a pushed message joins the always-visible
    /// set immediately — `SetContents` arms `_timeLeft = 600`
    /// (`ChatMessageContainer.cs:33`), so `CanBeShownWhenChatIsClosed`
    /// (`:23`) is true and `DrawChat(false)` draws it at full alpha.
    #[test]
    fn a_new_message_is_corner_visible_with_the_full_timer() {
        let mut chat = ChatMonitor::default();
        chat.send(
            "hello corner",
            "Belsin",
            &mut tr_core::UnifiedRandom::new(1),
            &mut crate::emotes::EmoteSystem::default(),
        );
        assert_eq!(chat.messages[0].time_left, MESSAGE_SHOW_TIME);
        assert!(chat.messages[0].can_be_shown_when_chat_is_closed());
        // The closed-chat draw window carries the line (the HUD reads this
        // iterator — `ui::draw_chat` via `visible_lines(chat.open)`).
        let visible: Vec<_> = chat.visible_lines(false).collect();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].text, "Belsin hello corner");
        // The vanilla draw is full-color: the container carries no alpha
        // decay (`RemadeChatMonitor.cs:83` draws `Color` as stored).
        assert_eq!(visible[0].color, [255, 255, 255]);
    }

    /// `ChatMessageContainer.Update` (`:45-52`): one tick per update, the
    /// line stays corner-visible for exactly 600 updates (10 s at 60 Hz),
    /// then the closed chat hides it while the open chat keeps it.
    #[test]
    fn the_corner_visibility_expires_after_600_updates() {
        let mut chat = ChatMonitor::default();
        chat.new_text("fading line", COLOR_DEATH);
        for i in 1..=599 {
            chat.update();
            assert_eq!(chat.messages[0].time_left, MESSAGE_SHOW_TIME - i);
            assert!(
                chat.messages[0].can_be_shown_when_chat_is_closed(),
                "still visible at update {i}"
            );
        }
        assert_eq!(chat.visible_lines(false).count(), 1);
        // The 600th update zeroes the timer — the corner walk hides it.
        chat.update();
        assert_eq!(chat.messages[0].time_left, 0);
        assert!(!chat.messages[0].can_be_shown_when_chat_is_closed());
        assert_eq!(chat.visible_lines(false).count(), 0);
        // The open chat's scrollback is intact (`drawingPlayerChat` bypasses
        // the gate, `RemadeChatMonitor.cs:79`).
        assert_eq!(chat.visible_lines(true).count(), 1);
        // The timer never underflows (`if (_timeLeft > 0)`, `:47`).
        chat.update();
        assert_eq!(chat.messages[0].time_left, 0);
    }

    /// `RemadeChatMonitor.DrawChat` (`:76-97`): the window is `_showCount =
    /// 10` (`:25`), and the closed walk **breaks** at the first expired
    /// container (`:79-82`) — an expired line hides everything behind it,
    /// it is not skipped over.
    #[test]
    fn the_closed_window_breaks_at_the_first_expired_line() {
        let mut chat = ChatMonitor::default();
        // 12 lines, newest first: "line 11" .. "line 0".
        for i in 0..12 {
            chat.new_text(format!("line {i}"), COLOR_DEATH);
        }
        // Open: exactly the 10-line window, newest first.
        let open: Vec<_> = chat.visible_lines(true).map(|l| l.text.clone()).collect();
        assert_eq!(open.len(), SHOW_COUNT);
        assert_eq!(open[0], "line 11");
        assert_eq!(open[9], "line 2");
        // Expire "line 8" (index 3) only: the closed walk draws indices
        // 0..2 then breaks — indices 4+ stay hidden behind the expired row.
        chat.messages[3].time_left = 0;
        let closed: Vec<_> = chat.visible_lines(false).map(|l| l.text.clone()).collect();
        assert_eq!(closed, ["line 11", "line 10", "line 9"]);
        // Expire the newest line: nothing shows in the corner at all.
        chat.messages[0].time_left = 0;
        assert_eq!(chat.visible_lines(false).count(), 0);
        // The open chat still shows its full 10-line window.
        assert_eq!(chat.visible_lines(true).count(), SHOW_COUNT);
    }

    /// A fresh line inserted in front of expired ones re-opens the corner
    /// window up to the first expired row (the same `:79-82` break).
    #[test]
    fn a_fresh_line_shows_even_when_older_lines_expired() {
        let mut chat = ChatMonitor::default();
        chat.new_text("old line", COLOR_DEATH);
        for _ in 0..MESSAGE_SHOW_TIME {
            chat.update();
        }
        assert_eq!(chat.visible_lines(false).count(), 0);
        chat.new_text("new line", COLOR_WORLD);
        let closed: Vec<_> = chat.visible_lines(false).map(|l| l.text.clone()).collect();
        assert_eq!(closed, ["new line"]);
        assert_eq!(chat.messages[0].time_left, MESSAGE_SHOW_TIME);
        // The expired line stays in the scrollback (the 500-line log).
        assert_eq!(chat.messages.len(), 2);
        assert_eq!(chat.visible_lines(true).count(), 2);
    }

    /// W4-g23 — the `NewText` cue tail wiring (`Main.cs:64632-64642`): every
    /// append queues exactly one legacy 12 (`:64635`/`:64641`), the drain
    /// empties the queue in append order, and `update` (the timer decay)
    /// queues nothing.
    #[test]
    fn w4_g23_every_new_text_append_queues_one_12_and_the_drain_empties() {
        let mut chat = ChatMonitor::default();
        chat.new_text("first", COLOR_DEATH);
        chat.new_text("second", COLOR_WORLD);
        assert_eq!(chat.take_pending_cues(), vec![12, 12], "one per append");
        assert!(chat.take_pending_cues().is_empty(), "the drain empties");
        // The timer decay is not an append — no cue.
        chat.update();
        assert!(chat.take_pending_cues().is_empty());
        // The send path folds into new_text, so it cues too.
        chat.send(
            "hello",
            "Belsin",
            &mut tr_core::UnifiedRandom::new(1),
            &mut crate::emotes::EmoteSystem::default(),
        );
        assert_eq!(chat.take_pending_cues(), vec![12]);
    }
}
