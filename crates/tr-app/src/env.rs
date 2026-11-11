//! The `TR_*` environment pins: the deterministic screenshot / benchmark / test
//! instrumentation the app reads from its process environment.
//!
//! Every pin lives here as one named accessor so the whole dev surface is
//! discoverable in one file (the 2026-11-01 architecture plan's A6, the
//! `REFACTOR-env` row of `docs/ROADMAP.md`): a pin is set before the process
//! starts, so an accessor may resolve it once into a `OnceLock` and hand out
//! the same value for the rest of the run — the read semantics (`env::var` vs
//! `env::var_os`, presence vs value, the parse shape) of the original inline
//! call site are kept per accessor.
//!
//! Two pins are deliberately **uncached**: `TR_SUNDIAL`/`TR_MOONDIAL` are the
//! only flags any test mutates (`set_var`/`remove_var` in
//! `app/mod.rs`'s DW-1.6 pair, serialized by its `ENV_LOCK`), so
//! [`sundial_pin`]/[`moondial_pin`] re-read the environment on every call —
//! a `OnceLock` there would make the second test observe the first one's
//! value. The dynamic-key gateways ([`var_raw`], [`fake_count`]) stay live
//! for the same reason: they are keyed at runtime.
//!
//! Infra module, not a port: no C# counterpart exists. The original read sites
//! are cited in each accessor's doc comment so the port's divergences (a
//! `TR_*` pin is a harness affordance, not game logic) stay auditable.

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::OnceLock;

use glam::Vec2;
use tr_core::lighting::LightMode;

/// Uncached `std::env::var` — the escape hatch for the flags the tests mutate
/// and for the dynamic-key gateways (a `OnceLock` would latch the first
/// reader's value and break `app/mod.rs`'s env-mutating pair).
pub(crate) fn var_raw(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Uncached presence (`std::env::var_os(name).is_some()`).
pub(crate) fn present_raw(name: &str) -> bool {
    std::env::var_os(name).is_some()
}

/// Uncached `T: FromStr` parse — keeps the call site's inference (the shape of
/// `TR_EQUIPPAGE`'s struct-field default).
pub(crate) fn parsed_raw<T: FromStr>(name: &str) -> Option<T> {
    var_raw(name).and_then(|v| v.parse().ok())
}

/// The live lookup the injectable seams (`EnvPins::from_lookup`,
/// `apply_scene_forces`) bind to the environment.
pub(crate) fn lookup() -> impl Fn(&str) -> Option<String> {
    |name| var_raw(name)
}

/// Presence via `std::env::var(name).is_ok()` (`TR_STATS`, `TR_CART`, …): a
/// set-but-unparsable value still counts, unlike [`pin_bool`]'s `var_os`.
macro_rules! pin_flag {
    ($(#[$doc:meta])* $name:ident, $flag:literal) => {
        $(#[$doc])*
        pub(crate) fn $name() -> bool {
            static V: OnceLock<bool> = OnceLock::new();
            *V.get_or_init(|| std::env::var($flag).is_ok())
        }
    };
}

/// Presence via `std::env::var_os(name).is_some()` — the dominant pin shape.
macro_rules! pin_bool {
    ($(#[$doc:meta])* $name:ident, $flag:literal) => {
        $(#[$doc])*
        pub(crate) fn $name() -> bool {
            static V: OnceLock<bool> = OnceLock::new();
            *V.get_or_init(|| std::env::var_os($flag).is_some())
        }
    };
}

/// `Option<T>`: the raw string parsed as `T` without trimming (the shape of
/// `std::env::var(..).ok().and_then(|v| v.parse().ok())`).
macro_rules! pin_num {
    ($(#[$doc:meta])* $name:ident, $flag:literal, $ty:ty) => {
        $(#[$doc])*
        pub(crate) fn $name() -> Option<$ty> {
            static V: OnceLock<Option<$ty>> = OnceLock::new();
            *V.get_or_init(|| std::env::var($flag).ok().and_then(|v| v.parse().ok()))
        }
    };
}

/// `Option<String>`: the raw value, resolved once.
macro_rules! pin_str {
    ($(#[$doc:meta])* $name:ident, $flag:literal) => {
        $(#[$doc])*
        pub(crate) fn $name() -> Option<String> {
            static V: OnceLock<Option<String>> = OnceLock::new();
            V.get_or_init(|| std::env::var($flag).ok()).clone()
        }
    };
}

/// Comma-separated `i64`s (`None` when any token is not a number) — the shared
/// tail of the `TR_POOL`/`TR_PAINT`/`TR_ECHO` rectangle parsers.
pub(crate) fn tr_nums(s: &str) -> Option<Vec<i64>> {
    s.split(',').map(|v| v.trim().parse::<i64>().ok()).collect()
}

/// One `TR_PLACE` group: place this tile kind through the registry route.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TrPlace {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) kind: u16,
    pub(crate) style: i32,
    pub(crate) dir: i32,
}

/// One `TR_TOGGLE` group: `n` interaction presses at one tile.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TrToggle {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) n: u32,
}

/// One `TR_TILES` cell: write this exact tile kind with these exact frames.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TrTile {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) w: i32,
    pub(crate) h: i32,
    pub(crate) kind: i64,
    pub(crate) fx: i16,
    pub(crate) fy: i16,
}

/// `parse_xy` (app/mod.rs:640): an `x,y` pair, both tokens trimmed.
fn parse_xy(v: &str) -> Option<(i32, i32)> {
    let (a, b) = v.split_once(',')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

/// `parse_light_mode` (app/mod.rs:643): the `TR_LIGHT_MODE` name table.
fn parse_light_mode(v: &str) -> Option<LightMode> {
    match v.trim().to_ascii_lowercase().as_str() {
        "color" | "colour" | "0" => Some(LightMode::Color),
        "white" | "1" => Some(LightMode::White),
        "retro" | "2" => Some(LightMode::Retro),
        "trippy" | "glitch" | "3" => Some(LightMode::Trippy),
        _ => None,
    }
}

/// The `TR_DYE=k1,k2,...` pin parse (app/mod.rs:664): the first ten
/// comma-separated kinds fill `Player.dye` positionally; garbage and 0 leave
/// the slot empty.
pub(crate) fn parse_tr_dye(list: &str) -> [u16; 10] {
    let mut out = [0u16; 10];
    for (slot, kind) in out.iter_mut().zip(list.split(',')) {
        *slot = kind.trim().parse().unwrap_or(0);
    }
    out
}

/// The `TR_DMOUNT=<kind>` pin parse (app/mod.rs:673: `App.equip.misc_dyes[3]`);
/// garbage → 0.
pub(crate) fn parse_tr_dmount(s: &str) -> u16 {
    s.trim().parse().unwrap_or(0)
}

/// `TR_MOUNT=<id|name>` (app/mod.rs:1569): the mount name table; an unknown
/// name resolves to no pin.
fn parse_mount_id(name: &str) -> Option<i32> {
    Some(match name {
        "rudolph" => 0,
        "bunny" => 1,
        "slime" => 3,
        "scutlix" => 9,
        "unicorn" => 10,
        "basilisk" => 14,
        "horse" | "painted_horse" => 40,
        "majestic_horse" => 41,
        "dark_horse" => 42,
        "pogo" | "pogo_stick" => 43,
        "wofgoat" | "goat" => 47,
        "queenslime" => 50,
        "pigron" => 2,
        "bee" => 5,
        "ufo" => 7,
        "drill" => 8,
        "cutefishron" => 12,
        "broom" | "witchbroom" => 23,
        "pirateship" => 44,
        "book" | "darkmagebook" => 48,
        "lavashark" => 49,
        "bat" => 56,
        "pixie" => 61,
        _ => return name.parse::<i32>().ok(),
    })
}

/// `TR_ARMOR=<set>` (app/mod.rs:2055): the four early-metal armor triples;
/// any other value is no pin.
fn parse_armor_set(set: &str) -> Option<[u16; 3]> {
    let ids = match set.trim() {
        "copper" => [89, 80, 76],
        "iron" => [90, 81, 77],
        "silver" => [91, 82, 78],
        "gold" => [92, 83, 79],
        _ => return None,
    };
    Some(ids)
}

// --- Startup pins (App::new / main / load_world) -----------------------------

/// `TR_WORLD=<path>`: the `.trwld` to load (the argv fallback in `main`).
/// `var_os` so a non-UTF-8 path survives.
pub(crate) fn world_path() -> Option<PathBuf> {
    static V: OnceLock<Option<PathBuf>> = OnceLock::new();
    V.get_or_init(|| std::env::var_os("TR_WORLD").map(PathBuf::from))
        .clone()
}

pin_str! {
    /// `TR_LANG=<culture>`: the startup culture (`-lang`'s stand-in), applied
    /// ahead of the saved `Settings.language`.
    lang,
    "TR_LANG"
}

/// `TR_LOCALE=<dir>`: the first localization resource probe dir (`var_os`, the
/// path survives non-UTF-8).
pub(crate) fn locale_dir() -> Option<PathBuf> {
    static V: OnceLock<Option<PathBuf>> = OnceLock::new();
    V.get_or_init(|| std::env::var_os("TR_LOCALE").map(PathBuf::from))
        .clone()
}

pin_bool! {
    /// `TR_NOHUD=1` hides the HUD, hotbar and minimap so screenshots show only
    /// the world. The debug overlay (`TR_DEBUG`) is independent and still drawn.
    no_hud,
    "TR_NOHUD"
}

pin_bool! {
    /// `TR_NOPLAYER=1` loads the world without the player body.
    no_player,
    "TR_NOPLAYER"
}

pin_bool! {
    /// `TR_PAN` is set — the camera pans deterministically instead of following.
    pan_set,
    "TR_PAN"
}

pin_num! {
    /// `TR_PAN=<tiles/frame>`: the per-frame camera pan step (benchmarks use
    /// it to measure the moving-camera path, where the static-scene caches
    /// (D2/G6) mostly miss).
    pan_step,
    "TR_PAN",
    f32
}

pin_bool! {
    /// `TR_CAM=x,y` is set — the camera is pinned instead of following the player.
    cam_set,
    "TR_CAM"
}

/// `TR_CAM=x,y`: the pinned camera centre (`loop.rs`'s startup read) — the
/// first two comma tokens decide the pin, exactly like the original read.
pub(crate) fn camera_pin() -> Option<(f32, f32)> {
    static V: OnceLock<Option<(f32, f32)>> = OnceLock::new();
    *V.get_or_init(|| {
        let s = std::env::var("TR_CAM").ok()?;
        let mut it = s.split(',').map(|v| v.trim().parse::<f32>());
        match (it.next(), it.next()) {
            (Some(Ok(x)), Some(Ok(y))) => Some((x, y)),
            _ => None,
        }
    })
}

pin_bool! {
    /// `TR_HOLD=right,jump`: deterministic held input for screenshots.
    hold_set,
    "TR_HOLD"
}

pin_num! {
    /// `TR_ZOOM=<f>`: the pinned in-game zoom (`loop.rs`, clamped to the
    /// 1.0–2.0 in-game range there).
    zoom_pin,
    "TR_ZOOM",
    f32
}

pin_bool! {
    /// `TR_ZOOM` is set — the zoom is pinned rather than user-controlled.
    zoom_pinned,
    "TR_ZOOM"
}

pin_bool! {
    /// `TR_NOMENU=1` skips the front-end menu straight into the world.
    no_menu,
    "TR_NOMENU"
}

pin_bool! {
    /// `TR_MENU=1` forces the front-end menu even with the skip flags set.
    force_menu,
    "TR_MENU"
}

pin_bool! {
    /// `TR_MAP=1` opens the world map on start.
    map_open,
    "TR_MAP"
}

pin_bool! {
    /// `TR_JOINED=1`: the session already joined once (skips the connect flow).
    joined_before,
    "TR_JOINED"
}

pin_str! {
    /// `TR_NAME=<name>`: the player name (menu/random fallback lives at the site).
    player_name,
    "TR_NAME"
}

pin_str! {
    /// `TR_JOIN=<addr>`: the auto-join target.
    join_target,
    "TR_JOIN"
}

pin_str! {
    /// `TR_JOIN_PW=<pw>`: the auto-join password (empty when unset).
    join_password,
    "TR_JOIN_PW"
}

pin_num! {
    /// `TR_HOST=<port>`: auto-host a server on this port.
    auto_host,
    "TR_HOST",
    u16
}

pin_str! {
    /// `TR_UI=<screen>`: the front-end preset name (decoded by
    /// `flow::tr_ui_preset`).
    ui_preset_name,
    "TR_UI"
}

pin_num! {
    /// `TR_COINS=<n>`: the starting coin purse.
    coins,
    "TR_COINS",
    u64
}

pin_num! {
    /// `TR_DASH=<0|1|2|3|6>`: the pinned dash type (clamped to the ported set).
    dash_type,
    "TR_DASH",
    i32
}

pin_str! {
    /// `TR_JUMPS=cloud,sandstorm,...`: the armed extra jumps (the comma list,
    /// the `balloon` arm included — the consumer maps the names).
    jumps_list,
    "TR_JUMPS"
}

pin_num! {
    /// `TR_WINGS=<ArmorIDs.Wing>`: the pinned wing logic (clamped to the table).
    wings,
    "TR_WINGS",
    i32
}

pin_num! {
    /// `TR_BOOTS=<1..5>`: the pinned rocket-boot tier (clamped to the table).
    rocket_boots,
    "TR_BOOTS",
    i32
}

/// `TR_MOUNT=<id|name>`: the pinned mount (numeric id or the name table).
pub(crate) fn mount_pin() -> Option<i32> {
    static V: OnceLock<Option<i32>> = OnceLock::new();
    *V.get_or_init(|| {
        let v = std::env::var("TR_MOUNT").ok()?;
        parse_mount_id(v.trim().to_ascii_lowercase().as_str())
    })
}

/// `TR_SPOUT_KIND=<0..3>`: the fall's liquid kind (1 lava, 2 honey, 3
/// shimmer) for the typed waterfall branches.
pub(crate) fn spout_kind() -> u8 {
    static V: OnceLock<u8> = OnceLock::new();
    *V.get_or_init(|| {
        use tr_core::tile::{FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, FLAG_LIQUID_SHIMMER};
        match std::env::var("TR_SPOUT_KIND").as_deref() {
            Ok("1") => FLAG_LIQUID_LAVA,
            Ok("2") => FLAG_LIQUID_HONEY,
            Ok("3") => FLAG_LIQUID_SHIMMER,
            _ => 0,
        }
    })
}

/// `TR_XY=x,y`: the debug spawn override — place the player at a tile
/// coordinate (the camera follows).
pub(crate) fn spawn_xy() -> Option<(i32, i32)> {
    static V: OnceLock<Option<(i32, i32)>> = OnceLock::new();
    *V.get_or_init(|| std::env::var("TR_XY").ok().and_then(|v| parse_xy(&v)))
}

/// `TR_SPOUT=x,y`: build the `FindWaterfalls` half-brick case at a fixed tile.
pub(crate) fn spout_xy() -> Option<(i32, i32)> {
    static V: OnceLock<Option<(i32, i32)>> = OnceLock::new();
    *V.get_or_init(|| std::env::var("TR_SPOUT").ok().and_then(|v| parse_xy(&v)))
}

/// `TR_BREAK=x,y`: the dev break-target cell.
pub(crate) fn dev_break_xy() -> Option<(i32, i32)> {
    static V: OnceLock<Option<(i32, i32)>> = OnceLock::new();
    *V.get_or_init(|| std::env::var("TR_BREAK").ok().and_then(|v| parse_xy(&v)))
}

/// `TR_PLACE=x,y,kind[,style[,dir]][;...]`: the registry-route placements (the
/// `load_world` half; `App::new`'s simpler three-token tuple list is
/// [`dev_place_list`]).
pub(crate) fn place_ops() -> Vec<TrPlace> {
    std::env::var("TR_PLACE")
        .ok()
        .map(|v| {
            v.split(';')
                .filter_map(|group| {
                    let nums = tr_nums(group)?;
                    match nums[..] {
                        [x, y, kind] => Some(TrPlace {
                            x: x as i32,
                            y: y as i32,
                            kind: kind as u16,
                            style: 0,
                            dir: 1,
                        }),
                        [x, y, kind, style] => Some(TrPlace {
                            x: x as i32,
                            y: y as i32,
                            kind: kind as u16,
                            style: style as i32,
                            dir: 1,
                        }),
                        [x, y, kind, style, dir] => Some(TrPlace {
                            x: x as i32,
                            y: y as i32,
                            kind: kind as u16,
                            style: style as i32,
                            dir: dir as i32,
                        }),
                        _ => None,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `TR_PLACE=x,y,kind[;...]` (app/mod.rs:1770): the simplified dev-list shape.
pub(crate) fn dev_place_list() -> Vec<(i32, i32, u16)> {
    std::env::var("TR_PLACE")
        .ok()
        .map(|v| {
            v.split(';')
                .filter_map(|entry| {
                    let mut it = entry.split(',');
                    let x = it.next()?.trim().parse::<i32>().ok()?;
                    let y = it.next()?.trim().parse::<i32>().ok()?;
                    let k = it.next()?.trim().parse::<u16>().ok()?;
                    Some((x, y, k))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `TR_TOGGLE=x,y[,n][;...]`: the interaction-dispatcher presses.
pub(crate) fn toggle_ops() -> Vec<TrToggle> {
    std::env::var("TR_TOGGLE")
        .ok()
        .map(|v| {
            v.split(';')
                .filter_map(|group| match tr_nums(group)?[..] {
                    [x, y] => Some(TrToggle {
                        x: x as i32,
                        y: y as i32,
                        n: 1,
                    }),
                    [x, y, n] => Some(TrToggle {
                        x: x as i32,
                        y: y as i32,
                        n: n.max(0) as u32,
                    }),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `TR_TILES=x,y,kind,fx,fy[;...]`: the explicit-frame world writes (the
/// 4-token `x,y,w,h` form clears a rectangle to air).
pub(crate) fn tile_ops() -> Vec<TrTile> {
    std::env::var("TR_TILES")
        .ok()
        .map(|v| {
            v.split(';')
                .filter_map(|group| match tr_nums(group)?[..] {
                    [x, y, w, h] => Some(TrTile {
                        x: x as i32,
                        y: y as i32,
                        w: w as i32,
                        h: h as i32,
                        kind: -1,
                        fx: 0,
                        fy: 0,
                    }),
                    [x, y, kind, fx, fy] => Some(TrTile {
                        x: x as i32,
                        y: y as i32,
                        w: 1,
                        h: 1,
                        kind,
                        fx: fx as i16,
                        fy: fy as i16,
                    }),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

pin_str! {
    /// `TR_FILL=tx,ty,w,h,kind[;...]`: the raw group list (`apply_tr_fill` parses
    /// each group through [`tr_nums`]).
    fill_rects,
    "TR_FILL"
}

pin_str! {
    /// `TR_POOL=tx,ty,w,h,kind`: the raw value (the liquid rectangle's parse runs
    /// at the site over [`tr_nums`], length-checked).
    pool_rect,
    "TR_POOL"
}

pin_str! {
    /// `TR_PAINT=tx,ty,w,h,paint[,wall]`: the raw value (length 5 or 6).
    paint_rect,
    "TR_PAINT"
}

pin_str! {
    /// `TR_ECHO=tx,ty,w,h`: the raw value (length 4).
    echo_rect,
    "TR_ECHO"
}

pin_bool! {
    /// `TR_CHEST`: the chest-panel screenshot fixture — a stocked, opened
    /// style-0 chest on the spawn plateau's floor.
    chest_fixture,
    "TR_CHEST"
}

pin_bool! {
    /// `TR_OPTIONS`: opens the in-game options window at boot so
    /// `TR_SHOT` captures the `IngameOptions` panel (M7-UI-4's fixture).
    ingame_options_open,
    "TR_OPTIONS"
}

pin_bool! {
    /// `TR_POSE`: keeps the capture scene empty — a contact-damage NPC
    /// would knock the pinned player off the furniture.
    pose_set,
    "TR_POSE"
}

pin_str! {
    /// `TR_POSE=sit|sleep`: the pose to apply through the real interaction decode.
    pose_mode,
    "TR_POSE"
}

pin_bool! {
    /// `TR_THROW`: the dev throw-arm fixture.
    dev_throw,
    "TR_THROW"
}

pin_bool! {
    /// `TR_INV`: opens the inventory on start.
    inv_open,
    "TR_INV"
}

pin_bool! {
    /// `TR_BESTIARY`: opens the bestiary on start.
    bestiary_open,
    "TR_BESTIARY"
}

pin_num! {
    /// `TR_BESTIARY=<n>`: the selected bestiary entry (1-based; the site bounds-checks it).
    bestiary_selected,
    "TR_BESTIARY",
    usize
}

pin_bool! {
    /// `TR_CHAT`: opens the chat panel with seeded lines.
    chat_open,
    "TR_CHAT"
}

pin_bool! {
    /// `TR_EMOTES`: opens the emote wheel.
    emotes_open,
    "TR_EMOTES"
}

pin_bool! {
    /// `TR_ACHIEVEMENTS`: opens the achievements menu on start.
    achievements_open,
    "TR_ACHIEVEMENTS"
}

pin_num! {
    /// `TR_ACHIEVEMENTS_CATS=<bitmask>`: the category filter bits of the
    /// opened achievements menu, Slayer = 1, Collector = 2, Explorer = 4,
    /// Challenger = 8 (all on by default, `InitializePage`'s
    /// `SetState(value: true)`).
    achievements_categories,
    "TR_ACHIEVEMENTS_CATS",
    u32
}

pin_bool! {
    /// `TR_NOCRAFT=1` turns the crafting panel off (`TR_CRAFT=1` forces it back on).
    no_craft,
    "TR_NOCRAFT"
}

pin_bool! {
    /// `TR_CRAFT=1` forces the crafting panel on.
    craft_open,
    "TR_CRAFT"
}

pin_bool! {
    /// `TR_VK_GRID`: shows the virtual-keyboard grid.
    vk_grid,
    "TR_VK_GRID"
}

/// `TR_MAPSTYLE=<0|1|2>`: the minimap style (an invalid value falls back to
/// `Main.mapStyle`'s default 1).
pub(crate) fn map_style() -> i32 {
    static V: OnceLock<i32> = OnceLock::new();
    *V.get_or_init(|| {
        std::env::var("TR_MAPSTYLE")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .filter(|v| (0..=2).contains(v))
            .unwrap_or(1)
    })
}

/// `TR_DEBUG`: the debug overlay's on/off — unset is off and only an explicit
/// `0` turns a set value off.
pub(crate) fn debug() -> bool {
    static V: OnceLock<bool> = OnceLock::new();
    *V.get_or_init(|| std::env::var("TR_DEBUG").map(|v| v != "0").unwrap_or(false))
}

pin_num! {
    /// `TR_BENCH=<frames>`: the benchmark length (the app's frame gate).
    bench_frames,
    "TR_BENCH",
    u32
}

pin_str! {
    /// `TR_SHOT=<path>`: the deterministic screenshot target.
    shot_path,
    "TR_SHOT"
}

pin_num! {
    /// `TR_SHOT_FRAME=<n>`: the capture frame (default 20).
    shot_at,
    "TR_SHOT_FRAME",
    u64
}

pin_flag! {
    /// `TR_STATS`: the per-frame stats overlay (a set value counts, even garbage).
    stats_on,
    "TR_STATS"
}

/// `TR_LIGHT_MODE=<color|white|retro|trippy>`: the lighting mode (the
/// color/white 9-slice path).
pub(crate) fn light_mode() -> Option<LightMode> {
    static V: OnceLock<Option<LightMode>> = OnceLock::new();
    *V.get_or_init(|| {
        std::env::var("TR_LIGHT_MODE")
            .ok()
            .and_then(|v| parse_light_mode(&v))
    })
}

/// `TR_AMBIENCE=<SkyEntityType>[,seed]`: one forced ambience spawn with a
/// pinned seed (the `AmbienceServer.ForceEntitySpawn` path).
pub(crate) fn ambience_pin() -> Option<(crate::ambience::SkyEntityType, i32)> {
    static V: OnceLock<Option<(crate::ambience::SkyEntityType, i32)>> = OnceLock::new();
    *V.get_or_init(|| {
        let v = std::env::var("TR_AMBIENCE").ok()?;
        let mut it = v.split(',');
        let ty = it
            .next()
            .map(str::trim)
            .and_then(crate::ambience::parse_sky_entity_type);
        let seed = it
            .next()
            .map(str::trim)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);
        ty.map(|ty| (ty, seed))
    })
}

/// `TR_ARMOR=<set>`: the full early-metal set to equip at load.
pub(crate) fn armor_set() -> Option<[u16; 3]> {
    static V: OnceLock<Option<[u16; 3]>> = OnceLock::new();
    *V.get_or_init(|| {
        let v = std::env::var("TR_ARMOR").ok()?;
        parse_armor_set(&v)
    })
}

/// `TR_DYE=k1,k2,...`: the dye slots filled positionally (garbage and 0 leave
/// the slot empty).
pub(crate) fn dye_kinds() -> [u16; 10] {
    static V: OnceLock<[u16; 10]> = OnceLock::new();
    *V.get_or_init(|| {
        std::env::var("TR_DYE")
            .map(|v| parse_tr_dye(&v))
            .unwrap_or([0u16; 10])
    })
}

/// `TR_DMOUNT=<kind>`: the mount-dye slot kind (garbage → 0).
pub(crate) fn misc_dye_mount() -> u16 {
    static V: OnceLock<u16> = OnceLock::new();
    *V.get_or_init(|| {
        std::env::var("TR_DMOUNT")
            .map(|v| parse_tr_dmount(&v))
            .unwrap_or(0)
    })
}

pin_flag! {
    /// `TR_REGEN`: regenerate the scratch world even when the file exists.
    regen,
    "TR_REGEN"
}

pin_num! {
    /// `TR_SEED=<n>`: the scratch-world generator seed.
    seed,
    "TR_SEED",
    u64
}

// --- load_world pins ---------------------------------------------------------

/// `TR_DAY=<0..1>` overrides the time of day (0 = midnight, 1 = noon): the
/// sky colour and the world clock follow the same pinned half.
pub(crate) fn day_override() -> Option<f32> {
    static V: OnceLock<Option<f32>> = OnceLock::new();
    *V.get_or_init(|| {
        std::env::var("TR_DAY")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .map(|v| v.clamp(0.0, 1.0))
    })
}

pin_num! {
    /// `TR_MOON=<0..7>` pins `Main.moonPhase` (the night light floor follows it).
    moon_phase,
    "TR_MOON",
    u8
}

pin_num! {
    /// `TR_TIME=<ticks>` pins the raw world clock (the dawn/dusk windows the
    /// `TR_DAY` half mapping cannot reach).
    time_pin,
    "TR_TIME",
    f64
}

/// `TR_SUNDIAL=1` arms the dawn fast-forward at load — **live read**: the
/// DW-1.6 tests set and remove this var between `load_world` calls.
pub(crate) fn sundial_pin() -> bool {
    std::env::var("TR_SUNDIAL").is_ok()
}

/// `TR_MOONDIAL=1` arms the dusk fast-forward at load — **live read** (see
/// [`sundial_pin`]).
pub(crate) fn moondial_pin() -> bool {
    std::env::var("TR_MOONDIAL").is_ok()
}

// --- Diagnostics / atlas pins ------------------------------------------------

pin_flag! {
    /// `TR_DUMP_TILES`: dumps the near-spawn tile column histogram.
    dump_tiles,
    "TR_DUMP_TILES"
}

pin_num! {
    /// `TR_DUMP_X=<col>`: the dump window's column (defaults to the middle).
    dump_x,
    "TR_DUMP_X",
    i32
}

pin_num! {
    /// `TR_DUMP_Y=<row>`: the dump window's row (defaults below the spawn).
    dump_y,
    "TR_DUMP_Y",
    i32
}

pin_flag! {
    /// `TR_SCAN_WATER`: reports the liquid-window scan.
    scan_water,
    "TR_SCAN_WATER"
}

pin_flag! {
    /// `TR_SCAN_TORCH`: reports the torch-density window scan.
    scan_torch,
    "TR_SCAN_TORCH"
}

pin_flag! {
    /// `TR_SCAN_LIGHT`: reports the 40×40 window with the most self-lit tiles.
    scan_light,
    "TR_SCAN_LIGHT"
}

/// `TR_SCAN_Y=<a>,<b>`: the scan's row bounds (defaults to the whole world).
pub(crate) fn scan_y() -> Option<(i32, i32)> {
    static V: OnceLock<Option<(i32, i32)>> = OnceLock::new();
    *V.get_or_init(|| std::env::var("TR_SCAN_Y").ok().and_then(|v| parse_xy(&v)))
}

pin_flag! {
    /// `TR_CHECK_FRAMES`: warns about atlas frames with no user.
    check_frames,
    "TR_CHECK_FRAMES"
}

pin_str! {
    /// `TR_ATLAS_DUMP=<path>`: dumps the built atlas PNG.
    atlas_dump,
    "TR_ATLAS_DUMP"
}

// --- Per-frame sim / render pins --------------------------------------------

pin_bool! {
    /// `TR_DAY` also freezes the clock: time-of-day advancement pauses.
    day_pinned,
    "TR_DAY"
}

pin_bool! {
    /// `TR_NOCACHE=1` disables D2 and G6, so benchmarks measure the full
    /// per-frame work instead of a cached scene. Also used to A/B the
    /// optimisations.
    no_cache,
    "TR_NOCACHE"
}

pin_bool! {
    /// `TR_NO_BG`: skips the whole background/parallax pass.
    no_bg,
    "TR_NO_BG"
}

/// `TR_NO_WALLS`: the wall pass runs unless this pin is set —
/// `std::env::var("TR_NO_WALLS").is_err()` (a set value, even an empty one,
/// disables it).
pub(crate) fn walls_enabled() -> bool {
    static V: OnceLock<bool> = OnceLock::new();
    *V.get_or_init(|| std::env::var("TR_NO_WALLS").is_err())
}

pin_num! {
    /// `TR_WATER_STYLE=<n>`: pins the water style (capped at 14).
    water_style,
    "TR_WATER_STYLE",
    usize
}

/// `TR_LIGHT_AT=x,y`: log the light at one tile once (the one-shot guard
/// lives at the call site).
pub(crate) fn light_at() -> Option<(i32, i32)> {
    static V: OnceLock<Option<(i32, i32)>> = OnceLock::new();
    *V.get_or_init(|| std::env::var("TR_LIGHT_AT").ok().and_then(|v| parse_xy(&v)))
}

pin_bool! {
    /// `TR_DUMP_LIQUID`: per-frame dump of the liquid draw list near the camera.
    dump_liquid,
    "TR_DUMP_LIQUID"
}

pin_bool! {
    /// `TR_NO_LIQBACK`: skips the background liquid pass.
    no_liqback,
    "TR_NO_LIQBACK"
}

pin_bool! {
    /// `TR_NO_LIQFRONT`: skips the foreground liquid pass.
    no_liqfront,
    "TR_NO_LIQFRONT"
}

pin_bool! {
    /// `TR_NO_LIQBEHIND`: suppresses the per-tile water-behind-self quads.
    no_liqbehind,
    "TR_NO_LIQBEHIND"
}

pin_bool! {
    /// `TR_NO_FOLIAGE`: skips the tree foliage pass.
    no_foliage,
    "TR_NO_FOLIAGE"
}

pin_bool! {
    /// `TR_NO_WATERFALL`: skips the waterfall walk (and its light feed).
    no_waterfall,
    "TR_NO_WATERFALL"
}

pin_bool! {
    /// `TR_NO_WIND`: disables the sway counters (the walk itself stays).
    no_wind,
    "TR_NO_WIND"
}

pin_num! {
    /// `TR_WIND_FORCE=<speed>`: pins `windSpeedCurrent` (the sway and leaf
    /// frequency strength).
    wind_force,
    "TR_WIND_FORCE",
    f32
}

/// `TR_HOLD=right,jump`: the deterministic held input pair (presence of the
/// substring decides each half).
pub(crate) fn hold_input() -> (bool, bool) {
    static V: OnceLock<(bool, bool)> = OnceLock::new();
    *V.get_or_init(|| {
        let v = std::env::var("TR_HOLD")
            .unwrap_or_default()
            .to_ascii_lowercase();
        (v.contains("right"), v.contains("jump"))
    })
}

/// `TR_HOOK=<ProjectileID>`: the pinned grappling-hook type (default 13).
pub(crate) fn hook_type() -> i32 {
    static V: OnceLock<i32> = OnceLock::new();
    *V.get_or_init(|| {
        std::env::var("TR_HOOK")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(13)
    })
}

pin_flag! {
    /// `TR_CART`: pins the minecart mount (mounted idempotently, so a
    /// dismount re-mounts on the next tick).
    cart_pin,
    "TR_CART"
}

pin_bool! {
    /// `TR_HARDMODE=1`: runs the hardmode runner half once.
    hardmode_forced,
    "TR_HARDMODE"
}

pin_bool! {
    /// `TR_SHOW_INVISIBLE=1`: draws the invisible blocks/walls for A/B shots.
    show_invisible,
    "TR_SHOW_INVISIBLE"
}

/// `TR_NET_EDIT=tx,ty`: one client edit request at frame 90 (the round-trip
/// verification fixture).
pub(crate) fn net_edit() -> Option<(i32, i32)> {
    static V: OnceLock<Option<(i32, i32)>> = OnceLock::new();
    *V.get_or_init(|| std::env::var("TR_NET_EDIT").ok().and_then(|v| parse_xy(&v)))
}

pin_bool! {
    /// `TR_POST=1`: the post-pass debug capture.
    post_capture,
    "TR_POST"
}

/// `TR_MOUSE=x,y`: the pinned logical pointer position, ignoring real cursor
/// motion, so screenshots can deterministically show the cursor and a hovered
/// item tooltip (like `TR_CAM`/`TR_DAY` do for the world).
pub(crate) fn mouse_pin() -> Option<Vec2> {
    static V: OnceLock<Option<Vec2>> = OnceLock::new();
    *V.get_or_init(|| {
        let s = std::env::var("TR_MOUSE").ok()?;
        let mut it = s.split(',').map(|v| v.trim().parse::<f32>());
        match (it.next(), it.next()) {
            (Some(Ok(x)), Some(Ok(y))) => Some(Vec2::new(x, y)),
            _ => None,
        }
    })
}

/// True when a `TR_SHOT` capture runs without an explicit `TR_MOUSE` pin: the
/// OS pointer is then parked offscreen and `CursorMoved` is ignored, so two
/// runs of the same command produce SHA256-equal screenshots no matter where
/// the desktop mouse happens to be. The pointer is harness input, not scene
/// content; an explicit `TR_MOUSE` still wins (the cursor/tooltip shots).
pub(crate) fn deterministic_pointer_park() -> bool {
    static V: OnceLock<bool> = OnceLock::new();
    *V.get_or_init(|| shot_set() && mouse_pin().is_none())
}

pin_bool! {
    /// `TR_SHOT` is set — a capture is pending, so the front-end flow skips the menu.
    shot_set,
    "TR_SHOT"
}

pin_bool! {
    /// `TR_BOSS` is set — a boss fixture is placed beside the player.
    boss_set,
    "TR_BOSS"
}

/// `TR_BOSS=<boss>`: the boss fixture name (`golem`, `deerclops[,ai0]`, else
/// the King Slime stand-in).
pub(crate) fn boss_mode() -> String {
    static V: OnceLock<String> = OnceLock::new();
    V.get_or_init(|| std::env::var("TR_BOSS").unwrap_or_default())
        .clone()
}

pin_str! {
    /// `TR_SLIME_AT=x[,x...]`: the dev slime-spawn columns.
    slime_at,
    "TR_SLIME_AT"
}

pin_str! {
    /// `TR_NPC=<kind>[,<kind>…]`: the humanoid-walker fixture names (the consumer
    /// maps the names).
    npc_names,
    "TR_NPC"
}

pin_num! {
    /// `TR_RAIN=<0..1.75>`: the forced rain strength at session start.
    rain_forced,
    "TR_RAIN",
    f32
}

pin_bool! {
    /// `TR_NO_WEATHER`: the session starts calm (no rain, no storm).
    no_weather,
    "TR_NO_WEATHER"
}

pin_num! {
    /// `TR_FOCUS=<index>`: the focused menu row (the keyboard nav's start).
    menu_focus,
    "TR_FOCUS",
    usize
}

pin_str! {
    /// `TR_PAGE=<page>`: the menu page name (`menu::page_from_name` decodes it).
    menu_page,
    "TR_PAGE"
}

/// `TR_FAKE_CHARS`/`TR_FAKE_WORLDS=<n>`: the screenshot lists' fake counts
/// (`save_list::list_players`/`list_worlds` prepend `Player n`/`World n`).
/// Keyed at runtime, so it keeps a live read.
pub(crate) fn fake_count(key: &str) -> usize {
    var_raw(key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}
