//! The named draw-layer stack — `GameInterfaceLayer` / `LegacyGameInterfaceLayer`
//! / `InterfaceScaleType` (`Terraria/UI/GameInterfaceLayer.cs`,
//! `LegacyGameInterfaceLayer.cs`, `InterfaceScaleType.cs`) and the vanilla
//! 43-layer table `Main.SetupDrawInterfaceLayers` builds
//! (`Main.cs:44712-44921`), plus `DrawInterface`'s stop-on-false walk
//! (`Main.cs:44930-44935`).
//!
//! Screen-model ruling: `app/frame.rs` still draws the HUD in its fixed code
//! order (research 18 deviation 6 / the "Layer stack" seed); this ports the
//! *machinery + data* — the layer names, their scale spaces and the walk
//! semantics — so a screen that must draw between existing HUD layers can be
//! inserted by name against the vanilla order instead of by code position.
//! Wiring `frame.rs`'s passes through the table is the follow-up (the draw
//! bodies are the per-screen immediate ports, not `DrawInterface_N` methods).

/// `Terraria/UI/InterfaceScaleType.cs:3-8` — the three scale spaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterfaceScaleType {
    /// `PlayerInput.SetZoom_World()` + `Main.GameViewMatrix.ZoomMatrix`
    /// (`GameInterfaceLayer.cs:23-27`) — world pixels under the camera zoom.
    Game,
    /// `PlayerInput.SetZoom_UI()` + `Main.UIScaleMatrix` (`:28-32`) — UI pixels
    /// under `Main.UIScale`.
    Ui,
    /// `PlayerInput.SetZoom_Unscaled()` + `Matrix.Identity` (`:33-37`) — raw
    /// backbuffer pixels.
    None,
}

/// Which transform `GameInterfaceLayer.Draw` begins the batch with
/// (`GameInterfaceLayer.cs:22-37`). The `SetZoom_*` side effects are the
/// caller's (they set `PlayerInput`'s zoom context for input scaling).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerMatrix {
    /// `Main.GameViewMatrix.ZoomMatrix`.
    WorldZoom,
    /// `Main.UIScaleMatrix`.
    UiScale,
    /// `Matrix.Identity`.
    Identity,
}

/// `GameInterfaceLayer.Draw`'s matrix selection (`GameInterfaceLayer.cs:22-37`).
pub fn layer_matrix(scale: InterfaceScaleType) -> LayerMatrix {
    match scale {
        InterfaceScaleType::Game => LayerMatrix::WorldZoom,
        InterfaceScaleType::Ui => LayerMatrix::UiScale,
        InterfaceScaleType::None => LayerMatrix::Identity,
    }
}

/// `GameInterfaceLayer` (`GameInterfaceLayer.cs:8-18`) — a named layer with a
/// scale space. The `DrawSelf` body is the caller's closure (the C#'s
/// `LegacyGameInterfaceLayer._drawMethod`, `LegacyGameInterfaceLayer.cs:7-14`,
/// whose default scale type is `Game`, `:9`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameInterfaceLayer {
    pub name: &'static str,
    pub scale: InterfaceScaleType,
}

/// `Main.SetupDrawInterfaceLayers` (`Main.cs:44718-44918`) — the 43 vanilla
/// layers in draw order. Scale types are `Game` unless the ctor's third
/// argument says `InterfaceScaleType.UI` (the `LegacyGameInterfaceLayer` default
/// is `Game`, `LegacyGameInterfaceLayer.cs:9`); no vanilla layer uses `None`.
pub const VANILLA_LAYERS: [GameInterfaceLayer; 43] = [
    // `Main.cs:44720-44769` — the world-space head of the stack.
    GameInterfaceLayer {
        name: "Vanilla: Interface Logic 1",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: MP Player Names",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: Emote Bubbles",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: Entity Markers",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: Smart Cursor Targets",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: Laser Ruler",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: Ruler",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: Gamepad Lock On",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: Tile Grid Option",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: Town NPC House Banners",
        scale: InterfaceScaleType::Game,
    },
    // `Main.cs:44770-44783`.
    GameInterfaceLayer {
        name: "Vanilla: Hide UI Toggle",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Wire Selection",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Capture Manager Check",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: Ingame Options",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Fancy UI",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Achievement Complete Popups",
        scale: InterfaceScaleType::Ui,
    },
    // `Main.cs:44784-44808`.
    GameInterfaceLayer {
        name: "Vanilla: Entity Health Bars",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: Invasion Progress Bars",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Map / Minimap",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Diagnose Net",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Sign Tile Bubble",
        scale: InterfaceScaleType::Game,
    },
    // `Main.cs:44809-44833`.
    GameInterfaceLayer {
        name: "Vanilla: Hair Window",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Dresser Window",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: NPC / Sign Dialog",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Interface Logic 2",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: Resource Bars",
        scale: InterfaceScaleType::Ui,
    },
    // `Main.cs:44834-44873`.
    GameInterfaceLayer {
        name: "Vanilla: Interface Logic 3",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: Inventory",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Info Accessories Bar",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Settings Button",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Control Hints",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Hotbar",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Builder Accessories Bar",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Radial Hotbars",
        scale: InterfaceScaleType::Ui,
    },
    // `Main.cs:44874-44918`.
    GameInterfaceLayer {
        name: "Vanilla: Mouse Text",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Player Chat",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Death Text",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Cursor",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Debug Stuff",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Mouse Item / NPC Head",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Mouse Over",
        scale: InterfaceScaleType::Game,
    },
    GameInterfaceLayer {
        name: "Vanilla: Interact Item Icon",
        scale: InterfaceScaleType::Ui,
    },
    GameInterfaceLayer {
        name: "Vanilla: Interface Logic 4",
        scale: InterfaceScaleType::Ui,
    },
];

/// `Main.DrawInterface`'s layer walk (`Main.cs:44930-44935`):
/// `while (enumerator.MoveNext() && enumerator.Current.Draw())` — a layer whose
/// `DrawSelf` returns **false** stops the whole stack (the hide-UI / capture
/// gates use this). `draw_self(index)` is the layer's body
/// (`LegacyGameInterfaceLayer.DrawSelf`, `:16-19`); the C#'s exception catch
/// (`GameInterfaceLayer.cs:44-47`) also yields false, which the caller models by
/// returning false. Returns the number of layers drawn (the stopping layer
/// counts — its `Draw` ran).
pub fn draw_interface(
    layers: &[GameInterfaceLayer],
    mut draw_self: impl FnMut(usize) -> bool,
) -> usize {
    let mut drawn = 0;
    for (i, _layer) in layers.iter().enumerate() {
        drawn += 1;
        if !draw_self(i) {
            break;
        }
    }
    drawn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_vanilla_table_is_43_layers_in_main_order() {
        // `Main.cs:44718` — `new GameInterfaceLayer[43]`; spot-pin head, tail
        // and the seams against the ctor list.
        assert_eq!(VANILLA_LAYERS.len(), 43);
        assert_eq!(VANILLA_LAYERS[0].name, "Vanilla: Interface Logic 1");
        assert_eq!(VANILLA_LAYERS[10].name, "Vanilla: Hide UI Toggle");
        assert_eq!(VANILLA_LAYERS[27].name, "Vanilla: Inventory");
        assert_eq!(VANILLA_LAYERS[40].name, "Vanilla: Mouse Over");
        assert_eq!(VANILLA_LAYERS[42].name, "Vanilla: Interface Logic 4");
    }

    #[test]
    fn the_scale_spaces_match_the_ctor_third_arguments() {
        // Default `Game` (`LegacyGameInterfaceLayer.cs:9`); explicit `UI` at
        // `Main.cs:44770,44775,44777-44778,...`; `Mouse Over` is world-space
        // (`:44904-44908`, no third argument) while its neighbours are UI.
        assert_eq!(VANILLA_LAYERS[0].scale, InterfaceScaleType::Game);
        assert_eq!(VANILLA_LAYERS[10].scale, InterfaceScaleType::Ui);
        assert_eq!(VANILLA_LAYERS[12].scale, InterfaceScaleType::Game); // Capture Manager Check
        assert_eq!(VANILLA_LAYERS[16].scale, InterfaceScaleType::Game); // Entity Health Bars
        assert_eq!(VANILLA_LAYERS[20].scale, InterfaceScaleType::Game); // Sign Tile Bubble
        assert_eq!(VANILLA_LAYERS[40].scale, InterfaceScaleType::Game); // Mouse Over
        assert_eq!(VANILLA_LAYERS[41].scale, InterfaceScaleType::Ui); // Interact Item Icon
        assert!(VANILLA_LAYERS
            .iter()
            .all(|l| l.scale != InterfaceScaleType::None));
        assert_eq!(
            VANILLA_LAYERS
                .iter()
                .filter(|l| l.scale == InterfaceScaleType::Game)
                .count(),
            16
        );
    }

    #[test]
    fn the_matrix_selection_follows_game_interface_layer_draw() {
        // `GameInterfaceLayer.cs:22-37`.
        assert_eq!(
            layer_matrix(InterfaceScaleType::Game),
            LayerMatrix::WorldZoom
        );
        assert_eq!(layer_matrix(InterfaceScaleType::Ui), LayerMatrix::UiScale);
        assert_eq!(
            layer_matrix(InterfaceScaleType::None),
            LayerMatrix::Identity
        );
    }

    #[test]
    fn the_walk_stops_on_the_first_false_and_counts_the_stopping_layer() {
        // `Main.cs:44932` — `Current.Draw()` false ends the while; the layer
        // that returned false DID run (its Draw executed).
        let mut visited = Vec::new();
        let drawn = draw_interface(&VANILLA_LAYERS, |i| {
            visited.push(i);
            i != 2
        });
        assert_eq!(drawn, 3);
        assert_eq!(visited, vec![0, 1, 2]);
        // All-true runs the whole stack.
        assert_eq!(draw_interface(&VANILLA_LAYERS, |_| true), 43);
        // An empty stack draws nothing.
        assert_eq!(draw_interface(&[], |_| true), 0);
    }
}
