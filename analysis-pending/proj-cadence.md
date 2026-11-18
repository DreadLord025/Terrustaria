# INTENT: ANALYSIS note for S1-I-4 wave C (task/S1-I-4c)

Append near the "Projectile + strike sync (M6-NET-5)" section
(`docs/ANALYSIS.md:9364`):

## Projectile SendData cadence (S1-I-4 wave C)

`app/net_proj_sync.rs` ports the projectile sync's three cadences as pure
functions (`ProjSync`), disjoint from `netglue.rs` (wave A) and
`net_npc_sync.rs` (wave B):

- **Spawn** (`Projectile.cs:11096`): one `27` per `NewProjectile`, after
  `FinalizeProjectile` + the modifier; the only gate is `SendData`'s
  `netMode == 0` return (`NetMessage.cs:83-86`).
- **Kill** (`Projectile.cs:77254-77257`): one `29` when
  `owner == Main.myPlayer && netMode != 0`, carrying the **entry** position
  snapshot (`:70648`), not the death position.
- **Per-update** (`Projectile.cs:17210-17244`): the `netUpdate`/`netSpam`
  budget — cost 5, cap 60, refill 1/tick — runs **per sub-step** (inside the
  `while` at `:16676`, `1 + extraUpdates` times); the `netUpdate = false`
  tail (`:17244`) is outside the loop and the owner gate, so it clears every
  slot's flag every tick. A parked send (`netUpdate2`) promotes and retries
  next tick.
- **Section culling** (`NetMessage.cs:1774-1800`): flag3 (`type == 12 ||
  projPet || aiStyle == 11 || netImportant`, `:1777`) bypasses culling;
  skipped clients are remembered per slot and re-served by
  `RecheckSectionsForSkippedUpdates` (`Projectile.cs:17246-17256`, host
  only), the targeted send clearing the flag (`NetMessage.cs:1821-1823`).

**Deliberate deviations (until the wave merge):**

1. The engine is **dead-code parity** (the R-36-2 film-framework pattern):
   the live loop still runs `netglue.rs::projectile_sync_plan`'s 10 Hz diff.
   Wiring both would double-send; the swap is the merge follow-up.
2. The 337 C# `netUpdate = true` AI-arm producers are not wired — the port's
   AI arms skip the writes today; `ProjSync::mark_dirty` is the seam.
3. `bannerIdToRespondTo` rides as 0 (no pool field; the banner response is
   unported) and `Main.projPet` is a `false` parameter (no port table yet).
4. The per-slot bookkeeping lives beside the pool (`ProjSync`) instead of on
   `Projectile` so `projectile.rs` stays untouched by this wave.

Verification: `cargo test -p tr-app net_proj_sync` — 10 per-site tests green
(`test result: ok. 10 passed; 0 failed`).
