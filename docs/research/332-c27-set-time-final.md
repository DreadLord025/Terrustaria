# 332 — W4-c27 SEAL: the 18 SetTime lane FINAL + the clock-family bible

Number **332** — chain **248→332** per the research/150 registry §9.2
UNIFIED FINAL (d54 tip b243c7086, row 332): 248's first-add belongs to
e26-luck (a23c64933), so this doc renumbers to 332. The seal-time scan
below is chain history, kept verbatim.

Chain history — number **248** per the research/150 registry rule (38bfd1d22): the
151–236 block consumed; 237 = e23 (g61 238 / g48 239 renames), 240 =
c42/d53/g58 triple, 241–243 = g58, **244 = g48's 244-c28-18-send-switch
(first-add 2b61530b4 14:30:16) beats f39's 244-f39-v18-claim-input
(8704691a3 14:32:01)** — first-pushed-wins per registry-150, so g48
KEEPS 244 and f39's renumber is ITS lane's obligation (recorded here,
not acted on), 245 = g48, 246 = d10
(d89380ca1 14:52:30), **247 = g67** (local tip, post-union-folds —
unpushed at the scan but claimed on `task/W4-g67-weather-rain-splash`),
262–280 claimed by local in-flight lanes (c8 262, g87 263, f22 264,
e20 266+269, c10 270, g50 271, g61 273, g56 274, f21 276, f45 279,
c42 280). Full remote-tip ls-tree scan + local-branch scan at the seal:
**248 is the first number free at BOTH.** This document is the c27
lane's FINAL — no further c27 commits planned.

**Mode: code-only.** No cargo/rustc/tests/rustfmt/graphify invoked.
The three new witnesses below are WRITTEN NOT RUN.

## 1. The git-log census — what landed on the clock lane

Branch `task/W4-c27-net-entity-state-ids`, four batches, tip
**412b6bc0d** (base a725acb04, wave-3 complete):

| Batch | Commit | Ids | Shape |
| --- | --- | --- | --- |
| 1 | **6fb7bcb25** | **18 SetTime** + 36 SyncPlayerZone | bodies/codecs verbatim (`NetMessage.cs:531-536`/`:933-944`, `MessageBuffer.cs:1290-1298`/`:2178-2202`) + host apply/relay arms; client clock apply (`== 1` day flag); sunModY/moonModY parked; faeling edge parked |
| 2 | 8bc1a1f63 | 41 ItemRotationAndAnimation + 43 ManaEffect | id forced to sender, channel re-derive `:2283`, HealMana peer text |
| 3 | 8ce6ba1bc | 51 MiscDataSync + 55 AddPlayerBuffPvP | Sundialing/Moondialing cooldown-0 gate + dawn/dusk ff + cooldown 8 + SendData(7) re-push (`Main.cs:6169-6197`), TorchGod kill, dual-hostile+pvpBuff gate |
| 4 | **412b6bc0d** | 62 SyncDodge + 84 PlayerStealth | ids forced, client own-id dodge via `Player.apply_net_dodge`, stealth float on `PlayerNet.stealth`; entity-state census complete 18/36/41/43/51/55/62/84 |

The 18 landing itself (batch 1, re-read at this seal):

- `crates/tr-app/src/net.rs:151` — `pub const SET_TIME: u8 = 18`
  (`MessageID.cs:45`).
- `net.rs:1574-1581` — `SetTime { day_time: bool, time: i32, sun_mod_y:
  i16, moon_mod_y: i16 }`, doc-cited `NetMessage.cs:531-536`.
- `net.rs:1941` variant; `:2474` id arm; `:2626-2630` writer
  (`u8::from(day)` + `i32` + two `i16`, the exact C# order);
  `:2857-2862` reader (`r.u8()? == 1` — the `== 1` flag, NOT `!= 0`).
- `crates/tr-app/src/app/netglue.rs:2029-2032` — client apply:
  `world.day_time = t.day_time; world.time = t.time as f64;` — the
  parked offsets read off the wire and dropped (section 4).
- Witnesses (landed): `net.rs::w4_c27_set_time_round_trips` (byte pin
  incl. the byte-2-is-night discriminator + truncated-body refusal)
  and `netglue.rs::w4_c27_set_time_applies_the_hosts_clock`.
- `anchors.txt:899` — the batch-1 canary row (`SET_TIME` marker).

C# windows re-read direct at this seal (≤100-line windows):
`NetMessage.cs:531-536` (writer), `MessageBuffer.cs:1290-1298`
(reader — `netMode == 1` only), `Main.cs:1185/:1187` (the two `short`
fields), `Main.cs:62937-63031` (DrawSunAndMoon's mod-offset tail).
Both match the landed codec byte-for-byte.

## 2. The c28 cadence hand-off — 205→18-send-switch PENDING

Re-probed at c28 tip **1ac5929f3** (`task/W4-c28-net-send-cadences`,
origin-synced; c28c cadence-final: the 30 pvp click send + 157/19
decisions + expiry-54 wiring):

- `net.rs:160` — `pub const INT_TIME: u8 = 205` STILL LIVE; `:2192`
  id arm, `:2279` writer (`w.f64(*time).bool(*day)` — not the C# 18
  layout), `:2439` reader.
- `netglue.rs:3168-3170` — the `% 3600` wrap still resets the counter
  with NO send inside; `:3204-3207` — the UNCONDITIONAL 10 Hz
  `server.broadcast(&S2c::Time { .. })` still fires; `:1474-1477` —
  the client `S2c::Time` arm still stores time/day; `:1273` — the
  `S2c::WorldInfo` arm is still join-FSM-only (no in-world re-apply).
- `merge-base --is-ancestor 412b6bc0d 1ac5929f3` → exit 1: **the c27
  reader is NOT yet in c28's history** — the two lanes meet at the
  union/merge, not before.
- c28's own `net_player_sync.rs:24-25` doc explicitly assigns the
  `36`/`18` BODIES to c27 ("this module carries the send DECISIONS
  only") — the lane boundary both sides record identically.

**The switch instruction is g48's research/244** (at g48 tip
2b61530b4, `244-c28-18-send-switch.md`): the C# has ZERO `SendData(18)`
sites — the clock rides **7 WorldData** (the 3600-tick `UpdateServer`
heartbeat `Main.cs:64560` + the event edges), so the faithful switch is
internal-205 → the id-7 cadence with the c27 18 reader KEPT as the
wire-compat leg. c27 owes nothing further: the reader/codec/apply stay
as landed; the switch (heartbeat send, edge drains, in-world WorldInfo
re-apply, 205 retirement) executes on c28's tip after the union, per
244's five steps. **c27's 18-reader-landed is the receipt c28's step 4
consumes** ("the c27 SetTime codec + apply arm STAY").

## 3. The g48 NET-APPLY FINAL cross-receipt — 18 = c27-landed

Re-probed at g48 tip **2b61530b4** (correction phase; the SEAL is
9460c85d9, research/17 tail frozen):

- research/17 row 18 reads: "PARKED on this tip — FINAL re-probe: the
  reader/codec LANDED on the clock lane's c27 tip `412b6bc0d` (batch 1
  `6fb7bcb25`: `id::SET_TIME = 18` + `SetTime` body + client apply arm
  + `w4_c27_set_time_applies_the_hosts_clock`; sunModY/moonModY parked,
  no port field)." — the hashes and symbols match section 1 exactly;
  the row's PARKED refers to g48's OWN tip (which still rides 205),
  not to the lane.
- Deviations row 13 stays admitted until c28's switch lands; the
  correction re-probe (zero 18 send sites, id-7 cadence) is recorded
  there and in 244.
- The g48 census FINAL (ef9a9aef0) mechanical count: 30 WIRED / 4 N/A
  / 4 DEAD / 2 PARKED = 40/40; 18's reader half is c27's contribution
  to the WIRED set at the union, its send half c28's.
- **Numbering note for the merger:** g48's 244 doc collides with
  f39's `244-f39-v18-claim-input.md` — the first-add timestamps
  (`--diff-filter=A` lineage): g48 2b61530b4 **14:30:16** vs f39
  8704691a3 **14:32:01**, so per registry-150 first-pushed-wins **g48
  keeps 244** and f39's renumber is ITS lane's obligation (recorded
  here for the f39 lane/merger, not acted on from c27). This seal
  took 248, clear of both at seal time (chain history — renumbered
  to 332 by the registry §9.2 UNIFIED FINAL: 248's first-add is e26's).

## 4. sunModY/moonModY — PARKED, with the full C# scope pin

The park is CORRECT and now scoped to the whole writer family, not
just "no port field":

- The ONLY writers of `Main.sunModY`/`moonModY` in the entire C# are
  (grep census over `decompiled/Terraria/*.cs`):
  1. `Main.cs:62983-62984` — the per-frame decay
     `(short)((double)mod * 0.999)` inside `DrawSunAndMoon`;
  2. `Main.cs:62998`/`:63007` — the title-screen drag
     `(short)(Main.mouseY - num4/num7)`, gated
     `Main.gameMenu && Main.netMode != 1` (`:62985`) with the
     `alreadyGrabbingSunOrMoon` latch (`:1189`, set `:63017` region);
     the same block drives `Main.time` from the drag x
     (`:62997`/`:63006`, clamps 53990/32390 and floor 10);
  3. `MessageBuffer.cs:1295-1296` — the 18 reader itself (the sync
     leg, client-only).
- So outside the title screen both fields are 0 (the port's
  `tr-core/src/sky.rs:20-21` doc already records this), and a CLIENT
  (`netMode == 1`) never runs the drag — it only ever receives the
  values. The offsets feed the celestial draw positions
  (`:62944`/`:62963`/`:62981`/`:63026` `LastCelestialBodyPosition`)
  — pure render, no sim, no RNG.
- **Park contract:** the wire keeps both shorts (landed codec; the
  new full-range witness pins them), the client apply drops them
  (landed arm; the new drop witness pins it). The port home arrives
  with the title-screen celestial-drag port (the front-end lane owns
  `DrawSunAndMoon`'s interactive block + `starGame`); at that point
  the apply arm gains two field writes and nothing else changes.
  No journey-mode connection: the creative time powers
  (`creative.rs:151` `button_target` → `App::set_time_of_day`
  `app/mod.rs:1875-1882`) set day/noon/night/midnight directly and
  never touch the offsets. The batch-1 commit message's
  "journey-mode sun/moon drag offsets" wording is corrected here:
  it is the TITLE-SCREEN drag, gated `gameMenu && netMode != 1`.

## 5. The clock-family bible — one owner per body

The full clock family, C# anchor → port → owner lane. No two rows
own the same body; the family's lanes compose without a wire change.

| C# (Main.cs unless noted) | Port | Owner / state |
| --- | --- | --- |
| `dayLength`/`nightLength` consts `:1173`/`:1175` (54000/32400) | `tr-core/src/sky.rs:9-11` `DAY_LENGTH`/`NIGHT_LENGTH` | main (render maths, M1.4-BG era) |
| `UpdateTime` add + strict boundaries `:64960-64964`, `:65137`/`:65148` | `world_sim/clock.rs::advance_time` (the `dt*60*dayRate` add, `t > 54000`/`t > 32400` STRICT, overshoot discarded) | main (`6122c416d` daycycle; boundary rolls `649583848`) |
| `UpdateTimeRate` `:6199-6226` + `IsFastForwardingTime` `:6160-6167` | `tr-core/src/world.rs:811-833` `is_fast_forwarding`/`day_rate` (ff 60 > asleep ×5 > 1) | main (M1.4-BG-21 `49f427229`); creative freeze/slider fold in `creative.rs:114` `time_rate` + `frame/world.rs:355-358` (journey lane `01d3cb93a`) |
| `Sundialing`/`Moondialing` SP arms `:6169-6197` (cooldown-0 gate, ff flag, cooldown 8) | `world.rs:1059-1078` `sundialing`/`moondialing`; triggers `edit.rs::interact_at` + `wiring.rs` | main (M1.4-BG-21); the `netMode == 1` SendData(51, kind 3/6) legs + host kind-3/6 apply = **c27 batch 3** (`netglue.rs:858`/`:874`, 8ce6ba1bc) |
| `UpdateTime_StartNight`/`StartDay` `:65210`/`:65365` (flag clears, cooldown--, moonPhase++ wrap 8, event rolls) | `clock.rs` boundary arms + `advance_time_with_events` (achievement rows, party/lantern schedulers) | main (S1-F-13/14, S1-E-18f `14a8f7bc3`); meteor bookend = g85 (`21e8eb1d7`); rain/cloudbg head clamp + lantern gates = c39→d130 (`ec27d8cfd`, weather family SEALED b8f655708) |
| `SkipToTime` `:65178-65199` (the boundary-crossing while loop + `TrySendData(7)`) | `App::set_time_of_day` (`app/mod.rs:1875`) sets the pair directly — the while-loop's StartNight/StartDay side effects are the d15 seam (`57aa7763a`: `set_time_of_day`'s flip rolls NightSetup) | creative buttons main; the full SkipToTime loop + net send = d15/c28 edges |
| `UpdateServer` `% 3600` heartbeat `:64555-64562` | `netglue.rs:2965-2968` counter wrap (S1-I-4a item heartbeat) — the SendData(7) inside = **c28 switch step 1** (research/244) | c28 PENDING |
| The id-7 event edges (`:64956` maxRaining, `:65361`/`:65409`/`:65439` boundaries, `:6180`/`:6195` dials, `:65197` SkipToTime, weather/invasion/cloud `:59004`/`:59035`/`:64359`/`:13198`/`:13220`/`:13228`/`:64782`/`:64818`, ded CLI `:5896-5914`) | the "world flags dirty → one WorldInfo broadcast" drain = **c28 switch step 2**; the client in-world re-apply = step 3 | c28 PENDING (g48 244's five steps) |
| `18 SetTime` writer `NetMessage.cs:531-536` / reader `MessageBuffer.cs:1290-1298` (ZERO C# send sites — wire-compat leg only) | `net.rs` SetTime codec + `netglue.rs:2029` apply | **c27 LANDED** (this seal); stays through the c28 switch |
| internal `205 Time` 10 Hz broadcast (the admitted stand-in, ANALYSIS `:11557-11560`) | `net.rs:170`/`:2547` + `netglue.rs:3002-3005`/`:1614-1617` | c28 switch step 4 RETIRES it (one clock path, never both) |
| `DrawSunAndMoon` arc maths `:62848-62877` | `tr-core/src/sky.rs::celestial_body` + `render/sky.rs:800-880` | main (render lane) |
| `sunModY`/`moonModY` decay + title-screen drag `:62983-63026`, `alreadyGrabbingSunOrMoon` `:1189` | UNPORTED — parked with the wire fields (section 4) | front-end/title-screen lane (unclaimed); the c27 wire leg is the only net surface |
| `Main.time` consumers: `SetBackColor` tints `:63452+`, fishing (`fishing.rs:58`), light params (`lighting/params.rs:169`), ambience (`audio.rs:2413`), sleep/tax (`clock.rs::update_taxes` `:64995-65003`) | the named modules read `world.time`/`day_rate` | main; no lane owns a second copy |

**Merge order for the family:** c27 (reader) + c28 (cadence switch,
consumes 244) meet at the union — c28's step-4 retirement deletes ONLY
the 205 constant/variant/arms/broadcast, never the 18 codec; the
weather family (d130-canonical) and the boundary-event lanes touch
`clock.rs`'s neighbours, not the 18 path; the title-screen drag port
later adds the two apply writes. The double-send ban holds: after the
switch exactly ONE clock broadcast path exists (id 7), with 18 as the
inert wire-compat reader.

## 6. The seal payload — witnesses WRITTEN NOT RUN

Three new witnesses (code-only, no compiler invoked):

1. `net.rs::w4_c27_set_time_mod_offsets_ride_the_full_short_range` —
   `i16::MIN`/`i16::MAX` offsets round-trip byte-exact (body bytes
   `00 80 ff 7f` pinned); the parked fields are wire-complete.
2. `netglue.rs::w4_c27_set_time_apply_drops_the_parked_offsets` —
   wild offsets on an 18 frame apply the clock and nothing else
   (no cooldown writes — the C# reader has no cooldown arm); follows
   the landed convention (host world values ride the same clock so
   the still-live 10 Hz 205 broadcast cannot race the assertion).
3. `netglue.rs::w4_c27_set_time_int_truncation_keeps_the_strict_boundary`
   — the `(int)` truncation × strict-boundary composition: a wire
   54000 keeps the client on day; the client's own `advance_time`
   then crosses exactly once with the overshoot discarded
   (`Main.cs:65137`/`:65148` semantics through the net path).

Landed witnesses re-verified by re-read (not re-run): the 8
`w4_c27_*_round_trips` codec pins + the 7 `w4_c27_*` netglue apply
pins; the batch-1 pair specifically (`w4_c27_set_time_round_trips`,
`w4_c27_set_time_applies_the_hosts_clock`). The landed apply
witness's comment ("the host's own 10 Hz clock broadcast rides the
same values") stays TRUE on this tip; g48 244 step 4 re-pins it at
the switch (its instruction, not c27's).

## 7. Verification (code-only)

- C# windows re-read direct: `NetMessage.cs:531-536`,
  `MessageBuffer.cs:1290-1298`/`:473-497`/`:2482-2530`,
  `Main.cs:1173-1189`/`:5880-5920`/`:6165-6226`/`:62841-63031`/
  `:64553-64566`/`:64822-64970`/`:65130-65240`/`:65353-65444` —
  all ≤100-line windows, junction paths read directly, no recursion.
- Port seams re-read at HEAD: `net.rs:151/:1574-1581/:1941/:2474/
  :2626-2630/:2857-2862`, `netglue.rs:2029-2032/:2965-2968/:3002-3005/
  :1614-1617`, `world_sim/clock.rs` (whole), `world.rs:811-833/
  :1059-1078`, `sky.rs` (whole), `creative.rs:114/:151`,
  `app/mod.rs:1875-1882`.
- Cross-lane tips re-read via `git show` (read-only): c28 1ac5929f3
  (section 2 probes), g48 2b61530b4 (research/17 row 18 + deviation
  13 + 244), registry scan over every pushed tip + local branch.
- NO cargo/tests/rustfmt/graphify run; the witnesses are WRITTEN NOT
  RUN. The catalog gate command for the eventual merger:
  `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`
  (judge by the `test result: ok` lines).
