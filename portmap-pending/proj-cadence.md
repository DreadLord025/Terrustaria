# PORT_MAP pending entry — S1-I-4 wave C (projectile SendData cadence)

Append to the `Net/*` row's family (the `NetMessage.cs` row at
`docs/PORT_MAP.md:332`):

| C# source (under `Terraria/`) | Rust target | State |
| --- | --- | --- |
| `Projectile.cs:11096` (NewProjectile spawn 27), `:77254-77257` + `:70648` (Kill 29 + entry position), `:17210-17244` (the netUpdate/netSpam budget, per sub-step `:16676`), `:17246-17256` (RecheckSectionsForSkippedUpdates), `:23744`/`:23760`/`:23780`/`:52472`/`:57818`/`:73866`/`:74845` (AI immediate 27s); `Main.cs:37899-37906` (the yoyo despawn mark); `NetMessage.cs:776-841`/`:852-855` (the 27/29 writer bodies), `:1774-1800`/`:1821-1823` (flag3 section culling + the targeted-send flag clear) | `crates/tr-app/src/app/net_proj_sync.rs` (`ProjSync`: `spawn_sync`, `kill_sync`, `update_cadence`, `recheck_sections`, `note_broadcast`, `sync_now`, `despawn_mark_applies`, `state_of`, `sync_ignores_sections`, the `step` hook; 10 per-site tests) | ported, dead-code parity (S1-I-4c; the live-loop swap from `netglue.rs::projectile_sync_plan` + the 337 `netUpdate = true` AI producers are the wave-merge follow-up) |
