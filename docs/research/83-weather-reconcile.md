# 83 — Weather reconcile: the g11-side addendum (snowing keep-one)

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
