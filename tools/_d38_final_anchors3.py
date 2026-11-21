# W4-d38 FINAL sheet 3 - append the canary row (one-shot).
import io
row = ('crates/tr-app/src/bestiary.rs|w4_d38_final_the_unlock_ladders_match_the_banner_chain|'
       'W4-d38 FINAL sheet 3 - the bestiary unlock census (research/125 s8): all 35 C# banner '
       'chains agree with the port kills_needed_for (Paladin 25 the one non-default, the other '
       '34 default 50), the tier thresholds pinned at 1/fifth/half/full; GAP-1 the critter '
       'sight-provider (pre-existing, the base Goldfish 55 row inherits it), GAP-2 rarity stars '
       'display-only (5 kinds), GAP-3 the 328 Hide extra card pinned for the wiring-round drop')
p = 'anchors.txt'
s = io.open(p, encoding='utf-8').read()
assert s.endswith('\n')
path, marker, _ = row.split('|', 2)
assert marker in io.open(path, encoding='utf-8').read(), 'marker missing'
io.open(p, 'a', encoding='utf-8', newline='').write(row + '\n')
b = io.open(p, 'rb').read().replace(b'\r\n', b'\n')
io.open(p, 'wb').write(b)
print('appended 1 canary row; marker verified; LF rows:', b.count(b'\n'))
