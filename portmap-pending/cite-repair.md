# PORT_MAP pending entry — cite-repair

Comment-only citation repair of four C# ranges. The ports themselves are unchanged; each row corrects the cited C# lines to the code that is actually being mirrored.

| C# source (under `Terraria/`) | Rust target | State |
| --- | --- | --- |
| `GameDifficultyData.cs:81-92` (static ctor; `DebuffTimeMultiplier` on `:89`) | `crates/tr-app/src/buffs.rs` (`add_buff_determine_buff_time_to_add`) | repaired (cite-repair) |
| `WorldMap.cs:44-47` (`IsRevealed`) | `crates/tr-app/src/map.rs` (`tile_revealed`) | repaired (cite-repair) |
| `Projectile.cs:13904-14263` (per-type whip switch `:13904-13956` + `MinionAttackTargetNPC` write `:14257-14263`) | `crates/tr-app/src/projectile.rs` (`whip_on_hit`) | repaired (cite-repair) |
| `LiquidPlacement.cs:3-9` (enum declaration) | `crates/tr-core/src/tileobject.rs` (`LiquidPlacement`) | repaired (cite-repair) |

---

**Notes:**
- Four ranges were wrong before: `96-100` in a 93-line file, `406-409` in a 291-line file and `5-10` in a 9-line file were all out of range; `14258-13956` was reversed.
- Comment-only: no code, no tests touched, nothing built.
