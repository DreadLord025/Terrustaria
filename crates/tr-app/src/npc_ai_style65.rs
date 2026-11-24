//! W4-d30 (aiStyle-65) — the butterfly family, `NPC.AI`'s
//! `else if (this.aiStyle == 65) { this.AI_065_Butterflies(); }` dispatch
//! (`NPC.cs:34249-34252`) + the whole `AI_065_Butterflies` method
//! W4-f18 (aiStyle-65) — the butterfly family, `NPC.AI`'s
//! `else if (this.aiStyle == 65)` dispatch into `AI_065_Butterflies`
//! (`NPC.cs:44896-45119`), ported 1:1 as a PURE velocity machine
//! (code-only mode: no compiler/tests were run on this branch; the
//! witness tests below are written, not executed).
//!
//! ## Census (every `this.aiStyle = 65;` row in `NPC.cs`)
//!
//! Four kinds, all `SetDefaults`-verified:
//! * **Butterfly 356** (`NPCID.Butterfly`, `ID/NPCID.cs:11389`; row
//!   `NPC.cs:13531-13544`) — 10×10, damage 0, defense 0, lifeMax 5,
//!   npcSlots 0.25, `noGravity`, catchItem 1994 (the per-tick colour
//!   re-derive below shifts it 1994..2001).
//! * **GoldButterfly 444** (`ID/NPCID.cs:11561`; row `:14275-14289`) —
//!   the same zero-stat 10×10 row, catchItem 2891, rarity 3.
//! * **HellButterfly 653** (`ID/NPCID.cs:11983`; row `:17250-17264`) —
//!   10×10, `lavaImmune`, catchItem 4845 (lava bait). The ember glow +
//!   the 1/60 dust tail are 653-only.
//! * **EmpressButterfly 661** (`ID/NPCID.cs:11999`; row `:17351-17366`)
//!   — 10×10, `lavaImmune`, catchItem 4961, rarity 4. The Prismatic
//!   Lacewing: the rainbow glow + the fade/despawn machine are 661-only;
//!   its death spawns the Empress of Light (the `DoDeathEvents` case 661,
//!   `NPC.cs:79493-79503` — the boss-spawn seam, NOT this arm).
//!
//! NOT the projectile aiStyle 65 — that is the Sharknado Bolt 385
//! (`Projectile.cs:29535+`, ported as `projectile.rs::ai_sharknado_bolt`,
//! W-9), a different dispatch.
//!
//! ## The machine (C# order)
//!
//! 1. `:44900-44955` — type 661 ONLY: the rainbow `AddLight(Center, …)`,
//!    the fade counter `ai[2]` (clamped 0..num4, num4 = 50, or 60 when
//!    the target is a live player OUTSIDE the Hallow on a non-remix
//!    world — that case also forces the rise), the `ai[2] >= 60`
//!    despawn EARLY RETURN, the `Opacity` fade
//!    (`GetLerpValue(60, num4/2, ai[2], clamped)` — 1 at ai2=0, 0 at
//!    ai2=num4/2), the 1/5-gated type-267 shimmer dust + its half-scale
//!    clone, and `dontTakeDamage = ai[2] >= num4/2`.
//! 2. `:44956-45014` — `Main.netMode != 1` folds true (single-player):
//!    the one-time colour roll `ai[2]` (non-661, `Next(100)` ladder),
//!    the one-time scale latch `ai[3]` (`Next(75,111)*0.01f`), the
//!    `localAI[0]--` rearm timer; on expiry `Next(90,240)` +
//!    `TargetClosest()` + the FAR (>700 px, `localAI[3]` unset) speed
//!    ladder / NEAR wander quad, `netUpdate = true`.
//! 3. `:45015-45018` — `scale = ai[3]`; the 1/60 blend toward the aim.
//! 4. `:45019-45057` — the tile bounces: falling → the 3-tile down-scan
//!    (solid OR liquid flips the aim and damps ×0.9 INSIDE the loop,
//!    once per blocking tile); rising → the 30-tile down-scan for ANY
//!    solid, and only when none is found the single flip + damp.
//! 5. `:45058-45088` — the threat flee on a 15-tick cooldown (661: 10):
//!    the slot-order scan of hostile NPCs within 100 px of the centre
//!    (`Hitbox.Distance`), the averaged `DirectionFrom` push ×2, the
//!    16-speed `SafeNormalize` cap.
//! 6. `:45089-45103` — the `collideX` bounce (aim away from the
//!    velocity sign, velocity ×-0.2), the two facing ifs, the aim
//!    write-back LAST.
//! 7. `:45104-45107` — type 356: `catchItem = (short)(1994f + ai[2] -
//!    1f)`. The port's `bug_net.rs::catch_item(356, ai2)` already
//!    re-derives this every catch (`bug_net.rs:139`), so the module
//!    writes no item field — the wiring reads `ai[2]` through that seam.
//! 8. `:45108-45118` — type 653: the `netOffset` add, the ember
//!    `AddLight((int)Center.X/16, (int)Center.Y/16, 0.6, 0.3, 0.1)`,
//!    the 1/60-gated type-6 dust (alpha 254, velocity zeroed), the
//!    `netOffset` subtract.
//!
//! ## RNG order (by construction)
//!
//! The arm's `Main.rand` draws, in per-tick C# order:
//! 1. 661, `ai[2] > 0` after the clamp (`:44929-44953`): `Next(5)`
//!    (`:44934`); on 0: the `Dust.NewDust` internals (`:44937`, the
//!    real `DustPool::new_dust` — pressure bands + the default block),
//!    `NextVector2Circular(width, height)` (`:44938` — the angle
//!    `NextFloat` then the radial `NextFloat`, `Utils.cs:1266-1269`;
//!    the C# passes the FULL box as the half-extents, kept),
//!    `NextFloat()` (`:44939`, the velocity scale), `NextFloat()`
//!    (`:44942`, the fadeIn), then — only when the slot is not the
//!    6000 sentinel — `Dust.CloneDust`'s own `NewDust` internals
//!    (`:44946`, `Dust.cs:263-286`).
//! 2. non-661, `ai[2] == 0` (`:44958-44970`): ONE `Next(100)` — the
//!    colour ladder (roll 0 → ai2 6, 1-2 → 2, 3-8 → 3, 9-18 → 8,
//!    19-33 → 4, 34-52 → 7, 53-74 → 5, 75-99 → 1).
//! 3. `ai[3] == 0` (`:44971-44974`): ONE `Next(75, 111)` — the scale
//!    latch, drawn every tick until latched.
//! 4. rearm expiry (`:44976-45013`): `Next(90, 240)` (the timer,
//!    AFTER the RNG-free `TargetClosest()`), then the FAR branch:
//!    `Next(50, 151)` / `Next(150, 201)` (>1000) / `Next(100, 151)`
//!    (>850), `Next(100, 251)` (direction-scaled X), `Next(-50, 51)`,
//!    the CONDITIONAL `Next(100, 251)` Y-bias (only when
//!    `position.Y > player.Y - 100`); or the NEAR branch:
//!    `Next(26, 301)`, `Next(-100, 101)` ×2.
//! 5. 653 tail (`:45112-45116`): ONE `Next(60)`; on 0 the
//!    `Dust.NewDust` internals (type 6, alpha 254 — the type-6 tweak
//!    chain adds its own `Next(-10, 6)`).
//! The 661 despawn tick (`ai[2] >= 60`) returns BEFORE 2-5 — zero
//! further draws. Everything else (the blend, the bounces, the threat
//! scan, the facing) is pure maths. `UnifiedRandom` is the exact
//! `Main.rand` stream; the witnesses mirror the cited draws on a
//! cloned stream (and a twin `DustPool` for the dust internals) and
//! compare state, so a drifted order fails by construction.
//!
//! ## Documented folds (all cited)
//!
//! * `Main.netMode != 1` → true (single-player): the rearm block
//!   always runs; the despawn's `NetMessage.SendData(23)` (`:44924`)
//!   is the net seam — returned as [`Ai65Effects::despawn`].
//! * `GetTargetData()` (`:44908`, `NPC.cs:6901-6912`): 661 is NOT in
//!   `NPCID.Sets.UsesNewTargeting` (`ID/NPCID.cs:10434`), so the data
//!   is the player target when `HasValidTarget` (`:6657-6671` —
//!   target in 0..255, player active, !dead, !ghost) and `Invalid`
//!   otherwise; `Type == Player` always holds on the valid path. The
//!   caller resolves `Main.player[target].ZoneHallow` into
//!   [`Ai65Ctx::target_zone_hallow`] (the port's live zone scan).
//! * `Lighting.AddLight` → the [`Ai65Effects::light`] request (the
//!   `npc_lights.rs`/style64 seam). The 661 head light rides the
//!   `Vector2` overload (`Lighting.cs:140-143` — `(int)(v/16f)` per
//!   axis, the quotient truncation); the 653 tail truncates the
//!   centre BEFORE the divide (`(int)Center.X/16`). Both computed
//!   here; the two are mutually exclusive (different types).
//! * `this.Opacity` (`:44928`) — `Npc` has no opacity field; the value
//!   is RETURNED ([`Ai65Effects::opacity`]) for the wiring's draw
//!   fade. `dontTakeDamage` IS a live field, written in place.
//! * `active = false` (`:44921`) — the port's death resolution is the
//!   sim's retain; the module returns `despawn` and writes nothing
//!   else on that tick (the C# returns before every later block).
//! * The dust chains run the REAL `DustPool::new_dust`/`clone_dust`
//!   (the style75/frost-boss-b precedent) on the caller's stream —
//!   the draws are IN the arm, so a wholesale skip would break the
//!   RNG census. `Main.hslToRgb` is the byte-quantised
//!   `tile_light::hsl_to_rgb`; the `Color * 0.5f` multiplies are XNA's
//!   byte truncation (`(byte)(255f * 0.5f) = 127`, alpha included).
//! * `Main.tileSolid[type]` is the RUNTIME table (the style64
//!   precedent): `TILE_SOLID` minus the living-tree leaves 192
//!   (`WorldGen.cs:15714`). Platforms (`tileSolidTop`) do not bounce;
//!   liquid does, down-scan only (`:45026`).
//! * The normalize divides are f64 `Math.Sqrt` over an INT sum cast
//!   to f32 BEFORE the divide (`:44998`/`:45008`); NO zero guard —
//!   the C# NaN/Inf pass-through is kept (the style64 trap).
//! * `Entity.DirectionFrom` (`Entity.cs:210-213`) is XNA
//!   `Vector2.Normalize(Center - Source)` — the zero-vector NaN is
//!   kept; the 16-cap uses `SafeNormalize(Vector2.Zero)`
//!   (`Utils.cs:1837-1844` — zero/NaN falls back to (0,0)).
//! * The threat scan reuses `npc_ai_style64::Ai64NpcView` +
//!   `rect_distance` UNMODIFIED (the same `Main.npc` slot view and
//!   `Utils.cs:1507-1542` rect-to-point distance; the butterfly's own
//!   `damage == 0` excludes it, so the caller may include or drop its
//!   own view — byte-identical).
//!
//! WIRED for the GoldButterfly 444 (W4-c10 roster tail): the style-axis
//! `65 =>` claim (`npc.rs::ai_065_butterfly`) runs this module, the sim
//! pre-stamps the slot-ordered threat views (`butterfly_threat_views`,
//! the c9 Shimmerfly convention) and the FindFrame case 444
//! (`NPC.cs:72739-72772` — the 7-tick flap cycle, `rotation = vx*0.3`;
//! the 356-only `ai[2]-1` colour-row offset never fires for 444) landed
//! in `npc.rs::find_frame`. The 356/653/661 trio stays the d39/d43
//! roster lanes (the 661 fade/despawn + dust arms keep their seams).
//! UNWIRED (the style66/67/68 precedent): none of the four kinds has
//! an `NpcKind` yet — the roster lanes (W4-d39/d40/d43) own them; the
//! style-axis `65 =>` claim + the FindFrame case 356/444/653/661
//! (`NPC.cs:72739-72772` — the 7-tick flap cycle, `rotation = vx*0.3`,
//! the 356-only `ai[2]-1` colour-row offset) land with that wave.
//!
//! See docs/ANALYSIS.md "The butterfly family (W4-d30 aiStyle-65,
//! code-only)".

#![allow(dead_code)] // the 356/653/661 arms keep their seams (the d39/d43 roster lanes)
#![allow(dead_code)] // UNWIRED — the kinds + axis claim are the roster wave (W4-d39/d40/d43)

use crate::npc::Npc;
use crate::npc_ai_style64::{rect_distance, Ai64NpcView};
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::particles::{Color, DustPool, SpawnCtx, DUST_SENTINEL};
use tr_core::scene::get_lerp_value;
//! S1-D-3ff d39 wave 1 (aiStyle-65) — the butterfly, `NPC.AI_065_Butterflies`
//! (`NPC.cs:44896-45119`), ported 1:1 for the type-356 branches as a PURE
//! velocity machine (code-only mode: no compiler/tests were run on this
//! branch; the witness tests below are written, not executed).
//!
//! Family identity (`NPC.SetDefaults`'s `this.aiStyle = 65;` rows):
//! * `NPCID.Butterfly` (356) — `ID/NPCID.cs:11389`, `NPC.cs:13531-13544`:
//!   10x10, damage 0, defense 0, lifeMax 5, npcSlots 0.25, `noGravity`,
//!   catchItem 1994 (+ai[2]-1 per tick, `:45106`).
//! * `NPCID.GoldButterfly` (444), `NPCID.HellButterfly` (653),
//!   `NPCID.EmpressButterfly` (661) — the other three members; NOT ported
//!   in this wave (their kinds land with their own rounds), so their
//!   branches are skipped here and cited as exclusions:
//!   - the 661 head block (`:44900-44955`: the hsl light, the fade-out
//!     despawn counter, the Opacity/dust/dontTakeDamage writes);
//!   - the 661 `localAI[1] = 10f` override (`:45065-45068`);
//!   - the 653 tail (`:45108-45118`: the ember light + the 1/60 dust 6);
//!   - the FindFrame `type != 444 && != 653 && != 661` colour-band gate
//!     (`:72768`) — for 356 the band ALWAYS applies (the frame wiring's
//!     obligation, `npc.rs::find_frame`).
//!
//! Seam: the module writes only the fields the C# arm writes — velocity,
//! `ai[0]`/`ai[1]`/`ai[2]`/`ai[3]`, `localAI[0]`/`localAI[1]`/`localAI[3]`,
//! `scale`, `direction` (`Npc::facing`) and `netUpdate`. It does NOT
//! integrate position: the arm ends before `UpdateCollision`. The wiring
//! rides `integrate_flyer_collision` (the noGravity flyer seam, the bat
//! family's precedent). `TargetClosest()` (`:44979`) rides
//! `npc_target::target_closest` UNMODIFIED.
//!
//! The `catchItem = (short)(1994f + ai[2] - 1f)` per-tick re-derive
//! (`:45104-45107`) has no `Npc` field — the port's catch path re-derives
//! it from `ai[2]` at catch time (`bug_net::catch_item`'s 356 row,
//! `1994 + ai2 as i32 - 1`), so the write is a cited no-op here.
//!
//! Documented folds (all cited):
//! * `Main.netMode != 1` → true (the port is single-player) — the rearm
//!   block (`:44956-45014`) always runs.
//! * The threat scan loops `Main.npc[0..maxNPCs]` (`:45071-45079`) in slot
//!   order; the port's NPC list is a compacted `Vec`, so the caller hands
//!   the arm slot-ordered [`Ai65NpcView`]s (the f32 sum order is part of
//!   the behaviour — the style64 threat-scan precedent). The butterfly
//!   itself is in the C# loop but its `damage == 0` excludes it
//!   (`:45074`); the caller may include it.
//! * `Entity.DirectionFrom` (`Entity.cs:210-213`) is XNA
//!   `Vector2.Normalize(Center - Source)` — the f32 sum, the f64 sqrt cast
//!   to f32, the reciprocal; a zero vector yields NaN (0 × ∞), which the
//!   threat sum then carries. Kept (the style64 module's identical fold).
//! * `Vector2.Length()` (`:45083`) is `(float)Math.Sqrt(X*X + Y*Y)`;
//!   `SafeNormalize(Vector2.Zero)` (`Utils.cs:1837-1844`) returns Zero on
//!   a zero/NaN vector, else `Normalize`. Both kept verbatim.
//! * The down-scan is THREE tiles (`num27 = 3`, `:45021`) — NOT the
//!   firefly's four (`AI_064`'s `num975 = 4`); the aim flip + the 0.9 damp
//!   sit INSIDE the loop (`:45026-45033`), one per blocking tile. The
//!   up-scan's 30-tile loop (`:45036-45057`) flips + damps ONCE after the
//!   scan, liquid does not count there.
//! * `Main.tileSolid[type]` is the RUNTIME table (the style64 module's
//!   `tile_solid_runtime` fold — `WorldGen.cs:15714` clears 192).
//! * The tile coordinates TRUNCATE: `(int)Center.X / 16`
//!   (`:45022-45023`/`:45040-45041`), NOT the `>>4` floor.
//!
//! RNG draws, in C# order (the caller's `Main.rand` / `UpdateNPCs`
//! stream): the variant roll `Next(100)` (`:44968`, only while
//! `ai[2] == 0`), the scale latch `Next(75, 111)` (`:44973`, only while
//! `ai[3] == 0`), then on the rearm expiry (`localAI[0] <= 0`): the timer
//! `Next(90, 240)` (`:44978`), the FAR ladder `Next(50, 151)` +
//! conditional `Next(150, 201)`/`Next(100, 151)` + `Next(100, 251)` +
//! `Next(-50, 51)` + conditional `Next(100, 251)`
//! (`:44983-44996`), or the NEAR `Next(26, 301)` + `Next(-100, 101)` ×2
//! (`:45005-45007`). Nothing else draws.
//! Family identity, verified from the C# `SetDefaults` census (the four
//! `this.aiStyle = 65;` rows in `NPC.cs`):
//!
//! * `NPCID.Butterfly` (356) — `ID/NPCID.cs:11389`, `NPC.cs:13531-13544`:
//!   10×10, damage 0, defense 0, lifeMax 5, npcSlots 0.25, `noGravity`,
//!   catchItem 1994 (re-derived per tick from the colour style `ai[2]`,
//!   `:45104-45107` — the port's `bug_net::catch_item` already reads the
//!   LIVE `ai[2]`, so no field write is owed here).
//! * `NPCID.GoldButterfly` (444) — `NPC.cs:14288-14302` (unported).
//! * `NPCID.HellButterfly` (653) — `NPC.cs:17251-17264` (unported; the
//!   `:45108-45118` orange-light + dust-6 tail is its own).
//! * `NPCID.FairyButterfly` (661) — `NPC.cs:17359-17375` (unported; the
//!   `:44900-44955` hsl-glow + fade-despawn head is its own).
//!
//! Only 356 has an `NpcKind` (W4-f18 first-20 port), so the 661 head and
//! the 653 tail are NOT ported — the fn is keyed on the vanilla type and
//! debug-asserts the ported member (the `npc_ai_style64` precedent).
//!
//! Seam: the machine writes only the fields the C# arm writes — velocity,
//! `ai[0..3]`, `localAI[0]/[1]/[3]`, `scale`, `direction` (`Npc::facing`)
//! and `netUpdate`. It does NOT integrate position: the arm ends before
//! `UpdateCollision` (`NPC.cs:92523`). The caller runs the no-gravity
//! tile-collision tail (`npc.rs::integrate_flyer_collision`).
//!
//! Documented folds (all cited):
//! * `Main.netMode != 1` → true (the port is single-player) — the colour
//!   roll / scale latch / rearm block (`:44956-45014`) always runs.
//! * The threat scan loops `Main.npc[0..maxNPCs]` (`:45071-45079`) in slot
//!   order; the caller hands the arm slot-ordered [`Ai65NpcView`]s (the
//!   f32 sum order is part of the behaviour). The butterfly itself is in
//!   the C# loop but its `damage == 0` excludes it (`:45074`). The SOLO
//!   `update_rng` path passes an EMPTY slice (the style64 677 wiring
//!   precedent — the sibling pool seam lands with the sim wiring; the
//!   flee kick then never fires, documented deviation).
//! * `Main.tileSolid[type]` is the RUNTIME table: `WorldGen.cs:15714`
//!   clears `[192]` (living-tree leaves) after generation (the
//!   `npc_ai_style64::tile_solid_runtime` precedent). Platforms are
//!   `tileSolidTop`, NOT `tileSolid`; liquid does block the down-scan
//!   (`:45026`'s `|| liquid > 0`), not the up-scan.
//! * The 356 `catchItem` re-derive (`:45104-45107`) is the catch path's
//!   (`bug_net::catch_item(356, ai[2])` reads the live style) — no write.
//!
//! ## Faithfulness traps kept
//!
//! * The colour roll (`:44968-44969`) is ONE `Next(100)` through the
//!   8-step ladder: 0→5, <3→1, <9→2, <19→7, <34→3, <53→6, >=75→0,
//!   else 4 — then `ai[2] = 1 + pick` (so the styles are 1..8, the
//!   Empress/Julia/Gold rows' order is NOT sequential).
//! * The rearm timer draw `Next(90, 240)` (`:44978`) comes BEFORE
//!   `TargetClosest()` (`:44979`) — the style64 arm draws it AFTER; the
//!   two families' orders differ and both are kept.
//! * The near-wander speed is `Next(26, 301)` (`:45005`) — NOT the
//!   firefly's `Next(5, 151)`.
//! * The normalize divides are f64 `Math.Sqrt` over an INT sum, cast to
//!   f32 BEFORE the divide (`:44998`/`:45008`); NO zero guard — two zero
//!   draws give Infinity/NaN and the C# passes them through. Kept.
//! * The down-scan's `velocity.Y *= 0.9f` sits INSIDE the 3-tile loop
//!   (`:45024-45034`) — up to three damps in one tick; the up-scan's
//!   single damp (`:45049-45056`) fires once after the 30-tile scan.
//! * The threat flee (`:45058-45088`) runs on a 15-tick `localAI[1]`
//!   cadence (the 661's 10 is unported): `velocity += sum/count * 2f`,
//!   then the 16-cap via `SafeNormalize(Vector2.Zero)` (`Utils.cs:
//!   1837-1844` — zero/NaN falls back to (0,0), so a NaN velocity
//!   zeroes). `Entity.DirectionFrom` (`Entity.cs:210-213`) is XNA
//!   `Vector2.Normalize(Center - Source)` — the f32 sum, the f64 sqrt
//!   cast to f32, the reciprocal, the multiply; a zero vector yields
//!   NaN, which the threat sum then carries. Kept.
//! * `Hitbox.Distance(Center)` is the rect-to-point distance
//!   (`Utils.cs:1507-1542`) — shared with the firefly arm through
//!   `npc_ai_style64::rect_distance` (the minion-aim seam, S1-P-i27).
//!
//! ## RNG by construction
//!
//! The arm's ONLY `Main.rand` draws for type 356, in C# order:
//! 1. the colour roll `Next(100)` (`:44968`) — once, when `ai[2] == 0`.
//! 2. the scale latch `Next(75, 111)` (`:44973`) — once, when
//!    `ai[3] == 0`, AFTER the colour roll.
//! 3. on rearm (`localAI[0]` counts to `<= 0`, `:44976`): `Next(90, 240)`
//!    (the timer, `:44978`) — BEFORE `TargetClosest()` (`:44979`, which
//!    draws nothing) — then the FAR branch (`> 700` px and
//!    `localAI[3] == 0`): the speed `Next(50, 151)` / `Next(150, 201)`
//!    (`> 1000`) / `Next(100, 151)` (`> 850`) (`:44983-44991`), then
//!    `Next(100, 251)` (the direction-scaled X, `:44992`), then
//!    `Next(-50, 51)` (`:44993`), then the CONDITIONAL `Next(100, 251)`
//!    Y-bias (`:44996`); or the NEAR branch: `Next(26, 301)` (`:45005`),
//!    `Next(-100, 101)` ×2 (`:45006-45007`).
//! Nothing else draws: the blend, the scans, the threat flee, the wall
//! bounce and the facing tail are pure maths. `UnifiedRandom`
//! (`tr_core::unified_random`) is the exact `Main.rand` stream.
#![allow(dead_code)] // the 444/653/661 members are unported (the census above)

use crate::npc::Npc;
use crate::npc_ai_style64::rect_distance;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::tiledata::TILE_SOLID;
use tr_core::unified_random::UnifiedRandom;
use tr_core::World;

/// `NPCID.Butterfly` (`ID/NPCID.cs:11389`; `SetDefaults` `NPC.cs:13531-13544`).
pub const BUTTERFLY: i32 = 356;
/// `NPCID.GoldButterfly` (`ID/NPCID.cs:11561`; `NPC.cs:14275-14289`).
pub const GOLD_BUTTERFLY: i32 = 444;
/// `NPCID.HellButterfly` (`ID/NPCID.cs:11983`; `NPC.cs:17250-17264`).
pub const HELL_BUTTERFLY: i32 = 653;
/// `NPCID.EmpressButterfly` (`ID/NPCID.cs:11999`; `NPC.cs:17351-17366`).
pub const EMPRESS_BUTTERFLY: i32 = 661;

/// The per-tick context the arm reads besides the NPC and the world.
pub struct Ai65Ctx<'a> {
    /// The `Main.player` slots — feeds `TargetClosest` (`:44979`) and
    /// the 661 target-data read (`:44908`). Non-empty (the C# always
    /// holds the 255 slots).
    pub players: &'a [TargetCandidate],
    /// `Main.npc[0..maxNPCs]` in slot order (the threat scan,
    /// `:45071-45079`) — the f32 sum accumulates in it.
    pub npcs: &'a [Ai64NpcView],
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp
    /// (`NPC.cs:77780`) — 0 until the ticker is plumbed.
    pub ticker: u64,
    /// `Main.GlobalTimeWrappedHourly` — the 661 rainbow hue
    /// (`:44902`/`:44937`).
    pub global_time_wrapped_hourly: f32,
    /// `Main.remixWorld` (`:44913`) — the world's REMIX seed flag.
    pub remix_world: bool,
    /// `Main.player[this.target].ZoneHallow` (`:44913`), resolved by
    /// the caller for the CURRENT target slot (the port's live zone
    /// scan; false when the target is invalid — the gate short-circuits
    /// on `Invalid` first, so the value is unread there).
    pub target_zone_hallow: bool,
}

/// The side effects the pure arm cannot apply itself.
#[derive(Default, Debug, PartialEq)]
pub struct Ai65Effects {
    /// The tick's `Lighting.AddLight` — `(tile_x, tile_y, [r, g, b])`.
    /// The 661 head light (`:44904`, the rainbow ×0.3 + 0.1) or the
    /// 653 tail light (`:45111`, the constant ember); never both (the
    /// types are disjoint). `None` for 356/444 and on the despawn tick.
    pub light: Option<(i32, i32, [f32; 3])>,
    /// The 661 despawn (`:44919-44927`): `ai[2] >= 60` — the C#
    /// `active = false` + `SendData(23)` + RETURN. The caller
    /// deactivates the slot; nothing else ran this tick.
    pub despawn: bool,
    /// `this.Opacity` (`:44928`) — the 661 draw fade, 1 at ai2=0 → 0
    /// at ai2=num4/2. `Npc` has no opacity field; 0.0 for non-661
    /// (the C# never writes it there — the stock 1 is the draw-side
    /// default; the wiring reads this only for 661).
    pub opacity: f32,
}

/// `Main.tileSolid[type]` — the RUNTIME table (the style64 precedent:
/// `WorldGen.cs:15714` clears the living-tree leaves 192 after gen).
/// `NPCID.Butterfly` (`ID/NPCID.cs:11389`; `NPC.cs:13531-13544`).
pub const BUTTERFLY: i32 = 356;

/// One `Main.npc[slot]` as the threat scan reads it (`:45071-45079`):
/// `active`, `damage > 0`, `!friendly`, the `Hitbox` (int-truncated
/// position, `Entity.cs:171-183`) and the `Center` (float halves,
/// `Entity.cs:50-60`). Slot order — the f32 sum accumulates in it.
/// The `npc_ai_style64::Ai64NpcView` shape, re-declared so the module
/// keeps its own seam vocabulary.
#[derive(Clone, Copy, Debug)]
pub struct Ai65NpcView {
    pub active: bool,
    pub damage: i32,
    pub friendly: bool,
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
}

impl Ai65NpcView {
    /// `Entity.Center` — `position + (float)width / 2f` (FLOAT halves).
    fn center(&self) -> (f32, f32) {
        (
            self.x + self.width as f32 / 2.0,
            self.y + self.height as f32 / 2.0,
        )
    }
}

/// The per-tick context the arm reads besides the NPC and the world.
pub struct Ai65Ctx<'a> {
    /// The `Main.player` slots, indexed by player slot — feeds
    /// `TargetClosest` (`:44979`) and the rearm's target-Center read
    /// (`:44980`/`:44994`). The C# arrays always hold 255 slots, so slot
    /// 0 exists even when inactive (the no-target fallback reads it).
    pub players: &'a [TargetCandidate],
    /// `Main.npc[0..maxNPCs]` in slot order (the threat scan,
    /// `:45071-45079`). Empty on a solo tick — the scan finds nothing.
    pub npcs: &'a [Ai65NpcView],
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp — 0 until
    /// the ticker is plumbed (the S1-D-3p precedent).
    pub ticker: u64,
}

/// `WorldGen.InWorld(x, y, 2)` (`WorldGen.cs:9688-9695`, fluff 2) — the
/// style64 module's identical fold.
fn in_world2(world: &World, x: i32, y: i32) -> bool {
    x >= 2 && x < world.width - 2 && y >= 2 && y < world.height - 2
}

/// The arm's `Main.tileSolid[type]` read — the RUNTIME table (the
/// style64 module's fold: the generated `TILE_SOLID` minus the
/// living-tree leaves 192 that `WorldGen.cs:15714` clears).
    /// The `Main.player` slots — feeds `TargetClosest` (`:44979`, the
    /// `npc_target.rs` seam) and the rearm's `Main.player[target]` reads.
    pub players: &'a [TargetCandidate],
    /// `Main.npc[0..maxNPCs]` in slot order (the threat scan). Empty on
    /// the solo path (the module doc's documented deviation).
    pub npcs: &'a [Ai65NpcView],
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp
    /// (`NPC.cs:77780`) — 0 until the ticker is plumbed (S1-D-3p).
    pub ticker: u64,

/// The arm's `Main.tileSolid[type]` read — the RUNTIME table (the
/// `npc_ai_style64::tile_solid_runtime` precedent, `WorldGen.cs:15714`).
fn tile_solid_runtime(kind: u16) -> bool {
    TILE_SOLID.has(kind) && kind != 192
}

/// `Tile.nactive()` (`Tile.cs:240-247`): active AND not actuated-off.
fn nactive(t: &tr_core::tile::Tile) -> bool {
    t.active() && !t.inactive()
}

/// XNA `Vector2.Normalize` — the f32 sum, the f64 sqrt cast to f32,
/// the reciprocal, the two multiplies. A zero vector yields NaN
/// (0 × ∞); NO guard (the C# has none). The style64 twin, kept local
/// so this module does not edit that lane's file.
/// XNA `Vector2.Normalize` — the f32 sum, the f64 sqrt cast to f32, the
/// reciprocal, then the two multiplies. A zero vector yields NaN; NO
/// guard (the C# has none — `Entity.DirectionFrom`).
/// XNA `Vector2.Normalize` — the f32 sum, the f64 sqrt cast to f32, the
/// reciprocal, then the two multiplies. A zero vector yields NaN
/// (0 × ∞); NO guard (the C# has none).
fn xna_normalize(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// `Utils.SafeNormalize(Vector2.Zero)` (`Utils.cs:1837-1844`): the
/// zero/NaN fallback to (0,0), else the XNA normalize.
fn safe_normalize_zero(x: f32, y: f32) -> (f32, f32) {
    if (x == 0.0 && y == 0.0) || x.is_nan() || y.is_nan() {
        (0.0, 0.0)
    } else {
        xna_normalize(x, y)
    }
}

/// XNA `Vector2.Distance` — f32 sum, f64 sqrt cast back.
fn vector_distance(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;
    ((dx * dx + dy * dy) as f64).sqrt() as f32
}

/// `Utils.NextVector2Circular(halfWidth, halfHeight)`
/// (`Utils.cs:1266-1269`): `NextVector2Unit() * (hw, hh) * NextFloat()`
/// — the angle draw then the radial draw (the editing.rs twin).
fn next_vector2_circular(rng: &mut UnifiedRandom, hw: f32, hh: f32) -> (f32, f32) {
    let angle = rng.next_float() * std::f32::consts::TAU;
    let (ux, uy) = (angle.cos(), angle.sin());
    let radial = rng.next_float();
    (ux * hw * radial, uy * hh * radial)
}

/// `Main.hslToRgb(h, 1, 0.5) * mul` as XNA bytes — the palette's
/// byte-quantised floats recovered to bytes via `round` FIRST (the
/// `render/special_tiles.rs::cage_halo_color` precedent: a plain
/// `as u8` on the divide/multiply round-trip can truncate a hair-below
/// integer down a whole byte), then the `Color * float` operator's
/// per-component TRUNCATION (alpha included: `(byte)(255f * 0.5f) = 127`).
fn hsl_color_scaled(hue: f32, mul: f32) -> Color {
    let [r, g, b] = tr_core::tile_light::hsl_to_rgb(hue, 1.0, 0.5);
    Color::new(
        ((r * 255.0).round() * mul) as u8,
        ((g * 255.0).round() * mul) as u8,
        ((b * 255.0).round() * mul) as u8,
        (255.0 * mul) as u8,
    )
}

/// The family entry — `AI_065_Butterflies` (`NPC.cs:44896-45119`).
/// `ty` is the vanilla `NPC.type`; `npc` must carry the kind's 10×10
/// `SetDefaults` box (via `size_override` until the kinds land). The
/// dust chains roll the caller's stream through the real pool (the
/// style75 precedent — the draws are IN the arm).
pub fn ai_065_butterflies(
/// The family entry — `AI_065_Butterflies` (`NPC.cs:44896-45119`), the
/// type-356 branches. `npc` must carry the kind's `SetDefaults` box
/// (10x10 — `NpcKind::Butterfly.size()` once wired).
/// `AI_065_Butterflies` (`NPC.cs:44896-45119`), the type-356 path. `npc`
/// must carry the kind's `SetDefaults` box (10×10 — `kind.size()` once
/// the kind lands, `size_override` in the witnesses).
pub fn ai_065_step(
    ty: i32,
    npc: &mut Npc,
    world: &World,
    ctx: &Ai65Ctx,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
) -> Ai65Effects {
    debug_assert!(
        ty == BUTTERFLY || ty == GOLD_BUTTERFLY || ty == HELL_BUTTERFLY || ty == EMPRESS_BUTTERFLY,
        "aiStyle 65 has four members (356/444/653/661)"
    );
    let mut fx = Ai65Effects::default();
) {
    debug_assert!(ty == BUTTERFLY, "this wave ports the type-356 branches only");
) {
    debug_assert!(ty == BUTTERFLY, "only the Butterfly 356 is ported (W4-f18)");

    // `:44898-44899` — the aim locals ride ai[0]/ai[1].
    let mut num = npc.ai[0];
    let mut num2 = npc.ai[1];

    if ty == EMPRESS_BUTTERFLY {
        // `:44902-44904` — the rainbow glow: hsl(time*0.33 % 1) ×0.3
        // + 0.1 per channel, at the Centre (the Vector2 overload's
        // `(int)(v/16f)` quotient truncation, Lighting.cs:140-143).
        let hue = (ctx.global_time_wrapped_hourly * 0.33) % 1.0;
        let [r, g, b] = tr_core::tile_light::hsl_to_rgb(hue, 1.0, 0.5);
        let (cx, cy) = npc.center();
        fx.light = Some((
            (cx / 16.0) as i32,
            (cy / 16.0) as i32,
            [r * 0.3 + 0.1, g * 0.3 + 0.1, b * 0.3 + 0.1],
        ));
        // `:44905-44917` — the fade gate. num3 = 60 (the despawn
        // threshold), num4 = 50 (the clamp ceiling); a live player
        // target OUTSIDE the Hallow on a non-remix world raises the
        // ceiling to 60 AND forces the rise.
        let num3 = 60;
        let mut flag = false;
        let mut num4 = 50;
        // `GetTargetData()` (`:44908`): 661 is not in UsesNewTargeting,
        // so the data is the player target iff HasValidTarget
        // (`NPC.cs:6657-6671`).
        let valid = npc.target >= 0
            && npc.target < 255
            && (npc.target as usize) < ctx.players.len()
            && {
                let p = &ctx.players[npc.target as usize];
                p.active && !p.dead && !p.ghost
            };
        if !valid {
            flag = true; // `targetData.Invalid` (`:44909`).
        } else {
            let p = &ctx.players[npc.target as usize];
            let (pcx, pcy) = (
                p.x + p.width as f32 / 2.0,
                p.y + p.height as f32 / 2.0,
            );
            if vector_distance(pcx, pcy, cx, cy) >= 300.0 {
                flag = true; // `:44909` — the 300-px distance gate.
            }
            // `:44913-44917` — Type == Player always holds here.
            if !ctx.remix_world && !ctx.target_zone_hallow {
                num4 = num3;
                flag = true;
            }
        }
        // `:44918` — Clamp(ai[2] + (flag ? 1 : -1), 0, num4).
        npc.ai[2] = (npc.ai[2] + if flag { 1.0 } else { -1.0 }).clamp(0.0, num4 as f32);
        // `:44919-44927` — the despawn EARLY RETURN (SendData(23) is
        // the net seam; nothing below runs on this tick).
        if npc.ai[2] >= num3 as f32 {
            fx.despawn = true;
            return fx;
        }
        // `:44928` — Opacity = GetLerpValue(60, num4/2, ai[2], true).
        fx.opacity = get_lerp_value(num3 as f32, num4 as f32 / 2.0, npc.ai[2], true);
        // `:44929-44953` — the shimmer dust while fading.
        if npc.ai[2] > 0.0 {
            // `:44931-44932` — the flat 1-iteration loop.
            if rng.next_max(5) == 0 {
                // `:44936` — num6 = Lerp(0.9, 0.6, Opacity).
                let num6 = 0.9 + (0.6 - 0.9) * fx.opacity;
                // `:44937` — the type-267 dust, hue time*0.3 % 1, ×0.5.
                let dust_hue = (ctx.global_time_wrapped_hourly * 0.3) % 1.0;
                let (w, h) = npc.size();
                let num7 = dust.new_dust(
                    rng,
                    dust_ctx,
                    world,
                    glam::Vec2::new(npc.x, npc.y),
                    w as i32,
                    h as i32,
                    267,
                    0.0,
                    0.0,
                    0,
                    hsl_color_scaled(dust_hue, 0.5),
                    1.0,
                );
                // `:44938-44943` — the post-writes (the sentinel slot
                // included — the C# writes Main.dust[6000] too).
                {
                    let d = dust.slot_mut(num7);
                    let (ox, oy) = next_vector2_circular(rng, w, h);
                    d.position = glam::Vec2::new(cx + ox, cy + oy);
                    let vs = rng.next_float() * 0.8;
                    d.velocity *= vs;
                    d.velocity += glam::Vec2::new(npc.vx, npc.vy) * 0.6;
                    d.no_gravity = true;
                    d.fade_in = 0.6 + rng.next_float() * 0.7 * num6;
                    d.scale = 0.35;
                }
                // `:44944-44950` — the half-scale white clone.
                if num7 != DUST_SENTINEL {
                    let clone = dust.clone_dust(rng, dust_ctx, world, num7);
                    let d = dust.slot_mut(clone);
                    d.scale /= 2.0;
                    d.fade_in *= 0.85;
                    // `new Color(255,255,255,255) * 0.5f` — XNA byte
                    // truncation: 127 on all four channels.
                    d.color = Color::new(127, 127, 127, 127);
                }
            }
        }
        // `:44954` — dontTakeDamage = ai[2] >= num4/2 (INT division).
        npc.dont_take_damage = npc.ai[2] >= (num4 / 2) as f32;
    }

    // `:44956-45014` — the rearm block (netMode != 1 folds true).
    // `:44958-44970` — the one-time colour roll (non-661).
    if npc.ai[2] == 0.0 && ty != EMPRESS_BUTTERFLY {
        let num16 = rng.next_max(100);
        // `:44969` — the nested-ternary ladder, verbatim.
    // `:44956-45014` — the host block (`Main.netMode != 1` folds true).
    // `:44958-44970` — the one-time colour roll: ONE Next(100) through
    // the 8-step ladder, ai[2] = 1 + pick.
    if npc.ai[2] == 0.0 {
        let num16 = rng.next_max(100);
        let pick = if num16 == 0 {
            5
        } else if num16 < 3 {
            1
        } else if num16 < 9 {
            2
        } else if num16 < 19 {
            7
        } else if num16 < 34 {
            3
        } else if num16 < 53 {
            6
        } else if num16 >= 75 {
            0
        } else {
            4
        };
        npc.ai[2] = (1 + pick) as f32;
    }
    // `:44971-44974` — the one-time scale latch.
    if npc.ai[3] == 0.0 {
        npc.ai[3] = rng.next_range(75, 111) as f32 * 0.01;
    }
    // `:44975-44976` — the decrement runs EVERY tick.
    npc.local_ai[0] -= 1.0;
    if npc.local_ai[0] <= 0.0 {
        // `:44978` — the timer draw, BEFORE TargetClosest (`:44979`,
        // RNG-free — the npc_target.rs port, unmodified).
        npc.local_ai[0] = rng.next_range(90, 240) as f32;
        target_closest(npc, world, ctx.players, true, 65, ctx.ticker);
        // `:44980` — the horizontal gap to the (possibly new) target.
        let p = &ctx.players[npc.target as usize];
        let (cx, _cy) = npc.center();
        let num17 = (cx - (p.x + p.width as f32 / 2.0)).abs();
        // `:44981-45001` — the FAR wander (>700, near-latch unset).
        if num17 > 700.0 && npc.local_ai[3] == 0.0 {
    // `:44956-45014` — the rearm block (`Main.netMode != 1` folds true).
    // `:44958-44970` — the one-time variant roll: Next(100), the ladder
    // picks the colour band (ai[2] = 1..7; the FindFrame band offset and
    // the catchItem re-derive both read it).
    if npc.ai[2] == 0.0 {
        let num8 = 0;
        let num9 = 4;
        let num10 = 6;
        let num11 = 3;
        let num12 = 7;
        let num13 = 2;
        let num14 = 1;
        let num15 = 5;
        let num16 = rng.next_max(100);
        npc.ai[2] = (1 + if num16 == 0 {
            num15
        } else if num16 < 3 {
            num14
        } else if num16 < 9 {
            num13
        } else if num16 < 19 {
            num12
        } else if num16 < 34 {
            num11
        } else if num16 < 53 {
            num10
        } else if num16 >= 75 {
            num8
        } else {
            num9
        }) as f32;
        }
    }
    }
    // `:44971-44974` — the one-time scale latch: Next(75, 111) * 0.01f.
    if npc.ai[3] == 0.0 {
        npc.ai[3] = rng.next_range(75, 111) as f32 * 0.01;
    }
    // `:44975-45013` — the rearm timer. The decrement runs EVERY tick.
    npc.local_ai[0] -= 1.0;
    if npc.local_ai[0] <= 0.0 {
        npc.local_ai[0] = rng.next_range(90, 240) as f32;
        // `:44979` — TargetClosest BEFORE the gap read (it draws nothing).
        target_closest(npc, world, ctx.players, true, 65, ctx.ticker);
        // `:44980` — the horizontal gap to the (possibly new) target's
        // Center.X (float halves).
        let p = &ctx.players[npc.target as usize];
        let (cx, _cy) = npc.center();
        let num17 = (cx - (p.x + p.width as f32 / 2.0)).abs();
        // `:44981-45001` — the FAR wander: > 700 px and the near-latch
        // (localAI[3]) never set.
        if num17 > 700.0 && npc.local_ai[3] == 0.0 {
            // `:44983-44991` — the speed ladder, one draw per tier.
    // `:44975-44976` — the rearm countdown, EVERY tick.
    npc.local_ai[0] -= 1.0;
    if npc.local_ai[0] <= 0.0 {
        // `:44978` — the timer draw BEFORE TargetClosest (the style64
        // arm's order is the reverse; both kept).
        npc.local_ai[0] = rng.next_range(90, 240) as f32;
        // `:44979`.
        target_closest(npc, world, ctx.players, true, 65, ctx.ticker);
        // `:44980` — the horizontal gap to the target's Center.X.
        let p = &ctx.players[npc.target as usize];
        let (cx, _cy) = npc.center();
        let num17 = (cx - (p.x + p.width as f32 / 2.0)).abs();
        // `:44981-45001` — the FAR wander.
        if num17 > 700.0 && npc.local_ai[3] == 0.0 {
            // `:44983-44991` — the speed ladder, one draw per tick.
            let mut num18 = rng.next_range(50, 151) as f32 * 0.01;
            if num17 > 1000.0 {
                num18 = rng.next_range(150, 201) as f32 * 0.01;
            } else if num17 > 850.0 {
                num18 = rng.next_range(100, 151) as f32 * 0.01;
            }
            let num19 = npc.facing * rng.next_range(100, 251);
            let mut num20 = rng.next_range(-50, 51);
            // `:44994-44997` — the CONDITIONAL Y-bias draw.
            if npc.y > p.y - 100.0 {
                num20 -= rng.next_range(100, 251);
            }
            // `:44998-45000` — the int-sum f64 sqrt, cast BEFORE the
            // divide; NO zero guard (the C# NaN/Inf pass-through).
            let num21 = num18 / ((num19 * num19 + num20 * num20) as f64).sqrt() as f32;
            num = num19 as f32 * num21;
            num2 = num20 as f32 * num21;
        } else {
            // `:45002-45011` — the NEAR wander; the latch sticks at 1.
            // `:44992-44993` — the direction-scaled X, the Y spread.
            let num19 = npc.facing * rng.next_range(100, 251);
            let mut num20 = rng.next_range(-50, 51);
            // `:44994-44997` — the CONDITIONAL downward-bias draw: only
            // when the butterfly sits below player.Y - 100.
            // `:44992-44993` — the direction-scaled X, the Y spread.
            let num19 = npc.facing * rng.next_range(100, 251);
            let mut num20 = rng.next_range(-50, 51);
            // `:44994-44997` — the CONDITIONAL downward-bias draw.
            if npc.y > p.y - 100.0 {
                num20 -= rng.next_range(100, 251);
            }
            // `:44998-45000` — the int-sum f64 sqrt, cast to f32 BEFORE
            // the divide; NO zero guard (the C# NaN/Inf pass-through).
            let num21 =
                num18 / ((num19 * num19 + num20 * num20) as f64).sqrt() as f32;
            num = num19 as f32 * num21;
            num2 = num20 as f32 * num21;
        } else {
            // `:45002-45011` — the NEAR wander: the latch sticks at 1
            // for the NPC's whole life (nothing in the arm clears it).
            // `:45002-45011` — the NEAR wander: the latch sticks at 1.
            npc.local_ai[3] = 1.0;
            let num22 = rng.next_range(26, 301) as f32 * 0.01;
            let num23 = rng.next_range(-100, 101);
            let num24 = rng.next_range(-100, 101);
            let num25 = num22 / ((num23 * num23 + num24 * num24) as f64).sqrt() as f32;
            let num25 =
                num22 / ((num23 * num23 + num24 * num24) as f64).sqrt() as f32;
            num = num23 as f32 * num25;
            num2 = num24 as f32 * num25;
        }
        // `:45012`.
        npc.net_update = true;
    }

    // `:45015` — scale rides the latched ai[3] every tick.
    npc.scale = npc.ai[3];
    // `:45016-45018` — the 1/60 blend, f32 in the C#'s order.

    // `:45016-45018` — the 1/60 blend toward the aim, f32 in the C#'s
    // order: (v * (float)(60 - 1) + aim) / (float)60.
    let num26 = 60.0f32;
    npc.vx = (npc.vx * (num26 - 1.0) + num) / num26;
    npc.vy = (npc.vy * (num26 - 1.0) + num2) / num26;

    // `:45019-45035` — falling: the 3-tile down-scan. A blocking tile
    // (solid OR liquid) flips the aim AND damps ×0.9 INSIDE the loop
    // (up to three damps in one tick). `(int)Center.X / 16` truncates
    // BEFORE the divide; the null-tile check is the C#'s own.
    // `:45019-45035` — falling: the THREE-tile down-scan. A blocking tile
    // (solid OR liquid) flips the Y aim AND damps the velocity ×0.9 —
    // INSIDE the loop, once per blocking tile (up to three damps).
    // (solid OR liquid) flips the aim AND damps the velocity ×0.9 —
    // INSIDE the loop, once per blocking tile.
    if npc.vy > 0.0 {
        let num27 = 3;
        let (cx, cy) = npc.center();
        let num28 = (cx as i32) / 16;
        let num29 = (cy as i32) / 16;
        for j in num29..num29 + num27 {
            if let Some(t) = world.tile(num28, j) {
                if (nactive(t) && tile_solid_runtime(t.kind)) || t.liquid > 0 {
                    num2 *= -1.0;
                    if npc.vy > 0.0 {
                        npc.vy *= 0.9;
            if in_world2(world, num28, j) {
                if let Some(t) = world.tile(num28, j) {
                    if (nactive(t) && tile_solid_runtime(t.kind)) || t.liquid > 0 {
                        num2 *= -1.0;
                        if npc.vy > 0.0 {
                            npc.vy *= 0.9;
                        }
                    }
                }
            }
            }
        }
        }
    }
    }
    // `:45036-45057` — rising: the 30-tile down-scan for ANY solid;
    // only when none is found, the single flip + damp (liquid does
    // not count here).
                    }
                }
            }
        }
    }
    // `:45036-45057` — rising: the 30-tile down-scan for ANY solid; if
    // none, flip the aim and damp ONCE (liquid does not count here).
    if npc.vy < 0.0 {
        let num30 = 30;
        let mut flag2 = false;
        let (cx, cy) = npc.center();
        let num31 = (cx as i32) / 16;
        let num32 = (cy as i32) / 16;
        for k in num32..num32 + num30 {
            if let Some(t) = world.tile(num31, k) {
                if nactive(t) && tile_solid_runtime(t.kind) {
                    flag2 = true;
            if in_world2(world, num31, k) {
                if let Some(t) = world.tile(num31, k) {
                    if nactive(t) && tile_solid_runtime(t.kind) {
                        flag2 = true;
                    }
                }
            }
        }
        }
                }
            }
        }
        if !flag2 {
            num2 *= -1.0;
            if npc.vy < 0.0 {
                npc.vy *= 0.9;
            }
        }

    // `:45058-45088` — the threat flee.
    if npc.local_ai[1] > 0.0 {
        npc.local_ai[1] -= 1.0;
    } else {
        // `:45064-45068` — the cooldown: 15, or 10 for the Empress.
        npc.local_ai[1] = if ty == EMPRESS_BUTTERFLY { 10.0 } else { 15.0 };
        let mut num33 = 0.0f32;
        let mut zero = (0.0f32, 0.0f32);
        let (cx, cy) = npc.center();
        // `:45071-45079` — the slot-order scan: active, damage > 0,
        // not friendly, Hitbox.Distance(Center) <= 100.
        for view in ctx.npcs {
            if view.active && view.damage > 0 && !view.friendly {
                let d = rect_distance(
    // `:45058-45088` — the threat flee, every 15th tick (the 661's 10 is
    // not this wave's). The scan sums `DirectionFrom(nPC.Center)` over
    // every live hostile within Hitbox.Distance(Center) <= 100.
    // `:45058-45088` — the 15-tick threat flee (the 661's 10-tick arm is
    // unported). The scan reads the slot-ordered sibling views; the
    // butterfly's own damage 0 excludes it.
    if npc.local_ai[1] > 0.0 {
        npc.local_ai[1] -= 1.0;
    } else {
        npc.local_ai[1] = 15.0;
        let mut num33 = 0.0f32;
        let mut zero = (0.0f32, 0.0f32);
        let (cx, cy) = npc.center();
        for view in ctx.npcs {
            if view.active && view.damage > 0 && !view.friendly {
                let (nx, ny) = view.center();
                // `Entity.Hitbox` — the int-truncated position.
                if rect_distance(
                let d = rect_distance(
                    view.x as i32,
                    view.y as i32,
                    view.width,
                    view.height,
                    cx,
                    cy,
                );
                if d <= 100.0 {
                    num33 += 1.0;
                    // `Entity.DirectionFrom` (Entity.cs:210-213).
                    let (sx, sy) = (
                        view.x + view.width as f32 / 2.0,
                        view.y + view.height as f32 / 2.0,
                    );
                    let (dx, dy) = xna_normalize(cx - sx, cy - sy);
                ) <= 100.0
                {
                    num33 += 1.0;
                    // `base.DirectionFrom(nPC.Center)` —
                    // Normalize(ourCenter - theirCenter), NO guard.
                    let (dx, dy) = xna_normalize(cx - nx, cy - ny);
                    // `base.DirectionFrom(nPC.Center)` — the XNA
                    // normalize of (own Centre - threat Centre).
                    let (tcx, tcy) = view.center();
                    let (dx, dy) = xna_normalize(cx - tcx, cy - tcy);
                    zero.0 += dx;
                    zero.1 += dy;
                }
            }
        }
        }
        // `:45080-45087` — the averaged ×2 push + the 16-cap via
        // SafeNormalize(Zero) (a NaN velocity zeroes).
        if num33 > 0.0 {
            npc.vx += zero.0 / num33 * 2.0;
            npc.vy += zero.1 / num33 * 2.0;
            let len = ((npc.vx * npc.vx + npc.vy * npc.vy) as f64).sqrt() as f32;
            if len > 16.0 {
                let (nx, ny) = safe_normalize_zero(npc.vx, npc.vy);
                npc.vx = nx * 16.0;
                npc.vy = ny * 16.0;
            }
        }
    }
    }

    // `:45089-45093` — the wall bounce: the aim turns AWAY from the
    // velocity's sign (>= 0 → -|aim|, the NaN compare folds to the
    // first arm), the velocity kicks back ×-0.2.
    if npc.collide_x {
        num = if !(npc.vx < 0.0) { 0.0 - num.abs() } else { num.abs() };
        npc.vx *= -0.2;
    }
    // `:45094-45101` — the facing: two separate ifs; at vx == 0 (or
    // NaN) neither fires.
        if num33 > 0.0 {
            // `:45082` — velocity += zero / num33 * 2f.
            npc.vx += zero.0 / num33 * 2.0;
            npc.vy += zero.1 / num33 * 2.0;
            // `:45083-45086` — the 16 cap: Length() > 16 →
            // SafeNormalize(Zero) * 16.
            let len = ((npc.vx * npc.vx + npc.vy * npc.vy) as f64).sqrt() as f32;
            if len > 16.0 {
                let (sx, sy) = if (npc.vx == 0.0 && npc.vy == 0.0)
                    || npc.vx.is_nan()
                    || npc.vy.is_nan()
                {
                    (0.0, 0.0)
                } else {
                    xna_normalize(npc.vx, npc.vy)
                };
                npc.vx = sx * 16.0;
                npc.vy = sy * 16.0;
        if num33 > 0.0 {
            // `:45082` — velocity += zero / num33 * 2f (the C#'s
            // divide-then-multiply order).
            npc.vx += zero.0 / num33 * 2.0;
            npc.vy += zero.1 / num33 * 2.0;
            // `:45083-45086` — the 16-cap via SafeNormalize(Zero)
            // (`Utils.cs:1837-1844` — zero/NaN falls back to (0,0)).
            let len = ((npc.vx * npc.vx + npc.vy * npc.vy) as f64).sqrt() as f32;
            if len > 16.0 {
                if (npc.vx == 0.0 && npc.vy == 0.0)
                    || npc.vx.is_nan()
                    || npc.vy.is_nan()
                {
                    npc.vx = 0.0;
                    npc.vy = 0.0;
                } else {
                    let (nx, ny) = xna_normalize(npc.vx, npc.vy);
                    npc.vx = nx * 16.0;
                    npc.vy = ny * 16.0;
                }
            }
        }
    }
    // `:45089-45093` — the wall bounce: the aim turns AWAY from the
    // velocity's sign (>= 0 → -|aim|, the NaN compare folds to the first
    // arm), the velocity kicks back ×-0.2.
    // velocity's sign (>= 0 → -|aim|, the NaN compare folds to the
    // first arm), the velocity kicks back ×-0.2.
    if npc.collide_x {
        num = if !(npc.vx < 0.0) {
            0.0 - num.abs()
        } else {
            num.abs()
        };
        npc.vx *= -0.2;
    }
    // `:45094-45101` — the facing: two separate ifs; at vx == 0 (or NaN)
    // neither fires and the direction keeps its old value.
    // `:45094-45101` — the facing: two separate ifs; at vx == 0 (or
    // NaN) neither fires and the direction keeps its old value.
    if npc.vx < 0.0 {
        npc.facing = -1;
    }
    if npc.vx > 0.0 {
        npc.facing = 1;
    }
    // `:45102-45103` — the aim write-back, LAST.
    npc.ai[0] = num;
    npc.ai[1] = num2;
    // `:45104-45107` — the 356 catchItem re-derive is the
    // `bug_net.rs::catch_item(356, ai2)` seam (module docs) — no item
    // field write here.
    if ty == HELL_BUTTERFLY {
        // `:45110` — position += netOffset (the net-smoothing offset).
        npc.x += npc.net_offset.x;
        npc.y += npc.net_offset.y;
        // `:45111` — the ember light, `(int)Center.X / 16` (truncate
        // BEFORE the divide — NOT the 661 head's quotient form).
        let (cx, cy) = npc.center();
        fx.light = Some(((cx as i32) / 16, (cy as i32) / 16, [0.6, 0.3, 0.1]));
        // `:45112-45116` — the 1/60-gated type-6 dust, alpha 254.
        if rng.next_max(60) == 0 {
            let (w, h) = npc.size();
            let num34 = dust.new_dust(
                rng,
                dust_ctx,
                world,
                glam::Vec2::new(npc.x, npc.y),
                w as i32,
                h as i32,
                6,
                0.0,
                0.0,
                254,
                Color::default(),
                1.0,
            );
            dust.slot_mut(num34).velocity *= 0.0;
        }
        // `:45117` — position -= netOffset.
        npc.x -= npc.net_offset.x;
        npc.y -= npc.net_offset.y;
    }
    fx
    // `:45104-45107` — the catchItem re-derive: no Npc field; the catch
    // path re-derives from ai[2] (`bug_net::catch_item`'s 356 row).
}
}
    // `:45104-45107` — the 356 catchItem re-derive is the catch path's
    // (`bug_net::catch_item(356, ai[2])` reads the live style); the
    // 653/661 tails (`:45108-45118` / `:44900-44955`) are unported.
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one
    //! C# branch against its citation; the RNG witnesses mirror the
    //! cited draws on a cloned `UnifiedRandom` (and a twin `DustPool`
    //! for the dust internals) and compare state, so a drifted draw
    //! order fails by construction.
    use super::*;
    use crate::npc::NpcKind;
    //! cited `Next` calls on a cloned `UnifiedRandom` and compare the
    //! stream state so a drifted draw order fails by construction.
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    //! cited `Next` calls on a cloned `UnifiedRandom` and compare the
    //! stream state so a drifted draw order fails by construction.
    use super::*;

    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    fn cand(x: f32, y: f32) -> TargetCandidate {
        TargetCandidate {
            active: true,
            dead: false,
            ghost: false,
            x,
            y,
            width: 20,
            height: 42,
            aggro: 0,
            item_animation: 0,
            npc_type_no_aggro: false,
            tank_pet: None,
        }
    }

    /// A stand-in butterfly body: no aiStyle-65 kind exists yet, so the
    /// witnesses drive `ai_065_butterflies(ty, ...)` on a spare kind
    /// with the C#'s 10×10 box (`NPC.cs:13533-13534`) pinned via
    /// `size_override`. ai[2]/ai[3] pre-set so the one-time rolls stay
    /// out of the way unless a test wants them; live timers by default.
    fn fly(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Slime, x, y, 1, 0.0);
        n.size_override = Some((10.0, 10.0));
        n.hp = 5;
        n.max_hp = 5;
        n.ai[2] = 4.0; // a rolled colour
        n.ai[3] = 1.0; // a latched scale
        n.local_ai[0] = 50.0; // a live rearm timer
        n.local_ai[1] = 5.0; // a live flee cooldown
        n
    }

    fn ctx<'a>(players: &'a [TargetCandidate], npcs: &'a [Ai64NpcView]) -> Ai65Ctx<'a> {
        Ai65Ctx {
            players,
            npcs,
            ticker: 0,
            global_time_wrapped_hourly: 0.0,
            remix_world: false,
            target_zone_hallow: false,
        }
    }

    fn spawn_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: glam::Vec2::ZERO,
            screen_size: glam::Vec2::new(6400.0, 4800.0),
            max_dust_to_draw: 6000,
        }
    }

    fn solid(w: &mut World, x: i32, y: i32) {
        let t = w.tile_mut(x, y).unwrap();
        *t = Tile {
            kind: 1, // stone — in TILE_SOLID
            flags: FLAG_ACTIVE,
            ..Tile::default()
        };
    }

    /// Find a seed whose FIRST `Next(bound)` is 0 — the dust-gate
    /// witnesses need the gated branch without hand-computing the
    /// stream (the search runs at test time).
    fn seed_first_zero(bound: i32) -> u64 {
        let mut seed = 1u64;
        loop {
            let mut r = UnifiedRandom::new(seed as i32);
            if r.next_max(bound) == 0 {
                return seed;
            }
            seed += 1;
        }
    }

    /// The census: the four `this.aiStyle = 65;` rows and their NPCIDs.
    #[test]
    fn w4_d30_s65_the_family_is_the_four_butterflies() {
        assert_eq!(BUTTERFLY, 356, "NPCID.cs:11389, NPC.cs:13535");
        assert_eq!(GOLD_BUTTERFLY, 444, "NPCID.cs:11561, NPC.cs:14279");
        assert_eq!(HELL_BUTTERFLY, 653, "NPCID.cs:11983, NPC.cs:17254");
        assert_eq!(EMPRESS_BUTTERFLY, 661, "NPCID.cs:11999, NPC.cs:17355");
    }

    /// `NPC.cs:44958-44970` — the colour roll: ONE Next(100), the
    /// nested-ternary ladder. The witness replays the draw and checks
    /// the mapping for every band boundary via the mirrored roll.
    #[test]
    fn w4_d30_s65_the_colour_roll_draws_once_and_maps_the_ladder() {
        let w = air();
        let players = [cand(1620.0, 1500.0)];
        let mut n = fly(1600.0, 1500.0);
        n.ai[2] = 0.0;
        let mut rng = UnifiedRandom::new(7);
        let mut mirror = rng.clone();
        let roll = mirror.next_max(100);
        let want = 1 + if roll == 0 {
            5
        } else if roll < 3 {
            1
        } else if roll < 9 {
            2
        } else if roll < 19 {
            7
        } else if roll < 34 {
            3
        } else if roll < 53 {
            6
        } else if roll >= 75 {
            0
        } else {
            4
        };
        ai_065_butterflies(
            BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert_eq!(rng, mirror, "exactly one Next(100), nothing else");
        assert_eq!(n.ai[2], want as f32, ":44969");
        // The ladder's band edges, pinned as pure maths (`:44969`).
        let map = |r: i32| {
            1 + if r == 0 { 5 } else if r < 3 { 1 } else if r < 9 { 2 }
            else if r < 19 { 7 } else if r < 34 { 3 } else if r < 53 { 6 }
            else if r >= 75 { 0 } else { 4 }
        };
        assert_eq!((map(0), map(2), map(3), map(8)), (6, 2, 3, 3));
        assert_eq!((map(9), map(18), map(19), map(33)), (8, 8, 4, 4));
        assert_eq!((map(34), map(52), map(53), map(74)), (7, 7, 5, 5));
        assert_eq!((map(75), map(99)), (1, 1));
        // The next tick: rolled — no draw.
        let untouched = rng.clone();
        ai_065_butterflies(
            BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert_eq!(rng, untouched, "the colour never re-rolls");
    }

    /// `NPC.cs:44971-44974` + `:45015` — the scale latch draws
    /// Next(75,111) ONCE (ai[3] == 0), then scale rides it every tick.
    #[test]
    fn w4_d30_s65_the_scale_latch_draws_once_and_sticks() {
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        let mut n = fly(1600.0, 1500.0);
        n.ai[3] = 0.0;
        let mut rng = UnifiedRandom::new(7);
        let mut mirror = rng.clone();
        let want = mirror.next_range(75, 111) as f32 * 0.01;
        ai_065_butterflies(
            BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert_eq!(rng, mirror, "exactly one Next(75,111)");
        assert_eq!(n.ai[3], want, ":44973");
        assert_eq!(n.scale, want, ":45015");
        let untouched = rng.clone();
        ai_065_butterflies(
            BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert_eq!(rng, untouched, "the latch never re-draws");
    }

    /// `NPC.cs:44976-45012` — the NEAR rearm: Next(90,240), then
    /// Next(26,301), Next(-100,101) ×2, in order; localAI[3] latches
    /// to 1; netUpdate fires. (The 65 near speed is 26..300 — NOT the
    /// firefly's 5..150, `:45005` vs `:34059`.)
    #[test]
    fn w4_d30_s65_the_near_rearm_draws_the_cited_quad_in_order() {
        let w = air();
        let players = [cand(1620.0, 1500.0)]; // 25 px away — near
        let mut n = fly(1600.0, 1500.0);
        n.local_ai[0] = 1.0; // decrements to 0 → rearm
        let mut rng = UnifiedRandom::new(42);
        let mut mirror = rng.clone();
        let timer = mirror.next_range(90, 240);
        let speed = mirror.next_range(26, 301) as f32 * 0.01;
        let ax = mirror.next_range(-100, 101);
        let ay = mirror.next_range(-100, 101);
        ai_065_butterflies(
            BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert_eq!(rng, mirror, "the quad, in C# order, nothing else");
        assert_eq!(n.local_ai[0], timer as f32, ":44978");
        assert_eq!(n.local_ai[3], 1.0, ":45004 — the near latch");
        let norm = speed / ((ax * ax + ay * ay) as f64).sqrt() as f32;
        assert_eq!(n.ai[0], ax as f32 * norm, ":45009");
        // The air world's up-scan flips a negative aim once.
        assert_eq!(n.ai[1], (ay as f32 * norm).abs(), ":45010 + the up-scan flip");
        assert!(n.net_update, ":45012");
    }

    /// `NPC.cs:44981-45000` — the FAR rearm (>700, latch unset): the
    /// base speed Next(50,151), the direction-scaled Next(100,251),
    /// Next(-50,51), and the CONDITIONAL Next(100,251) Y-bias when the
    /// fly sits below player.Y - 100. Stream-clone replay pins order.
    #[test]
    fn w4_d30_s65_the_far_rearm_draws_the_y_bias_only_below_the_player() {
        let w = air();
        // The player 800 px right and 200 px ABOVE the fly: far, and
        // fly.y > player.y - 100 → the bias draw fires.
        let players = [cand(2400.0, 1300.0)];
        let mut n = fly(1600.0, 1500.0);
        n.local_ai[0] = 1.0;
        let mut rng = UnifiedRandom::new(11);
        let mut mirror = rng.clone();
        let timer = mirror.next_range(90, 240);
        let speed = mirror.next_range(50, 151) as f32 * 0.01;
        let ax = n.facing * mirror.next_range(100, 251);
        let mut ay = mirror.next_range(-50, 51);
        ay -= mirror.next_range(100, 251); // the conditional draw
        ai_065_butterflies(
            BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert_eq!(rng, mirror, "the five draws, in C# order");
        assert_eq!(n.local_ai[0], timer as f32);
        assert_eq!(n.local_ai[3], 0.0, "the far branch never latches");
        let norm = speed / ((ax * ax + ay * ay) as f64).sqrt() as f32;
        assert_eq!(n.ai[0], ax as f32 * norm);
        assert_eq!(n.ai[1], (ay as f32 * norm).abs(), "the up-scan flip");

        // The player 200 px BELOW: fly.y (1500) <= player.y - 100
        // (1600) → NO bias draw; the stream is one draw shorter.
        let players2 = [cand(2400.0, 1700.0)];
        let mut n2 = fly(1600.0, 1500.0);
        n2.local_ai[0] = 1.0;
        let mut rng2 = UnifiedRandom::new(11);
        let mut mirror2 = rng2.clone();
        let _ = mirror2.next_range(90, 240);
        let speed2 = mirror2.next_range(50, 151) as f32 * 0.01;
        let ax2 = n2.facing * mirror2.next_range(100, 251);
        let ay2 = mirror2.next_range(-50, 51);
        ai_065_butterflies(
            BUTTERFLY, &mut n2, &w, &ctx(&players2, &[]), &mut rng2,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert_eq!(rng2, mirror2, "no bias draw above the player");
        let norm2 = speed2 / ((ax2 * ax2 + ay2 * ay2) as f64).sqrt() as f32;
        assert_eq!(n2.ai[0], ax2 as f32 * norm2);
    }

    /// `NPC.cs:45016-45018` — the 1/60 blend toward the aim, and
    /// `:45094-45101` the facing writes (two separate ifs).
    #[test]
    fn w4_d30_s65_the_blend_is_one_sixtieth_and_the_facing_follows_vx() {
        let w = air();
        let players = [cand(1620.0, 1500.0)];
        let mut n = fly(1600.0, 1500.0);
        n.ai[0] = 6.0;
        n.ai[1] = 0.0;
        n.vx = 0.0;
        n.vy = 0.0;
        let mut rng = UnifiedRandom::new(3);
        let untouched = rng.clone();
        ai_065_butterflies(
            BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert_eq!(rng, untouched, "a fully-latched idle tick draws nothing");
        assert_eq!(n.vx, (0.0 * 59.0 + 6.0) / 60.0, ":45017");
        assert_eq!(n.facing, 1, ":45098-45101");
        // vx == 0 keeps the old facing (neither if fires).
        let mut n2 = fly(1600.0, 1500.0);
        n2.ai[0] = 0.0;
        n2.ai[1] = 0.0;
        n2.facing = -1;
        ai_065_butterflies(
            BUTTERFLY, &mut n2, &w, &ctx(&players, &[]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert_eq!(n2.facing, -1, "vx == 0 keeps the facing");
    }

    /// `NPC.cs:45019-45035` — the down-scan damp fires INSIDE the loop,
    /// once per blocking tile (three stacked solids → 0.9³), and the
    /// aim flips once per tile (three flips → back to the original
    /// sign). Liquid alone bounces too (`:45026`).
    #[test]
    fn w4_d30_s65_the_down_scan_damps_once_per_blocking_tile() {
        let mut w = air();
        // The fly centre at tile (100, 100); solids at rows 100..102.
        solid(&mut w, 100, 100);
        solid(&mut w, 100, 101);
        solid(&mut w, 100, 102);
        let players = [cand(1620.0, 1500.0)];
        let mut n = fly(1600.0 - 5.0, 1600.0 - 5.0);
        n.ai[0] = 0.0;
        n.ai[1] = 2.0; // a positive aim
        n.vy = 1.0; // falling
        let mut rng = UnifiedRandom::new(3);
        ai_065_butterflies(
            BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        // The blend nudges vy toward 2 first: (1*59+2)/60.
        let blended = (1.0f32 * 59.0 + 2.0) / 60.0;
        assert_eq!(n.vy, blended * 0.9 * 0.9 * 0.9, "three in-loop damps");
        assert_eq!(n.ai[1], -2.0, "three flips invert the aim (odd count)");
    }

    /// `NPC.cs:45036-45057` — the up-scan: ANY solid in the 30-tile
    /// column suppresses the bounce; an empty column flips the aim
    /// once and damps once.
    #[test]
    fn w4_d30_s65_the_up_scan_bounces_only_over_an_empty_column() {
        let players = [cand(1620.0, 1500.0)];
        // Empty air: the bounce fires.
        let w = air();
        let mut n = fly(1595.0, 1595.0);
        n.ai[1] = -2.0;
        n.vy = -1.0;
        let mut rng = UnifiedRandom::new(3);
        ai_065_butterflies(
            BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        let blended = (-1.0f32 * 59.0 + -2.0) / 60.0;
        assert_eq!(n.vy, blended * 0.9, "the single up-scan damp");
        assert_eq!(n.ai[1], 2.0, "the aim flips positive");
        // A solid 20 tiles below: no bounce.
        let mut w2 = air();
        solid(&mut w2, 100, 120);
        let mut n2 = fly(1595.0, 1595.0);
        n2.ai[1] = -2.0;
        n2.vy = -1.0;
        ai_065_butterflies(
            BUTTERFLY, &mut n2, &w2, &ctx(&players, &[]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert_eq!(n2.vy, blended, "no damp over a solid column");
        assert_eq!(n2.ai[1], -2.0, "the aim keeps its sign");
    }

    /// `NPC.cs:45089-45093` — the collideX bounce: the aim turns away
    /// from the velocity's sign, the velocity kicks ×-0.2.
    #[test]
    fn w4_d30_s65_the_collide_x_bounce_turns_the_aim_away() {
        let w = air();
        let players = [cand(1620.0, 1500.0)];
        let mut n = fly(1600.0, 1500.0);
        n.ai[0] = 3.0;
        n.vx = 2.0; // moving right into the wall
        n.collide_x = true;
        let mut rng = UnifiedRandom::new(3);
        ai_065_butterflies(
            BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        // The blend runs first: (2*59+3)/60, then the ×-0.2 kick.
        let blended = (2.0f32 * 59.0 + 3.0) / 60.0;
        assert_eq!(n.vx, blended * -0.2, ":45092");
        assert_eq!(n.ai[0], -3.0, ":45091 — vx >= 0 → -|aim|");
    }

    /// `NPC.cs:45058-45088` — the threat flee: the 15-tick cooldown,
    /// the averaged DirectionFrom push ×2, the 16-speed cap.
    #[test]
    fn w4_d30_s65_the_threat_flee_pushes_away_and_caps_at_sixteen() {
        let w = air();
        let players = [cand(3000.0, 1500.0)];
        let mut n = fly(1600.0, 1500.0);
        n.local_ai[1] = 0.0; // the cooldown expired
        n.ai[0] = 0.0;
        n.ai[1] = 0.0;
        // One hostile 50 px to the LEFT — the push is to the RIGHT.
        let threat = Ai64NpcView {
            active: true,
            damage: 10,
            friendly: false,
            x: 1550.0,
            y: 1595.0,
            width: 20,
            height: 20,
        };
        let mut rng = UnifiedRandom::new(3);
        let untouched = rng.clone();
        ai_065_butterflies(
            BUTTERFLY, &mut n, &w, &ctx(&players, &[threat]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert_eq!(rng, untouched, "the flee draws nothing");
        assert_eq!(n.local_ai[1], 15.0, ":45064 — the cooldown resets");
        assert!(n.vx > 0.0, "pushed away from the threat");
        // One threat: the unit DirectionFrom ×2 — speed exactly 2.
        let push_len = (n.vx * n.vx + n.vy * n.vy).sqrt();
        assert!((push_len - 2.0).abs() < 1e-4, "the unit push ×2, :45082");
        // A pre-existing 20-speed velocity caps to 16 along its own
        // direction (the push adds first, then the cap).
        let mut n2 = fly(1600.0, 1500.0);
        n2.local_ai[1] = 0.0;
        n2.vx = 20.0;
        n2.vy = 0.0;
        n2.ai[0] = 0.0;
        ai_065_butterflies(
            BUTTERFLY, &mut n2, &w, &ctx(&players, &[threat]), &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        let len = (n2.vx * n2.vx + n2.vy * n2.vy).sqrt();
        assert!((len - 16.0).abs() < 1e-4, "the 16-cap, :45083-45086");
    }

    /// `NPC.cs:44905-44928` + `:44954` — the 661 fade: a live player
    /// target outside the Hallow raises the ceiling to 60 and forces
    /// the rise; the despawn at ai2 >= 60 returns early; Opacity is
    /// the clamped reversed lerp; dontTakeDamage at ai2 >= num4/2.
    #[test]
    fn w4_d30_s65_the_empress_fade_rises_outside_the_hallow_and_despawns_at_sixty() {
        let w = air();
        let players = [cand(1620.0, 1500.0)]; // 25 px — inside 300
        let mut n = fly(1600.0, 1500.0);
        n.target = 0;
        n.ai[2] = 59.0;
        let mut c = ctx(&players, &[]);
        c.target_zone_hallow = false; // outside the Hallow → num4 = 60
        let mut rng = UnifiedRandom::new(3);
        let untouched = rng.clone();
        let fx = ai_065_butterflies(
            EMPRESS_BUTTERFLY, &mut n, &w, &c, &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert!(fx.despawn, ":44919-44927 — ai2 hits 60");
        assert_eq!(rng, untouched, "the despawn tick draws nothing further");
        assert!(fx.light.is_some(), "the head light ran before the return");
        // In the Hallow the ceiling stays 50 and a near target makes
        // the counter FALL; at ai2 = 25 the shield drops (25 >= 25).
        let mut n2 = fly(1600.0, 1500.0);
        n2.target = 0;
        n2.ai[2] = 26.0;
        let mut c2 = ctx(&players, &[]);
        c2.target_zone_hallow = true;
        let fx2 = ai_065_butterflies(
            EMPRESS_BUTTERFLY, &mut n2, &w, &c2, &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert!(!fx2.despawn);
        assert_eq!(n2.ai[2], 25.0, "the near-Hallow fall, :44918");
        assert!(n2.dont_take_damage, ":44954 — 25 >= 50/2");
        // Opacity at ai2 = 25, num4 = 50: GetLerpValue(60, 25, 25) = 1.
        assert_eq!(fx2.opacity, 1.0, ":44928 — the reversed lerp at the shield edge");
        // One more fall: ai2 = 24 → the shield drops.
        let fx3 = ai_065_butterflies(
            EMPRESS_BUTTERFLY, &mut n2, &w, &c2, &mut rng,
            &mut DustPool::new(), &spawn_ctx(),
        );
        assert_eq!(n2.ai[2], 24.0);
        assert!(!n2.dont_take_damage, "24 < 25 — hittable again");
        // GetLerpValue(60, 25, 24) = 36/35 > 1 → clamped to 1.
        assert_eq!(fx3.opacity, 1.0, ":44928 — the clamped lerp");
    }

    /// `NPC.cs:44929-44953` — the 661 dust chain's RNG census: the
    /// Next(5) gate, the NewDust internals, the circular offset (angle
    /// then radial), the velocity NextFloat, the fadeIn NextFloat, and
    /// the CloneDust internals — mirrored on a twin pool + stream.
    #[test]
    fn w4_d30_s65_the_empress_dust_chain_matches_the_mirrored_stream() {
        let w = air();
        let players = [cand(1620.0, 1500.0)];
        let seed = seed_first_zero(5);
        let mut n = fly(1600.0, 1500.0);
        n.target = 0;
        n.ai[2] = 10.0; // fading, > 0
        let mut c = ctx(&players, &[]);
        c.target_zone_hallow = true; // the fall path keeps ai2 > 0
        let mut rng = UnifiedRandom::new(seed as i32);
        let mut pool = DustPool::new();
        let sc = spawn_ctx();
        ai_065_butterflies(EMPRESS_BUTTERFLY, &mut n, &w, &c, &mut rng, &mut pool, &sc);

        // The mirror: the same draws in the same order on a twin.
        let mut mrng = UnifiedRandom::new(seed as i32);
        let mut mpool = DustPool::new();
        assert_eq!(mrng.next_max(5), 0, "the gate");
        let opacity = get_lerp_value(60.0, 25.0, 9.0, true); // ai2 fell 10→9
        let num6 = 0.9 + (0.6 - 0.9) * opacity;
        let idx = mpool.new_dust(
            &mut mrng, &sc, &w, glam::Vec2::new(1600.0, 1500.0),
            10, 10, 267, 0.0, 0.0, 0, hsl_color_scaled(0.0, 0.5), 1.0,
        );
        {
            let d = mpool.slot_mut(idx);
            let (ox, oy) = next_vector2_circular(&mut mrng, 10.0, 10.0);
            d.position = glam::Vec2::new(1605.0 + ox, 1505.0 + oy);
            d.velocity *= mrng.next_float() * 0.8;
            d.no_gravity = true;
            d.fade_in = 0.6 + mrng.next_float() * 0.7 * num6;
            d.scale = 0.35;
        }
        if idx != DUST_SENTINEL {
            let cl = mpool.clone_dust(&mut mrng, &sc, &w, idx);
            let d = mpool.slot_mut(cl);
            d.scale /= 2.0;
            d.fade_in *= 0.85;
            d.color = Color::new(127, 127, 127, 127);
        }
        assert_eq!(rng, mrng, "the dust chain's draw order, pinned");
        // The spawned dust carries the post-writes.
        let d = pool.slot(idx);
        assert!(d.no_gravity, ":44941");
        assert_eq!(d.scale, 0.35, ":44943");
    }

    /// `NPC.cs:45108-45118` — the 653 tail: the netOffset round-trip,
    /// the constant ember light at the truncated centre tile, and the
    /// 1/60-gated type-6 dust (alpha 254, velocity zeroed) — the gate
    /// draw happens EVERY 653 tick, the dust internals only on 0.
    #[test]
    fn w4_d30_s65_the_hell_butterfly_tail_lights_and_gates_the_dust() {
        let w = air();
        let players = [cand(1620.0, 1500.0)];
        let mut n = fly(1600.0, 1500.0);
        n.net_offset = glam::Vec2::new(3.0, -2.0);
        let mut rng = UnifiedRandom::new(3);
        let mut mirror = rng.clone();
        let gate = mirror.next_max(60);
        let mut pool = DustPool::new();
        let sc = spawn_ctx();
        let fx = ai_065_butterflies(
            HELL_BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng, &mut pool, &sc,
        );
        // The light reads the netOffset-SHIFTED centre (`:45110` before
        // `:45111`): cx = 1605+3 = 1608 → 100, cy = 1505-2 = 1503 → 93.
        assert_eq!(fx.light, Some((100, 93, [0.6, 0.3, 0.1])), ":45111");
        assert_eq!((n.x, n.y), (1600.0, 1500.0), "the netOffset round-trip, :45110/:45117");
        if gate == 0 {
            // The dust internals on the mirror (position carries the
            // offset at spawn time).
            let idx = pool.new_dust(
                &mut mirror, &sc, &w, glam::Vec2::new(1603.0, 1498.0),
                10, 10, 6, 0.0, 0.0, 254, Color::default(), 1.0,
            );
            pool.slot_mut(idx).velocity *= 0.0;
        }
        assert_eq!(rng, mirror, "the gate draw (+ internals on 0), in order");
    }

    /// `NPC.cs:45104-45107` — the 356 catchItem re-derive rides the
    /// `bug_net.rs::catch_item` seam: 1994 + ai[2] - 1.
    #[test]
    fn w4_d30_s65_the_butterfly_catch_item_rides_the_bug_net_seam() {
        assert_eq!(crate::bug_net::catch_item(356, 1.0), 1994);
        assert_eq!(crate::bug_net::catch_item(356, 6.0), 1999);
        assert_eq!(crate::bug_net::catch_item(356, 8.0), 2001);
        assert_eq!(crate::bug_net::catch_item(653, 0.0), 4845, "NPC.cs:17263");
        assert_eq!(crate::bug_net::catch_item(661, 0.0), 4961, "NPC.cs:17364");
    }

    /// The RNG-free idle: a fully-latched butterfly with live timers,
    /// no threats and no collideX draws NOTHING (the 356/444 path).
    #[test]
    fn w4_d30_s65_the_latched_idle_tick_draws_no_rng() {
        let w = air();
        let players = [cand(1620.0, 1500.0)];
        let mut n = fly(1600.0, 1500.0);
        let mut rng = UnifiedRandom::new(99);
        let untouched = rng.clone();
        for _ in 0..5 {
            ai_065_butterflies(
                GOLD_BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng,
                &mut DustPool::new(), &spawn_ctx(),
            );
        }
        assert_eq!(rng, untouched, "five idle ticks, zero draws");
    /// A stand-in butterfly body: the 10x10 box (`NPC.cs:13533-13534`)
    /// pinned via `size_override` until the kind's own `size()` lands;
    /// `ai[2]`/`ai[3]` pre-latched so the one-time draws stay out of the
    /// way unless a test wants them.
    fn flutter(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(crate::npc::NpcKind::Slime, x, y, 1, 0.0);
        n.size_override = Some((10.0, 10.0));
        n.hp = 5;
        n.max_hp = 5;
        n.ai[2] = 4.0;
        n.ai[3] = 1.0;
        n.local_ai[0] = 50.0; // a live rearm timer by default
        n.local_ai[1] = 5.0; // a live threat-pause by default
    fn butterfly(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(crate::npc::NpcKind::Butterfly, x, y, 1, 0.0);
        n.size_override = Some((10.0, 10.0));
        n
    }

    fn ctx<'a>(players: &'a [TargetCandidate], npcs: &'a [Ai65NpcView]) -> Ai65Ctx<'a> {
        Ai65Ctx { players, npcs, ticker: 0 }
    }

    /// `NPC.cs:44958-44970` — the variant ladder: Next(100) == 0 picks
    /// band 5 (ai[2] = 6), 1-2 → 1, 3-8 → 2, 9-18 → 7, 19-33 → 3,
    /// 34-52 → 6, >= 75 → 0, else 4. The roll draws exactly once and
    /// only while ai[2] == 0.
    #[test]
    fn the_variant_roll_maps_the_next100_ladder() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        // Find a seed whose first Next(100) is 0 (the rarest band).
        let mut seed = 0;
        loop {
            let mut probe = UnifiedRandom::new(seed);
            if probe.next_max(100) == 0 {
                break;
            }
            seed += 1;
        }
        let mut n = flutter(160.0, 160.0);
        n.ai[2] = 0.0;
        let mut rng = UnifiedRandom::new(seed);
        let mut mirror = rng.clone();
        let band = 1 + 5; // num16 == 0 → num15
        mirror.next_max(100);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.ai[2], band as f32, ":44969");
        assert_eq!(rng, mirror, "one Next(100), no other draw");
        // A second tick with ai[2] set draws nothing for the variant.
        let before = rng.clone();
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, before, "the roll is one-time");
    }

    /// `NPC.cs:44971-44974` — the scale latch: Next(75, 111) * 0.01f,
    /// once; `:45015` writes scale = ai[3] every tick.
    #[test]
    fn the_scale_latches_once_and_rides_every_tick() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.ai[3] = 0.0;
        let mut rng = UnifiedRandom::new(7);
        let mut mirror = rng.clone();
        let latch = mirror.next_range(75, 111) as f32 * 0.01;
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.ai[3], latch, ":44973");
        assert_eq!(n.scale, latch, ":45015");
        assert_eq!(rng, mirror, "one Next(75,111)");
    }

    /// `NPC.cs:44975-44978` — the rearm timer decrements every tick and
    /// re-rolls Next(90, 240) on expiry; the NEAR wander (the target
    /// within 700 px) latches localAI[3] = 1 and draws Next(26, 301) +
    /// Next(-100, 101) ×2 (`:45005-45007`).
    #[test]
    fn the_rearm_expiry_draws_the_near_wander_in_order() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.local_ai[0] = 1.0; // expires THIS tick (the decrement first)
        let mut rng = UnifiedRandom::new(11);
        let mut mirror = rng.clone();
        let timer = mirror.next_range(90, 240) as f32;
        let speed = mirror.next_range(26, 301) as f32 * 0.01;
        let ax = mirror.next_range(-100, 101);
        let ay = mirror.next_range(-100, 101);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.local_ai[0], timer, ":44978");
        assert_eq!(n.local_ai[3], 1.0, ":45004");
        let norm = speed / ((ax * ax + ay * ay) as f64).sqrt() as f32;
        assert_eq!(n.ai[0], ax as f32 * norm, ":45009");
        assert_eq!(n.ai[1], ay as f32 * norm, ":45010");
        assert!(n.net_update, ":45012");
        assert_eq!(rng, mirror, "the near-wander draw order");
    }

    /// `NPC.cs:45016-45018` — the 1/60 blend: (v * 59 + aim) / 60, f32.
    #[test]
    fn the_velocity_blends_one_sixtieth_toward_the_aim() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.vx = 2.0;
        n.vy = -1.0;
        n.ai[0] = 5.0;
        n.ai[1] = 3.0;
        let mut rng = UnifiedRandom::new(3);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.vx, (2.0f32 * 59.0 + 5.0) / 60.0, ":45017");
        assert_eq!(n.vy, (-1.0f32 * 59.0 + 3.0) / 60.0, ":45018");
    }

    /// `NPC.cs:45019-45035` — the THREE-tile down-scan: each blocking
    /// tile flips the Y aim and damps vy ×0.9 INSIDE the loop (two
    /// stacked solid tiles → two flips cancel, two damps compound).
    #[test]
    fn the_down_scan_damps_once_per_blocking_tile() {
        let mut w = air();
        // The butterfly's centre tile column: centre (165, 165) → tile
        // (10, 10). Stack solid tiles at rows 10 and 11.
        for ty in [10, 11] {
            let i = w.index(10, ty);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.vy = 2.0;
        n.ai[1] = 4.0;
        let mut rng = UnifiedRandom::new(5);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        // Two flips: 4 → -4 → 4; two damps: 2 * 0.9 * 0.9 (the blend ran
        // first: vy = (2*59 + 4)/60 before the scan).
        let blended = (2.0f32 * 59.0 + 4.0) / 60.0;
        assert_eq!(n.vy, blended * 0.9 * 0.9, ":45029-45032");
        assert_eq!(n.ai[1], 4.0, "two flips cancel");
    }

    /// `NPC.cs:45058-45088` — the threat flee: every 15th tick a live
    /// hostile within Hitbox.Distance <= 100 pushes the velocity by
    /// DirectionFrom * 2 (one threat → the full away-vector ×2).
    #[test]
    fn the_threat_scan_pushes_away_from_a_nearby_hostile() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.local_ai[1] = 1.0; // expires THIS tick → the scan runs
        n.vx = 0.0;
        n.vy = 0.0;
        n.ai[0] = 0.0;
        n.ai[1] = 0.0;
        // A hostile 50 px to the RIGHT: its centre (215, 165), our
        // centre (165, 165) → DirectionFrom = (-1, 0).
        let threat = Ai65NpcView {
            active: true,
            damage: 10,
            friendly: false,
            x: 205.0,
            y: 155.0,
            width: 20,
            height: 20,
        };
        let mut rng = UnifiedRandom::new(9);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[threat]), &mut rng);
        assert_eq!(n.local_ai[1], 15.0, ":45064");
        assert_eq!(n.vx, -2.0, ":45082 — away ×2");
        assert_eq!(n.vy, 0.0);
        // A friendly neighbour never scares (the !friendly gate).
        let mut n2 = flutter(160.0, 160.0);
        n2.local_ai[1] = 1.0;
        let friendly = Ai65NpcView { friendly: true, ..threat };
        ai_065_step(BUTTERFLY, &mut n2, &w, &ctx(&players, &[friendly]), &mut rng);
        assert_eq!(n2.vx, 0.0, ":45074 — the !friendly gate");
    }

    /// `NPC.cs:45089-45093` — the wall bounce: collideX turns the aim
    /// away from the velocity's sign and kicks vx ×-0.2.
    #[test]
    fn the_wall_bounce_flips_the_aim_and_kicks_back() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.collide_x = true;
        n.vx = 1.0;
        n.ai[0] = 3.0;
        let mut rng = UnifiedRandom::new(13);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        // The blend ran first: vx = (1*59 + 3)/60 > 0 → aim = -|3|.
        let blended = (1.0f32 * 59.0 + 3.0) / 60.0;
        assert_eq!(n.vx, blended * -0.2, ":45092");
        assert_eq!(n.ai[0], -3.0, ":45091");
    }

    /// `NPC.cs:45094-45101` — the facing: two separate ifs; vx == 0
    /// keeps the old direction.
    #[test]
    fn the_facing_follows_the_velocity_sign_only() {
        let w = air();
        let players = [cand(160.0, 160.0)];
        let mut n = flutter(160.0, 160.0);
        n.facing = 1;
        n.vx = -1.0;
        n.ai[0] = -60.0; // the blend keeps vx negative
        let mut rng = UnifiedRandom::new(17);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.facing, -1, ":45096");
        // Zero velocity: neither if fires.
        let mut n2 = flutter(160.0, 160.0);
        n2.facing = -1;
        n2.vx = 0.0;
        n2.ai[0] = 0.0;
        ai_065_step(BUTTERFLY, &mut n2, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n2.facing, -1, "vx == 0 keeps the direction");
    }
}
}
    /// The `:44969` ladder as a pure fn — the witness pins every bucket
    /// boundary against the citation (the arm's own copy is inline).
    fn colour_pick(num16: i32) -> i32 {
        if num16 == 0 {
            5
        } else if num16 < 3 {
            1
        } else if num16 < 9 {
            2
        } else if num16 < 19 {
            7
        } else if num16 < 34 {
            3
        } else if num16 < 53 {
            6
        } else if num16 >= 75 {
            0
        } else {
            4
        }
    }

    #[test]
    fn the_colour_roll_ladder_maps_the_cited_buckets() {
        // `:44968-44969` — Next(100): 0→style 6, 1..2→2, 3..8→3,
        // 9..18→8, 19..33→4, 34..52→7, 53..74→5, 75..99→1.
        for (roll, style) in [
            (0, 6),
            (1, 2),
            (2, 2),
            (3, 3),
            (8, 3),
            (9, 8),
            (18, 8),
            (19, 4),
            (33, 4),
            (34, 7),
            (52, 7),
            (53, 5),
            (74, 5),
            (75, 1),
            (99, 1),
        ] {
            assert_eq!(1 + colour_pick(roll), style, "roll {roll} bucket");
        }
    }

    #[test]
    fn the_first_tick_colour_style_rides_the_streams_first_draw() {
        // The arm's ai[2] must equal 1 + ladder(the stream's first
        // Next(100)) — the colour roll is the FIRST draw of a fresh tick.
        let w = air();
        let players = [cand(2000.0, 1500.0)];
        let mut n = butterfly(1000.0, 1500.0);
        let mut rng = UnifiedRandom::new(0);
        let mut mirror = rng.clone();
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        let first = mirror.next_max(100);
        assert_eq!(n.ai[2], (1 + colour_pick(first)) as f32);
        assert!((1.0..=8.0).contains(&n.ai[2]));
    }

    #[test]
    fn the_first_tick_draws_colour_then_scale_then_rearm_in_order() {
        // `:44968` Next(100), `:44973` Next(75,111), `:44978`
        // Next(90,240) — the rearm fires on tick 1 (localAI[0] 0 → -1),
        // then the NEAR wander (the target is close): Next(26,301),
        // Next(-100,101) ×2. Six draws, no more.
        let w = air();
        let players = [cand(1010.0, 1500.0)];
        let mut n = butterfly(1000.0, 1500.0);
        let mut rng = UnifiedRandom::new(123);
        let mut mirror = rng.clone();
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        let _colour = mirror.next_max(100);
        let scale = mirror.next_range(75, 111);
        let timer = mirror.next_range(90, 240);
        let _speed = mirror.next_range(26, 301);
        let _dx = mirror.next_range(-100, 101);
        let _dy = mirror.next_range(-100, 101);
        assert_eq!(rng, mirror, "the arm's draw order drifted");
        assert_eq!(n.ai[3], scale as f32 * 0.01);
        assert_eq!(n.scale, n.ai[3]);
        assert_eq!(n.local_ai[0], timer as f32);
        assert_eq!(n.local_ai[3], 1.0, "the near latch sticks");
        assert!(n.net_update);
    }

    #[test]
    fn the_far_wander_draws_the_speed_ladder_and_the_conditional_bias() {
        // `:44981-45000` — a target > 1000 px away with localAI[3] == 0:
        // Next(50,151) THEN Next(150,201) (both draws — the ladder
        // overwrites), Next(100,251), Next(-50,51), and the conditional
        // Next(100,251) when the butterfly sits below player.Y - 100.
        let w = air();
        let players = [cand(3000.0, 1000.0)];
        let mut n = butterfly(1000.0, 1500.0);
        let mut rng = UnifiedRandom::new(7);
        let mut mirror = rng.clone();
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        let _colour = mirror.next_max(100);
        let _scale = mirror.next_range(75, 111);
        let _timer = mirror.next_range(90, 240);
        let _base = mirror.next_range(50, 151);
        let speed = mirror.next_range(150, 201);
        let dx = mirror.next_range(100, 251);
        let mut dy = mirror.next_range(-50, 51);
        dy -= mirror.next_range(100, 251); // y 1500 > 1000 - 100
        assert_eq!(rng, mirror, "the far ladder's draw order drifted");
        let k = speed as f32 * 0.01
            / ((dx * dx + dy * dy) as f64).sqrt() as f32;
        assert_eq!(n.ai[0], dx as f32 * k);
        // The blended vy is negative (the bias pulls the aim down-hard),
        // so the 30-tile up-scan finds no solid in the air world and
        // flips the aim's sign ONCE (`:45049-45056`) before the write-back.
        assert_eq!(n.ai[1], -(dy as f32 * k));
        assert_eq!(n.local_ai[3], 0.0, "the far branch keeps the latch clear");
    }

    #[test]
    fn the_blend_is_one_sixtieth_and_the_wall_bounce_kicks_back() {
        // `:45016-45018` — (v * 59 + aim) / 60; `:45089-45093` — the
        // collideX aim flip + the ×-0.2 kick.
        let w = air();
        let players = [cand(1010.0, 1500.0)];
        let mut n = butterfly(1000.0, 1500.0);
        n.ai[2] = 4.0; // the colour roll already latched
        n.ai[3] = 1.0; // the scale latch already latched
        n.local_ai[0] = 50.0; // no rearm this tick
        n.ai[0] = 6.0;
        n.ai[1] = -3.0;
        n.vx = 1.0;
        n.vy = 0.0;
        n.collide_x = true;
        let mut rng = UnifiedRandom::new(0);
        let mut mirror = rng.clone();
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(rng, mirror, "a latched tick draws NOTHING");
        // The blend ran BEFORE the bounce: vx = (1*59 + 6)/60, then
        // ×-0.2; the aim flips to -|6| (vx >= 0 at the bounce read).
        let blended = (1.0f32 * 59.0 + 6.0) / 60.0;
        assert_eq!(n.vx, blended * -0.2);
        assert_eq!(n.ai[0], -6.0);
        // The blended vy = -0.05 < 0 → the up-scan's single aim flip
        // (`:45049-45056`) + the ×0.9 velocity damp fire in the air world.
        let blended_vy = (0.0f32 * 59.0 + -3.0) / 60.0;
        assert_eq!(n.ai[1], 3.0);
        assert_eq!(n.vy, blended_vy * 0.9);
    }

    #[test]
    fn the_threat_flee_averages_the_push_and_caps_at_16() {
        // `:45058-45088` — localAI[1] == 0 arms the 15-tick scan; one
        // hostile view 50 px to the right pushes LEFT (DirectionFrom is
        // own - threat), ×2, and the 16-cap normalizes.
        let w = air();
        let players = [cand(1010.0, 1500.0)];
        let mut n = butterfly(1000.0, 1500.0);
        n.ai[2] = 4.0;
        n.ai[3] = 1.0;
        n.local_ai[0] = 50.0;
        n.local_ai[1] = 0.0;
        // ai[0]/ai[1] stay 0 → the blended vy is 0, so neither tile scan
        // fires (both gate on vy != 0) and the cap is a pure X normalize.
        n.vx = -20.0; // already over the cap after the push
        let views = [Ai65NpcView {
            active: true,
            damage: 10,
            friendly: false,
            x: 1050.0,
            y: 1500.0,
            width: 20,
            height: 20,
        }];
        let mut rng = UnifiedRandom::new(0);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &views), &mut rng);
        assert_eq!(n.local_ai[1], 15.0);
        // The blended vx = -20*59/60 = -19.6667; the push is (-1,0)*2 =
        // -2 → -21.6667; the blended vy stays 0 (ai[1] 0), so the cap
        // normalizes to exactly (-16, 0).
        assert_eq!(n.vx, -16.0);
        assert_eq!(n.vy, 0.0);
    }

    #[test]
    fn the_facing_follows_the_velocity_sign_only() {
        // `:45094-45101` — two separate ifs; vx == 0 keeps the old
        // direction (the firefly arm's shared tail).
        let w = air();
        let players = [cand(1010.0, 1500.0)];
        let mut n = butterfly(1000.0, 1500.0);
        n.ai[2] = 4.0;
        n.ai[3] = 1.0;
        n.local_ai[0] = 50.0;
        n.local_ai[1] = 5.0;
        n.facing = 1;
        let mut rng = UnifiedRandom::new(0);
        ai_065_step(BUTTERFLY, &mut n, &w, &ctx(&players, &[]), &mut rng);
        assert_eq!(n.facing, 1, "a zero-velocity tick keeps the direction");
    }

    /// SEAL (research/338 §4, renumbered 281→338 per §8.5): the 444 catch row rides the gold range —
    /// `NPC.cs:14287` writes catchItem 2891, `bug_net.rs:152`'s
    /// `442..=448 => 2889 + type - 442` covers it (2889 + 2).
    #[test]
    fn w4_d30_seal_the_gold_butterfly_catch_row_rides_the_gold_range() {
        assert_eq!(crate::bug_net::catch_item(GOLD_BUTTERFLY, 0.0), 2891);
        // The gold row ignores ai[2] (only 356 re-derives, `:45106`).
        assert_eq!(crate::bug_net::catch_item(GOLD_BUTTERFLY, 7.0), 2891);
    }

    /// SEAL (research/338 §4, renumbered 281→338 per §8.5): the HellButterfly's item is lava bait
    /// (`ItemID.cs:1306`), the GoldButterfly's is not — the plain net
    /// burns on 653's catch, never on 444's.
    #[test]
    fn w4_d30_seal_the_hell_butterfly_is_lava_bait_the_gold_is_not() {
        let hell_item = crate::bug_net::catch_item(HELL_BUTTERFLY, 0.0);
        let gold_item = crate::bug_net::catch_item(GOLD_BUTTERFLY, 0.0);
        assert!(crate::bug_net::is_lava_bait(hell_item), "4845");
        assert!(!crate::bug_net::is_lava_bait(gold_item), "2891");
    }

    /// SEAL (research/338 §3c/§3d, renumbered 281→338 per §8.5): the g65 "Kill-309" arm is projectile
    /// 309 (FrostBlastFriendly) and d77's 661 cue is the Onyx Black Bolt —
    /// neither shares an id space with this module's NPC kinds. Pin the
    /// projectile const so a future census cannot fold the axes.
    #[test]
    fn w4_d30_seal_the_g65_309_and_d77_661_are_other_id_spaces() {
        assert_eq!(crate::projectile::FROST_BLAST_FRIENDLY, 309);
        // The module's census stays NPC-side (NPC.cs SetDefaults rows).
        assert_eq!(
            (BUTTERFLY, GOLD_BUTTERFLY, HELL_BUTTERFLY, EMPRESS_BUTTERFLY),
            (356, 444, 653, 661)
        );
        assert_ne!(EMPRESS_BUTTERFLY as u16, crate::projectile::FROST_BLAST_FRIENDLY);
    }
}
