# W4-f12 BUFF CLOSEOUT — the f49d bridge hunk (2026-11-21)

Branch `task/W4-f12-buff-frame-wiring`, closeout phase (after 61724db38).
Code-only: witnesses written, not run.

## Item 1 — f49d nebula-exec status: NOT landed → publish the hunk

Read `origin/task/W4-f49-mana-regen-2` tip **6c0ee2aa2** ("W4-f49 residuals
witness fix: the cuffs end-to-end rig…"):

* The three inline scan arms are STILL LIVE in their `player.rs`
  (`self.nebula_level_mana = self.update_buffs_nebula_buffs(176, i)` :6864,
  life :6869, damage :6874) — the R3-exec deletion has NOT happened.
* `update_survival` is still called INSIDE their `update_with_grapple`
  (:7766) — the R3 step-4 lift has NOT happened.
* Their R3 recipe (research/85 ADDENDUM, ce71ba61a) is unchanged since
  the sweep-phase read: keep f12's g27 `apply_orb_arm` as the ONE tier
  writer, bridge `orb_scan.nebula_level_mana` → `Player.nebula_level_mana`
  in `fold_orb` (an ASSIGN), delete their three scan arms + the helper,
  lift `update_survival` after the wiring seam, keep their `items.rs`
  wholesale.

**f12-side action per the closeout contract: publish the exact hunk** (the
`Player.nebula_level_mana` field exists only on f49's tree, so the bridge
cannot compile here; it lands with the merge). The hunk, verbatim, for
`crates/tr-app/src/buff_frame_wiring.rs::fold_orb` (this branch :457-465):

```diff
 fn fold_orb(p: &mut Player, e: &OrbEffects) {
     p.beetle_buff |= e.beetle_buff;
+    // R3 bridge (f49 research/85): the ONE tier writer is `nebula_buffs`
+    // inside `apply_orb_arm` (OrbScanState, zero-seeded every tick); the
+    // regen read side reads the d117 Player field — ASSIGN, never +=.
+    p.nebula_level_mana = p.orb_scan.nebula_level_mana; // `:12072`
     p.melee_damage += e.melee_damage;
```

Semantics pinned by witness `w4_f12_closeout_orb_scan_is_assign_shaped`
(written not run): a buff-less frame after a tier-2 frame leaves
`orb_scan.nebula_level_mana == 0` (the reset + assign-derivation — a stale
field cannot survive), and the step-down edge fires exactly once per
writer (one dispatch: `is_orb_id` routing exclusivity, the final-phase
witness). The Life/Damage twins stay scan-local (their consumers are
`fold_orb`'s `buff_life_regen`/four-class adds; nothing else reads them —
R3 step 2 permits leaving the Player twins unbridged).

**Merge-order reminders (f49's R3 steps 1/3/4/5 — their side or the
merger's, recorded here so the hunk is not applied alone):** deleting
their scan arms WITHOUT this bridge strands the regen engine at 0;
applying this bridge WITHOUT deleting their arms double-writes the field
(their scan runs later in `update_with_grapple` than our fold seam —
last-writer-wins would mask the double-step on `buffTime == 2`, but the
tier assign would run twice). The `update_survival` lift also repairs
f12's own `buff_life_regen` feed (our fold lands after their regen read
today). Both halves land in the SAME merge commit.

## Item 2 — the buff-frame ANIMATION census

**The lane-name premise is FALSE against the source:** there is NO
`Main.buffFrame` and NO `buffFrameCounter` anywhere in the decompiled
tree (case-insensitive grep over `decompiled/` — zero hits). The
:43300-region (`DrawBuffIcon`, `Main.cs:43307-43368`) draws
`TextureAssets.Buff[num]` with the FULL-sheet rect
`Rectangle(0, 0, Width(), Height())` (`:43315`) — **all 400 buff icons
are static single-frame sheets; there is no icon-frame-cycle to wire.**
The only per-frame icon dynamics are the `buffAlpha` hover tint
(`:43332/:43353`, clamped [0.4, 1] `:43355-43361` — already ported 1:1,
final-phase icon census) and the icon TEXT (handler counter / duration —
items 3/4 below). `AssetInitializer.cs:420` loads one `Images\Buff_<id>`
per id; no frame metadata exists.

**The C# animated-buff list (the full census — buffs whose STATE drives a
frame animation elsewhere):**

| buff | anim state | writer (C#) | consumer (C#) | port status |
| --- | --- | --- | --- | --- |
| Ice Barrier 62 | `iceBarrierFrame`/`FrameCounter` (12 frames, advance on tick 3) | `UpdateBuffs` :10469-10478 (below-half-life gate :10464) | `PlayerDrawLayers.DrawPlayer_35_IceBarrier` :3918-3923 (`sourceRect.Y = height/12 * frame`) | SIM ported + wired (`buff_effects_boosters.rs:307-314` via the wiring's 62 arm; pair persists across ticks, seeded :167-170, folded :486-487); DRAW layer unported (player-draw lane) |
| Beetle Endurance 95-97 / Might 98-100 | `beetleFrame` (3 frames, every tick) + `beetlePos/Vel` orbits | `UpdateArmorSets_Always_Beetle` :16448-16496 (NOT UpdateBuffs — gated on `beetleDefense‖beetleOffense`) | `DrawPlayer_37_BeetleBuff` :4007-4028 (`Height()/3 * beetleFrame + 1`) | counters (`beetleOrbs`) ported (g27 scan); frame/orbits UNPORTED — f50's `update_armor_sets_always_beetle` ported only the counter-zero half (:16443-16446), the frame tick + `Next(-100,101)` orbit draws are the armor-set/visual lane's remainder (their doc note: "cosmetic and unported") |
| Solar Shield 170-172 | `solarShields` count selects the sheet (`Extra[61+shields-1]`), `solarShieldPos/Vel` orbits | `UpdateArmorSets_Always_Solar` region :16403-16428 | `DrawPlayer_26_SolarShield` :3026-3030 | count ported (g27 scan + f50 ladder); orbits/draw unported (same lane note) |
| Nebula tiers 173-181 | tier count → floating boost orbs (draw-side) | `UpdateBuffs_NebulaBuffs` :12070-12079 | player-draw nebula layers | tier ported (g27); draw unported |
| Titanium Storm 306 | render-target shader pulse (`PlayerTitaniumStormBuffTextureContent`, "TitaniumStorm" shader) | flag :10088, `Main.cs:33376-33381` request | render-target draw | flag ported (`has_titanium_storm_buff`, DEBUFF_DISPATCH 306); shader unported (render lane) |

**Verdict:** every frame-anim CONSUMER that exists in the port's scope is
wired: the 62 pair ticks through `update_buff_frame` and lands on the
live `Player.ice_barrier_frame{,_counter}` fields (the draw reader's
inputs). The 400-icons census needs NO cycle code — static is 1:1. The
beetle/solar/nebula orbit+frame draws are player-draw-layer work (their
sim counters are live), recorded as the remainder, not a wiring gap.

Witness: `w4_f12_closeout_ice_frame_persists_and_cycles_across_ticks`
(written not run) — 36 below-half ticks wrap the 12-frame cycle to
(0, 0), tick 37 resumes at counter 1 (persistence), the above-half delete
arm freezes the pair.
