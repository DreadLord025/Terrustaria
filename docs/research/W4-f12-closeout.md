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

## Item 3 — buff-stack display: the stack-count SOURCE census (COMPLETE)

**The xN source is NOT buff stacks.** `DrawBuffIcon`'s text precedence
(`Main.cs:43316-43324`): (1) `BuffID.Sets.BuffTextHandlers[id].
HandleBuffText()` — all 28 handlers are
`CachedProjectileCounterBuffTextHandler` (`BuffID.cs:54-168`), which sums
`Main.LocalPlayer.ownedProjectileCounts` over its projectile list and
returns `"x" + num` when > 0 (`CachedProjectileCounterBuffTextHandler.cs:
12-30` — the sum is a float but the counts are ints, so the text is an
integer count); (2) else the duration over 2 ticks. Port: `buff_text` +
`BUFF_TEXT_HANDLERS` (`ui/resources.rs:411-454`), verified 1:1 in the
final-phase icon census (28 ids, exact projectile lists, the xN-over-
timer precedence witness). The handler-owned ids are exactly the
summon/minion families (the 28 SUMMON_OWNED overlap minus the non-handler
rows) — the count SOURCE is the per-tick projectile recount
(`update_projectile_caches`, C# `:12184-12215`), never `buffTime`.

**The per-buff stack semantics census (`AddBuff`, `Player.cs:5263-5449`
— there is NO stack-count field anywhere):**

| family | re-grant semantics | C# | port |
| --- | --- | --- | --- |
| DEFAULT (every id not below) | refresh-to-max: `buffTime[i] < time → = time`, never additive, never a 2nd slot | :5419-5421 | `buffs.rs:248-249` ✓ |
| Mana Sickness 94 | ADDITIVE `+= time` capped 600 | :5410-5417 + BuffID.cs:53 | `buffs.rs:242-247` ✓ (f49's drink re-arms 300/drink — stacks TIME) |
| Kite 383 | ADDITIVE capped 43200 | same | same ✓ |
| fed 26/206/207/332/333/334 | IsFedState wipe-then-add: the new tier REPLACES every fed slot | :5277-5286 | `buffs.rs:196-202` ✓ |
| light pets (12) / vanity pets (74) | family-exclusive wipe (the no-rescan skip quirk preserved) | :5377-5400 | `buffs.rs:259+` ✓ |
| flasks 71-79 (meleeBuff) | family-exclusive wipe WITH `i--` rescan | :5361-5375 | ✓ |
| Warmth 124 | side-effect: cuts existing 46 to max(5, t/4), 47 to max(5, t/2) | :5300-5323 | `buffs.rs:296+` ✓ |
| orb tiers 95-100/170-172 | ladder: clear lower rows + re-grant the new tier id at 5 ticks (scan pins) | :10342-10443 + :16366-16581 | g27 scan ✓ (the machines f50's) |
| nebula tiers 173-181 | pickup `NebulaLevelup` clears + re-grants tier+1; expiry steps DOWN (buffType--, time 480) instead of dying | :57030-57059 + :12070-12079 | g27 `nebula_buffs` ✓ (the pickup f49's) |
| mounts (66) | scan re-affirms `buffTime = 10` every tick | :9881-9884 | `player.rs:5897-5904` ✓ |
| TimeLeftDoesNotDecrease (28/334/29/159/150/93/348/366) | timer never ticks down | :9871 + BuffID.cs:47 | `buffs.rs:388-397` ✓ |

**Verdict: census COMPLETE, zero gaps.** No buff has an increment-style
stack counter in the C#; the display's only "xN" is the projectile-count
handler. The 94-additive semantics is the one place "stack" language
applies, and it stacks TIME (the tooltip's percentage derives from
`buffTime`, the final-phase tooltip census). Witness:
`w4_f12_closeout_regrant_is_refresh_except_the_two_additive_ids`
(buffs.rs, written not run).
