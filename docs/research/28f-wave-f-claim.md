# S1-E-7 wave F (W4-d72) — fifth-block census: every arm range is claimed; the FRAME is the genuinely unclaimed remainder

> Worker W4-d72, branch `task/W4-d72-buff-effects-wave-f` @ main `a725acb0`.
> Code-only wave: tests WRITTEN NOT RUN (no cargo/rustc per the worker
> contract); compile-by-inspection. This note is the early claim commit.

## The in-flight census FIRST (2026-11-20, re-derived at `a725acb0`)

The brief's "fifth missing buff block, Player.cs:9855-12069" was re-censused
against ALL in-flight branches before taking any id. Every `buffType[j]` arm
head in the region was extracted (254 heads, the c17/c18/d71 census
reproduced) and diffed against the landed dispatch sets AND the five
in-flight claim commits:

| wave | branch | claim commit | C# range | ids |
| --- | --- | --- | --- | --- |
| A (landed) | c16 | in main | `:9875-10048` potion block | player.rs match |
| B (landed) | c16 | in main | `:10049-10317` debuff block | buff_effects.rs |
| C (landed) | c17 first | in main | `:10800-11400` + imbues `:12013-12044` | buff_effects_accessory.rs |
| D (landed) | c18 first | in main | `:11400-12069` + pet head `:11005-11241` | buff_effects_misc.rs |
| C2 | c17 | `fa1e8930` | `:10318-10461` | 94/396/95-100/170-181 |
| D2 | c18 | `02d946e1` | `:10462-10976` | 62 + 29 summon keep-alives |
| E | d71 | `e2dc1fcf` | `:11406-12012` | 37 (late pets + status tail + Werewolf + hunger tiers) |
| G | d73 | `7cf886f9` | `:10328-10485` + `:9935-9946` | 22 (boosters + Sugar Rush 192 + Brain of Confusion 321) |
| H | d74 | `dc654c62` | `:10318-10808` | 40 (tail census; overlaps C2/D2/G by id — their merge problem, not this wave's) |

**Result: ZERO unclaimed arm ids.** The union of the landed waves and the
five in-flight claims covers every arm head in `:9875-12044`. The c16
branch itself has no commits past main (its waves A/B landed already).
Per the brief ("take genuinely unclaimed"), this wave does NOT re-port any
arm id. What IS genuinely unclaimed is the **`UpdateBuffs` FRAME** — the
statements around the arm chain that no wave's range includes:

## The claimed remainder, exactly (the frame, no arm ids)

| piece | C# | live today |
| --- | --- | --- |
| soulDrain → `AddBuff(151, 2)` head | `:9857-9860` | NONE — no `soul_drain` field, no buff-151 grant anywhere (`player.rs:3852` only excludes 151 from a clear; the regen consumer `:19121/:19160` rides `RegenInputs.extra_*`, unfilled for soulDrain) |
| `dontStarveWorld` → `UpdateStarvingState(true)` head | `:9861-9864` + helper `:12152-12182` | NONE — `potions.rs` ports `TryToResetHungerToNeutral`/`UpdateHungerBuffs` but nothing calls `UpdateStarvingState`; the DontStarve world never grants the 332 Well Fed chain |
| per-tick `UpdateHungerBuffs()` tail | `:12046` + helper `:12098-12150` | PARTIAL — the helper is `potions.rs::update_hunger_buffs` (private, drink-path only via `try_to_reset_hunger_to_neutral`); the PER-TICK call site is missing, so hunger tiers never expire on their own |
| luckPotion diff → `luckNeedsSync` + `oldLuckPotion` | `:12047-12051` | NONE — `luck.rs` documents the sync as net-only; the diff bookkeeping itself is unported |
| lavaOpacity fade ladder | `:12052-12067` | NONE — `recall_teleport.rs:491` explicitly records "lavaOpacity :37590 has no port field"; `render/liquids.rs:137` notes the factor is not applied |

The loop preamble (`:9865-9874`: slot skip + `TimeLeftDoesNotDecrease`
decrement) is ALREADY live (`buffs.rs::tick`, `:384-395`) — witness-only
here, not re-ported.

## RNG census of the frame

Direct `Main.rand` draws: **ZERO** (`:9857-9864`, `:12046-12067`,
`:12098-12182` scanned). Indirect:
- `AddBuff(151/332/333/334)` — `Player.AddBuff` (`:5263-5295`) draws no rand
  (the port's `buffs.rs::add_buff` is the verbatim port).
- `EmoteBubble.MakeLocalPlayerEmote(146/147/148)` — `EmoteBubble.cs` draws
  no rand on the make path (`NewBubble` `:116-131` is deterministic; the
  port's `emotes.rs::new_bubble` mirrors it). Recorded as REQUESTS.
- `SoundEngine.PlaySound(SoundID.Hungry)` (`:12138`) — the legacy sound
  style's pitch roll rides the audio seam; recorded as a REQUEST (the
  `SoundID.Hungry` trackable, `SoundID.cs:1711`).

No stream is touched by this wave.

## Shape

Pure accumulator module `crates/tr-app/src/buff_effects_frame.rs`
(`FrameEffects` + `apply_buff_frame`), the wave-B/C/D/E/G/H seam pattern:
- head grants + hunger expiry grants + emote/sound requests fold into the
  accumulator; slot writes (the hunger zeroing) run on `BuffSlots` via the
  existing `potions.rs` helper shape — the module REUSES
  `potions::update_hunger_buffs` logic through a pub re-export rather than
  a second copy (keep-one).
- `FrameState` carries the persistent `oldLuckPotion` + `lavaOpacity`
  (the C# `Player` fields the port lacks).
- Wiring into the live tick rides the waves' merge step — the module is
  test-only until then; no runtime behaviour change, no RNG or
  fixed-step-order change.
Witnesses: one `s1_e_7f_*` test per piece + the frame-order/RNG pins.