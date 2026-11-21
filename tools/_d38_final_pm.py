# W4-d38 FINAL - append the PORT_MAP census row (CRLF, byte-exact).
import io
row = ('| `NPC.FindFrame` (`NPC.cs:66684-77370`) + the bestiary unlock chain '
       '(`CommonEnemyUICollectionInfoProvider.cs:21-75`, `BannerSystem.cs:341/:403`, '
       '`ItemID.cs:1303-1304`, `CritterUICollectionInfoProvider.cs:14-20`) + '
       '`AssetInitializer.cs:127-129` / `Main.LoadNPC` (`Main.cs:59084-59090`) — the '
       'd38 band 281-337 censuses | `crates/tr-app/src/npc.rs::find_frame` (35/35 arms; '
       'the 7 humanoid town stand-ins the wiring-round gap list) + '
       '`crates/tr-app/src/bestiary.rs::unlock_state` / `tr-core/bestiary.rs::kills_needed_for` '
       '(all 35 banner chains agree; GAP-1 critter sight-provider, GAP-2 rarity stars, '
       'GAP-3 the 328 Hide extra card) + `tools/_d38_final_lazy.js` (the 113 §10 lazy '
       're-projection: E35 8115 / L91 1554 one page) | census FINAL (W4-d38 2026-11-21, '
       'code-only — research/125 §§6-10; the census scripts RUN, the 2 witnesses '
       '`w4_d38_final_the_seven_town_standins_are_the_wiring_gap` + '
       '`w4_d38_final_the_unlock_ladders_match_the_banner_chain` WRITTEN NOT RUN) |\n')
p = 'docs/PORT_MAP.md'
b = io.open(p, 'rb').read()
assert b.endswith(b'\r\n')
io.open(p, 'ab').write(row.replace('\n', '\r\n').encode('utf-8'))
print('PORT_MAP row appended (CRLF)')
