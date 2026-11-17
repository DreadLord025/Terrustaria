//! `Terraria.GameContent.Golf.GolfState` + `GolfBallTrackRecord` - the per-client
//! golf scoring state (`decompiled/Terraria/GameContent/Golf/GolfState.cs:1-173`,
//! `GolfBallTrackRecord.cs:1-33`).
//!
//! This is the piece that makes the Golfer's shop tiers reachable:
//! `GetGolfBallScore` feeds `Player.AccumulateGolfingScore`
//! (`crates/tr-app/src/angler.rs::accumulate_golfing_score`), which
//! `shop.rs`'s `StockWhen::GolfScore(500/1000/2000)` gates on.
//!
//! The C# holds a live `Projectile` reference and re-reads it (`GetLastHitBall`
//! re-validates `active`/`type`/`owner`/`ai[1]` every call). `tr-core` has no
//! projectile pool, so the ball is a value snapshot taken by the caller -
//! `&GolfBall` - and the re-validation reads that snapshot.
//!
//! `GolfState.cs:8`'s `BALL_RETURN_PENALTY = 1` is declared and never read;
//! omitting it is exact.

use glam::Vec2;

/// `ProjectileID.Sets.IsAGolfBall` (`ProjectileID.cs:428`).
pub const GOLF_BALL_PROJECTILES: &[i32] = &[
    721, 739, 740, 741, 742, 743, 744, 745, 746, 747, 748, 749, 750, 751, 752,
];

/// `ProjectileID.Sets.IsAGolfBall[kind]`.
#[inline]
pub fn is_a_golf_ball(kind: i32) -> bool {
    GOLF_BALL_PROJECTILES.contains(&kind)
}

/// `Item.IsAGolfingItem` (`Item.cs:47952-47962`): a golf ball by its `shoot`,
/// or a club/whistle id. `shoot` is `Item.shoot`.
pub fn is_a_golfing_item(item_type: i32, shoot: i32) -> bool {
    is_a_golf_ball(shoot)
        || item_type == 4039
        || (4092..=4095).contains(&item_type)
        || (4587..=4598).contains(&item_type)
}

/// The `Projectile` fields `GolfState` reads. A snapshot, not a reference.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GolfBall {
    /// `Projectile.whoAmI` - the C# indexes `_hitRecords` with it.
    pub who_am_i: usize,
    /// `Projectile.type`.
    pub kind: i32,
    /// `Projectile.owner`.
    pub owner: i32,
    /// `Projectile.active`.
    pub active: bool,
    /// `Projectile.position`.
    pub position: Vec2,
    /// `Projectile.Center`.
    pub center: Vec2,
    /// `Projectile.ai[1]` - the hit counter.
    pub ai1: f32,
    /// `Projectile.localAI[1]` - 1 once the ball has been hit.
    pub local_ai1: f32,
}

/// `GolfBallTrackRecord.cs:6-33` - the hit history and its distance score.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GolfBallTrackRecord {
    hit_locations: Vec<Vec2>,
}

impl GolfBallTrackRecord {
    /// `:10-13`.
    pub fn record_hit(&mut self, position: Vec2) {
        self.hit_locations.push(position);
    }

    /// `:21-32` - `(totalDist, hitsMade)` over consecutive hit pairs.
    fn track_info(&self) -> (f64, i32) {
        let mut hits = 0;
        let mut total = 0.0f64;
        let mut i = 0usize;
        while i + 1 < self.hit_locations.len() {
            total += (self.hit_locations[i + 1] - self.hit_locations[i]).length() as f64;
            i += 1;
            hits += 1;
        }
        (total, hits)
    }

    /// `:15-19` - `(int)(totalDistance / 16.0) / (hitsMade + 2)`.
    ///
    /// The C# truncates the tile count *then* integer-divides by `hits + 2`,
    /// so the two truncations do not collapse into one.
    pub fn get_accumulated_score(&self) -> i32 {
        let (total, hits) = self.track_info();
        ((total / 16.0) as i32) / (hits + 2)
    }
}

/// `GolfState.cs:6-173`.
#[derive(Clone, Debug)]
pub struct GolfState {
    golf_score_time: i32,
    golf_score_time_max: i32,
    golf_score_delay: i32,
    last_recorded_ball_time: f64,
    last_recorded_ball_location: Option<Vec2>,
    waiting_for_ball_to_settle: bool,
    last_swing_position: Vec2,
    last_hit_golf_ball: Option<GolfBall>,
    last_recorded_swing_count: i32,
    /// `GolfBallTrackRecord[1000]` (`:28`), keyed by `whoAmI`.
    hit_records: Vec<Option<GolfBallTrackRecord>>,
}

impl Default for GolfState {
    fn default() -> Self {
        Self::new()
    }
}

impl GolfState {
    pub fn new() -> Self {
        Self {
            golf_score_time: 0,
            golf_score_time_max: 3600, // `:12`
            golf_score_delay: 90,      // `:13`
            last_recorded_ball_time: 0.0,
            last_recorded_ball_location: None,
            waiting_for_ball_to_settle: false,
            last_swing_position: Vec2::ZERO,
            last_hit_golf_ball: None,
            last_recorded_swing_count: 0,
            hit_records: vec![None; 1000], // `:28`
        }
    }

    /// `:30` - `golfScoreTime / golfScoreTimeMax`.
    pub fn score_adjustment(&self) -> f32 {
        self.golf_score_time as f32 / self.golf_score_time_max as f32
    }

    /// `:32` - the hole only scores once the timer is past the 90-tick delay.
    pub fn should_score_hole(&self) -> bool {
        self.golf_score_time >= self.golf_score_delay
    }

    /// `:34-40`.
    fn update_score_time(&mut self) {
        if self.golf_score_time < self.golf_score_time_max {
            self.golf_score_time += 1;
        }
    }

    /// `:42-45`.
    pub fn reset_score_time(&mut self) {
        self.golf_score_time = 0;
    }

    /// `:47-50`.
    pub fn set_score_time(&mut self) {
        self.golf_score_time = self.golf_score_time_max;
    }

    /// `:52-67`. `total_game_time_seconds` is
    /// `Main.gameTimeCache.TotalGameTime.TotalSeconds`; `my_player` is
    /// `Main.myPlayer`.
    pub fn try_get_camera_tracking_position(
        &self,
        live_ball: Option<&GolfBall>,
        my_player: i32,
        total_game_time_seconds: f64,
    ) -> Option<Vec2> {
        let last_hit_ball = self.get_last_hit_ball(live_ball, my_player);
        if let Some(ball) = last_hit_ball {
            if self.waiting_for_ball_to_settle {
                return Some(ball.center);
            }
        }
        if self.last_recorded_ball_time + 2.0 >= total_game_time_seconds && last_hit_ball.is_none()
        {
            if let Some(loc) = self.last_recorded_ball_location {
                return Some(loc);
            }
        }
        None
    }

    /// `:69-76`.
    pub fn world_clear(&mut self) {
        self.last_hit_golf_ball = None;
        self.last_recorded_ball_location = None;
        self.last_recorded_ball_time = 0.0;
        self.last_recorded_swing_count = 0;
        self.waiting_for_ball_to_settle = false;
    }

    /// `:78-81`.
    pub fn cancel_ball_tracking(&mut self) {
        self.waiting_for_ball_to_settle = false;
    }

    /// `:83-95`.
    pub fn record_swing(&mut self, golf_ball: &GolfBall) {
        self.last_swing_position = golf_ball.position;
        self.last_hit_golf_ball = Some(*golf_ball);
        self.last_recorded_swing_count = golf_ball.ai1 as i32;
        self.waiting_for_ball_to_settle = true;
        let id = golf_ball.who_am_i;
        // `:90-93` - a fresh record on the first hit of a ball, or when the
        // existing slot is empty.
        if self.hit_records[id].is_none() || self.last_recorded_swing_count == 1 {
            self.hit_records[id] = Some(GolfBallTrackRecord::default());
        }
        if let Some(r) = self.hit_records[id].as_mut() {
            r.record_hit(golf_ball.position);
        }
    }

    /// `:102-109` - the stored ball is only "last hit" while it is still live,
    /// still a golf ball, still ours, and has not been re-hit since.
    fn get_last_hit_ball<'a>(
        &self,
        live_ball: Option<&'a GolfBall>,
        my_player: i32,
    ) -> Option<&'a GolfBall> {
        let stored = self.last_hit_golf_ball.as_ref()?;
        // The C# re-reads the *live* projectile; without a pool the caller
        // hands in the current state of the same slot.
        let live = live_ball.filter(|b| b.who_am_i == stored.who_am_i)?;
        if !live.active
            || !is_a_golf_ball(live.kind)
            || live.owner != my_player
            || self.last_recorded_swing_count != live.ai1 as i32
        {
            return None;
        }
        Some(live)
    }

    /// `:111-133`. `held_item_type`/`held_item_shoot` are the local player's
    /// held item (`Main.LocalPlayer.HeldItem`), and `my_player` is
    /// `Main.myPlayer`.
    pub fn update(
        &mut self,
        held_item_type: i32,
        held_item_shoot: i32,
        my_player: i32,
        live_ball: Option<&GolfBall>,
    ) {
        self.update_score_time();
        if self.get_last_hit_ball(live_ball, my_player).is_none() {
            self.waiting_for_ball_to_settle = false;
            return;
        }
        if self.waiting_for_ball_to_settle {
            if let Some(live) = live_ball {
                self.waiting_for_ball_to_settle = live.local_ai1 as i32 == 1;
            }
        }
        // `:124-132` - the 3611 (Golf Whistle) special case, then the item gate.
        let flag = held_item_type == 3611;
        if !is_a_golfing_item(held_item_type, held_item_shoot) && !flag {
            self.waiting_for_ball_to_settle = false;
        }
    }

    /// `:135-142`.
    pub fn record_ball_info(
        &mut self,
        golf_ball: &GolfBall,
        live_ball: Option<&GolfBall>,
        my_player: i32,
        total_game_time_seconds: f64,
    ) {
        let is_last = self
            .get_last_hit_ball(live_ball, my_player)
            .is_some_and(|b| b.who_am_i == golf_ball.who_am_i);
        if is_last && self.waiting_for_ball_to_settle {
            self.last_recorded_ball_location = Some(golf_ball.center);
            self.last_recorded_ball_time = total_game_time_seconds;
        }
    }

    /// `:144-148`.
    pub fn land_ball(&mut self, golf_ball: &GolfBall) {
        let id = golf_ball.who_am_i;
        if let Some(r) = self.hit_records[id].as_mut() {
            r.record_hit(golf_ball.position);
        }
    }

    /// `:150-159` - the accumulated distance score scaled by the timer.
    pub fn get_golf_ball_score(&self, golf_ball: &GolfBall) -> i32 {
        let id = golf_ball.who_am_i;
        match self.hit_records[id].as_ref() {
            None => 0,
            Some(r) => (r.get_accumulated_score() as f32 * self.score_adjustment()) as i32,
        }
    }

    /// `:161-172` - return the ball to the swing point unless it is already
    /// within 1 px of it. Returns `true` when the ball was moved, which is the
    /// caller's cue to bump `ai[1]`/`netUpdate2` (the C# writes the live
    /// projectile).
    pub fn reset_golf_ball(&mut self, live_ball: Option<&GolfBall>, my_player: i32) -> bool {
        let Some(ball) = self.get_last_hit_ball(live_ball, my_player) else {
            return false;
        };
        if (ball.position - self.last_swing_position).length() < 1.0 {
            return false;
        }
        self.last_recorded_swing_count = ball.ai1 as i32 + 1;
        true
    }

    /// The position the last swing was taken from (`:22`, `:85`).
    pub fn last_swing_position(&self) -> Vec2 {
        self.last_swing_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ball(id: usize, pos: Vec2) -> GolfBall {
        GolfBall {
            who_am_i: id,
            kind: 721,
            owner: 0,
            active: true,
            position: pos,
            center: pos,
            ai1: 1.0,
            local_ai1: 1.0,
        }
    }

    #[test]
    fn the_score_is_distance_over_sixteen_divided_by_hits_plus_two() {
        // `GolfBallTrackRecord.cs:15-19`. Three hits 160 px apart: total 320,
        // so 320/16 = 20 tiles, hits = 2, 20 / 4 = 5.
        let mut r = GolfBallTrackRecord::default();
        r.record_hit(Vec2::new(0.0, 0.0));
        r.record_hit(Vec2::new(160.0, 0.0));
        r.record_hit(Vec2::new(320.0, 0.0));
        assert_eq!(r.get_accumulated_score(), 5);

        // A single hit has no distance and no hits.
        let mut one = GolfBallTrackRecord::default();
        one.record_hit(Vec2::new(0.0, 0.0));
        assert_eq!(one.get_accumulated_score(), 0);
    }

    #[test]
    fn the_tile_count_truncates_before_the_hit_division() {
        // The two truncations do not commute: 15.9 px is 0 tiles (not 0.99),
        // and the `+2` divides the *truncated* count.
        let mut r = GolfBallTrackRecord::default();
        r.record_hit(Vec2::new(0.0, 0.0));
        r.record_hit(Vec2::new(15.9, 0.0));
        // 15.9/16 = 0.99 -> 0, hits = 1, 0/3 = 0.
        assert_eq!(r.get_accumulated_score(), 0);

        let mut r2 = GolfBallTrackRecord::default();
        r2.record_hit(Vec2::new(0.0, 0.0));
        r2.record_hit(Vec2::new(160.0, 0.0));
        // 160/16 = 10, hits = 1, 10/3 = 3 (integer division).
        assert_eq!(r2.get_accumulated_score(), 3);
    }

    #[test]
    fn the_score_timer_gates_the_hole_and_clamps_at_3600() {
        // `:12-13`, `:30-50` - delay 90, max 3600; `ScoreAdjustment` is
        // time/max, and `ShouldScoreHole` is `time >= 90`.
        let mut s = GolfState::new();
        assert!(!s.should_score_hole(), "a fresh state cannot score");
        assert_eq!(s.score_adjustment(), 0.0);

        s.set_score_time();
        assert!(s.should_score_hole());
        assert_eq!(s.score_adjustment(), 1.0);

        // update clamps at the max rather than overflowing.
        for _ in 0..10 {
            s.update(0, 0, 0, None);
        }
        assert_eq!(s.score_adjustment(), 1.0);

        s.reset_score_time();
        assert_eq!(s.score_adjustment(), 0.0);
        // 89 ticks is still short of the 90-tick delay.
        for _ in 0..89 {
            s.update(0, 0, 0, None);
        }
        assert!(!s.should_score_hole());
        s.update(0, 0, 0, None);
        assert!(s.should_score_hole(), "tick 90 crosses the delay");
    }

    #[test]
    fn the_accumulated_score_scales_by_the_timer() {
        // `:158` - `(int)(recordScore * ScoreAdjustment)`.
        let mut s = GolfState::new();
        let b = ball(7, Vec2::new(0.0, 0.0));
        s.record_swing(&b);
        let hit2 = ball(7, Vec2::new(320.0, 0.0));
        s.land_ball(&hit2);

        // Timer at 0: the score is scaled to nothing.
        assert_eq!(s.get_golf_ball_score(&b), 0);
        // Timer full: 320/16 = 20 tiles, hits = 1, 20/3 = 6.
        s.set_score_time();
        assert_eq!(s.get_golf_ball_score(&b), 6);
    }

    #[test]
    fn a_swing_restarts_the_record_on_the_balls_first_hit() {
        // `:90-93` - a fresh record when the slot is empty *or* the swing count
        // is 1, so re-teeing a ball does not inherit the old distance.
        let mut s = GolfState::new();
        let mut b = ball(3, Vec2::new(0.0, 0.0));
        s.record_swing(&b);
        s.land_ball(&ball(3, Vec2::new(160.0, 0.0)));
        s.set_score_time();
        assert!(s.get_golf_ball_score(&b) > 0);

        // First hit of a fresh ball in the same slot resets the track.
        b.ai1 = 1.0;
        s.record_swing(&b);
        assert_eq!(s.get_golf_ball_score(&b), 0, "the old distance is gone");
    }

    #[test]
    fn the_last_hit_ball_is_dropped_when_it_goes_inactive_or_is_re_hit() {
        // `:102-109` - four guards: active, IsAGolfBall, owner == myPlayer,
        // and the swing count matching `ai[1]`.
        let mut s = GolfState::new();
        let b = ball(5, Vec2::ZERO);
        s.record_swing(&b);

        let mut gone = b;
        gone.active = false;
        assert!(s.get_last_hit_ball(Some(&gone), 0).is_none());

        let mut wrong_kind = b;
        wrong_kind.kind = 1;
        assert!(s.get_last_hit_ball(Some(&wrong_kind), 0).is_none());

        let mut rehit = b;
        rehit.ai1 = 9.0;
        assert!(s.get_last_hit_ball(Some(&rehit), 0).is_none());

        assert!(s.get_last_hit_ball(Some(&b), 0).is_some());
        // A different slot is not the tracked ball.
        assert!(s.get_last_hit_ball(Some(&ball(6, Vec2::ZERO)), 0).is_none());
        // A ball owned by another player is not ours (`:104` against
        // `Main.myPlayer`); the guard is a real comparison, not a constant.
        let mut foreign = b;
        foreign.owner = 3;
        assert!(s.get_last_hit_ball(Some(&foreign), 0).is_none());
        assert!(s.get_last_hit_ball(Some(&foreign), 3).is_some());
    }

    #[test]
    fn the_ball_returns_to_the_swing_point_unless_already_there() {
        // `:161-172` - the 1 px tolerance.
        let mut s = GolfState::new();
        let b = ball(9, Vec2::new(100.0, 100.0));
        s.record_swing(&b);
        assert_eq!(s.last_swing_position(), Vec2::new(100.0, 100.0));

        // Within 1 px: no reset.
        let mut near = b;
        near.position = Vec2::new(100.5, 100.0);
        assert!(!s.reset_golf_ball(Some(&near), 0));

        // Further away: reset, and the swing count advances.
        let mut far = b;
        far.position = Vec2::new(300.0, 100.0);
        assert!(s.reset_golf_ball(Some(&far), 0));
    }

    #[test]
    fn only_golf_items_keep_the_ball_tracked() {
        // `Item.cs:47952-47962`.
        assert!(is_a_golfing_item(4039, 0), "the iron club");
        assert!(is_a_golfing_item(4092, 0) && is_a_golfing_item(4095, 0));
        assert!(is_a_golfing_item(4587, 0) && is_a_golfing_item(4598, 0));
        assert!(is_a_golfing_item(0, 721), "a ball by its shoot");
        assert!(!is_a_golfing_item(4091, 0));
        assert!(!is_a_golfing_item(4599, 0));
        assert!(!is_a_golfing_item(1, 1));
    }

    #[test]
    fn holding_a_non_golf_item_cancels_the_settle_wait() {
        // `:129-132` - `!IsAGolfingItem(held) && !whistle` clears the flag.
        let mut s = GolfState::new();
        let b = ball(11, Vec2::ZERO);
        s.record_swing(&b);
        assert!(s.waiting_for_ball_to_settle);
        s.update(1, 0, 0, Some(&b));
        assert!(!s.waiting_for_ball_to_settle);

        // The whistle (3611) is explicitly allowed through (`:124-128`).
        s.record_swing(&b);
        assert!(s.waiting_for_ball_to_settle);
        s.update(3611, 0, 0, Some(&b));
        assert!(s.waiting_for_ball_to_settle, "the whistle keeps tracking");
    }

    #[test]
    fn camera_tracking_follows_the_live_ball_then_the_last_recorded_point() {
        // `:52-67`.
        let mut s = GolfState::new();
        let b = ball(13, Vec2::new(500.0, 500.0));
        s.record_swing(&b);
        // Waiting for the ball to settle -> follow it.
        assert_eq!(
            s.try_get_camera_tracking_position(Some(&b), 0, 10.0),
            Some(Vec2::new(500.0, 500.0))
        );

        // The ball is gone, but the recorded location is within 2 s.
        s.record_ball_info(&b, Some(&b), 0, 100.0);
        let none: Option<&GolfBall> = None;
        assert_eq!(
            s.try_get_camera_tracking_position(none, 0, 101.0),
            Some(Vec2::new(500.0, 500.0))
        );
        // Past the 2 s window it stops.
        assert_eq!(s.try_get_camera_tracking_position(none, 0, 103.0), None);
    }

    #[test]
    fn world_clear_drops_the_ball_and_the_swing_count() {
        // `:69-76`.
        let mut s = GolfState::new();
        let b = ball(2, Vec2::ZERO);
        s.record_swing(&b);
        s.world_clear();
        assert!(!s.waiting_for_ball_to_settle);
        assert_eq!(s.try_get_camera_tracking_position(Some(&b), 0, 1.0), None);
    }
}
