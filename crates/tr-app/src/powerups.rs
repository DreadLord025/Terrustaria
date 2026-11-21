//! W4-a6 — the permanent power-up consumables: `Player.ItemCheck`'s
//! `ItemCheck_UseLifeCrystal` / `UseLifeFruit` / `UseManaCrystal` /
//! `UseDemonHeart` / `UseMinecartPowerUp` / `UseArtisanLoaf` /
//! `UseCombatBook` / `UseTorchGodsFavor` / `UsePeddlersSatchel` /
//! `UsePetLicenses` / `UseShimmerPermanentItems`
//! (`Player.cs:45192-45423`) — the full power-up cluster — plus the
//! `ItemCheck_TryDestroyingDrones` remote arms (`:45245-45272`, the
//! 5451/5738 weapons' mid-swing kill).
//!
//! Every arm rides the C# mid-swing gate `itemAnimation > 0 &&
//! ItemTimeIsZero` (`:4064`): the use START (`ApplyItemAnimation`) ran on
//! the press tick, the grant fires on the first action tick inside the
//! animation and `ApplyItemTime` (`:4416-4419`) re-arms the cadence. The
//! machine is the boss-spawner / recall shape (`editing.rs`'s summon gate,
//! `recall_teleport::recall_item_check`): a press edge starts the swing,
//! the grant lands when `item_time` reads zero inside the running swing.
//! A REFUSED arm (cap reached) keeps re-checking each action tick — the
//! C# gate never consumes on a refusal — and the generic consumable tail
//! (`:43280-43326`) only pays the stack when `ApplyItemTime` ran
//! (`itemTimeMax != 0 && itemTime == itemTimeMax`), so a refused use does
//! NOT consume.
//!
//! The stack consume folds onto the grant tick (the food/potion arms'
//! `TurnToAir` convention, `potions.rs:474-487`); the C# runs it one block
//! later in the same tick (`:43280`), no observable difference in SP.
//! The `NetMessage.SendData` rows are MP-only — the port is single-player
//! (`chat.rs`'s netMode-0 note), so they fold out.
//!
//! Achievement targeting (W4-a6 reconcile fix): the C#'s
//! `HandleSpecialEvent` completes ONE row's condition
//! (`GetCondition("STAR_POWER", "Use").Complete()`,
//! `AchievementsHelper.cs:288-379`); the port now carries the row-targeted
//! `AchievementManager::complete_condition` (the g88 `complete_named` /
//! c36 `complete_condition` shape, research/138 §3 keep-one) — the earlier
//! `notify_custom("Use")` broadcast completed every `Flag("Use")` row at
//! once and was LESS faithful than both claimants; replaced here.

use crate::app::App;
use crate::item::ItemSlot;
use crate::registry;
use crate::world_sim::editing::start_swing_extras;

/// One 60 Hz tick in seconds (the `editing.rs` constant).
const ONE_TICK_SECS: f32 = 1.0 / 60.0;

/// The integer tick count off a seconds timer (`editing.rs::ticks_of`).
fn ticks_of(secs: f32) -> i32 {
    (secs * 60.0).round() as i32
}

/// The held item's prefixed `useTime` in seconds — `Player.ApplyItemTime`
/// → `SetItemTime(sItem.useTime)` (`Player.cs:4416-4419`, the INSTANCE
/// value; `recall_teleport::use_time_secs` convention).
fn use_time_secs(held: ItemSlot) -> f32 {
    crate::prefix::prefixed_stats(held.kind, held.prefix)
        .map(|s| s.use_time as f32 * ONE_TICK_SECS)
        .filter(|t| *t > 0.0)
        .unwrap_or_else(|| {
            registry::get(held.kind)
                .map(|d| d.use_time as f32 * ONE_TICK_SECS)
                .filter(|t| *t > 0.0)
                .unwrap_or_else(|| crate::item::use_animation_secs_of(held.kind, held.prefix))
        })
}

/// The power-up kinds this machine owns — the `ItemCheck_Use*` cluster's
/// ids (`Player.cs:43595-43601`'s call row).
pub(crate) fn is_powerup(kind: u16) -> bool {
    matches!(
        kind,
        29 | 109 | 1291 | 3335 | 5289 | 5326 | 4382 | 5336 | 5043 | 5343
            | 4829 | 4830 | 4910 | 5337..=5342
    )
}

/// `Player.ItemCheck_CheckCanUse_Inner`'s Demon Heart refusal
/// (`Player.cs:52051-52054`): `extraAccessory || !Main.expertMode` refuses
/// the use START — no swing, no consume. The crystal/fruit caps gate the
/// GRANT, not the start (the C# has no cap leg here).
fn use_start_refused(app: &App, kind: u16) -> bool {
    if kind == 3335 {
        return app
            .player
            .as_ref()
            .is_some_and(|p| p.extra_accessory || !p.buffs.expert_mode);
    }
    false
}

/// The mid-swing grant dispatch — the C# call order
/// (`Player.cs:43595-43601`): LifeCrystal → LifeFruit → ManaCrystal →
/// DemonHeart → MinecartPowerUp → TorchGodsFavor → ArtisanLoaf. Returns
/// true when the grant fired (the stack pays one).
fn run_powerup_arm(app: &mut App, held: ItemSlot) -> bool {
    if ticks_of(app.swing) <= 0 || ticks_of(app.item_time) != 0 {
        return false;
    }
    let fired = match held.kind {
        29 => use_life_crystal(app),
        1291 => use_life_fruit(app),
        109 => use_mana_crystal(app),
        3335 => use_demon_heart(app),
        5289 => use_minecart_powerup(app),
        5043 => use_torch_gods_favor(app),
        5326 => use_artisan_loaf(app),
        4382 | 5336 => use_combat_book(app, held.kind),
        5343 => use_peddlers_satchel(app),
        4829 | 4830 | 4910 => use_pet_license(app, held.kind),
        5337..=5342 => use_shimmer_permanent(app, held.kind),
        _ => false,
    };
    if fired {
        // `ApplyItemTime(sItem)` (`:45310`/`:45278`) — the ACTION cadence
        // re-arms; the swing keeps running. Read off the pre-consume slot.
        app.item_time = use_time_secs(held);
        consume_one(app);
    }
    fired
}

/// The generic consumable tail (`Player.cs:43280-43326`): one per granted
/// use — `CanConsumeConsumableItem` is `return true` (`:5496-5499`), the
/// crystals are neither ranged nor coins nor ForceConsumption. The
/// 0-stack slot airs out on the consume tick (the food arm's `TurnToAir`
/// fold, `potions.rs:474-487`).
fn consume_one(app: &mut App) {
    let slot = &mut app.inventory[app.selected];
    if slot.count > 0 {
        slot.count -= 1; // `:43317-43320`
    }
    if slot.count == 0 {
        *slot = crate::item::EMPTY_SLOT; // `:42824-42828`
    }
}

/// `Player.ItemCheck_UseLifeCrystal` (`Player.cs:45306-45320`):
/// `statLifeMax < 400` → `statLifeMax += 20; statLifeMax2 += 20;
/// statLife += 20` + `HealEffect(20)`. `HandleSpecialEvent(this, 0)`
/// (`:45318`) is a NO-OP — the switch has no case 0
/// (`AchievementsHelper.cs:288-379`).
fn use_life_crystal(app: &mut App) -> bool {
    let Some(p) = app.player.as_mut() else {
        return false;
    };
    if p.life_max_base >= 400 {
        return false;
    }
    p.life_max_base += 20;
    p.max_hp += 20;
    p.hp += 20;
    // `HealEffect(20)` (`:45316` → `:4522-4529`): the RAW 20 text — the
    // crystal arm carries NO cap on `statLife` (the C# adds straight).
    crate::potions::spawn_heal_text(app, 20);
    true
}

/// `Player.ItemCheck_UseLifeFruit` (`Player.cs:45290-45304`):
/// `statLifeMax >= 400 && statLifeMax < 500` → `statLifeMax += 5;
/// statLifeMax2 += 5; statLife += 5` + `HealEffect(5)` +
/// `HandleSpecialEvent(2)` — GET_A_LIFE's "Use" flag, plus TOPPED_OFF when
/// both caps are full (`AchievementsHelper.cs:297-303`).
fn use_life_fruit(app: &mut App) -> bool {
    let (life_after, mana_base) = {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        if !(400..500).contains(&p.life_max_base) {
            return false;
        }
        p.life_max_base += 5;
        p.max_hp += 5;
        p.hp += 5;
        // `HealEffect(5)` (`:45300`) — the RAW 5 text, no statLife cap.
        crate::potions::spawn_heal_text(app, 5);
        (p.life_max_base, p.mana_max_base)
    };
    let _ = app.achievements.complete_condition("GET_A_LIFE", "Use"); // `:45302` case 2
    if life_after == 500 && mana_base == 200 {
        let _ = app.achievements.complete_condition("TOPPED_OFF", "Use");
    }
    true
}

/// `Player.ItemCheck_UseDemonHeart` (`Player.cs:45192-45200`):
/// `!extraAccessory && Main.expertMode` → `extraAccessory = true` (the 6th
/// accessory slot, `IsItemSlotUnlockedAndUsable` `:12587`). The start
/// refusal (`:52051-52054`) already shut the used/non-expert case; this
/// leg mirrors the C#'s own double gate. The `SendData(4)` sync (`:45198`)
/// is MP-only.
fn use_demon_heart(app: &mut App) -> bool {
    let Some(p) = app.player.as_mut() else {
        return false;
    };
    if p.extra_accessory || !p.buffs.expert_mode {
        return false;
    }
    p.extra_accessory = true;
    true
}

/// `Player.ItemCheck_UseMinecartPowerUp` (`Player.cs:45202-45214`): NO
/// flag gate — every granted use re-arms `ApplyItemTime`, raises
/// `unlockedSuperCart` once (`:45207-45211`, the `SendData(4)` sync is
/// MP-only) and `QuickSpawnItem(source, 3353)` (`:45212`) grants the
/// Minecart Mech — the Super Cart item itself, `MountID` 11.
fn use_minecart_powerup(app: &mut App) -> bool {
    if let Some(p) = app.player.as_mut() {
        if !p.unlocked_super_cart {
            p.unlocked_super_cart = true; // `:45209`
        }
    }
    quick_spawn_item(app, 3353, 1);
    true
}

/// `Player.QuickSpawnItem(source, item, stack)` → `GetOrDropItem(item,
/// PickupItemFromWorld)` (`Player.cs:7170-7200`): a fresh `Item` with
/// `Prefix(-1)` (the natural roll — `Item.cs:504-518`; the Minecart Mech
/// is in no prefixable set, so the roll draws NOTHING), then `GetItem`
/// (the deposit walk, `PickupItemFromWorld` = `CanGoIntoVoidVault: true`,
/// `GetItemSettings.cs:57` — the chime + `NotifyItemPickup` per accepted
/// slot, `:39491`/`:39505`), the void vault as the LAST resort
/// (`:39395-39398`), and the OVERFLOW spawns a world drop at the player's
/// centre with the prefix preserved (`Item.RequestNewItem`, `:7196-7199`
/// — the reforge-return overflow precedent, `app/mod.rs`'s
/// `return_reforge_slot`: the velocity/phase triplet on the port's pinned
/// `Item.NewItem` stand-in, here the `UpdatePlayers` stream — the use runs
/// inside that swap). `newAndShiny`/`enemyGrabDelayTime` are UI/MP-only
/// (the SP `ApplySpawnOwnership` arm is a no-op, `WorldItem.cs:296-305`).
fn quick_spawn_item(app: &mut App, kind: u16, count: u32) {
    let (vault_enabled, center) = app
        .player
        .as_ref()
        .map(|p| (p.void_vault_enabled, p.center()))
        .unwrap_or((false, (0.0, 0.0)));
    let inventory = &mut app.inventory;
    let ammo = &mut app.equip.ammo;
    let audio = &mut app.audio;
    let achievements = &mut app.achievements;
    let rngs = &mut app.rngs;
    let favorite_slots = &app.favorite_slots;
    // `Item.Prefix(-1)` on the fresh instance (`:7176`) — the roll draws
    // ride the use's `UpdatePlayers` swap (`Item.cs:528`'s `Main.rand`).
    let prefix =
        crate::prefix::roll_prefix(kind, -1, rngs.stream(crate::app::RngKey::UpdatePlayers));
    let favorite = |i: usize| favorite_slots.contains(&i);
    let mut remaining = crate::item::deposit_pickup(
        inventory,
        ammo,
        kind,
        prefix,
        count,
        favorite,
        &mut |moved| {
            // One chime + one notify per accepted slot (`:39491`/`:39505`,
            // the `world_sim/items.rs` pickup convention).
            audio.play_pickup_sound(false, rngs.stream(crate::app::RngKey::UpdatePlayers));
            let _ = moved;
            let _ = achievements.notify(crate::achievements::CondKind::ItemPickup(kind));
        },
    );
    // The void vault is GetItem's LAST resort (`:39395-39398`).
    if remaining > 0 && vault_enabled && crate::bank::can_void_vault_accept(kind) {
        let taken = crate::bank::void_vault_take(
            &mut app.player.as_mut().unwrap().banks[3],
            kind,
            prefix,
            remaining,
            crate::registry::max_stack(kind),
            false,
            audio,
            rngs.stream(crate::app::RngKey::UpdatePlayers),
        );
        remaining -= taken;
    }
    // `GetOrDropItem`'s overflow leg (`:7196-7199`): the remainder becomes
    // a world drop at `base.Center`, prefix preserved (no fresh roll).
    if remaining > 0 {
        let rng = rngs.stream(crate::app::RngKey::UpdatePlayers);
        let vx = (rng.next_float() * 2.0 - 1.0) * 1.2;
        let vy = -1.5 - rng.next_float();
        let phase = rng.next_float() * std::f32::consts::TAU;
        app.items.push(crate::item::ItemDrop::new(
            kind,
            remaining,
            prefix,
            center.0,
            center.1,
            vx,
            vy,
            phase,
        ));
    }
}

/// `Player.ItemCheck_UseArtisanLoaf` (`Player.cs:45216-45224`):
/// `!ateArtisanBread` → `ateArtisanBread = true` — the permanent +4-tile
/// crafting reach (`AdjTiles`, `:35671`, the `craft.rs` consumer). The
/// `SendData(4)` sync (`:45222`) is MP-only.
fn use_artisan_loaf(app: &mut App) -> bool {
    let Some(p) = app.player.as_mut() else {
        return false;
    };
    if p.ate_artisan_bread {
        return false;
    }
    p.ate_artisan_bread = true;
    true
}

/// `Player.ItemCheck_UseCombatBook` (`Player.cs:45322-45350`): the 4382
/// arm gates `!NPC.combatBookWasUsed`, the 5336 arm
/// `!NPC.combatBookVolumeTwoWasUsed` — each grants `ApplyItemTime`,
/// raises its flag and prints the `Misc.*Used` line in the World colour
/// (`Main.NewText`, `:45330`/`:45343`). The statics are World fields (the
/// S1-P-i35 standing-in); the MP `SendData(61, -11/-17)` legs fold out.
/// The town stat ladder (`NPC.cs:53018-53031`) reads both flags through
/// `TownEnv.combat_book`/`combat_book_two`.
fn use_combat_book(app: &mut App, kind: u16) -> bool {
    let key = if kind == 4382 {
        if app.world.combat_book_was_used {
            return false;
        }
        app.world.combat_book_was_used = true; // `:45329`
        "Misc.CombatBookUsed"
    } else {
        if app.world.combat_book_volume_two_was_used {
            return false;
        }
        app.world.combat_book_volume_two_was_used = true; // `:45342`
        "Misc.CombatBookVolumeTwoUsed"
    };
    app.chat
        .new_text(crate::lang::text(key), crate::chat::COLOR_WORLD);
    true
}

/// `Player.ItemCheck_UseTorchGodsFavor` (`Player.cs:45226-45243`):
/// `!unlockedBiomeTorches` → `unlockedBiomeTorches = true;
/// UsingBiomeTorches = true` (the property SETTER writes
/// `builderAccStatus[11] = 0`, `:3960-3963`), `HandleSpecialEvent(24)` →
/// GAIN_TORCH_GODS_FAVOR's "Use" flag (`AchievementsHelper.cs:367-369`)
/// and the SP bestiary kill for the Torch God (`SetDefaults(664)` →
/// `RegisterKill`, `:45234-45239` — the credit id is the name string,
/// `lang_tables.rs`'s `(664, "TorchGod")` row). The `SendData(4/51)` syncs
/// are MP-only.
fn use_torch_gods_favor(app: &mut App) -> bool {
    {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        if p.unlocked_biome_torches {
            return false;
        }
        p.unlocked_biome_torches = true; // `:45231`
        p.builder_acc_status[11] = 0; // `UsingBiomeTorches = true`, `:3962`
    }
    // `HandleSpecialEvent(24)` (`:45233`) — ONE row's condition
    // (`AchievementsHelper.cs:367-369`), the c36 keep-one shape.
    let _ = app
        .achievements
        .complete_condition("GAIN_TORCH_GODS_FAVOR", "Use");
    // `Main.netMode == 0` — the port is SP, the kill always registers.
    app.bestiary.kills.register_kill("TorchGod"); // `:45236-45238`
    true
}

/// `Player.ItemCheck_UsePeddlersSatchel` (`Player.cs:45368-45383`):
/// `!NPC.peddlersSatchelWasUsed` → the flag + the `Misc.PeddlersSatchelUsed`
/// World-colour line (the SP arm; the `SendData(61, -18)` MP leg folds
/// out). The travel shop's permanent extra stock slot reads the flag
/// (`Chest.cs:1303-1306`, `shop.rs::setup_travel_shop`).
fn use_peddlers_satchel(app: &mut App) -> bool {
    if app.world.peddlers_satchel_was_used {
        return false;
    }
    app.world.peddlers_satchel_was_used = true; // `:45375`
    app.chat.new_text(
        crate::lang::text("Misc.PeddlersSatchelUsed"),
        crate::chat::COLOR_WORLD,
    );
    true
}

/// `Player.ItemCheck_UsePetLicenses` → `LicenseOrExchangePet` →
/// `NPC.UnlockOrExchangePet` (`Player.cs:45352-45365`, `:45438-45445`,
/// `NPC.cs:80894-80918`): the mid-swing gate is `ItemTimeIsZero &&
/// (!petBoughtFlag || NPC.AnyNPCs(npcType))` — a bought license only
/// re-fires while its pet is resident (the EXCHANGE path). The SP
/// (`netMode != 1`) purchase arm (`:80904-80909`) raises the bought flag
/// and prints the `Misc.License*Used` line in the World colour; the
/// `TrySendData(7)` world sync is MP-only.
///
/// W4-f43b (research/138 §12.2's contract, leg 3 — the a6 else-arm
/// one-line dispatch): the g10 kinds carry 637/638/656 now, so the LIVE
/// `AnyNPCs` scan answers and the exchange leg is REACHABLE. The else-arm
/// dispatches to f43's wired `exchange_resident_pet`
/// (`town_arrival_pets.rs`, the live-Npc `RerollVariationForNPCType`
/// adapter — first resident, the ≤100-draw `Main.rand.Next(6)` reroll on
/// `RngKey::UpdatePlayers` (§10.2's use-context precedent), the
/// no-profile false, the legacy-0 roll, the heal + the genRand re-name)
/// and posts `Misc.PetExchangeSuccess` on true / `Misc.PetExchangeFail`
/// on false (`NPC.cs:80910-80917`) — the C# ALWAYS chats on the exchange
/// and consumes either way (`:45442`); no flag re-raise, no license
/// re-print. The bought flags are World fields (the
/// `combat_book_was_used` standing-in); the town arrival gates
/// (`Main.cs:65900-65911`) are LIVE on this branch (f43's splice).
fn use_pet_license(app: &mut App, kind: u16) -> bool {
    let (npc_type, key) = match kind {
        4829 => (637, "Misc.LicenseCatUsed"),
        4830 => (638, "Misc.LicenseDogUsed"),
        4910 => (656, "Misc.LicenseBunnyUsed"),
        _ => return false,
    };
    let bought = match kind {
        4829 => app.world.bought_cat,
        4830 => app.world.bought_dog,
        _ => app.world.bought_bunny,
    };
    // `NPC.AnyNPCs(npcType)` (`NPC.cs:89873-89883`): active && type match.
    // The g10 kinds carry the net ids — a resident TownCat/Dog/Bunny
    // answers (W4-f43b).
    let any = app
        .npcs
        .iter()
        .any(|n| n.hp > 0 && n.kind.net_id() == npc_type);
    if bought && !any {
        return false; // `:45440` — no resident pet, no exchange
    }
    if !bought {
        match kind {
            4829 => app.world.bought_cat = true,
            4830 => app.world.bought_dog = true,
            _ => app.world.bought_bunny = true,
        }
        app.chat
            .new_text(crate::lang::text(key), crate::chat::COLOR_WORLD);
    } else {
        // `RerollVariationForNPCType` (`:80910`) finds the resident pet
        // and rerolls it (`:80910-80917` — f43's wired
        // `exchange_resident_pet`, the roll on the use-context
        // `Main.rand` = `RngKey::UpdatePlayers`). Success/Fail both
        // consume one (`:45442`); no flag re-raise, no license re-print.
        let success = crate::town_arrival_pets::exchange_resident_pet(
            &mut app.npcs,
            npc_type,
            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
        );
        app.chat.new_text(
            crate::lang::text(if success {
                "Misc.PetExchangeSuccess" // `NPC.cs:80910-80912`
            } else {
                "Misc.PetExchangeFail" // `:80914-80917`
            }),
            crate::chat::COLOR_WORLD,
        );
    }
    true
}

/// `Player.ItemCheck_UseShimmerPermanentItems` (`Player.cs:45385-45423`):
/// six one-shot permanents, each gated on its own `used*` flag —
/// 5337 Aegis Crystal (`usedAegisCrystal`, the +0.2 lifeRegenTime,
/// `:19113-19116`), 5338 Aegis Fruit (`usedAegisFruit`, +4 defense,
/// `:12853-12856`), 5339 Arcane Crystal (`usedArcaneCrystal`, the mana
/// regen rows, `:19430-19433`/`:19453-19456`), 5340 Galaxy Pearl
/// (`usedGalaxyPearl`, +0.03 luck, `luck.rs`'s `RecalculateLuck` row),
/// 5341 Gummy Worm (`usedGummyWorm`, +3 fishing, `:12857-12860`),
/// 5342 Ambrosia (`usedAmbrosia`, the +0.05 speed triple, `:12861-
/// 12866`). The `SendData(4)` syncs are MP-only.
fn use_shimmer_permanent(app: &mut App, kind: u16) -> bool {
    let Some(p) = app.player.as_mut() else {
        return false;
    };
    match kind {
        5337 => {
            if p.used_aegis_crystal {
                return false;
            }
            p.used_aegis_crystal = true; // `:45390`
        }
        5338 => {
            if p.used_aegis_fruit {
                return false;
            }
            p.used_aegis_fruit = true; // `:45396`
        }
        5339 => {
            if p.used_arcane_crystal {
                return false;
            }
            p.used_arcane_crystal = true; // `:45402`
        }
        5340 => {
            if p.used_galaxy_pearl {
                return false;
            }
            p.used_galaxy_pearl = true; // `:45408` — the Player field is the
            // persistence owner; `update_luck` mirrors it into LuckState
            // (the g90 lane-7 shape, research/138 §2).
        }
        5341 => {
            if p.used_gummy_worm {
                return false;
            }
            p.used_gummy_worm = true; // `:45414`
        }
        5342 => {
            if p.used_ambrosia {
                return false;
            }
            p.used_ambrosia = true; // `:45420`
        }
        _ => return false,
    }
    true
}

/// `Player.ItemCheck_UseManaCrystal` (`Player.cs:45274-45288`):
/// `statManaMax < 200` → `statManaMax += 20; statManaMax2 += 20;
/// statMana += 20` + `ManaEffect(20)` + `HandleSpecialEvent(1)` —
/// STAR_POWER's "Use" flag, plus TOPPED_OFF when both caps are full
/// (`AchievementsHelper.cs:290-296`).
fn use_mana_crystal(app: &mut App) -> bool {
    let (life_base, mana_after) = {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        if p.mana_max_base >= 200 {
            return false;
        }
        p.mana_max_base += 20;
        p.mana_max += 20;
        p.mana += 20;
        // `ManaEffect(20)` (`:45284` → `:4531-4538`): the pending-text seam
        // the world_sim player pass drains (`world_sim/player.rs:1400`).
        p.queue_mana_effect(20);
        (p.life_max_base, p.mana_max_base)
    };
    let _ = app.achievements.complete_condition("STAR_POWER", "Use"); // `:45286` case 1
    if life_base == 500 && mana_after == 200 {
        let _ = app.achievements.complete_condition("TOPPED_OFF", "Use");
    }
    true
}

/// `Player.ItemCheck_TryDestroyingDrones` (`Player.cs:45245-45272`): the
/// Jim's Drone 5451 / Remote Control Car 5738 remotes KILL their owner's
/// live 1020/1105 projectiles mid-swing. The gate is `ownedProjectileCounts
/// [proj] > 0 && controlUseItem && ItemTimeIsZero && !mouseInterface` — NO
/// `itemAnimation > 0` leg (unlike the consumable arms), and the kill does
/// NOT `ApplyItemTime` (the drones are not consumable). `releaseUseItem =
/// false` (`:45257`/`:45271`) keeps the held button from re-arming a use
/// while the remote is held. The use START is refused while the projectile
/// is out (`ItemCheck_CheckCanUse_Inner`, `:52067-52074`) and the shot is
/// suppressed the same way (`:43519-43526`) — so the kill only ever runs
/// inside a swing that started with NO drone out.
///
/// Deviation (ANALYSIS "W4-a6 drones"): the 1020/1105 projectile AI is
/// unported (no `proj_data` row — the spawn seam returns `None`), so in
/// live play the counts never rise and the arm is dormant; the machine is
/// ported 1:1 against the pool state for when the drone AI lands. The
/// `mouseInterface` leg folds to `!inventory_open` (the port's standing-in
/// — the UI-hover flag has no field).
pub(crate) fn drone_destroy_check(app: &mut App) {
    let held = app.inventory[app.selected];
    let proj = match held.kind {
        5451 => 1020u16,
        5738 => 1105u16,
        _ => return,
    };
    if held.count == 0 {
        return;
    }
    // `:42598-42603` — the dead/CCed player runs UpdateDead, not ItemCheck.
    if app.player.as_ref().is_some_and(|p| p.dead || p.cced()) {
        return;
    }
    let owned = app
        .player
        .as_ref()
        .is_some_and(|p| p.owned_projectile_counts[usize::from(proj)] > 0);
    if !owned || !app.mine_down || ticks_of(app.item_time) != 0 || app.inventory_open {
        return;
    }
    // `:45249-45256`/`:45263-45270` — the 1000-slot scan: every ACTIVE
    // same-kind projectile of this owner dies.
    let owner = app.local_id as u8;
    for i in 0..app.projectiles.slots().len() {
        let p = &app.projectiles.slots()[i];
        if p.active && p.owner == owner && p.kind == proj {
            app.projectiles.kill(i);
        }
    }
    app.mine_released = false; // `releaseUseItem = false`, `:45257`/`:45271`
}

/// The power-up `ItemCheck` machine — called once per tick from
/// `update_editing` beside the boss-spawner block. `press_edge` is the
/// caller's `prev_mine_released && !prev_mine_down` pair (the
/// `recall_item_check` convention). Returns true when it STARTED a use
/// this tick (the caller consumes the tick like the boss arms); a live
/// swing's per-tick grant runs without consuming.
pub(crate) fn powerup_item_check(app: &mut App, press_edge: bool) -> bool {
    let held = app.inventory[app.selected];
    if held.count == 0 || !is_powerup(held.kind) {
        return false;
    }
    // `:42598-42603` CCed / the dead player runs UpdateDead, not ItemCheck
    // (the recall machine's gate).
    if app.player.as_ref().is_some_and(|p| p.dead || p.cced()) {
        return false;
    }
    if ticks_of(app.swing) > 0 {
        run_powerup_arm(app, held);
        return false;
    }
    if app.mine_down && !app.inventory_open && press_edge {
        if use_start_refused(app, held.kind) {
            return false;
        }
        // `ItemCheck_StartActualUse` → `ApplyItemAnimation` (`:51680`):
        // the swing lasts useAnimation; `itemTime` stays 0 (the `:42836`
        // decrement only runs when > 0), so the grant's `ItemTimeIsZero`
        // leg fires THIS tick — the boss-spawner start shape.
        app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
        app.swing = app.swing_max;
        start_swing_extras(app, held.kind);
        app.attack_cd = app.swing_max;
        app.reuse_delay = crate::prefix::prefixed_stats(held.kind, held.prefix)
            .map_or(0.0, |s| s.reuse_delay as f32 * ONE_TICK_SECS);
        app.item_time = 0.0;
        run_powerup_arm(app, held);
        app.mine_target = None;
        app.mine_timer = 0.0;
        return true;
    }
    false
}

// ---- witnesses (W4-a6) — WRITTEN NOT RUN (code-only swarm, no compiler) ----
//
// The rigs mirror `editing.rs::potion_press_app` / `recall_teleport`'s: the
// flow app's 8x8 air world, the player parked at the centre, the camera on
// the cursor cell, the SILENT listener so the use-sound draws nothing. The
// press edge is armed by one idle frame (`mine_prev_down = false`,
// `mine_released = true`), then `mine_down = true`.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use glam::Vec2;
    use tr_core::TILE_PX;

    const SCREEN: Vec2 = Vec2::new(1280.0, 720.0);

    fn rig(kind: u16, count: u32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut p = Player::spawn(&app.world);
        p.x = 4.0 * TILE_PX;
        p.y = 4.0 * TILE_PX;
        app.player = Some(p);
        app.camera = Vec2::new(4.0 * TILE_PX + 8.0, 4.0 * TILE_PX + 8.0);
        app.mouse_pos = Vec2::new(640.0, 360.0);
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (app.camera.x, app.camera.y),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app.inventory[app.selected] = ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        };
        app.mine_down = false;
        app.mine_prev_down = false;
        app.mine_released = true;
        app
    }

    fn tick(app: &mut App) {
        crate::world_sim::editing::update_editing(app, 1.0 / 60.0, SCREEN);
    }

    /// Life Crystal 29: +20 to the permanent base, the effective max and
    /// the live pool; one consumed; the RAW-20 HealLife text.
    #[test]
    fn life_crystal_grants_twenty_and_consumes_one() {
        let mut app = rig(29, 3);
        tick(&mut app); // idle: the press edge arms
        app.mine_down = true;
        tick(&mut app); // the use
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 120, "statLifeMax += 20");
        assert_eq!(p.max_hp, 120, "statLifeMax2 += 20");
        assert_eq!(p.hp, 120, "statLife += 20");
        assert_eq!(app.inventory[app.selected].count, 2, "one consumed");
        let t = app.combat_texts.iter().find(|t| t.active).expect("heal text");
        assert_eq!(t.text, "20");
        assert_eq!(t.color, crate::combat_text::HEAL_LIFE);
    }

    /// Holding LMB grants ONCE per press (the `:42733` press edge), not
    /// every tick — the stack drains by one, not thirty.
    #[test]
    fn holding_lmb_uses_one_crystal_not_every_tick() {
        let mut app = rig(29, 30);
        tick(&mut app);
        app.mine_down = true;
        for _ in 0..120 {
            tick(&mut app);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 120, "ONE grant per press");
        assert_eq!(app.inventory[app.selected].count, 29);
    }

    /// The 400 cap: a character already at 400 refuses the grant AND the
    /// consume (the C# `statLifeMax < 400` gate; `itemTimeMax` stays 0).
    #[test]
    fn life_crystal_at_cap_refuses_and_keeps_the_stack() {
        let mut app = rig(29, 3);
        {
            let p = app.player.as_mut().unwrap();
            p.life_max_base = 400;
            p.max_hp = 400;
            p.hp = 400;
        }
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 400, "no grant at the cap");
        assert_eq!(app.inventory[app.selected].count, 3, "no consume on refusal");
    }

    /// Fifteen crystals walk 100 → 400 and then stop (the cap).
    #[test]
    fn life_crystals_stop_at_four_hundred() {
        let mut app = rig(29, 20);
        for _ in 0..20 {
            // A fresh press each use: release, idle, press.
            app.mine_down = false;
            tick(&mut app);
            app.attack_cd = 0.0;
            app.reuse_delay = 0.0;
            app.swing = 0.0;
            app.mine_down = true;
            tick(&mut app);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 400, "15 grants then the cap");
        assert_eq!(app.inventory[app.selected].count, 5, "20 - 15");
    }

    /// Mana Crystal 109: +20 to base/effective/live mana, one consumed,
    /// the ManaEffect text queued, STAR_POWER's "Use" flag fired.
    #[test]
    fn mana_crystal_grants_twenty_and_consumes_one() {
        let mut app = rig(109, 3);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana_max_base, 40, "statManaMax += 20");
        assert_eq!(p.mana_max, 40, "statManaMax2 += 20");
        assert_eq!(p.mana, 40, "statMana += 20");
        assert_eq!(app.inventory[app.selected].count, 2);
        assert_eq!(p.take_mana_effect(), Some(20), "ManaEffect(20) queued");
        let star = app.achievements.list.iter().find(|a| a.name == "STAR_POWER").unwrap();
        assert!(star.completed, "HandleSpecialEvent(1) → STAR_POWER");
    }

    /// The 200 mana cap refuses the grant and the consume.
    #[test]
    fn mana_crystal_at_cap_refuses() {
        let mut app = rig(109, 3);
        {
            let p = app.player.as_mut().unwrap();
            p.mana_max_base = 200;
            p.mana_max = 200;
            p.mana = 200;
        }
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana_max_base, 200);
        assert_eq!(app.inventory[app.selected].count, 3, "no consume on refusal");
    }

    /// The permanent mana base survives the per-tick `statManaMax2` rebase
    /// (`ResetEffects` `:18389`) — the crystal's +20 is not wiped next tick.
    #[test]
    fn mana_crystal_base_survives_the_rebase() {
        let mut app = rig(109, 3);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_mut().unwrap();
        p.update(&app.world, crate::player::PlayerInput::default());
        assert_eq!(p.mana_max_base, 40, "the permanent cap persists");
        assert_eq!(p.mana_max, 40, "the rebase reads the grown base");
    }

    /// Life Fruit 1291: +5 to base/effective/live life at the 400 gate,
    /// one consumed, the RAW-5 heal text, GET_A_LIFE's "Use" flag.
    #[test]
    fn life_fruit_grants_five_at_four_hundred() {
        let mut app = rig(1291, 3);
        {
            let p = app.player.as_mut().unwrap();
            p.life_max_base = 400;
            p.max_hp = 400;
            p.hp = 400;
        }
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 405, "statLifeMax += 5");
        assert_eq!(p.max_hp, 405);
        assert_eq!(p.hp, 405);
        assert_eq!(app.inventory[app.selected].count, 2);
        let t = app.combat_texts.iter().find(|t| t.active).expect("heal text");
        assert_eq!(t.text, "5");
        let life = app.achievements.list.iter().find(|a| a.name == "GET_A_LIFE").unwrap();
        assert!(life.completed, "HandleSpecialEvent(2) → GET_A_LIFE");
    }

    /// Life Fruit below 400 refuses (the `statLifeMax >= 400` gate) — no
    /// grant, no consume.
    #[test]
    fn life_fruit_below_four_hundred_refuses() {
        let mut app = rig(1291, 3);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 100, "no grant below 400");
        assert_eq!(app.inventory[app.selected].count, 3, "no consume on refusal");
    }

    /// Life Fruit at 500 refuses (the `statLifeMax < 500` gate).
    #[test]
    fn life_fruit_at_five_hundred_refuses() {
        let mut app = rig(1291, 3);
        {
            let p = app.player.as_mut().unwrap();
            p.life_max_base = 500;
            p.max_hp = 500;
            p.hp = 500;
        }
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.player.as_ref().unwrap().life_max_base, 500);
        assert_eq!(app.inventory[app.selected].count, 3);
    }

    /// Twenty fruits walk 400 → 500 and then stop.
    #[test]
    fn life_fruits_stop_at_five_hundred() {
        let mut app = rig(1291, 25);
        {
            let p = app.player.as_mut().unwrap();
            p.life_max_base = 400;
            p.max_hp = 400;
            p.hp = 400;
        }
        for _ in 0..25 {
            app.mine_down = false;
            tick(&mut app);
            app.attack_cd = 0.0;
            app.reuse_delay = 0.0;
            app.swing = 0.0;
            app.mine_down = true;
            tick(&mut app);
        }
        assert_eq!(app.player.as_ref().unwrap().life_max_base, 500);
        assert_eq!(app.inventory[app.selected].count, 5, "25 - 20");
    }

    /// Demon Heart 3335 in expert mode: extraAccessory flips true, one
    /// consumed (the 6th accessory slot unlock).
    #[test]
    fn demon_heart_unlocks_the_extra_slot_in_expert() {
        let mut app = rig(3335, 2);
        app.player.as_mut().unwrap().buffs.expert_mode = true;
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert!(app.player.as_ref().unwrap().extra_accessory);
        assert_eq!(app.inventory[app.selected].count, 1);
    }

    /// Demon Heart outside expert mode refuses the use START
    /// (`:52051-52054`) — no swing, no grant, no consume.
    #[test]
    fn demon_heart_outside_expert_refuses() {
        let mut app = rig(3335, 2);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert!(!app.player.as_ref().unwrap().extra_accessory);
        assert_eq!(app.inventory[app.selected].count, 2, "no consume");
        assert_eq!(app.swing, 0.0, "no swing on the refused start");
    }

    /// A second Demon Heart refuses (extraAccessory already set) — the
    /// stack keeps its remainder.
    #[test]
    fn a_second_demon_heart_refuses() {
        let mut app = rig(3335, 2);
        app.player.as_mut().unwrap().buffs.expert_mode = true;
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 1);
        // Release + re-press with the flag already set.
        app.mine_down = false;
        tick(&mut app);
        app.attack_cd = 0.0;
        app.reuse_delay = 0.0;
        app.swing = 0.0;
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 1, "the second refuses");
    }

    /// Minecart Powerup 5289: unlockedSuperCart flips true and the Minecart
    /// Mech 3353 lands in the inventory (QuickSpawnItem → GetItem); one
    /// powerup consumed.
    #[test]
    fn minecart_powerup_unlocks_and_grants_the_mech() {
        let mut app = rig(5289, 2);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert!(app.player.as_ref().unwrap().unlocked_super_cart);
        assert_eq!(app.inventory[app.selected].count, 1, "one consumed");
        let mech = app
            .inventory
            .iter()
            .find(|s| s.kind == 3353)
            .expect("the Minecart Mech granted");
        assert_eq!(mech.count, 1);
    }

    /// The minecart powerup has NO flag gate (`:45204` — only the crystal
    /// arms carry one): a second use grants a SECOND mech (the stack
    /// merges) while the unlock flag stays true.
    #[test]
    fn a_second_minecart_powerup_grants_another_mech() {
        let mut app = rig(5289, 2);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        app.mine_down = false;
        tick(&mut app);
        app.attack_cd = 0.0;
        app.reuse_delay = 0.0;
        app.swing = 0.0;
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 0, "both consumed");
        let mech = app.inventory.iter().find(|s| s.kind == 3353).unwrap();
        assert_eq!(mech.count, 2, "the second use grants again");
    }

    /// The QuickSpawnItem overflow leg (`GetOrDropItem` `:7196-7199`): a
    /// full inventory with no vault spawns the mech as a world drop at the
    /// player's centre instead of losing it.
    #[test]
    fn minecart_powerup_overflow_spawns_a_world_drop() {
        let mut app = rig(5289, 1);
        // Fill every OTHER slot to its cap — no merge, no empty slot.
        let full = crate::registry::max_stack(9);
        for i in 0..app.inventory.len() {
            if i != app.selected {
                app.inventory[i] = ItemSlot { kind: 9, name: registry::name(9), count: full, prefix: 0 };
            }
        }
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert!(app.player.as_ref().unwrap().unlocked_super_cart);
        assert!(
            app.items.iter().any(|d| d.kind == 3353 && d.count == 1),
            "the overflow mech rides a world drop"
        );
    }

    /// Artisan Loaf 5326: ateArtisanBread flips true (the +4 crafting
    /// reach), one consumed.
    #[test]
    fn artisan_loaf_sets_the_reach_flag() {
        let mut app = rig(5326, 2);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert!(app.player.as_ref().unwrap().ate_artisan_bread);
        assert_eq!(app.inventory[app.selected].count, 1);
    }

    /// A second Artisan Loaf refuses the grant AND the consume (the
    /// `!ateArtisanBread` gate `:45218`).
    #[test]
    fn a_second_artisan_loaf_refuses() {
        let mut app = rig(5326, 2);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 1);
        app.mine_down = false;
        tick(&mut app);
        app.attack_cd = 0.0;
        app.reuse_delay = 0.0;
        app.swing = 0.0;
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 1, "the second refuses");
    }

    /// Combat Book 4382: the world flag flips, the World-colour chat line
    /// prints, one consumed (`:45324-45331`).
    #[test]
    fn combat_book_sets_the_world_flag_and_chats() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = rig(4382, 2);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert!(app.world.combat_book_was_used);
        assert_eq!(app.inventory[app.selected].count, 1);
        let line = app.chat.messages.first().expect("the chat line");
        assert_eq!(line.text, crate::lang::text("Misc.CombatBookUsed"));
        assert_eq!(line.color, crate::chat::COLOR_WORLD);
    }

    /// A second Combat Book refuses (the `!NPC.combatBookWasUsed` gate) —
    /// no second chat line, no consume.
    #[test]
    fn a_second_combat_book_refuses() {
        let mut app = rig(4382, 2);
        app.world.combat_book_was_used = true;
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 2, "no consume on refusal");
        assert!(app.chat.messages.is_empty(), "no chat on refusal");
    }

    /// Combat Book Volume Two 5336: its OWN flag (`:45337-45344`) — the
    /// vol.1 flag does not gate it, and it does not set vol.1's.
    #[test]
    fn combat_book_volume_two_sets_its_own_flag() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = rig(5336, 2);
        app.world.combat_book_was_used = true; // vol.1 already used
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert!(app.world.combat_book_volume_two_was_used);
        assert_eq!(app.inventory[app.selected].count, 1);
        let line = app.chat.messages.first().expect("the chat line");
        assert_eq!(line.text, crate::lang::text("Misc.CombatBookVolumeTwoUsed"));
        assert_eq!(line.color, crate::chat::COLOR_WORLD);
    }

    /// A second Volume Two refuses (the `!combatBookVolumeTwoWasUsed` gate).
    #[test]
    fn a_second_combat_book_volume_two_refuses() {
        let mut app = rig(5336, 2);
        app.world.combat_book_volume_two_was_used = true;
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 2, "no consume on refusal");
        assert!(app.chat.messages.is_empty());
    }

    /// Torch God's Favor 5043: the unlock flag + the builder-accessory
    /// status slot 11 flip, GAIN_TORCH_GODS_FAVOR completes, the TorchGod
    /// bestiary kill registers, one consumed (`:45226-45243`).
    #[test]
    fn torch_gods_favor_unlocks_the_biome_torches() {
        let mut app = rig(5043, 2);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_ref().unwrap();
        assert!(p.unlocked_biome_torches);
        assert_eq!(p.builder_acc_status[11], 0, "UsingBiomeTorches = true");
        assert_eq!(app.inventory[app.selected].count, 1);
        let torch = app
            .achievements
            .list
            .iter()
            .find(|a| a.name == "GAIN_TORCH_GODS_FAVOR")
            .unwrap();
        assert!(torch.completed, "HandleSpecialEvent(24)");
        assert_eq!(app.bestiary.kills.get_kill_count("TorchGod"), 1);
    }

    /// A second Torch God's Favor refuses (the `!unlockedBiomeTorches`
    /// gate) — no second bestiary kill, no consume.
    #[test]
    fn a_second_torch_gods_favor_refuses() {
        let mut app = rig(5043, 2);
        app.player.as_mut().unwrap().unlocked_biome_torches = true;
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 2, "no consume on refusal");
        assert_eq!(app.bestiary.kills.get_kill_count("TorchGod"), 0);
    }

    /// Peddler's Satchel 5343: the world flag flips, the World-colour chat
    /// line prints, one consumed (`:45368-45383`).
    #[test]
    fn peddlers_satchel_sets_the_world_flag_and_chats() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = rig(5343, 2);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert!(app.world.peddlers_satchel_was_used);
        assert_eq!(app.inventory[app.selected].count, 1);
        let line = app.chat.messages.first().expect("the chat line");
        assert_eq!(line.text, crate::lang::text("Misc.PeddlersSatchelUsed"));
        assert_eq!(line.color, crate::chat::COLOR_WORLD);
    }

    /// A second Peddler's Satchel refuses (the `!peddlersSatchelWasUsed`
    /// gate) — no second chat line, no consume.
    #[test]
    fn a_second_peddlers_satchel_refuses() {
        let mut app = rig(5343, 2);
        app.world.peddlers_satchel_was_used = true;
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 2, "no consume on refusal");
        assert!(app.chat.messages.is_empty());
    }

    /// Cat License 4829: boughtCat flips, the World-colour chat line
    /// prints, one consumed (`:45354-45357`, `NPC.cs:80904-80909`).
    #[test]
    fn cat_license_sets_bought_cat_and_chats() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = rig(4829, 2);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert!(app.world.bought_cat);
        assert!(!app.world.bought_dog);
        assert!(!app.world.bought_bunny);
        assert_eq!(app.inventory[app.selected].count, 1);
        let line = app.chat.messages.first().expect("the chat line");
        assert_eq!(line.text, crate::lang::text("Misc.LicenseCatUsed"));
        assert_eq!(line.color, crate::chat::COLOR_WORLD);
    }

    /// Dog License 4830 and Bunny License 4910 set their OWN flags.
    #[test]
    fn dog_and_bunny_licenses_set_their_own_flags() {
        let mut app = rig(4830, 1);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert!(app.world.bought_dog);
        assert!(!app.world.bought_cat);
        let mut app = rig(4910, 1);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert!(app.world.bought_bunny);
        assert!(!app.world.bought_cat);
        assert!(!app.world.bought_dog);
    }

    /// A bought license with no resident pet refuses (`:45440`'s
    /// `!petBoughtFlag || AnyNPCs(npcType)` gate — the port ships no town
    /// pets, so the exchange is unreachable): no consume, no chat.
    #[test]
    fn a_bought_license_without_a_resident_pet_refuses() {
        let mut app = rig(4829, 2);
        app.world.bought_cat = true;
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 2, "no consume on refusal");
        assert!(app.chat.messages.is_empty(), "no exchange chat");
    }

    /// A resident NON-pet NPC does not open the exchange gate — the LIVE
    /// scan matches net ids, not mere liveness (`NPC.cs:89873-89883`;
    /// the g87 closeout `26620538e`'s twin witness, research/138 §11.3).
    #[test]
    fn a_resident_non_pet_does_not_open_the_exchange() {
        let mut app = rig(4829, 2);
        app.world.bought_cat = true;
        let mut npc = crate::npc::Npc::new(crate::npc::NpcKind::Slime, 32.0, 32.0, 1, 0.4);
        npc.hp = 10;
        npc.max_hp = 10;
        app.npcs.push(npc);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(
            app.inventory[app.selected].count, 2,
            "a slime is not the cat — the exchange refuses, nothing consumed"
        );
        assert!(app.chat.messages.is_empty(), "no exchange chat on refusal");
    }

    // W4-f43b: the exchange legs are REACHABLE — the g10 kinds carry
    // 637/638/656 and the else-arm dispatches to f43's wired
    // `exchange_resident_pet` (research/138 §12.2's contract, leg 3). The
    // refusal witnesses above still hold (no resident / a non-pet); the
    // positive Success/Fail pair lands below.

    /// W4-f43b — the Success leg (`NPC.cs:80910-80912`): a bought license
    /// with a resident TownCat rerolls the variation (the ≤100-draw
    /// `Main.rand.Next(6)` on `RngKey::UpdatePlayers`), heals to full,
    /// re-names on the genRand alias and posts `Misc.PetExchangeSuccess`;
    /// one consumed (`:45442` — the C# consumes either way).
    #[test]
    fn a_resident_cat_exchanges_with_the_success_chat() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = rig(4829, 2);
        app.world.bought_cat = true;
        let mut cat = crate::npc::Npc::new(crate::npc::NpcKind::TownCat, 32.0, 32.0, 1, 0.0);
        cat.hp = 10; // hurt — the exchange heals (`NPC.cs:80950`)
        cat.variation_index = 0;
        app.npcs.push(cat);
        // Pin the roll stream so a change is immediate when the exchange is
        // the first draw: walk seeds until one answers != 0. The ≤100-draw
        // reroll cap (`NPC.cs:80941`) guarantees a change regardless of any
        // preceding use-context draws — the probability all 100 repeat the
        // old index is (1/6)^100, so the Success leg is certain; the pin
        // just makes the common case a single draw.
        let mut seed = 1i32;
        while tr_core::UnifiedRandom::new(seed).next_max(6) == 0 {
            seed += 1;
        }
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = tr_core::UnifiedRandom::new(seed);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 1, "one consumed");
        let line = app.chat.messages.first().expect("the exchange chat");
        assert_eq!(line.text, crate::lang::text("Misc.PetExchangeSuccess"));
        assert_eq!(line.color, crate::chat::COLOR_WORLD);
        let cat = &app.npcs[0];
        assert_ne!(cat.variation_index, 0, "the reroll changed the variation");
        assert_eq!(cat.hp, cat.max_hp, "life = lifeMax (:80950)");
        assert!(app.world.bought_cat, "no flag re-raise");
    }

    /// W4-f43b — the Fail leg (`NPC.cs:80914-80917`): a resident whose
    /// every reroll draw repeats the old index fails, posts
    /// `Misc.PetExchangeFail` and STILL consumes one — never a silent
    /// consume (the g87 closeout's fold, research/138 §11.3). A
    /// fixed-seed stream is walked to force 100 identical draws is
    /// impractical; the legacy-profile twin pins the deterministic fail:
    /// a resident TOWN SLIME at variation 0 has a LegacyNPCProfile whose
    /// RollVariation is the constant 0 (`Profiles.cs:83-86`) — the
    /// exchange fails with ZERO draws. (The slime's own license does not
    /// exist; the cat license's scan targets 637 only, so the witness
    /// drives `exchange_resident_pet` directly for the slime and the
    /// cat-license Fail path via the no-change stream below.)
    #[test]
    fn a_legacy_slime_exchange_fails_without_a_draw() {
        let mut app = rig(4829, 1);
        let mut slime =
            crate::npc::Npc::new(crate::npc::NpcKind::TownSlimeBlue, 32.0, 32.0, 1, 0.0);
        slime.hp = 5;
        slime.variation_index = 0;
        app.npcs.push(slime);
        let before = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        let ok = crate::town_arrival_pets::exchange_resident_pet(
            &mut app.npcs,
            670,
            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
        );
        assert!(!ok, "the constant-0 roll never changes index 0");
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            before,
            "the LegacyNPCProfile draws NOTHING"
        );
        assert_eq!(app.npcs[0].variation_index, 0);
        assert_eq!(app.npcs[0].hp, 5, "no heal on failure");
    }

    /// The six shimmer permanents each set their OWN flag and consume one
    /// (`:45385-45423`): 5337 aegis crystal, 5338 aegis fruit, 5339 arcane
    /// crystal, 5340 galaxy pearl (the luck-state field), 5341 gummy worm,
    /// 5342 ambrosia.
    #[test]
    fn the_shimmer_permanents_set_their_flags() {
        for (kind, check) in [
            (5337u16, 0),
            (5338, 1),
            (5339, 2),
            (5340, 3),
            (5341, 4),
            (5342, 5),
        ] {
            let mut app = rig(kind, 2);
            tick(&mut app);
            app.mine_down = true;
            tick(&mut app);
            let p = app.player.as_ref().unwrap();
            let flags = [
                p.used_aegis_crystal,
                p.used_aegis_fruit,
                p.used_arcane_crystal,
                p.used_galaxy_pearl,
                p.used_gummy_worm,
                p.used_ambrosia,
            ];
            assert!(flags[check], "kind {kind} sets flag {check}");
            assert_eq!(
                flags.iter().filter(|f| **f).count(),
                1,
                "kind {kind} sets ONLY its own flag"
            );
            assert_eq!(app.inventory[app.selected].count, 1, "one consumed");
        }
    }

    /// A second shimmer permanent refuses (its `!used*` gate) — no consume.
    #[test]
    fn a_second_shimmer_permanent_refuses() {
        let mut app = rig(5337, 2);
        app.player.as_mut().unwrap().used_aegis_crystal = true;
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 2, "no consume on refusal");
    }

    /// The Aegis Fruit's +4 defense + the Ambrosia speed triple land in the
    /// per-tick equip pass (`UpdatePermanentBoosters`, `:12853-12866`) and
    /// SURVIVE the next tick's ResetEffects (the permanents re-apply).
    #[test]
    fn aegis_fruit_and_ambrosia_boosters_apply_every_tick() {
        let mut app = rig(29, 1); // any held item; the flags drive the pass
        {
            let p = app.player.as_mut().unwrap();
            p.used_aegis_fruit = true;
            p.used_ambrosia = true;
        }
        tick(&mut app);
        {
            let p = app.player.as_mut().unwrap();
            p.update(&app.world, crate::player::PlayerInput::default());
            assert_eq!(p.stat_defense, 4, "the +4 defense");
            assert!((p.pick_speed_bonus - 0.05).abs() < 1e-6);
            assert!((p.tile_speed_bonus - 0.05).abs() < 1e-6);
            assert!((p.wall_speed_bonus - 0.05).abs() < 1e-6);
            // A second pass re-applies from the reset base (no stacking).
            p.update(&app.world, crate::player::PlayerInput::default());
            assert_eq!(p.stat_defense, 4, "no double-apply");
        }
    }

    /// The Galaxy Pearl's +0.03 luck rides `RecalculateLuck`
    /// (`luck.rs:201-203`) once the flag is set — the use latches the
    /// Player field, `update_luck` mirrors it into the LuckState consumer.
    #[test]
    fn galaxy_permanent_luck_rides_the_recalculation() {
        let mut app = rig(5340, 1);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_mut().unwrap();
        assert!(p.used_galaxy_pearl, "the Player field is the owner");
        p.update(&app.world, crate::player::PlayerInput::default());
        assert!(p.luck_state.used_galaxy_pearl, "the mirror seeds the state");
        assert!((p.luck_state.luck - 0.03).abs() < 1e-6, "the +0.03 row");
    }

    /// W4-a6 — the TRAINING_DAY hook (`Player.cs:25695-25698`): BOTH world
    /// book flags standing fires progression 40 through the player pass
    /// (the g90 lane-7 consumer); one book alone fires nothing.
    #[test]
    fn both_combat_books_fire_training_day() {
        let mut app = rig(2, 1); // any non-powerup held item
        app.world.combat_book_was_used = true;
        crate::world_sim::player::update_player(&mut app, 1.0 / 60.0);
        let training = |app: &App| {
            app.achievements
                .list
                .iter()
                .find(|a| a.name == "TRAINING_DAY")
                .unwrap()
                .completed
        };
        assert!(!training(&app), "one book alone fires nothing");
        app.world.combat_book_volume_two_was_used = true;
        crate::world_sim::player::update_player(&mut app, 1.0 / 60.0);
        assert!(training(&app), "the pair completes TRAINING_DAY (`:25697`)");
    }

    /// Plant a live owned projectile of `kind` in the pool + arm the count
    /// cache (the 1020/1105 drone AI is unported — the spawn seam returns
    /// None, so the witnesses drive the pool state directly).
    fn plant_drone(app: &mut App, kind: u16) {
        let slot = app
            .projectiles
            .slots_mut()
            .iter_mut()
            .find(|p| !p.active)
            .expect("a free slot");
        slot.active = true;
        slot.kind = kind;
        slot.owner = app.local_id as u8;
        slot.time_left = 3600;
        app.player.as_mut().unwrap().owned_projectile_counts[usize::from(kind)] = 1;
    }

    /// Jim's Drone 5451 with a live 1020: the mid-swing kill fires
    /// (`:45247-45257`) — the projectile dies, `mine_released` clears, the
    /// stack is untouched (the remote is not consumable).
    #[test]
    fn jims_drone_kills_the_live_drone_mid_swing() {
        let mut app = rig(5451, 1);
        plant_drone(&mut app, 1020);
        app.mine_down = true;
        app.item_time = 0.0;
        tick(&mut app);
        assert!(
            !app.projectiles.slots().iter().any(|p| p.active && p.kind == 1020),
            "the drone dies"
        );
        assert!(!app.mine_released, "releaseUseItem = false");
        assert_eq!(app.inventory[app.selected].count, 1, "no consume");
    }

    /// Remote Control Car 5738 with a live 1105: the same kill
    /// (`:45259-45271`).
    #[test]
    fn remote_control_car_kills_the_live_car_mid_swing() {
        let mut app = rig(5738, 1);
        plant_drone(&mut app, 1105);
        app.mine_down = true;
        app.item_time = 0.0;
        tick(&mut app);
        assert!(
            !app.projectiles.slots().iter().any(|p| p.active && p.kind == 1105),
            "the car dies"
        );
        assert!(!app.mine_released);
        assert_eq!(app.inventory[app.selected].count, 1, "no consume");
    }

    /// The kill gate needs `controlUseItem` — a released button does not
    /// destroy (`:45247`).
    #[test]
    fn the_drone_kill_needs_the_button_held() {
        let mut app = rig(5451, 1);
        plant_drone(&mut app, 1020);
        app.mine_down = false;
        app.item_time = 0.0;
        tick(&mut app);
        assert!(
            app.projectiles.slots().iter().any(|p| p.active && p.kind == 1020),
            "no kill without controlUseItem"
        );
    }

    /// The kill gate needs `ItemTimeIsZero` — a live action cadence does
    /// not destroy (`:45247`).
    #[test]
    fn the_drone_kill_waits_for_item_time_zero() {
        let mut app = rig(5451, 1);
        plant_drone(&mut app, 1020);
        app.mine_down = true;
        app.item_time = 0.5; // 30 ticks of cadence left
        tick(&mut app);
        assert!(
            app.projectiles.slots().iter().any(|p| p.active && p.kind == 1020),
            "no kill while itemTime > 0"
        );
    }

    /// Another owner's drone survives — the scan is `owner == whoAmI`
    /// (`:45252`).
    #[test]
    fn the_drone_kill_spares_other_owners() {
        let mut app = rig(5451, 1);
        plant_drone(&mut app, 1020);
        // A second 1020 owned by player 1.
        let slot = app
            .projectiles
            .slots_mut()
            .iter_mut()
            .find(|p| !p.active)
            .expect("a free slot");
        slot.active = true;
        slot.kind = 1020;
        slot.owner = 1;
        slot.time_left = 3600;
        app.mine_down = true;
        app.item_time = 0.0;
        tick(&mut app);
        assert!(
            app.projectiles
                .slots()
                .iter()
                .any(|p| p.active && p.kind == 1020 && p.owner == 1),
            "the other owner's drone lives"
        );
    }

    /// The use START is refused while the drone is out
    /// (`ItemCheck_CheckCanUse_Inner`, `:52067-52070`) — no swing arms.
    #[test]
    fn jims_drone_use_start_refuses_while_the_drone_is_out() {
        let mut app = rig(5451, 1);
        plant_drone(&mut app, 1020);
        tick(&mut app); // idle: the press edge arms
        app.mine_down = true;
        tick(&mut app);
        // The destroy arm kills the drone (mine_down + item_time 0), but
        // the use START never armed a swing — the refusal ran first.
        assert_eq!(app.swing, 0.0, "no swing on the refused start");
        assert_eq!(app.inventory[app.selected].count, 1, "no consume");
    }
}