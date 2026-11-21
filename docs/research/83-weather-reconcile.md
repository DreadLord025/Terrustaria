# 83 — Weather reconcile: the g11-side addendum (snowing keep-one)

**STATUS (union-exec phase): F1-F7 PUBLISHED as exact patch-hunks (§-patch
below) — d130 applies them mechanically on ITS branch (it owns execution,
tip `ec1d73c7` at publication; its `snow.rs` unchanged since `50c845e5`).
g11's own `main_dust.rs::snowing` + `SnowingEnv` retire at the MERGE
(keep-one); this branch's snow witnesses stay until the merger drops the
body. The teleport half is research/116 — EXECUTED on this branch.**

W4-g11 reconcile phase, code-only. This is the **g11 branch's addendum** to the
d130 reconcile doc (`docs/research/83-weather-reconcile.md` at `50c845e5` on
`task/W4-d130-weather-ambience` — not present on this branch; the merger unions
the two files). d130 picked **KEEP d130** for the `Main.snowing()` 3-way; this
addendum **agrees with the d130 base** and re-derives the arm-by-arm comparison
from the C# (`Main.cs:13032-13116`, re-read in full), correcting one claim in
d130's table and listing the **exact folds** the union needs. No code changes
in this commit — the folds are the merger's worklist.

Arms compared:

| arm | commit | body |
| --- | --- | --- |
| g11 (this branch) | `73545f0b` | `main_dust.rs::snowing` + `SnowingEnv`, wired at `scene_scan.rs:399-434` |
| d130 | `50c845e5` (body `5ae7f39a`) | `snow.rs::Snow::snowing` + `SnowInputs`, wired at its `scene_scan.rs:399-431` |
| d54 batch-4 | `48343a4c` | `world_dust_main.rs::snowing` + `SnowingEnv`, LATENT |

## 1. Arm-by-arm (every row re-derived against `Main.cs:13032-13116`)

| aspect | C# | g11 `73545f0b` | d130 `50c845e5` | d54 `48343a4c` |
| --- | --- | --- | --- | --- |
| wiring slot | `Main.Update:17238`, before `Sandstorm.EmitDust:17247`, outside every `SwapRandom` | WIRED — `scene_scan::update_scene`, before `emit_dust`, stored metrics (last-scan persistence) | WIRED — same slot, same order, stored metrics clone | LATENT (body + witnesses only) |
| RNG stream | bare `Main.rand` (pre-swap) | `NamedRngs::global()` — the port's bare-`Main.rand` stand-in; the pre-swap chest-eat cues already ride it (`frame/world.rs:580`) | own splitmix `Snow` stream (the WEATHER-2 per-consumer convention, `sand.rs` pattern) | stream parameter (caller picks; doc says `global`) |
| `Main.maxScreenW` (`:13045`) | static: 1920 at init (`Main.cs:66715`), **4096 under HiDef** (`SetGraphicsProfileInternal:3709`); the non-XNA default selects HiDef (`:67047`) | **4096** (`main_dust.rs:56`, cites `:3709`; the port runs HiDef — `frame/draw.rs:65`, `ambience.rs:184-187` "the Reach fallback is not used by modern builds") | **1920** Reach pin (`snow.rs:43`, "HiDef's 4096 arm is a documented drop", the `sand.rs:35`/`rain.rs:889` stand-in family) | `env.max_screen_w` parameter (caller picks) |
| budget `:13057` | `Main.snowDust` static — reset at the `UpdateDust` head (`Dust.cs:428`), `++` per live 76 slot (`:1747`); read once per attempt, constant within the call | **live-census stand-in** — `minecart::live_type(pool,76)` per attempt (O(6000) scan; the `SandStormCount` pattern) | **the real counter** — `DustPool.snow_dust` (`particles.rs:233/:329`), read once per call, fed by the 76 arm | `env.snow_dust` parameter (caller must supply the census) |
| `gamePaused` gate `:13040` | in-body early return BEFORE any draw | in-body (`env.game_paused`, `main_dust.rs:118`) | **folded out** — "folds into the caller's pause gate"; the `scene_scan` caller does NOT gate on pause (the fused per-frame scene update runs paused) | in-body (`env.game_paused`) |
| tile gate `:13086` | `InWorld && tile != null && !nactive() && wall == 0`; `nactive()` = active && !actuated (`Tile.cs:240-247`) | correct — `!(t.active() && !t.inactive()) && t.wall == 0`, OOB → no spawn | correct — `(active && !inactive) \|\| wall != 0 → continue`, OOB → continue | **WRONG** — `!t.active()` misses the actuated arm (d130's finding, confirmed) |
| attempt loop `:13053` | `for (int i = 0; (float)i < num3; i++)` | int counter, `(i as f32) < num3` — the C# shape | float `attempt += 1.0` — observation-equal (exact ≤ 2^24) | float `i += 1.0` — observation-equal |
| `Math.Sign(0)` `:13099` | three-way sign, 0 at 0 | explicit three-way match | explicit three-way match | `signum()` — observation-equal (sqrt(0)=0 factor), not the C# shape |
| intensity/recipe maths `:13044-13108` | f64 `Pow`, inner-`(int)` budget, `1+50*α` float bound, BREAK-not-skip budget gate, player-fall pull, else-if edge `Next(5)` pushes, off-screen y spread, double `+0.2*α` scale, Deerclops `/20` `/3` `-0.5` fades | identical | identical | identical |
| type-76 AI (`Dust.cs:1745-1763`) + tails (`:2071-2074`/`:2079-2082`/`:2119-2128`) | shared `UpdateDust` | none — "the 76 physics arm stays d50's dust_physics lane" (flakes ride the generic move/kills until it lands) | **ported** — `type_76_arm` (counter `++`, `+0.009` growth, the 0.04 lerp pull, the `!noLight` position push, the 10×10 solid damp) + `type_76_tails` (0.99 drag, rotation, `-0.01` scale), hooked in `minecart::tick:1106-1107` | none (d50 lane deferred) |
| draw (`Main.DrawDust` generic 76, `Main.cs:38968-39018`) | light at `(pos+4)/16`, (4,4)-origin rotated quad, full-dark kill | none (draw lane deferred) | **ported** — `snow::draw`, hooked at `frame/draw.rs:1292` | none (draw lane deferred) |
| `Snowflake` rain-rig stand-in | n/a | retired (keep-one, `73545f0b`) | retired (no `Snowflake` in its `rain.rs`) | untouched (latent note) |

**Correction to d130's §2 table:** its closing claim "the intensity/budget/
attempt-loop/recipe maths are identical in all three" is **wrong on the
budget** — the `maxScreenW` pins differ (4096 vs 1920), so `num2` differs by
~2.13× (at a 1600-px view, dry: `500*1600/4096 = 195` vs `500*1600/1920 =
416`), and the Deerclops `/3` arm compounds it. Every downstream budget
assertion (d130's snow witnesses, g11's
`w4_g11_snowing_breaks_on_the_live_76_budget` cap comment `25e180d4`) is
pin-dependent. The port runs HiDef (`frame/draw.rs:65`, `ambience.rs:184-187`
— both cite `Main.cs:67047`/`:3709`), so **4096 is the faithful pin**; d130's
1920 is a stand-in inherited from `sand.rs` (whose own 1920 pin is the same
deviation family — flagged for the sand/rain lanes, NOT changed here).

## 2. KEEP-ONE recommendation: d130 base + the g11 details folded

**Keep d130's `snow.rs` as the surviving module** — it is the only arm that is
wired AND carries the real `Main.snowDust` counter AND the flake AI AND the
draw (d130 §2's reasoning stands). **Fold the g11 body details into it:**

* **F1 — the body + stream.** Replace `Snow::snowing`'s body with g11's
  `main_dust.rs::snowing` body (they are line-for-line the same maths; g11's
  carries the corrections below). Re-sign to a free function taking
  `rng: &mut UnifiedRandom` (the caller passes `app.rngs.global()`) +
  `snow_dust: i32` (d130's counter param) + d130's `SnowInputs`/metrics/ctx/
  world/pool. Retire the `Snow` struct's private splitmix (`new`/`next_u64`/
  `rand01`/`next_int`) and the `App.snow` field (`app/mod.rs:890`,
  `flow.rs:2427`, `app/mod.rs:3550` on d130). Rationale: `:17238` sits before
  every `SwapRandom` — the bare `Main.rand` is the port's `global` stream
  (S1-A-10 one-stream-per-pass; the pre-swap chest-eat cues at
  `frame/world.rs:580` already ride it). The WEATHER-2 per-consumer convention
  is a documented deviation for the rain rig's own consumers; snowing is not
  one of them — its draws must be visible to the global stream's other
  consumers. (d130 §2 explicitly leaves this slot decision to the merger;
  this addendum recommends `global`.)
* **F2 — `MAX_SCREEN_W = 4096.0`** (g11 `main_dust.rs:54-56`, the HiDef pin
  `Main.cs:3709` + `:67047`) replaces d130's 1920 Reach pin (`snow.rs:42-43`).
  Re-derive the budget expectations in every snow witness under 4096.
* **F3 — restore the in-body `gamePaused` gate** (`:13040`): add
  `game_paused: bool` to `SnowInputs` (g11's `SnowingEnv` has it) and the
  early return. d130's "caller's pause gate" fold is unverified — the fused
  `scene_scan::update_scene` runs per-frame (`frame/world.rs:596`) and its
  pause gating is not established; the C# gates in-body before any draw.
* **F4 — the budget reads the counter, once per call** (d130's shape):
  `snow_dust` param, NOT g11's per-attempt `live_type(pool,76)` census. g11's
  census stand-in retires; its budget-break witness re-pins to the counter.
* **F5 — the int attempt-loop shape** (g11's `let mut i = 0; while (i as f32)
  < num3 { i += 1; … }`, the C# `for (int i…)`): optional, observation-equal
  to d130's float loop; take g11's for shape fidelity.
* **F6 — env sourcing stays d130's caller** (`wind.remix_world`,
  `rain.cloud_alpha()` — the raw unclamped intensity, identical to g11's
  `intensity_value()`; the live-Deerclops `hp > 0 && kind == Deerclops` scan
  is identical in both). g11's `scene_scan.rs:407-434` `SnowingEnv` block
  re-points at `crate::snow::SnowInputs` (+ `game_paused: app.game_paused`).
* **F7 — witness union.** g11's four snow witnesses port over re-pinned:
  `w4_g11_snowing_gates_before_any_draw` (add the paused case, F3),
  `w4_g11_snowing_mirrors_the_dry_arm_draw_sequence` and
  `…_the_storm_arm_with_deerclops` (re-derive the draw sequences on the
  `global` `UnifiedRandom` stream, F1), `…_breaks_on_the_live_76_budget`
  (counter input, F4; the `25e180d4` cap comment re-derives under F2 — at
  cloudAlpha 0 the 1280-px fixture gives `num2 = 156`, Deerclops `156/3 = 52`,
  cap `52*0.6 = 31.2` — unchanged, g11 already pinned 4096). d130's snow
  witnesses survive except budget-count expectations (F2).

**Retire at merge (keep-one):** g11 `main_dust.rs::snowing` + `SnowingEnv` +
its `MAX_SCREEN_W`/`SNOW_TILE_MAX` consts (the teleport half of `main_dust.rs`
is OUTSIDE this reconcile — see `116-teleport-reconcile.md`); d54
`world_dust_main.rs::snowing` + its `SnowingEnv` (latent + the `!t.active()`
tile-gate defect); d130's `Snow` RNG struct (F1). **Keep:** d130's
`type_76_arm`/`type_76_tails`/`draw` + `DustPool.snow_dust` (field, head reset
`particles.rs:329`, the `minecart::tick` hook) — g11's branch does not have
the counter yet, so the d130 `particles.rs`/`minecart.rs` hunks land with the
union; g11's `rain.rs` `Snowflake` retirement + the debug row's
`live_type(pool,76)` + the `gfx_quality.rs` `snow_cap` retirement comments
(re-point at `crate::snow`). The `sand.rs:35`/`rain.rs:889` 1920 pins are
flagged as the same HiDef deviation family for their own lanes — no change
here.

The rain/splash halves of d130's doc (its §1 impact-gate fix, §3 g67 splash
union) are untouched by this addendum — g11 has no rain-gate or splash code.

## Verification

Code-only mode per the task brief: no cargo/tests/graphify. Every row above
was re-derived by reading `Main.cs:13032-13116`, `:17225-17248`,
`:3702-3719`, `:66708-66716`, `:67040-67048`, `Dust.cs:1743-1764`,
`:2069-2084`, `Tile.cs:240-247` (via d130's cited re-read) and diffing the
three port bodies (`git show 73545f0b`/`50c845e5`/`48343a4c`) line-by-line.
The folds are a merger worklist, WRITTEN NOT APPLIED.

## §-patch — F1-F7 as EXACT patch-hunks (union-exec phase, W4-g11)

**d130 owns execution on ITS branch** (`task/W4-d130-weather-ambience`,
tip `ec1d73c7` at this writing — its `snow.rs` is byte-identical to
`50c845e5`'s, verified `git diff 50c845e5 ec1d73c7 -- crates/tr-app/src/snow.rs`
empty). Every hunk below quotes the OLD text exactly as it stands at that
tip (file + line anchors) and the NEW text to replace it with, so d130
applies mechanically. Line numbers are 1-based at `ec1d73c7`. The g11
`main_dust.rs::snowing` + `SnowingEnv` retirement is the MERGER's drop
(keep-one, §2) — not a d130 hunk. Body indentation inside the retired
`impl Snow` wrapper stays as-is after F1c (valid Rust; the merge-verify
wave's rustfmt normalises — no rustfmt in the code-only phase).

### F1 — the body + stream (global `UnifiedRandom`, the `Snow` splitmix retires)

**F1a — `crates/tr-app/src/snow.rs:9-11`** (module doc, the RNG bullet):

OLD:
```rust
//! * the RNG is this module's own splitmix stream (the per-consumer-stream
//!   convention — vanilla rolls `Main.rand`); the pool's `NewDust` rolls ride
//!   the same stream via `SplitmixRand`, exactly like `sand.rs`;
```
NEW:
```rust
//! * the RNG is the bare `Main.rand` — the caller passes
//!   `app.rngs.global()` (`:17238` sits BEFORE every `SwapRandom`, the
//!   S1-A-10 one-stream-per-pass contract; research/83 F1). The pool's
//!   `NewDust` rolls ride the same `UnifiedRandom`;
```

**F1b — `snow.rs:33`** (the use block; add the `UnifiedRandom` import):

OLD:
```rust
use tr_core::World;
```
NEW:
```rust
use tr_core::{UnifiedRandom, World};
```

**F1c — `snow.rs:71-113`** (retire the `Snow` struct + its splitmix; the
free-fn signature). OLD (the whole span, struct through the old signature):
```rust
pub(crate) struct Snow {
    rng: u64,
}

impl Snow {
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            rng: (seed ^ 0x2545_F491_4F6C_DD1D).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn rand01(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    /// `Main.rand.Next(max)` — `Next(0)` rolls 0 (XNA), the sand.rs convention.
    fn next_int(&mut self, max: u32) -> u32 {
        (self.next_u64() % max.max(1) as u64) as u32
    }

    /// `Main.snowing()` (`Main.cs:13032-13116`). `metrics` is the stored
    /// `Main.SceneMetrics` (the last scan), `snow_dust` the live type-76
    /// population the previous `UpdateDust` head counted (`Main.snowDust`),
    /// `ctx` the pool's spawn statics. The `gamePaused` early-out
    /// (`:13040`) folds into the caller's pause gate; `netMode != 2`
    /// (`:17234`) is the single-player fold.
    pub(crate) fn snowing(
        &mut self,
        metrics: &SceneMetrics,
        i: &SnowInputs,
        snow_dust: i32,
        ctx: &SpawnCtx,
        world: &World,
        pool: &mut DustPool,
    ) {
```
NEW:
```rust
/// `Main.snowing()` (`Main.cs:13032-13116`). `rng` is the bare
/// `Main.rand` (the caller passes `app.rngs.global()` — `:17238` pre-swap,
/// research/83 F1); `metrics` is the stored `Main.SceneMetrics` (the last
/// scan), `snow_dust` the live type-76 population the previous
/// `UpdateDust` head counted (`Main.snowDust`, read ONCE per call — F4),
/// `ctx` the pool's spawn statics. The `gamePaused` early-out (`:13040`)
/// is IN-BODY (F3); `netMode != 2` (`:17234`) is the single-player fold.
pub(crate) fn snowing(
    rng: &mut UnifiedRandom,
    metrics: &SceneMetrics,
    i: &SnowInputs,
    snow_dust: i32,
    ctx: &SpawnCtx,
    world: &World,
    pool: &mut DustPool,
) {
```

**F1c-tail — `snow.rs:239-241`** (drop the retired `impl`'s closing brace;
the loop's `}` and the fn's `}` stay). OLD:
```rust
        }
    }
}
```
NEW:
```rust
        }
}
```
(The body between F1c and this tail keeps its current indentation —
mechanical; rustfmt at merge-verify normalises.)

**F1d — the draw sites inside the body** (each OLD → NEW, exact):

`snow.rs:149`:
```rust
            if !(self.rand01() < num) {
```
→
```rust
            if !(rng.next_float() < num) {
```

`snow.rs:154-155`:
```rust
            let mut num4 = self.next_int(i.scaled_size.x as u32 + 1500) as i32 - 750;
            let mut num5 = i.scaled_position.y as i32 - self.next_int(50) as i32;
```
→
```rust
            let mut num4 = rng.next_max(i.scaled_size.x as i32 + 1500) - 750;
            let mut num5 = i.scaled_position.y as i32 - rng.next_max(50);
```

`snow.rs:162-166`:
```rust
            if self.next_int(5) == 0 {
                num4 = self.next_int(500) as i32 - 500;
            } else if self.next_int(5) == 0 {
                num4 = self.next_int(500) as i32 + i.scaled_size.x as i32;
            }
```
→
```rust
            if rng.next_max(5) == 0 {
                num4 = rng.next_max(500) - 500;
            } else if rng.next_max(5) == 0 {
                num4 = rng.next_max(500) + i.scaled_size.x as i32;
            }
```

`snow.rs:169-170`:
```rust
                num5 += self.next_int((i.scaled_size.y as f64 * 0.8) as u32) as i32
                    + (i.scaled_size.y as f64 * 0.1) as i32;
```
→
```rust
                num5 += rng.next_max((i.scaled_size.y as f64 * 0.8) as i32)
                    + (i.scaled_size.y as f64 * 0.1) as i32;
```

`snow.rs:189`:
```rust
                &mut crate::app::SplitmixRand(&mut self.rng),
```
→
```rust
                rng,
```

`snow.rs:205`:
```rust
            d.velocity.y = 3.0 + self.next_int(30) as f32 * 0.1;
```
→
```rust
            d.velocity.y = 3.0 + rng.next_max(30) as f32 * 0.1;
```

`snow.rs:210-211` (the dry drift takes the C# `Next(-10,10)` shape, not
the `Next(20)-10` fold — same values, the g11/C# form):
```rust
                d.velocity.x = i.wind_speed_current
                    + (self.next_int(20) as i32 - 10) as f32 * 0.1;
```
→
```rust
                d.velocity.x = i.wind_speed_current
                    + rng.next_range(-10, 10) as f32 * 0.1;
```

`snow.rs:229`:
```rust
                    + self.rand01() * 0.2
```
→
```rust
                    + rng.next_float() * 0.2
```

**F1e — the `App.snow` field retires** (three sites):

`crates/tr-app/src/app/mod.rs:889-890` — OLD:
```rust
    /// The ambient snowfall (`Main.snowing()`, W4-d130).
    pub(crate) snow: crate::snow::Snow,
```
NEW: (delete both lines)

`crates/tr-app/src/app/mod.rs:3550` — OLD:
```rust
            snow: crate::snow::Snow::new(1),
```
NEW: (delete the line)

`crates/tr-app/src/app/flow.rs:2427` — OLD:
```rust
            snow: crate::snow::Snow::new(1),
```
NEW: (delete the line)

### F2 — `MAX_SCREEN_W = 4096.0` (the HiDef pin)

`snow.rs:15-18` (module doc) — OLD:
```rust
//! * `Main.maxScreenW` is pinned at its Reach value 1920
//!   (`Main.cs:66715`/`:3715` — the port's `MaxWorldViewSize.X` convention,
//!   `rain.rs`'s spawn rate uses the same pin; HiDef's 4096 arm is a
//!   documented drop);
```
NEW:
```rust
//! * `Main.maxScreenW` is pinned at its HiDef value 4096
//!   (`Main.cs:3709` + `:67047` — the non-XNA default selects HiDef; the
//!   port renders HiDef, `frame/draw.rs:65` / `ambience.rs:184-187`;
//!   research/83 F2 — the `sand.rs`/`rain.rs` 1920 pins are their own
//!   lanes' recorded deviation family);
```

`snow.rs:42-43` — OLD:
```rust
/// `Main.maxScreenW` — the Reach pin (see the module doc).
const MAX_SCREEN_W: f32 = 1920.0;
```
NEW:
```rust
/// `Main.maxScreenW` under the HiDef profile (`Main.cs:3709` — the port
/// renders HiDef; research/83 F2).
const MAX_SCREEN_W: f32 = 4096.0;
```

### F3 — the in-body `gamePaused` gate

`snow.rs:67-68` (the `SnowInputs` tail) — OLD:
```rust
    /// `Main.remixWorld` (`:13034-13037`).
    pub remix_world: bool,
```
NEW:
```rust
    /// `Main.remixWorld` (`:13034-13037`).
    pub remix_world: bool,
    /// `Main.gamePaused` (`:13040`) — the IN-BODY early return (F3: the
    /// fused per-frame `scene_scan::update_scene` runs paused; the C#
    /// gates before any draw).
    pub game_paused: bool,
```

`snow.rs:118-121` — OLD:
```rust
        // `:13040-13043`: no snow tiles / below the surface — nothing.
        if metrics.snow_tile_count <= 0 || !metrics.surface_atmospherics {
            return;
        }
```
NEW:
```rust
        // `:13040-13043`: paused / no snow tiles / below the surface —
        // nothing (the pause gate is IN-BODY, research/83 F3).
        if i.game_paused || metrics.snow_tile_count <= 0 || !metrics.surface_atmospherics {
            return;
        }
```

### F4 — the budget reads the counter ONCE per call

**No body hunk** — d130's `snow_dust: i32` param + the constant-within-
call gate (`snow.rs:143-147`) IS the F4 shape; it survives verbatim. The
g11 per-attempt `live_type(pool,76)` census retires with the g11 body at
merge (the merger's drop). The F7 budget witnesses re-pin to the param.

### F5 — the int attempt-loop shape

`snow.rs:138-140` — OLD:
```rust
        let mut attempt = 0.0_f32;
        while attempt < num3 {
            attempt += 1.0;
```
NEW:
```rust
        let mut attempt = 0i32;
        while (attempt as f32) < num3 {
            attempt += 1;
```
(The C# `for (int i = 0; (float)i < num3; i++)` shape; the counter keeps
the `attempt` name — `i` is the `SnowInputs` param. Observation-equal,
exact ≤ 2^24.)

### F6 — env sourcing stays d130's caller (+ the pause field, the free call)

`crates/tr-app/src/world_sim/scene_scan.rs:399-429` — OLD:
```rust
    // `Main.snowing()` (`Main.cs:17238`, body `:13032-13116`): the ambient
    // snowfall spawns right BEFORE `Sandstorm.EmitDust` in the C# — the port
    // runs both in this slot (the spawn order between the two dust families
    // only matters for the shared pool's slot scan; the snow's own stream is
    // separate). The budget reads the previous tick's `Main.snowDust`
    // (`pool.snow_dust`, the shared tick's count). W4-d130.
    let snow_inputs = crate::snow::SnowInputs {
        scaled_position: *camera - screen * 0.5 / *zoom,
        scaled_size: screen / *zoom,
        cloud_alpha: rain.cloud_alpha(),
        raining: rain.is_raining(),
        wind_speed_current: wind.speed_current,
        player_vy,
        gfx_quality: app.gfx_quality.quality,
        // `NPC.IsADeerclopsNearScreen()` (`NPC.cs:44534-44537`): the active
        // 668 scan (`npcsFoundForCheckActive[668]`) — the destructure's npcs.
        deerclops_near: npcs
            .iter()
            .any(|n| n.hp > 0 && n.kind == crate::npc::NpcKind::Deerclops),
        remix_world: wind.remix_world,
    };
    let snow_dust = app.dust.snow_dust;
    let snow_metrics = app.scene_metrics.clone();
    app.snow.snowing(
        &snow_metrics,
        &snow_inputs,
        snow_dust,
        &ctx,
        world,
        &mut app.dust,
    );
```
NEW:
```rust
    // `Main.snowing()` (`Main.cs:17238`, body `:13032-13116`): the ambient
    // snowfall spawns right BEFORE `Sandstorm.EmitDust` in the C# — the port
    // runs both in this slot. The stream is the BARE `Main.rand` (the port's
    // `NamedRngs::global()` — `:17238` sits before every `SwapRandom`,
    // research/83 F1; the pre-swap chest-eat cues already ride it). The
    // budget reads the previous tick's `Main.snowDust` (`pool.snow_dust`,
    // the shared tick's count, ONCE per call — F4). W4-d130 + g11 F1-F6.
    let snow_inputs = crate::snow::SnowInputs {
        scaled_position: *camera - screen * 0.5 / *zoom,
        scaled_size: screen / *zoom,
        cloud_alpha: rain.cloud_alpha(),
        raining: rain.is_raining(),
        wind_speed_current: wind.speed_current,
        player_vy,
        gfx_quality: app.gfx_quality.quality,
        // `NPC.IsADeerclopsNearScreen()` (`NPC.cs:44534-44537`): the active
        // 668 scan (`npcsFoundForCheckActive[668]`) — the destructure's npcs.
        deerclops_near: npcs
            .iter()
            .any(|n| n.hp > 0 && n.kind == crate::npc::NpcKind::Deerclops),
        remix_world: wind.remix_world,
        // F3: the in-body pause gate reads the live flag.
        game_paused: app.game_paused,
    };
    let snow_dust = app.dust.snow_dust;
    let snow_metrics = app.scene_metrics.clone();
    crate::snow::snowing(
        app.rngs.global(),
        &snow_metrics,
        &snow_inputs,
        snow_dust,
        &ctx,
        world,
        &mut app.dust,
    );
```
(The g11 `scene_scan.rs:407-434` `SnowingEnv` block is the MERGER's drop —
keep-one takes this d130 block; the env values are identical per §1:
`rain.cloud_alpha()` = g11's `intensity_value()` raw intensity, the
Deerclops scan and the view-rect fold match.)

### F7 — the witness union (re-pins + the four g11 mirrors)

**F7a — `snow.rs:386-398`** (`inputs()` gains the pause field) — OLD:
```rust
    fn inputs() -> SnowInputs {
        SnowInputs {
            scaled_position: CAMERA - SCREEN * 0.5,
            scaled_size: SCREEN,
            cloud_alpha: 0.0,
            raining: false,
            wind_speed_current: 0.0,
            player_vy: 0.0,
            gfx_quality: 1.0,
            deerclops_near: false,
            remix_world: false,
        }
    }
```
NEW:
```rust
    fn inputs() -> SnowInputs {
        SnowInputs {
            scaled_position: CAMERA - SCREEN * 0.5,
            scaled_size: SCREEN,
            cloud_alpha: 0.0,
            raining: false,
            wind_speed_current: 0.0,
            player_vy: 0.0,
            gfx_quality: 1.0,
            deerclops_near: false,
            remix_world: false,
            game_paused: false,
        }
    }
```

**F7b — the mechanical call-site re-shape** (every remaining witness;
`Snow::new(N)` → a pinned `UnifiedRandom`, `x.snowing(A, …)` →
`snowing(&mut x, A, …)`). The exact pairs, by line at `ec1d73c7`:

| line | OLD | NEW |
| --- | --- | --- |
| 434 | `let mut dry = Snow::new(11);` | `let mut dry = UnifiedRandom::new(11);` |
| 438 | `dry.snowing(&m, &i, 0, &ctx(), &w, &mut dry_pool);` | `snowing(&mut dry, &m, &i, 0, &ctx(), &w, &mut dry_pool);` |
| 440 | `let mut storm = Snow::new(11);` | `let mut storm = UnifiedRandom::new(11);` |
| 445 | `storm.snowing(&m, &i, 0, &ctx(), &w, &mut storm_pool);` | `snowing(&mut storm, &m, &i, 0, &ctx(), &w, &mut storm_pool);` |
| 489 | `let mut near = Snow::new(17);` | `let mut near = UnifiedRandom::new(17);` |
| 494 | `near.snowing(&m, &i, 0, &ctx(), &w, &mut near_pool);` | `snowing(&mut near, &m, &i, 0, &ctx(), &w, &mut near_pool);` |
| 496 | `let mut far = Snow::new(17);` | `let mut far = UnifiedRandom::new(17);` |
| 500 | `far.snowing(&m, &i, 0, &ctx(), &w, &mut far_pool);` | `snowing(&mut far, &m, &i, 0, &ctx(), &w, &mut far_pool);` |
| 527 | `let mut dry = Snow::new(19);` | `let mut dry = UnifiedRandom::new(19);` |
| 530 | `dry.snowing(&m, &i, 0, &ctx(), &w, &mut dry_pool);` | `snowing(&mut dry, &m, &i, 0, &ctx(), &w, &mut dry_pool);` |
| 533 | `let mut storm = Snow::new(19);` | `let mut storm = UnifiedRandom::new(19);` |
| 536 | `storm.snowing(&m, &i, 0, &ctx(), &w, &mut storm_pool);` | `snowing(&mut storm, &m, &i, 0, &ctx(), &w, &mut storm_pool);` |
| 572 | `let mut s = Snow::new(23);` | `let mut s = UnifiedRandom::new(23);` |
| 575 | `s.snowing(&m, &i, 0, &ctx(), &w, &mut pool);` | `snowing(&mut s, &m, &i, 0, &ctx(), &w, &mut pool);` |
| 639 | `let mut s = Snow::new(29);` | `let mut s = UnifiedRandom::new(29);` |
| 642 | `s.snowing(&m, &i, 0, &ctx(), &w, &mut pool);` | `snowing(&mut s, &m, &i, 0, &ctx(), &w, &mut pool);` |

(The gates witness at 411-425 and the budget witness at 456-480 are
replaced WHOLESALE by F7d/F7c below — their `Snow::new` sites are not in
this table. The statistical expectations of the intensity/velocity/
determinism/recipe witnesses are stream-family-agnostic — they survive
the splitmix→UnifiedRandom swap unchanged.)

**F7c — `snow.rs:456-480`** (the budget witness re-derives under F2's
4096: `(int)(500*1600/4096) = 195`; quality-1 cap `195*1.1 = 214.5`;
quality-0.5 cap `195*0.75 + 19.5 = 165.75`) — OLD:
```rust
    #[test]
    fn the_budget_gate_breaks_the_loop() {
        // `Main.cs:13057`: at quality 1 the budget is num2 * 1.1; the dry
        // 1600-px num2 is `(int)(500*1600/1920) = 416` → 457.6. A snowDust
        // already at 458 spawns NOTHING (the break, not a skip).
        let w = snow_world();
        let m = metrics(6000, true); // intensity 1: every attempt passes
        let mut s = Snow::new(13);
        let mut pool = DustPool::new();
        let i = inputs();
        s.snowing(&m, &i, 458, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the budget break");
        // One under the budget the loop runs (and the pool fills).
        for _ in 0..30 {
            s.snowing(&m, &i, 457, &ctx(), &w, &mut pool);
        }
        assert!(live_type(&pool, 76) > 0);
        // The gfxQuality scale: at 0.5 the budget is 416*0.75 + 41.6 = 353.6.
        let mut q = Snow::new(13);
        let mut q_pool = DustPool::new();
        let mut qi = inputs();
        qi.gfx_quality = 0.5;
        q.snowing(&m, &qi, 354, &ctx(), &w, &mut q_pool);
        assert_eq!(live_type(&q_pool, 76), 0, "the quality-scaled budget");
    }
```
NEW:
```rust
    #[test]
    fn the_budget_gate_breaks_the_loop() {
        // `Main.cs:13057`: at quality 1 the budget is num2 * 1.1; the dry
        // 1600-px num2 under the HiDef 4096 pin (F2) is
        // `(int)(500*1600/4096) = 195` → 214.5. A snowDust already at 215
        // spawns NOTHING (the break, not a skip; F4 — the param is read
        // once per call).
        let w = snow_world();
        let m = metrics(6000, true); // intensity 1: every attempt passes
        let mut s = UnifiedRandom::new(13);
        let mut pool = DustPool::new();
        let i = inputs();
        snowing(&mut s, &m, &i, 215, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the budget break");
        assert_eq!(s, UnifiedRandom::new(13), "the break precedes any draw");
        // One under the budget the loop runs (and the pool fills).
        for _ in 0..30 {
            snowing(&mut s, &m, &i, 214, &ctx(), &w, &mut pool);
        }
        assert!(live_type(&pool, 76) > 0);
        // The gfxQuality scale: at 0.5 the budget is 195*0.75 + 19.5 = 165.75.
        let mut q = UnifiedRandom::new(13);
        let mut q_pool = DustPool::new();
        let mut qi = inputs();
        qi.gfx_quality = 0.5;
        snowing(&mut q, &m, &qi, 166, &ctx(), &w, &mut q_pool);
        assert_eq!(live_type(&q_pool, 76), 0, "the quality-scaled budget");
    }
```

`snow.rs:484-486` (the Deerclops comment re-derives) — OLD:
```rust
        // `Main.cs:13047-13052`: num /= 20, num2 /= 3 — at count 6000 the
        // dry intensity 1 → 0.05 and the budget 457.6 → 152*1.1 = 167.2.
        // The per-flake scale also drops 0.5 (`:13104-13107`).
```
NEW:
```rust
        // `Main.cs:13047-13052`: num /= 20, num2 /= 3 — at count 6000 the
        // dry intensity 1 → 0.05 and the budget 214.5 → 65*1.1 = 71.5
        // (the 4096 pin, F2). The per-flake scale also drops 0.5
        // (`:13104-13107`).
```

**F7d — `snow.rs:411-425`** (the gates witness gains the F3 pause case +
the zero-draw stream pin — g11 `w4_g11_snowing_gates_before_any_draw`
folded in) — OLD:
```rust
    #[test]
    fn the_gates_need_snow_tiles_at_the_surface_and_no_remix() {
        // `Main.cs:13034-13043`: the three early returns.
        let w = snow_world();
        let mut s = Snow::new(7);
        let mut pool = DustPool::new();
        let mut i = inputs();
        s.snowing(&metrics(0, true), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "no snow tiles");
        s.snowing(&metrics(3000, false), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "below the surface");
        i.remix_world = true;
        s.snowing(&metrics(3000, true), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the remix world");
    }
```
NEW:
```rust
    #[test]
    fn the_gates_need_snow_tiles_at_the_surface_and_no_remix() {
        // `Main.cs:13034-13043`: the FOUR early returns (F3 adds pause);
        // none draws (the g11 gates witness folded in, F7).
        let w = snow_world();
        let mut s = UnifiedRandom::new(7);
        let mut pool = DustPool::new();
        let mut i = inputs();
        snowing(&mut s, &metrics(0, true), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "no snow tiles");
        snowing(&mut s, &metrics(3000, false), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "below the surface");
        i.remix_world = true;
        snowing(&mut s, &metrics(3000, true), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the remix world");
        i.remix_world = false;
        i.game_paused = true;
        snowing(&mut s, &metrics(3000, true), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the pause gate (F3)");
        assert_eq!(s, UnifiedRandom::new(7), "no gate draws");
    }
```

**F7e — the three g11 draw-mirror witnesses port over** (INSERT before the
test module's closing brace, after `the_spawn_recipe_matches_the_csharp_order`).
Fixture constants re-derived: view top-left `CAMERA - SCREEN*0.5 =
(800, 1070)`, size 1600×900; the `NewDust` cull rect (`Dust.cs:97-101`,
dCount 0 → margin 400) is `(400, 670, 2400, 1700)`, XNA `Intersects`
strict `<`; the snow rows 95+ are ACTIVE (the tile gate refuses
`num5/16 >= 95`); every rolled x lands in-world (base `[50, 3149]`, the
edge pushes `[300, 799]`/`[2400, 2899]` — all < 3200). NEW (whole):
```rust
    /// g11 `w4_g11_snowing_mirrors_the_dry_arm_draw_sequence` (F7,
    /// re-pinned to the d130 fixture + the global UnifiedRandom stream):
    /// cloudAlpha 0 → ONE attempt, intensity 1 → the gate always passes;
    /// the mirror walks every draw in C# order and pins the flake —
    /// a culled spawn draws NO internals but the recipe rolls still ride
    /// the stream (the sentinel takes the writes, `Dust.cs:79`).
    #[test]
    fn the_dry_arm_mirrors_the_draw_sequence() {
        let w = snow_world();
        let m = metrics(6000, true);
        let mut i = inputs();
        i.player_vy = 5.0;
        i.wind_speed_current = 0.3;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(0x5EED);
        snowing(&mut rng, &m, &i, 0, &ctx(), &w, &mut pool);
        let mut mirror = UnifiedRandom::new(0x5EED);
        let _ = mirror.next_float(); // the spawn gate (`:13061`)
        let mut num4 = mirror.next_max(1600 + 1500) - 750;
        let mut num5 = 1070 - mirror.next_max(50);
        num5 -= 5; // player_vy > 0 (`:13067-13070`)
        if mirror.next_max(5) == 0 {
            num4 = mirror.next_max(500) - 500;
        } else if mirror.next_max(5) == 0 {
            num4 = mirror.next_max(500) + 1600;
        }
        if num4 < 0 || num4 as f32 > 1600.0 {
            num5 += mirror.next_max((900.0f64 * 0.8) as i32) + (900.0f64 * 0.1) as i32;
        }
        num4 += 800;
        let open = (num5 / 16) < 95; // rows 95+ are active snow
        let passes =
            400 < num4 + 10 && num4 < 400 + 2400 && 670 < num5 + 10 && num5 < 670 + 1700;
        let mut v = Vec2::ZERO;
        let mut s = 0.0;
        if open {
            if passes {
                // The six default-block draws (`Dust.cs:153-193`, 10×10).
                let px = num4 as f32 + mirror.next_max(6) as f32 + 4.0;
                let py = num5 as f32 + mirror.next_max(6) as f32 + 4.0;
                let vx = mirror.next_range(-20, 21) as f32 * 0.1;
                let vy = mirror.next_range(-20, 21) as f32 * 0.1;
                let _ = mirror.next_max(3);
                let scale = 1.0 + mirror.next_range(-20, 21) as f32 * 0.01;
                let d = pool.slots().iter().find(|d| d.active).expect("the flake");
                assert_eq!(d.dust_type, 76);
                assert_eq!(d.position, Vec2::new(px, py));
                v = Vec2::new(vx, vy);
                s = scale;
            } else {
                assert_eq!(live_type(&pool, 76), 0, "the cull refuses the spawn");
            }
            // `:13089-13095` + the shared tail at cloudAlpha 0.
            v.y = 3.0 + mirror.next_max(30) as f32 * 0.1;
            v.y *= s;
            v.x = 0.3 + mirror.next_range(-10, 10) as f32 * 0.1;
            v.x += 0.3 * 15.0;
            v.y *= 1.0;
            if passes {
                let d = pool.slots().iter().find(|d| d.active).unwrap();
                assert_eq!(d.velocity, v);
                assert_eq!(d.scale, s);
            } else {
                assert_eq!(
                    pool.slot(DUST_SENTINEL).velocity.x,
                    v.x,
                    "the sentinel took the writes"
                );
            }
        }
        assert_eq!(rng, mirror);
    }

    /// g11 `w4_g11_snowing_mirrors_the_storm_arm_with_deerclops` (F7):
    /// cloudAlpha 0.5 → 26 attempts, the 0.05 gate, the sqrt/sign storm
    /// drift, the double scale bump + the Deerclops −0.5 cut. F4: the
    /// budget reads the CONSTANT snow_dust param (0 here — num2 =
    /// (int)(500*1600/4096)*2/3 = 130, quality-1 cap 143, never reached
    /// in 26 attempts), NOT a growing live census.
    #[test]
    fn the_storm_arm_mirrors_with_deerclops() {
        let mut any_spawn = false;
        for seed in [99i32, 7, 0x5EED, 12345] {
            let w = snow_world();
            let m = metrics(6000, true);
            let mut i = inputs();
            i.cloud_alpha = 0.5;
            i.raining = true;
            i.wind_speed_current = -0.4;
            i.deerclops_near = true;
            i.player_vy = 5.0;
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            snowing(&mut rng, &m, &i, 0, &ctx(), &w, &mut pool);
            let mut mirror = UnifiedRandom::new(seed);
            let num = 1.0f32 / 20.0;
            let mut spawned = 0usize;
            let mut attempt = 0i32;
            while (attempt as f32) < 1.0 + 50.0 * 0.5 {
                attempt += 1;
                // F4: the constant param (0) — the gate never breaks here.
                if !(0.0f32 < 130.0 * 1.0 + 13.0) {
                    break;
                }
                if !(mirror.next_float() < num) {
                    continue;
                }
                let mut num4 = mirror.next_max(1600 + 1500) - 750;
                let mut num5 = 1070 - mirror.next_max(50) - 5;
                if mirror.next_max(5) == 0 {
                    num4 = mirror.next_max(500) - 500;
                } else if mirror.next_max(5) == 0 {
                    num4 = mirror.next_max(500) + 1600;
                }
                if num4 < 0 || num4 as f32 > 1600.0 {
                    num5 += mirror.next_max((900.0f64 * 0.8) as i32)
                        + (900.0f64 * 0.1) as i32;
                }
                num4 += 800;
                let open = (num5 / 16) < 95;
                let passes = open
                    && 400 < num4 + 10
                    && num4 < 400 + 2400
                    && 670 < num5 + 10
                    && num5 < 670 + 1700;
                let mut v = Vec2::ZERO;
                let mut s = 0.0;
                if open {
                    if passes {
                        let px = num4 as f32 + mirror.next_max(6) as f32 + 4.0;
                        let py = num5 as f32 + mirror.next_max(6) as f32 + 4.0;
                        let vx = mirror.next_range(-20, 21) as f32 * 0.1;
                        let vy = mirror.next_range(-20, 21) as f32 * 0.1;
                        let _ = mirror.next_max(3);
                        let scale = 1.0 + mirror.next_range(-20, 21) as f32 * 0.01;
                        let d = pool.slot(spawned);
                        assert!(d.active);
                        assert_eq!(d.dust_type, 76);
                        assert_eq!(d.position, Vec2::new(px, py));
                        v = Vec2::new(vx, vy);
                        s = scale;
                        spawned += 1;
                    }
                    s += 0.5 * 0.2;
                    v.y = 3.0 + mirror.next_max(30) as f32 * 0.1;
                    v.y *= s;
                    v.x = (0.4f64.sqrt() as f32) * -1.0 * (0.5 + 0.5) * 10.0
                        + mirror.next_float() * 0.2
                        - 0.1;
                    v.y *= 0.5;
                    v.y *= 1.0 + 0.3 * 0.5;
                    s += 0.5 * 0.2;
                    s -= 0.5; // the Deerclops cut
                    v *= 1.0 + 0.5 * 0.5;
                    if passes {
                        let d = pool.slot(spawned - 1);
                        assert_eq!(d.velocity, v);
                        assert_eq!(d.scale, s);
                    }
                }
            }
            assert_eq!(live_type(&pool, 76), spawned);
            assert_eq!(rng, mirror);
            any_spawn |= spawned > 0;
        }
        assert!(any_spawn, "the seed sweep must pass the 0.05 gate");
    }

    /// g11 `w4_g11_snowing_breaks_on_the_live_76_budget` (F7, re-pinned
    /// to the F4 counter input + the F2 4096 budget): Deerclops at
    /// cloudAlpha 0 → num2 = 195/3 = 65, quality-1 cap 65*1.1 = 71.5; a
    /// counter already at 72 spends nothing.
    #[test]
    fn the_budget_breaks_on_the_counter() {
        let w = snow_world();
        let m = metrics(6000, true);
        let mut i = inputs();
        i.deerclops_near = true;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(1);
        snowing(&mut rng, &m, &i, 72, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0);
        assert_eq!(rng, UnifiedRandom::new(1));
    }
```

### §-patch verification

Code-only (no cargo/tests/graphify per the lane directive). Every OLD
block was copied verbatim from `git show
origin/task/W4-d130-weather-ambience:crates/tr-app/src/snow.rs` (and
`scene_scan.rs`/`app/mod.rs`/`app/flow.rs`) at tip `ec1d73c7`; the NEW
blocks re-derive the g11 body (`73545f0b`, this branch's
`main_dust.rs:104-231` pre-union) against `Main.cs:13032-13116` and the
d130 fixture constants. The F7e mirrors' cull/tile-gate branches were
walked by hand for the fixture's roll ranges (no execution — WRITTEN NOT
RUN, the merge-verify wave executes).
