# W4-c3 merge-side E8 drops in npc/data.rs (329 s3): drop the dup identity
# arms (CursedSkull 355 -> d34@267, GiantCursedSkull 289 -> d38@289);
# WaterBoltMimic 694 keeps; shared stat arms re-keyed onto the canonical
# variants (they are the same variant names post-merge).
import io
p = "crates/tr-app/src/npc/data.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
def rep(old, new, n=1):
    global src
    import re as _re
    old = old.replace("\n", nl); new = new.replace("\n", nl)
    pat = _re.escape(old).replace(r"\-", "[-\u2013\u2014]")
    hits = list(_re.finditer(pat, src))
    assert len(hits) == n, f"count {len(hits)} != {n} for {old[:80]!r}"
    src = _re.sub(pat, lambda m: new, src)

# to_u16: drop the c3 dup block, keep 694 with a repointed comment
rep("""            // W4-c3 (the style-10 family): the CursedSkull's vanilla 34
            // is the Merchant's port-local slot above (`to_u16`'s
            // bijection guard), so it takes the next free port-local id
            // 355 (354 is W4-c4's in-flight FlyingFish claim - the
            // BlazingWheel 353 precedent); 289/694 keep their vanilla
            // ids (both free). `NPC.cs:9150-9165` / `:12753-12768` /
            // `:17737-17751`.
            NpcKind::CursedSkull => 355,
            NpcKind::GiantCursedSkull => 289,
            NpcKind::WaterBoltMimic => 694,
""", """            // W4-c3 (the style-10 family): the WaterBoltMimic keeps its
            // vanilla 694 (`NPC.cs:17737-17751`). The CursedSkull and
            // GiantCursedSkull ports dropped per the d37 E8 tables
            // (research/329 s3) - d34@267 / d38@289 own them above.
            NpcKind::WaterBoltMimic => 694,
""")
# from_u16: same drop
rep("""            // W4-c3 (the style-10 family): 355 is the CursedSkull's
            // port-local id (the vanilla 34 slot is the Merchant's);
            // 289/694 keep their vanilla ids (`to_u16` above).
            355 => NpcKind::CursedSkull,
            289 => NpcKind::GiantCursedSkull,
            694 => NpcKind::WaterBoltMimic,
""", """            // W4-c3: 694 keeps its vanilla id (`to_u16` above); the
            // 355/289 dup arms dropped per the d37 E8 tables (d34/d38 own).
            694 => NpcKind::WaterBoltMimic,
""")
# hit_sound: drop the shared 34/289 arm (d34/d38 rows identical), keep 694
rep("""            // W4-c3 (the style-10 family): 34/289 NPCHit2
            // (`NPC.cs:9158`/`:12761`), 694 DD2_BookStaffCast
            // (`:17745` - the `CreateTrackable("dd2_book_staff_cast", 3)`
            // style, `SoundID.cs:1676`; the `_0` pin, the DD2 families'
            // convention).
            NpcKind::CursedSkull | NpcKind::GiantCursedSkull => "NPC_Hit_2",
            NpcKind::WaterBoltMimic => "dd2_book_staff_cast_0",
""", """            // W4-c3: 694 DD2_BookStaffCast (`NPC.cs:17745` - the
            // `CreateTrackable("dd2_book_staff_cast", 3)` style,
            // `SoundID.cs:1676`; the `_0` pin, the DD2 families'
            // convention). The 34/289 NPCHit2 rows are d34/d38's above.
            NpcKind::WaterBoltMimic => "dd2_book_staff_cast_0",
""")
# death_sound
rep("""            // W4-c3 (the style-10 family): 34/289 NPCDeath2
            // (`NPC.cs:9159`/`:12762`), 694 NPCDeath52 (`:17746`).
            NpcKind::CursedSkull | NpcKind::GiantCursedSkull => "NPC_Killed_2",
            NpcKind::WaterBoltMimic => "NPC_Killed_52",
""", """            // W4-c3: 694 NPCDeath52 (`NPC.cs:17746`); the 34/289
            // NPCDeath2 rows are d34/d38's above.
            NpcKind::WaterBoltMimic => "NPC_Killed_52",
""")
# no_gravity group: keep-both composition - drop the two dups from the
# group (d38's GiantCursedSkull + d34's CursedSkull singles own them),
# keep WaterBoltMimic as its own arm with the citation.
rep("""            // W4-c3 (the style-10 family): 34/289/694 all write
            // `noGravity = true` + kb 0.2 (`NPC.cs:9160-9163`/
            // `:12763-12766`/`:17747-17750`).
            | NpcKind::CursedSkull
            | NpcKind::GiantCursedSkull
            | NpcKind::WaterBoltMimic
""", """            // W4-c3: the WaterBoltMimic writes `noGravity = true` +
            // kb 0.2 (`NPC.cs:17747-17750`); the 34/289 rows are
            // d34/d38's below (E8 keep-both).
            | NpcKind::WaterBoltMimic
""")
# ai_style: canonical kinds already have 10 arms (d34/d38); keep 694 only
rep("""            // W4-c3 (the style-10 family): aiStyle 10 - NPC.cs:9154
            // (the CursedSkull 34), :12757 (the GiantCursedSkull 289),
            // :17741 (the WaterBoltMimic 694) - the census is exactly
            // these three.
            NpcKind::CursedSkull | NpcKind::GiantCursedSkull | NpcKind::WaterBoltMimic => 10,
""", """            // W4-c3 (the style-10 family): aiStyle 10 - NPC.cs:17741
            // (the WaterBoltMimic 694); the 34/289 aiStyle-10 rows are
            // d34/d38's above (E8 keep-both, same axis machine).
            NpcKind::WaterBoltMimic => 10,
""")
# size
rep("""            // W4-c3 (the style-10 family): 34 26x28 (`NPC.cs:9152-9153`),
            // 289 44x44 (`:12755-12756`), 694 44x44 (`:17739-17740`).
            NpcKind::CursedSkull => (26.0, 28.0),
            NpcKind::GiantCursedSkull | NpcKind::WaterBoltMimic => (44.0, 44.0),
""", """            // W4-c3: 694 44x44 (`NPC.cs:17739-17740`); the 34/289 size
            // rows are d34/d38's above (identical values, E8 keep-both).
            NpcKind::WaterBoltMimic => (44.0, 44.0),
""")
# cs_id: drop CursedSkull dup (d34 has 34), GiantCursedSkull has no main
# arm? check below - d38 may lack it; keep GiantCursedSkull => 289 (main
# cs_id has no 289 arm - this ADDS the canonical row, superset-module).
rep("""            // W4-c3 (the style-10 family): the vanilla `NPC.type` ids
            // (`NPCID.cs:10745`/`:11255`/`:12065`). The CursedSkull's
            // protocol id is port-local 355, so its C#-type consumers
            // (the style-10 machine's type gates) read 34 here.
            NpcKind::CursedSkull => 34,
            NpcKind::GiantCursedSkull => 289,
            NpcKind::WaterBoltMimic => 694,
""", """            // W4-c3 (the style-10 family): the vanilla `NPC.type` ids
            // (`NPCID.cs:11255`/`:12065`); the CursedSkull's 34 row is
            // d34's above. The GiantCursedSkull 289 row rides here (the
            // superset-module rule - d38 carries no cs_id arm for it).
            NpcKind::GiantCursedSkull => 289,
            NpcKind::WaterBoltMimic => 694,
""")
# max_hp
rep("""            // W4-c3 (the style-10 family): 34 40 (`NPC.cs:9157`),
            // 289 400 (`:12760`), 694 60 (`:17744`).
            NpcKind::CursedSkull => 40,
            NpcKind::GiantCursedSkull => 400,
            NpcKind::WaterBoltMimic => 60,
""", """            // W4-c3: 694 60 (`NPC.cs:17744`); the 34/289 lifeMax rows
            // are d34/d38's above (identical values, E8 keep-both).
            NpcKind::WaterBoltMimic => 60,
""")
# contact_damage
rep("""            // W4-c3 (the style-10 family): 34 35 (`NPC.cs:9155`),
            // 289 60 (`:12758`), 694 20 (`:17742`).
            NpcKind::CursedSkull => 35,
            NpcKind::GiantCursedSkull => 60,
            NpcKind::WaterBoltMimic => 20,
""", """            // W4-c3: 694 20 (`NPC.cs:17742`); the 34/289 damage rows
            // are d34/d38's above (identical values, E8 keep-both).
            NpcKind::WaterBoltMimic => 20,
""")
open(p, "w", encoding="utf8", newline="").write(src)
print("c3 data.rs E8 drops applied")
