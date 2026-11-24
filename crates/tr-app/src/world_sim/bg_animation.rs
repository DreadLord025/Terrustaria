//! The forest background-set animation and change-flash caches (W4-g70):
//! `Main.DoUpdate_AnimateBackgrounds` (`Main.cs:19650-19735`) and
//! `WorldGen.BackgroundsCache` (`GameContent/BackgroundChangeFlashInfo.cs`).
//! Both are update-side: the draw path only reads their resolved ids/powers.

/// `Main.bgFrame[bgIndex]` / `Main.bgFrameCounter[bgIndex]` for the four
/// forest slots (bg indexes 0/10/11/12 — the array is indexed by the
/// *front-layer* slot, but only these four entries are ever touched).
#[derive(Default)]
pub(crate) struct ForestBgAnimation {
    pub frame: [u32; 4],
    pub frame_counter: [u32; 4],
}

impl ForestBgAnimation {
    /// `DoUpdate_AnimateBackgrounds` (`Main.cs:19650-19656`): one
    /// `UpdateForest` per slot, in slot order 0, 10, 11, 12.
    pub fn update(&mut self, hills: &mut [[i32; 2]; 4]) {
        for i in 0..4 {
            self.update_forest(i, &mut hills[i]);
        }
    }

    /// `DoUpdate_AnimateBackgrounds_UpdateForest` (`Main.cs:19658-19735`).
    /// Two independent arms: the 94/114-116 near-hill family (which also
    /// rewrites the FAR hill to the 93/168-170 row) and the 180-183 near-hill
    /// family (near only). The gate reads the CURRENT `bgSet[1]`, so once the
    /// first arm has rewritten it out of 180..183 the second arm no-ops.
    fn update_forest(&mut self, i: usize, bg_set: &mut [i32; 2]) {
        // `Main.cs:19660-19704`: the 94-family arm.
        if bg_set[1] == 94 || (114..=116).contains(&bg_set[1]) {
            self.frame_counter[i] += 1;
            if self.frame_counter[i] >= 6 {
                self.frame_counter[i] = 0;
                self.frame[i] += 1;
                if self.frame[i] >= 4 {
                    self.frame[i] = 0;
                }
            }
            bg_set[1] = match self.frame[i] {
                0 => 94,
                1 => 114,
                2 => 115,
                _ => 116,
            };
            bg_set[0] = match self.frame[i] {
                0 => 93,
                1 => 168,
                2 => 169,
                _ => 170,
            };
        }
        // `Main.cs:19705-19734`: the 180-family arm (early return in C#).
        if !(180..=183).contains(&bg_set[1]) {
            return;
        }
        self.frame_counter[i] += 1;
        if self.frame_counter[i] >= 6 {
            self.frame_counter[i] = 0;
            self.frame[i] += 1;
            if self.frame[i] >= 4 {
                self.frame[i] = 0;
            }
        }
        bg_set[1] = match self.frame[i] {
            0 => 180,
            1 => 181,
            2 => 182,
            _ => 183,
        };
    }
}

/// The live hill ids: `Main.treeMntBGSet1..4` as `setBG` left them, stepped
/// by the animation each update. Kept beside the styles on `App` because the
/// animation mutates the ids in place (`bgSet[1] = 94` etc.), exactly like
/// the C# statics.
pub(crate) fn hills_from_styles(styles: &[u8; 4]) -> [[i32; 2]; 4] {
    std::array::from_fn(|i| tr_core::world::World::forest_bg_set(styles[i] as i32).0)
}

/// `WorldGen.BackgroundsCache` (`BackgroundChangeFlashInfo.cs:5-58`): the
/// 13-area variation snapshot and its change flash. `UpdateCache` runs on
/// world load / net world-info (`WorldGen.cs:7944`, `MessageBuffer.cs:529`);
/// `UpdateFlashValues` decays every power 0.05 per update
/// (`Main.cs:17218`); the draw path reads `GetFlashPower` through
/// `DrawSurfaceBG_DrawChangeOverlay` (`Main.cs:61098-61107`).
#[derive(Default)]
pub(crate) struct BackgroundChangeFlash {
    variations: [i32; 13],
    flash_power: [f32; 13],
}

impl BackgroundChangeFlash {
    /// `UpdateCache` (`BackgroundChangeFlashInfo.cs:11-26`): the area order
    /// is treeBG1..4, corrupt, jungle, snow, hallow, crimson, desert, ocean,
    /// mushroom, underworld. The port owns treeBG1..4 + the five variants;
    /// snow/desert/ocean/underworld ride their constants-off 0.
    pub fn update_cache(&mut self, world: &tr_core::world::World) {
        let values = [
            world.forest_bg_styles[0] as i32,
            world.forest_bg_styles[1] as i32,
            world.forest_bg_styles[2] as i32,
            world.forest_bg_styles[3] as i32,
            world.corrupt_bg_style as i32,
            world.jungle_bg_style as i32,
            0, // snowBG — unowned
            world.hallow_bg_style as i32,
            world.crimson_bg_style as i32,
            0, // desertBG — unowned
            0, // oceanBG — unowned
            world.mushroom_bg_style as i32,
            0, // underworldBG — unowned
        ];
        for (area, &value) in values.iter().enumerate() {
            self.update_variation(area, value);
        }
    }

    /// `UpdateVariation` + `ValueChanged` (`:28-44`): a changed variation
    /// latches the power to 1 (the `!Main.gameMenu` gate holds in-world).
    fn update_variation(&mut self, area: usize, value: i32) {
        let old = self.variations[area];
        self.variations[area] = value;
        if old != value {
            self.flash_power[area] = 1.0;
        }
    }

    /// `UpdateFlashValues` (`:46-52`): `Clamp(power - 0.05, 0, 1)` per area.
    pub fn update_flash_values(&mut self) {
        for p in &mut self.flash_power {
            *p = (*p - 0.05).clamp(0.0, 1.0);
        }
    }

    /// `GetFlashPower` (`:54-57`).
    pub fn get_flash_power(&self, area: usize) -> f32 {
        self.flash_power[area]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `DoUpdate_AnimateBackgrounds_UpdateForest`'s 94-family arm: the frame
    /// advances every 6 ticks, wraps at 4, and rewrites BOTH hill slots
    /// (`Main.cs:19660-19704`).
    #[test]
    fn the_94_family_steps_both_hills_every_six_ticks() {
        let mut anim = ForestBgAnimation::default();
        let mut hills = [[7i32, 94], [7, 8], [7, 8], [7, 8]];
        // Tick 1..5 hold frame 0; tick 6 lands frame 1.
        for _ in 0..5 {
            anim.update(&mut hills);
            assert_eq!(hills[0], [93, 94]);
        }
        anim.update(&mut hills);
        assert_eq!(hills[0], [168, 114]);
        for _ in 0..6 {
            anim.update(&mut hills);
        }
        assert_eq!(hills[0], [169, 115]);
        for _ in 0..6 {
            anim.update(&mut hills);
        }
        assert_eq!(hills[0], [170, 116]);
        for _ in 0..6 {
            anim.update(&mut hills);
        }
        // The wrap: frame 3 -> 0, counter back to 0.
        assert_eq!(hills[0], [93, 94]);
        assert_eq!(anim.frame[0], 0);
        assert_eq!(anim.frame_counter[0], 0);
    }

    /// The 180-family arm rewrites only the near hill, and the 94 arm's gate
    /// reads the CURRENT id — a style-8 set (179/180) never grows a far row
    /// (`Main.cs:19705-19734`).
    #[test]
    fn the_180_family_keeps_the_far_hill() {
        let mut anim = ForestBgAnimation::default();
        let mut hills = [[179i32, 180], [7, 8], [7, 8], [7, 8]];
        for _ in 0..6 {
            anim.update(&mut hills);
        }
        assert_eq!(hills[0], [179, 181]);
        for _ in 0..18 {
            anim.update(&mut hills);
        }
        assert_eq!(hills[0], [179, 180]);
    }

    /// A non-animated near hill (style 0's `8`) burns no counter and stays.
    #[test]
    fn a_static_hill_is_untouched() {
        let mut anim = ForestBgAnimation::default();
        let mut hills = [[7i32, 8], [7, 8], [7, 8], [7, 8]];
        for _ in 0..100 {
            anim.update(&mut hills);
        }
        assert_eq!(hills[0], [7, 8]);
        assert_eq!(anim.frame_counter[0], 0);
    }

    /// `hills_from_styles` resolves through `SetForestBGSet`.
    #[test]
    fn hills_resolve_from_the_four_styles() {
        let hills = hills_from_styles(&[0, 4, 8, 13]);
        assert_eq!(hills, [[7, 8], [93, 94], [179, 180], [7, -1]]);
    }

    /// `BackgroundChangeFlashInfo`: a variation change latches 1, the decay
    /// is 0.05/update clamped at 0, and an unchanged cache update pays no
    /// flash (`BackgroundChangeFlashInfo.cs:28-52`).
    #[test]
    fn the_flash_latches_on_change_and_decays_by_five_hundredths() {
        let mut cache = BackgroundChangeFlash::default();
        let mut w = tr_core::world::World::new("t".into(), 40, 40);
        cache.update_cache(&w); // first snapshot: all zeros, no change
        assert_eq!(cache.get_flash_power(0), 0.0);
        w.forest_bg_styles[0] = 1;
        cache.update_cache(&w);
        assert_eq!(cache.get_flash_power(0), 1.0);
        assert_eq!(cache.get_flash_power(1), 0.0);
        // 20 updates drain it exactly to 0; the 21st clamps.
        for _ in 0..20 {
            cache.update_flash_values();
        }
        assert_eq!(cache.get_flash_power(0), 0.0);
        cache.update_flash_values();
        assert_eq!(cache.get_flash_power(0), 0.0);
        // Re-snapshotting the same styles pays nothing.
        cache.update_cache(&w);
        assert_eq!(cache.get_flash_power(0), 0.0);
    }

    /// The area order of `UpdateCache`: forest slots first, then the five
    /// variants at their C# areas (corrupt 4, jungle 5, hallow 7, crimson 8,
    /// mushroom 11).
    #[test]
    fn the_flash_areas_follow_the_update_cache_order() {
        let mut cache = BackgroundChangeFlash::default();
        let mut w = tr_core::world::World::new("t".into(), 40, 40);
        w.forest_bg_styles = [1, 2, 3, 4];
        w.corrupt_bg_style = 1;
        w.jungle_bg_style = 1;
        w.hallow_bg_style = 1;
        w.crimson_bg_style = 1;
        w.mushroom_bg_style = 1;
        cache.update_cache(&w);
        for area in [0, 1, 2, 3, 4, 5, 7, 8, 11] {
            assert_eq!(cache.get_flash_power(area), 1.0, "area {area}");
        }
        for area in [6, 9, 10, 12] {
            assert_eq!(cache.get_flash_power(area), 0.0, "area {area}");
        }
    }
}
