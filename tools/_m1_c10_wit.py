# Rebuild the c10 waterfowl defaults witness (the union interleaved it
# with the d42 batch witness body). Re-key the Duck quartet protocol pins.
import sys, re, subprocess
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
c10 = subprocess.run(["git","show","origin/task/W4-c10-style68-waterfowl:crates/tr-app/src/npc.rs"],capture_output=True).stdout.decode("utf8","replace")
# extract the c10 witness body
i = c10.index("fn w4_c10_the_waterfowl_defaults_match_the_csharp_rows")
j = c10.index("\n    }", i) + len("\n    }")
body = c10[i:j].replace("\r\n", "\n").replace("\n", nl)
# re-key the Duck quartet protocol pins (731-734)
body = body.replace(
    '            assert_eq!(kind.to_u16(), ty, "{kind:?} protocol id = vanilla");' + nl +
    '            assert_eq!(NpcKind::from_u16(ty), Some(kind), "{kind:?} decode");',
    '            // E8 merge renumber: the Duck quartet rides 731-734 (the' + nl +
    '            // d37 tables); the Seagull/Grebe pairs keep their vanilla ids.' + nl +
    '            let proto = match kind {' + nl +
    '                NpcKind::Duck => 731,' + nl +
    '                NpcKind::Duck2 => 732,' + nl +
    '                NpcKind::DuckWhite => 733,' + nl +
    '                NpcKind::DuckWhite2 => 734,' + nl +
    '                _ => ty,' + nl +
    '            };' + nl +
    '            assert_eq!(kind.to_u16(), proto, "{kind:?} protocol id");' + nl +
    '            assert_eq!(NpcKind::from_u16(proto), Some(kind), "{kind:?} decode");')
# replace the broken witness in the current file
i2 = src.index("fn w4_c10_the_waterfowl_defaults_match_the_csharp_rows")
# find the end of the broken witness (the next #[test] or fn after the d42 batch tail)
j2 = src.index("    /// W4-c10 — the axis claim:", i2)
# back up to the doc comment start
k2 = src.rfind(nl, 0, j2)
# the broken witness ends right before the c10 axis-claim doc
src = src[:i2] + body + nl + nl + src[j2:]
open(p, "w", encoding="utf8", newline="").write(src)
print("waterfowl witness rebuilt")
