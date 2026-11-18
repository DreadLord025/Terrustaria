# INTENT: ROADMAP update for S1-I-4 wave C (task/S1-I-4c)

The S1-I-4 row (`docs/ROADMAP.md:454`) stays OPEN — this wave maps and ports
the PROJECTILE subsystem's cadence only. Append to the row's evidence (or a
sibling note) at the wave merge:

- **Wave C landed (task/S1-I-4c, pending merge):** the projectile `SendData`
  cadence is ported into the new disjoint module
  `crates/tr-app/src/app/net_proj_sync.rs` (the `ProjSync` engine + per-site
  send fns + the `step` hook), with 10 per-site tests
  (`app::net_proj_sync::tests::*`). Sites: spawn `Projectile.cs:11096`,
  kill `:77254-77257` (entry position `:70648`), the per-update budget
  `:17210-17244` (cost 5 / cap 60 / refill 1, per sub-step `:16676`, tail
  clear outside the owner gate), the section recheck `:17246-17256` +
  `NetMessage.cs:1774-1800`/`:1821-1823` (flag3 `:1777`), the AI immediate
  sites `:23744`/`:23760`/`:23780`/`:52472`/`:57818`/`:73866`/`:74845`, and
  the despawn mark `Main.cs:37899-37906`.
- **Still open for S1-I-4:** the live-loop swap (the 10 Hz
  `netglue.rs::projectile_sync_plan` diff still runs; the engine is
  dead-code parity until the waves merge — swapping both would double-send),
  the 337 `netUpdate = true` AI-arm producers, and waves A (world-item) /
  B (NPC). The message-id table (S1-I-1) is untouched.
