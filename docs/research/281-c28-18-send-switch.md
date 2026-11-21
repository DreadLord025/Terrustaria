# 281 — W4-g48: the 18 SetTime send-switch instruction for the c28 lane (upstream publish)

Number **281** — RENAMED from 244 (REGISTRY-RENAME round, 2026-11-21).
The 244 pick was made on the correction-phase 358-tip scan (pushed
`2b61530b4` 14:30:16) BEFORE the research/150 round-2 refresh (d54 tip
`72226e500`) was visible: that refresh CONSUMED the whole 244–280
assignment block (244 = c31 jungle-residuals per §6.3), so 244 was
never free. Per the registry cutoff note (post-sweep pushes
self-adjudicate; losers take free numbers **≥281** after a fresh
all-tip sweep), a fresh 362-tip sweep found 244 additionally claimed by
f39 `244-f39-v18-claim-input.md` (`8704691a3` 14:32:01) and d53
`244-proj-dust-retire.md` (`a106d13e8` 14:56:57), and **281 unclaimed
at every tip** (the only 280+ doc anywhere: c42's assigned
`280-c42-wiring-seal.md`). This doc takes **281**; the sibling
item-lane seed 245 → **282** in the same round (245 = the d81/g53
18b-fork assignment). Winner context per the refresh: 237 = e23,
238+239 = d42, 240–243 = g58; this lane's f11-tail-fix chain
237→239→272. Published by the g48 NET-APPLY-1 correction phase (the
SEALED research/17 tail's external-dependency hand-off, re-probe
citations only).

**Audience:** `task/W4-c28-net-send-cadences` (c28c-cadence-resume-active).
The g48 census pinned the lane's remaining 18 scope as "the 205→18 send
switch" (research/17 row 18 + the FINAL verdict). This publish CORRECTS
that framing against the C# and hands over the exact switch.

## 1. The C# truth: id 18 has ZERO send sites — the clock rides id 7

Re-probe evidence (c28's own census tool + direct C# reads):

- `tools/c28_send_census.json` at c28 tip `6379f881b`: `per_id_counts`
  has **no key 18** (918 sites, 148 ids — 18 absent); the `sites` array
  has zero id-18 rows. The writer arm EXISTS (`NetMessage.cs:531-536`:
  `(byte)(dayTime?1:0)` + `(int)time` + `sunModY` + `moonModY`) and the
  reader arm EXISTS (`MessageBuffer.cs:1290-1298`, netMode==1: dayTime /
  time / sunModY / moonModY) — but NO C# code path ever sends 18.
- The live clock sync is **`7 WorldData`** (`NetMessage.cs:224-412` —
  `(int)Main.time` first, then the dayTime/bloodMoon/eclipse BitsByte;
  the reader `MessageBuffer.cs:473-497` applies time + flags + the whole
  world header on every receive, netMode==1). The time-carrying send
  sites (census id-7 rows, Main.cs):
  - `UpdateServer` `Main.cs:64560` — the `% 3600` heartbeat (60 s at
    60 Hz; `netPlayCounter` resets to 0 on fire),
  - `UpdateTime` `Main.cs:64956` — the `maxRaining != oldMaxRaining`
    edge (netMode==2 gate `:64954`),
  - `SkipToTime` `Main.cs:65197` — `TrySendData(7)` after
    `Main.time = timeToSet` (netMode==2 `:65195`),
  - `UpdateTime_StartNight` `Main.cs:65361` — after `time = 0;
    dayTime = false` (netMode==2 `:65359`),
  - `UpdateTime_StartDay` `Main.cs:65409` + `:65439` — the day-flip
    send + the eclipse-roll send (netMode==2 gates `:65407`/`:65437`),
  - `Sundialing`/`Moondialing` `Main.cs:6180`/`:6195` — the
    fast-forward triggers (the netMode==1 legs send `51` instead,
    `:6175`/`:6190`),
  - the weather/invasion/cloud edges (`UpdateWeather` `:59004`/`:59035`,
    `UpdateInvasion` `:64359`, `updateCloudLayer`
    `:13198`/`:13220`/`:13228`, slime-rain `:64782`/`:64818`, the ded
    input callbacks `:5896-5914`) — id 7 doubles as the world-flags
    broadcast; the clock fields ride along.

So "switch 205→18" is NOT a 1:1 C# port: the faithful switch is
**internal 205 → the C# id-7 cadence**, with the c27 `SetTime` codec
kept as the landed 18 reader (wire-compat for any future/external 18
sender; the C# itself never emits it).

## 2. The g48 tip state (what the switch replaces)

- `crates/tr-app/src/net.rs:285` — `pub const INT_TIME: u8 = 205; //
  clock broadcast`; the `S2c::Time { time: f64, day: bool }` variant
  (`:2538`), the id arm `:3129`, the writer `:3245`
  (`w.f64(*time).bool(*day)` — NOT the C# 18 layout), the reader `:3477`.
- `crates/tr-app/src/app/netglue.rs:3098-3101` — the host broadcasts
  `S2c::Time` UNCONDITIONALLY on every 10 Hz cadence tick (the tail of
  the item-diff block); the client apply `:1588-1591` stores
  `world.time`/`world.day_time`.
- The port ALREADY carries the full `7 WorldInfo` codec
  (`net.rs:1383-1432` struct — `time: i32` first + `day_time` +
  `blood_moon`/`eclipse`/`moon_phase` + the whole header incl.
  `max_raining`/`sundial_cooldown`/`moondial_cooldown`; writer
  `write_to`/reader `read_from`, round-trip witness
  `world_info_round_trip` `:5057`). The host builds it via
  `App::world_info()` (`netglue.rs:2391`) and sends it ONCE at join
  (`:330`); the client's `S2c::WorldInfo` arm (`:1385-1395`) only stores
  `net_info` + advances the join FSM — the in-world RE-apply of the
  clock/flags fields (the C# reader's unconditional `Main.time =
  ReadInt32()` etc.) is the missing half.
- The `net_play_counter` `% 3600` wrap already exists at
  `netglue.rs:3061-3064` (the S1-I-4a item-heartbeat port) — the
  heartbeat send hooks into that exact arm.

## 3. The exact switch (for c28 — apply on the c28 tip after the union)

1. **Host heartbeat:** in the `% 3600 == 0` arm
   (`netglue.rs:3062-3064`), before the counter reset, add
   `server.broadcast(&S2c::WorldInfo(Box::new(self.world_info())));`
   — the C# `UpdateServer` `SendData(7)` (`Main.cs:64560`).
2. **Host event edges:** the clock lane's `world_sim/clock.rs` flips
   (`:54` StartNight / `:70` StartDay) and the SkipToTime/sundial/
   moondial paths raise a "world flags dirty" request the net tick
   drains as one `WorldInfo` broadcast per edge (the C# `:65361`/
   `:65409`/`:65197`/`:6180`/`:6195` sites; the weather/invasion edges
   ride the same broadcast when those lanes land — one truth, the
   `world_info()` builder).
3. **Client in-world re-apply:** extend the `S2c::WorldInfo` arm — when
   `net_state >= IN_WORLD`, apply `world.time = info.time as f64`,
   `world.day_time`, `blood_moon`, `eclipse`, `moon_phase` (+ the
   weather/cooldown fields the port rows) BEFORE the join-FSM branch
   (the C# reader applies on every receive; the join install
   `client_install_world` `:2337-2343` already reads the same fields).
4. **Retire internal 205:** delete the unconditional `S2c::Time`
   broadcast (`netglue.rs:3098-3101`), the `INT_TIME` constant + the
   `Time` variant/arms (`net.rs:285`/`:2538`/`:3129`/`:3245`/`:3477`)
   and the client `S2c::Time` arm (`netglue.rs:1588-1591`) — ONE clock
   path, never both (the double-send ban, the net_npc_sync SWAP
   precedent). The c27 `SetTime` codec + apply arm STAY (the 18
   reader is wire-compat; its witness
   `w4_c27_set_time_applies_the_hosts_clock` notes the host's 10 Hz
   broadcast rides the same values — after the switch the assertion
   holds via the WorldInfo re-apply instead; re-pin that witness
   comment at the switch).
5. **Docs:** research/17 row 18 PARKED→WIRED + deviations row 13
   (`:449`, the internal-205 admission) closed with the switch cite;
   the ANALYSIS internal-clock deviation note (ANALYSIS.md:7275-7278
   region) repointed.

Cadence note: the C# has NO periodic sub-minute clock send — between
edges the client simulates its own clock (the port already does,
`world_sim/clock.rs`); the 60 s heartbeat is the only drift correction.
The port's 10 Hz unconditional broadcast is the admitted stand-in this
switch removes.

## 4. Verification

Code-only — no cargo/rustc/tests/rustfmt/graphify invoked. The c28
census json re-read at tip `6379f881b` (no 18 key, 58 id-7 sites); the
C# windows re-read direct (`NetMessage.cs:224-260`/`:531-536`,
`MessageBuffer.cs:473-497`/`:1290-1298`, `Main.cs:64556-64562`/
`:64952-64958`/`:65190-65199`/`:65356-65362`/`:65405-65441`/
`:6173-6199`); the g48 port seams re-read at HEAD (`net.rs`/
`netglue.rs` line cites above); the c27 landed shape re-read at tip
`412b6bc0d` (`SetTime` struct `net.rs:1575-1581`, apply
`netglue.rs:2027-2030`, sunModY/moonModY parked — no port field).
