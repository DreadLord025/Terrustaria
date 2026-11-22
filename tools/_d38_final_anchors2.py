# W4-d38 FINAL sheet 2 - append the canary row (one-shot).
import io
row = ('crates/tr-app/src/npc.rs|w4_d38_final_the_seven_town_standins_are_the_wiring_gap|'
       'W4-d38 FINAL sheet 2 - the FindFrame status census (research/293 s7): 34 C# cases + '
       'the 328 no-case pin, 35/35 port arms, the 7 humanoid town stand-ins (287/294/295/296/'
       '322/323/324) pinned as the wiring-round gap list (no grounded flip, no air tail; the '
       'extra6/wave-B/extra7 modules supersede per the d44 7c430def6 master table)')
p = 'anchors.txt'
s = io.open(p, encoding='utf-8').read()
assert s.endswith('\n')
path, marker, _ = row.split('|', 2)
assert marker in io.open(path, encoding='utf-8').read(), 'marker missing'
io.open(p, 'a', encoding='utf-8', newline='').write(row + '\n')
print('appended 1 canary row; marker verified present')
