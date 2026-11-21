# W4-e10 VOICE SEAL — the mutual receipts + the voice-family bible

**Lane:** `task/W4-e10-player-sounds-wave`, seal phase over the DONE code tip
`8e0d951fc` (the 13 landed files: the voice picker + `.plr` voice pair + the
hurt/death voice trees + the roar family + the doors/BuffStation chain + the
mount voice census + the smart-cursor twin + the buff-arms 192/366).
**Producer:** W4-e10 seal pass, 2026-11-21, code-only (no compiler invoked).
**Consumers:** the f44 merger (research/132 §4), the d79 pitch/pool merger
(research/W4-d79-pitch-final.md §F4), the g21c resume, the f3 HeightOffset
seam task (research/65).

Sibling tips READ this pass (origin, after a successful fetch):
f44 `0d1ee2045` (SEAL 4 — 132 SEALED, the hydration LIVE at `5c8bbb160`),
d79 `03ebefad7` (PITCH FINAL 4), g21 `7596fb262` (batch 5 — g21c NOT pushed),
f3 `75f3f7347` (retire 4). main moved to `051600da8` (S1-L-11g merge); this
branch's base stays `a725acb04` — no merge is performed in the seal phase.

## 1. The f44g hydration mutual receipt (research/132 §4, the e10c closure)

f44's SEAL 4 (`0d1ee2045`, over the code port `5c8bbb160`) declares the
shared hydration hunk **CLOSED on f44** and consumed our published cast-seam
(ANALYSIS "The voice hydration cast-seam", `88df88473`) verbatim. Re-read
against their live code, the receipt holds line-for-line:

- **The apply** — `Player::hydrate_from_file` (f44 player.rs:2516) carries
  EXACTLY the predicted hunk: `self.voice_variant = file.voice_variant as
  i32` (the 1..4 clamp already applied by `finish_player_file`, `:55831`) +
  `self.voice_pitch_offset = file.voice_pitch_offset` (the direct f32 copy).
  No further transform — the consumers read the pair as pinned by our
  `w4_e10_the_hydrated_voice_pair_drives_the_default_voice_tail` witness.
- **The park/carry** — `CharPlay(i)` parks the card's `.plr` on
  `App.play_player` (`Main.SelectPlayer` → `SetAsActive`,
  `UICharacterListItem.cs:256-261`), `swap_world` carries it through the
  `App::new` reset, hydration runs FIRST (right after `Player::spawn`), the
  `.trplayer` sidecar restore SECOND and wins on shared fields (132 §2c).
- **The declarations** — f44 declares `voice_variant: i32` (ctor 1,
  `Player.cs:56045`) + `voice_pitch_offset: f32` at the SAME ctor defaults as
  our live fields (player.rs:2000/:2004) → the declaration hunks dedup
  textually at the merge. `voice_override` stays OURS (f44 does not declare
  it — it is the per-tick voice-accessory consumer, `Player.cs:1706`).
- **The keep-one (132 §3-L1 rule 7, unchanged)** — DELETE our
  `player_file.rs` transport diff (the i32 `PlayerFile`/`PlayerFileWrite`
  fields + the `finish_player_file` `voice: Option<(i32, f32)>` parameter),
  KEEP f44's u8 `PlayerTail` walk (the file-faithful `(byte)` width,
  `:54924`; the inc1/inc2 depths ride `PlayerTail::default` + the shared
  clamp — the superset). KEEP our picker (`ui/char_creation.rs`) + the
  `NewPlayerDraft` fields + the `session.rs` threading re-pointed with
  `voice_variant: draft.voice_variant as u8` (the picker offers 1..3, the
  cast is lossless; a hand-edited draft clamps on the next read like the
  C#). Our three `player_file.rs` witnesses re-pin onto f44's
  `the_voice_pair_round_trips_clamps_and_derives` (its first arm == our
  round-trip witness; clamp + gender-derive already covered there). Our
  `save_list.rs` fixture literals dedup onto f44's fixtures (same values,
  the u8 type).
- **The consumers stay ours wholesale** — `hurt_voice`/`death_voice`/
  `default_voice_hurt`/`voice_pitch_remap` + the drain
  (`world_sim/player.rs:1480-1541`) + `play_female_hurt_sound`/
  `play_voice_style`/`play_hurt_npc_int` + the `PLAYER_VOICE_CUES` bank.
  f44's own note ("the hurt cue keeps its variant-1 stand-in until that
  fold") is the d79 stand-in question — §2's voice-picker ruling settles
  it: the merged hurt path is OURS.
- **The exit-save** — the one remaining shared remainder (103 §2b rule 3
  condition (b)); condition (a) the hydration is MET. The save direction
  casts back (`file.voice_variant = p.voice_variant as u8`) when that lane
  lands.

**VERDICT: the OPEN-BOTH hydration hunk is CLOSED (by f44's port, on our
published seam); e10 owes NO hydration code.** The stale "f44c/f44e seam,
CharPlay routing-only" notes on THIS branch (ANALYSIS ×2, PORT_MAP ×2,
ROADMAP ×1, the player.rs field doc, the anchors row) are re-stamped to the
closure in this seal's companion commit. The dh-line DEFAULT-DROP (132 §2f)
is safe once the exit-save lands: both flags ride L1 + this hydration.

## 2. The d79 pitch rewire receipt (W4-d79-pitch-final.md §F4, e10 hunks)

d79's PITCH FINAL 4 (`03ebefad7`) census classifies our tip as **91 prod +
2 test bare sites, +10 prod menu added, −7 Roar, −1 cart rewrite** (the
cart `play_at` is a rewrite of the base site, not an addition). Every
verdict re-verified against our landed code:

### The +10 prod menu rewires (all `play` → `play_menu`, draw-free pool)

| e10 site (this tip) | C# anchor | bare call today |
| --- | --- | --- |
| app/flow.rs:492 | `UICharacterCreation.cs:1517-1527` pitch-bar tick | `d.audio.play("Menu_Tick")` |
| app/flow.rs:711 | `:1288` cloth-style click | `play("Menu_Tick")` |
| app/flow.rs:742 | `:1042` voice-pitch click | `play("Menu_Tick")` |
| app/flow.rs:801 | `:1547` randomize player | `play("Menu_Tick")` |
| app/loop.rs:710 | `Player.cs:4976` ToggleInv map-close | `play("Menu_Close")` |
| app/loop.rs:719 | `:5010` ToggleInv talk-close | `play("Menu_Close")` |
| app/loop.rs:726 | `:5056`/`:5087` ToggleInv toggle | `play(if open …)` |
| app/loop.rs:765 | `:24973`/`:29111` fullscreen map | `play(if open …)` |
| app/loop.rs:778 | `:24953` map-style cycle | `play("Menu_Tick")` |
| app/loop.rs:833 | `:5087`/`:5056` rebound cInv | `play(if open …)` |

The smart-cursor press tick (loop.rs:800, `:28820` + the hold twin `:28827`,
one press-edge play covers the pair — `11793ad44`) rides the SAME
`play_menu` rewrite inside d79's hunk window.

### The −7 Roar removals — keep-one: either side's hunk survives

Our six `editing.rs` summon sites + the `doors.rs:62` golem-altar site
already play `play_roar(0, plx, ply)` — the SAME façade + arguments d79's
phase-3 rewire uses. All seven C# sites carry an EXPLICIT `0` style
(`Player.cs:43760`…`:44044` + `:32782` re-read: every call is
`PlaySound(15, x, y, 0)`), so `Roar_0` is faithful HERE.

**The Roar_0→Roar_1 fix is g21's Destroyer site, NOT ours**: `NPC.cs:50093`
is the 3-arg `PlaySound(15, x, y)` — the int overload's default `Style = 1`
(`SoundEngine.cs:102`), so d79's `play_roar(1, …)` (`Roar_1`) wins over
g21's bare `play_at("Roar_0", …)` (wrong twice: no guard, wrong slot).
Our `play_roar` already models the case-15 playing guard + the style-4
remap (`sound.rs::roar_case15`) and draws NOTHING — the pitch/pool contract
holds on every roar site we own.

### The cart rewrite (world_sim/player.rs:1365-1380)

Our drain matches `Item_53/55/56` → `play_proj_sound((2, style), …)` (the
same case-2 path as d79's `play_npc_item_cue`) with a `_ => play_at`
fallback. **DELETE the fallback at the merge** (the queue only ever carries
those three); keep ONE drain — d79's `play_npc_item_cue` form per its F4
ruling. The per-style pitch windows (53 `Next(-20,-11)*0.02`, 55 negated,
56 `Next(-50,51)*0.002` + the 0.6 volume) are identical on both lanes.

### The façade keep-one rulings (re-confirmed against our code)

- **MaxMana (case 25): d79's pool wins the PLAY, our mana-blade PRODUCER
  keeps.** Our `play_max_mana_sound` (audio.rs:2119, emit-only — stacks
  voices) is superseded by d79's `play_single_voice("MaxMana", -1, -1)`
  (the no-Stop single-slot overwrite, the C# `LegacySoundPlayer.cs:964`).
  Our full-bar refill producer (`pending_mana_full` +
  `take_mana_full`, player.rs:1964/:3542, drain world_sim/player.rs:1637)
  is d79's `pending_mana_refill` duplicate — DELETE our queue + drain,
  keep d79's (it carries the live box for the dust burst). **KEEP our
  mana-blade expiry producer** (`editing.rs:1787-1841`, `Player.cs:42837-
  42844` → `EmitMaxManaEffect` `:49798-49808` — a seam d79 does NOT have):
  re-point its play at `play_single_voice("MaxMana", -1, -1)`; the 5×dust-45
  burst body (`Next(20,26)*0.1` scale before each NewDust, noLight/noGravity/
  velocity×0.5) is identical on both lanes — one copy survives.
- **Hungry (case 42): BOTH producers keep, ONE façade — ours.** Our three
  drink callers (`potions.rs:454/:767/:850`, the 333-expiry flag from
  `apply_potion_delay`/`try_to_reset_hunger_to_neutral`, `Player.cs:5524`/
  `:51873`/`:51879`) play `play_voice_style("Hungry", -1, -1, 0, rng)` —
  the case-42 contract (the NextFloat on the audio-only stream, zero
  gameplay draws). f12's `pending_hungry_sounds` counter (the UpdateBuffs
  333-expiry, `Player.cs:12138`) keeps its producer; its drain
  (`world_sim/player.rs:1242`, bare `play("hungry")` — bypasses the
  tracked-style volume/variance) re-points at our façade.
- **The voice picker (case 20 + the hurt family): OURS wholesale.**
  `hurt_voice`'s C#-priority chain + `play_female_hurt_sound` (the ONE
  `Next(3)` slot pick) + `play_hurt_npc_int` + `play_voice_style` +
  `play_no_space_sound` (stoned) are d79-compatible façades; d79's
  `pending_hurt_sound = Some("Player_Hit_0")` default-variant stand-in
  (its player.rs:4582) is superseded — the merged tree keeps OUR drain
  (world_sim/player.rs:1480-1541). The picker's preview runs the SAME
  `default_voice_hurt` tail through `drain_voice_preview` (flow.rs:404) on
  the bare-`Main.rand` stand-in — no world draws.

## 3. The g21 chat-handlers receipt (g21c resume input)

Probe: g21's pushed tip is still batch 5 `7596fb262` (g21c NOT pushed); the
chat chain (`chat_key`, loop.rs) rides its batch 2-5 code. Our
doors/buff-grant vs their chat handlers:

- **BuffStation routing is DISJOINT.** g21's diff touches `editing.rs`
  only at `place_wall_route` (~:5364, the replacement-toggle gate) and
  `place_tile` (~:7509, the biome-torch swap toggle); it does NOT touch
  `doors.rs` or `tr-core/edit.rs` at all. Our BuffStation chain
  (`edit.rs:255/:316` routing + `doors.rs:402-445` grant/play + the 192/366
  buff arms) lands in files/hunks g21 never opens → zero overlap, both
  sides land as-is. Their `player.rs` hunks (toggle_pet/toggle_light,
  tile_replacement_enabled/using_biome_torches + tests) sit outside our
  voice-field block (:1989-2010) and our voice trees (:3589-3670).
- **The chat pair is a DELETE for g21, not a rewire.** Their
  `world_sim/npcs.rs` `play("Chat")` at talk-open (`Main.cs:43083`) and
  Angler turn-in (`:40243`) duplicate d79's phase-7 seams
  (`play_single_voice("Chat", -1, -1)`) — d79's Chat pair wins (its F4
  ledger: −2 deletes). Their `npc_chat.rs` witness hunk (the played_count
  assertions on `talk_to_npc`) re-pins onto d79's seam — the count contract
  (one cue on the fresh talk, silent re-click) is unchanged.
- **loop.rs hunk adjacency, not collision.** g21 extracts the inline chat
  keyboard block (~:153-253) into `chat_key` and adds the F7/F8/F10/F11
  arms; our ToggleInv/map/smart-cursor/cInv hunks sit at :704-841 (base
  coordinates :677+). Different regions of the same file — a textual merge
  resolves cleanly; the g21c resume must re-run its chat witnesses against
  the merged `world_key` (our Escape chain closes map/talk BEFORE the
  inventory toggle; their chat gate `!drawingPlayerChat` reads the same
  `chat.open` state).
- **flow.rs**: g21 only extends the `front_flow_tests` App literal (new
  hover/diag fields); our voice-picker hunks (:308-828) are disjoint.

## 4. The mount-census × HeightOffset-seam compose-note (f3 research/65)

Our mount VOICE census (ANALYSIS "The mount voice-override census",
witness `w4_e10_the_mount_voice_override_census_is_complete`) and f3's
mount OFFSET census (research/65, retire pass `75f3f7347`) are disjoint
axes of the same mount rows:

- **Voice axis (ours, COMPLETE):** the hurt tree's mount block
  (`Player.cs:38651-38678`) covers EXACTLY {52 Wolf→NPC_Hit_6, 54
  Velociraptor→NPC_Hit_47, 55 Rat / 56 Bat→NPC_Killed_4, 61 Pixie→
  NPC_Hit_5}; the death tree has NO mount block; every other mount —
  incl. the whole `MountID.Sets.Cart` set (6, 38, …) — falls through to
  the default voice. Reads only `mount.is_active()`/`mount.kind()` —
  NO offset getter.
- **Offset axis (f3's 65-seam, DEFERRED):** the five getters
  (`HeightOffsetVisual` :3237 / `HeightOffsetHitboxCenter` :3253 /
  `MountXOffset` :3269 / `HeightOffsetBoost` :3281 / `HeightMapOffset`
  :3297 + `BaseHeight` :3335) feed the DRAW/hitbox/minimap consumers —
  `mounted_center`/`mounted_top` (our player.rs:4997/:5007) fold to 0
  today; the seam task re-sites them. **The voice trees never read
  `MountedCenter`/`MountedTop`** (the local-player hurt cue is
  UNPOSITIONED, `:38572-38575`; the death cue plays at `base.position`,
  `:38745`) — so the 65-seam's re-siting cannot change a voice outcome,
  and our census cannot constrain the seam. No shared hunk.
- **portableStool stays deferred WITH the seam** (f3's verdict,
  re-confirmed): the stool machine (`UpdatePortableStoolUsage` :29193-
  29205 + `ResizeHitbox` + `CanFitSpace` + the jump tail/jump-off/dye
  slot) is not self-contained; the stool is NOT a mount, so it never
  enters the voice mount block either — the voice census needs no stool
  arm.
- **One adjacent consumer to note at the seam merge:** the DST accessory
  recharge cues (`Player.cs:28333`/`:28396`/`:28401`/`:28406`) play at
  `this.MountedCenter` — they are in our 51-site DEFERRED list (the
  `Update_TimerCrit`/`Update_SnappingStone` machines unported). When both
  the machines and the 65-seam land, those four sites take the re-sited
  accessor — the seam's leverage covers them without a voice-lane change.

## 5. The voice-family bible (the merged-tree contract)

The single entry point for the voice family at the merge, in fold order:

1. **Transport** — f44's u8 `PlayerTail` walk (132 §3-L1 rule 7); our
   `player_file.rs` diff DELETES, the picker/draft/session hunks KEEP with
   the `as u8` cast (§1).
2. **Live fields** — one declaration set: `voice_variant: i32` (ctor 1) +
   `voice_pitch_offset: f32` (ctor 0) dedup textually (f44 == ours);
   `voice_override: i32` is OURS alone (the per-tick accessory slot,
   `ApplyEquipFunctional` :14382-14385 / `ResetVisibleAccessories` :31146).
3. **Hydration** — f44's `hydrate_from_file` (§1); our cast-seam witness
   pins the consumer contract; no e10 code owed.
4. **Decision trees** — OURS wholesale: `hurt_voice` (stoned 156 →
   voiceOverride 19 arms → mount {52,54,55,56,61} → werewolf → Frost →
   Necro → DST-world → voiceVariant 1/2/3 with the
   `Utils.Remap(-1,1,-0.7,0.7)` pitch) + `death_voice` (voiceOverride
   {1,2,3,4,5,9,15} → DST/anniversary → default case 5) +
   `default_voice_hurt` (the shared preview tail). d79's variant-1
   stand-in DELETES.
5. **Playback façades** — case 1 `play_hurt_sound` (`Next(3)`), case 20
   `play_female_hurt_sound` (`Next(3)`, no pitch draw), case 3/4
   `play_hurt_npc_int` (the one `Next(-10,11)*0.01`), case 2/42
   `play_voice_style` (the NextFloat audio-only), case 0
   `play_no_space_sound` (stoned), case 5 `play_death_sound` (top-left);
   the chicken `Next(5)==0` rare pick rides the drain
   (world_sim/player.rs:1502-1510). All draws on `UpdatePlayers` AFTER
   the attenuation gate; the bank rows `Female_Hit_0..2`, `NPC_Hit_5/47`,
   `Item_35` + `PLAYER_VOICE_CUES` (audio.rs:551) keep.
6. **The picker** — `ui/char_creation.rs` (the `CycleVoiceStyle` row over
   `VariantOrder` {1,2,3}, the play/pitch-slider 3-tick debounce
   `PlayVoicePreview`, `UpdateSelectedGender` 1↔2, the 1/20 banker's
   quantize, `Sets.Colors` tint) + `drain_voice_preview` (flow.rs:404) on
   the bare-`Main.rand` stand-in. The four `Menu_Tick` sites rewire onto
   d79's `play_menu` (§2).
7. **The census canaries** — `w4_e10_the_hurt_voice_follows_the_c_priority_order`,
   `..._death_voice_...`, `..._mount_voice_override_census_is_complete`,
   `..._male_set_...`, `..._hydrated_voice_pair_drives_the_default_voice_tail`
   (player.rs:15643-15845) + the 13 picker witnesses (char_creation.rs) +
   the 3 transport witnesses (re-pin per §1). All WRITTEN NOT RUN
   (code-only lane).

### Verification log (this seal pass)

- `git fetch origin --prune` — two connection resets, third attempt green
  (the 90 s retry rule); sibling tips re-probed on origin afterwards.
- f44 `hydrate_from_file` body re-read at `0d1ee2045` — the voice hunk
  matches the published cast-seam byte-for-byte (§1).
- d79 §F4 tables re-read at `03ebefad7`; our +10/−7/−1 sites re-counted
  against the landed code (§2's table matches the tip line numbers).
- `Player.cs:43760`…`:44044` + `:32782` re-read — all 19 case-15 sites
  carry the explicit `0`; `NPC.cs:50093` re-read — the 3-arg default-1
  call (§2's Roar ruling).
- g21 diff `a725acb04..7596fb262` re-read — `editing.rs`/`player.rs` hunk
  coordinates vs ours (§3); `doors.rs`/`edit.rs` untouched by g21.
- `python tools/_e10_final_census.py` re-run: 211 sites, 160 claimed /
  51 unclaimed — unchanged from the DONE tip (the seal adds docs only).
- research/32 rows + INDEX re-stamped SEALED for the voice lane (§5's
  contract is the row evidence).