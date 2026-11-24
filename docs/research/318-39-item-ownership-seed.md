# 318 — W4-g48: the 39 ReleaseItemOwnership item-lane task-seed (FindOwner/22 scope)

Number **318** — RENAMED chain **245 → 282 → 318** (UNIFIED FINAL per
research/150 §9.2, d54 tip `b243c7086`, 2026-11-21). The 282 step was
this lane's contested self-rename (`9c776ca2f` 15:50:24) INTO f26's
§7.3-assigned 281–303 block (282 = f26 `68-findframe-census-addendum-7i.md`),
so §9 does NOT honor it: this doc takes the §9.2-assigned **318**.
**g48's 245 KEY WIN stands** (§7.1, pre-publish — the key verdict is
unaffected; only the doc filename renumbers, 245 stays winner-consumed).
Chain history: the initial 245 pick was made on the correction-phase
358-tip scan (pushed `2b61530b4` 14:30:16) BEFORE the research/150
round-2 refresh (d54 tip `72226e500`) was visible: that refresh
CONSUMED the whole 244–280 assignment block (245 = the d81/g53
`18b-element-census` fork copy per §6.3), so 245 was never free; a
fresh 362-tip sweep found 282 unclaimed at every tip (the only 280+
doc anywhere: c42's assigned `280-c42-wiring-seal.md`) and it was taken
per the round-2 cutoff note (post-sweep pushes self-adjudicate) — but
281–303 was RESERVED by §7.2/§7.3, the self-pick collided and §9.2
re-assigns **318**. The sibling c28 send-switch publish rides the
parallel chain 244 → 281 → **310** in the same round (281 = f26
addendum-7h; g48's 244 KEY WIN stands likewise). Winner context per
the refresh: 237 = e23, 238+239 = d42, 240–243 = g58; this lane's
f11-tail-fix chain 237→239→272.
Published by the g48 NET-APPLY-1 correction phase — the SEALED
research/17 tail's row-39 PARKED dependency hand-off (re-probe
citations only, never re-open scope).

## 1. Which lane owns it (the adjudication)

- **NOT g48/g49/g50/g51** (the net-apply-arms family): grep for
  `FindOwner`/`RELEASE_ITEM_OWNERSHIP` at the three sibling tips
  (`af1f172f5`/`714f1f736`/`fcfb92f26`) is EMPTY; g48's row 39 is
  PARKED-with-owner by the FINAL census.
- **NOT M6-NET-5 as rowed**: the ROADMAP S1-I-4a wave-A note parks
  "151 slot-reuse/expiry (WorldItem.cs:351, Item.cs:49723) and 39
  ownership request (WorldItem.cs:256/:264)" under "stay M6-NET-5", but
  the ATLAS M6-NET-5 row (`ATLAS.md:1616`, `M6-NET-5-back-targets`) is
  the **27 Projectile / 28 NPCStrike / 29 ProjectileDead** sync — DONE
  2026-11-09, no item-ownership scope. The ROADMAP pointer is a
  forward-reference to the milestone's REMAINDER, not the landed row.
- **The ITEM lane owns it**: the 21/22/39/151 world-item region is the
  d98 lane's census scope (`task/W4-d98-net-ids-items-chests`, S1-I-1
  wave I — 21 enemy-grab bit + 22 verified faithful), and the FindOwner
  body is `WorldItem.cs` simulation, not a message codec. The seed
  below is the item lane's remaining apply-arm scope; the 151
  slot-reuse/expiry remainder rides the same lane (same `UpdateServer`
  loop, `Main.cs:64582-64604`).

## 2. What is already landed (do NOT re-port)

- **The 39 codec** — f21 tip `5274a097` (`e1d90bcd`):
  `id::RELEASE_ITEM_OWNERSHIP = 39` (`net.rs:180`), the
  `ReleaseItemOwnership { slot: i16, force_assign_to_server: bool }`
  body (`:1842`, `NetMessage.cs:948-951` — `(short)number` +
  `number2 != 0f`), both-direction codecs + the PARKED netglue no-op
  arms (`netglue.rs:823`/:1952 — `C2s/S2c::ReleaseItemOwnership(_) =>
  {}`). The dispatcher arms are what this seed fills.
- **The 22 ItemOwner store** — pre-g48 WIRED: the codec
  (`net.rs:1179-1221` — slot i16 + reserved_for u8 + hold u7 +
  grab_delay_player u8 + grab_delay u7 + position f32 pair,
  `NetMessage.cs:675-684`/`MessageBuffer.cs:1552-1562` byte-verbatim),
  the client `net_reserved` store (`netglue.rs:1786-1797` — the 255
  remove / insert + the grab-delay pair write onto `net_items`).
- **The 21 ItemDrop store** — pre-g48 WIRED + the d98 enemy-grab bit.
- **The pull-gate helpers** — `item::item_space` (`Player.ItemSpace`
  `Player.cs:39153-39197`, S1-P-i47), `item::get_item_grab_range`
  (`Player.cs:35130-35166`) + the grab-range constants
  (`item.rs:550-558` — DEFAULT 42 / COIN 350 / MANA 300 / LIFE 250 /
  TREASURE 150), the heart/star kind families (`HEART_KINDS`/
  `STAR_KINDS`), `DEFAULT_GRAB_DELAY = 100`.

## 3. What is UNROWED (the seed's scope — the peer scan)

`WorldItem.FindOwner` (`WorldItem.cs:188-271`) + `ReserveFor`
(`:273-285`) — the full window `:188-285`:

1. **Reservation fields on the host drop** — `ItemDrop`
   (`item.rs:845-895`) carries NO reservation state: add
   `reserved_for: u8` (255 default, `WorldItem.cs:18`),
   `time_since_reserved: i32` (-1 default, `:20`),
   `time_to_keep_reservation: i32` (`:22`), `instanced: bool` (`:36`).
   The client-side `net_reserved` map is NOT the host store — the host
   needs per-drop fields (the 22 writer reads them).
2. **The FindOwner body** (`:188-271`), verbatim order:
   - `forceAssignToServer` → `timeToKeepReservation = 0` (`:191-194`);
   - the `instanced || timeToKeepReservation > 0` early return
     (`:195-198`);
   - the server-force gates (`:201-204`): `forceAssignToServer ||
     shimmerTime > 0 || (grabDelayTime > 0 && grabDelayPlayer == 255)
     || EmergencyStacking.HasPendingTransferInvolving(this)` → owner
     255 (the EmergencyStacking module is UNPORTED — the gate's
     stand-in is false until that lane lands, record the deviation);
   - the 255-player peer scan (`:207-236`): skip the grab-delay-locked
     player, `active && !dead && CanPullItem(this, ItemSpace(this))`,
     the Manhattan distance (`:217` — centre-x delta + the
     top-y-to-bottom-y delta, NOT centre-to-centre y), the manaMagnet
     star-trio −300 / lifeMagnet heart-trio −250 / the Mana Cloak star
     4143 −300 subtractions (`:218-229`), the `NPC.sWidth = 1920`
     initial best (`NPC.cs:6767`);
   - the hopper veto (`:237-247`): the winner's
     `GetItemGrabRange`-inflated hitbox must intersect the drop, ELSE
     `Wiring.IsHopperInRangeOf` (`Wiring.cs:2610-2635` — the
     `ItemsThatShouldNotBeInInventory` refusal + the 192 px
     `HopperGrabHitboxSize` (`:3429`) inflated tile scan for a wired
     chest 21/467) → owner 255;
   - the `num2 == num` no-change return (`:249-252`);
   - the netMode legs (`:253-270`): client → clear reservation + send
     `39` (force flag as number2); host → the `num != myPlayer &&
     active` reassign + the `timeSince >= 0` → −1 + `39` to the old
     owner, ELSE `ReserveFor(num2)`.
3. **ReserveFor** (`:273-285`): the already-reserved invariant guard,
   `reserved_for = player`, `time_since_reserved = 0`, the `22`
   broadcast with `timeToKeepReservation = 15` default.
4. **The 39 dispatcher arms** (`MessageBuffer.cs:2232-2255`): host leg
   = the reservation-is-mine gate (`== whoAmI`) → clear (255, timer 0)
   → `FindOwner(force)` → if still 255 re-announce via `22`
   (`TrySendData(22, -1, whoAmI, null, slot)`); client leg = the
   reservation-is-mine gate (`== myPlayer`) →
   `FindOwner(forceAssignToServer: true)`.
5. **The UpdateServer cadence** (`Main.cs:64582-64604`): the
   `time_since_reserved++` tick, the unreserved `% 5 == 1` FindOwner
   retry, the reserved owner-inactive-or-`% 300 == 0` re-check — plus
   the client's own-reserved sweep (`Main.cs:64546-64552`).
6. **The other FindOwner callers** (the seed's activation census):
   `WorldItem.cs:317` (the grab-delay expiry), `:529` (the shimmer
   transform's client release), `:703` (VoodooDollLavaDeath),
   `EmergencyStacking.cs:403-404` (the pending-transfer pair) — each
   activates when its sim half lands; the 39 arm itself needs only
   items 1-5.

**Peer-scan dependency:** the host scan needs each peer's
`CanPullItem`/`ItemSpace` answer — the port has NO peer-inventory store
(the id-5 gap, research/17's task seeds; `remote_players` carries
`PlayerNet` state only). The seed's first slice is the peer-inventory
store (the join-bundle 59-slot push, `NetMessage.cs:2806-2962`) OR the
recorded stand-in (host-local answer for every active peer, deviation
admitted) — the item lane picks; the FindOwner maths is exact either
way.

## 4. Verification

Code-only — no cargo/rustc/tests/rustfmt/graphify invoked. The C#
windows re-read direct (`WorldItem.cs:188-285`/`:18-36`/`:317`/`:529`/
`:703`, `MessageBuffer.cs:2232-2255`/`:1552-1562`,
`NetMessage.cs:948-951`/`:675-684`, `Main.cs:64546-64604`,
`Wiring.cs:2610-2635`/`:3429`, `NPC.cs:6767`,
`Player.cs:39139-39146`); the sibling-tip greps ran at
`af1f172f5`/`714f1f736`/`fcfb92f26` (empty); the f21 codec re-read at
tip `5274a097`; the g48 port seams re-read at HEAD (the `ItemDrop`
struct, the 22 store, the item.rs helpers — line cites above); the
ATLAS M6-NET-5 row + the ROADMAP S1-I-4a remainder note re-read at
HEAD.
