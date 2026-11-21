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
