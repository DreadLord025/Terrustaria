//! The world clock (plan stage A4): `Main.time`'s day/night wrap, the `TR_DAY`
//! freeze pin and the `Main.Disco*` colour cycle.

use std::sync::OnceLock;

use tr_core::World;

use crate::App;

/// `TR_DAY` pins the sky for screenshots; time-of-day advancement pauses.
pub(crate) fn time_frozen() -> bool {
    static FROZEN: OnceLock<bool> = OnceLock::new();
    *FROZEN.get_or_init(|| std::env::var_os("TR_DAY").is_some())
}

/// `Main.time`: a 15-minute day (`0..54000`) and a 9-minute night (`0..32400`) at 60
/// ticks per second.
pub(crate) fn advance_time(world: &mut World, dt: f32) {
    let mut t = world.time + dt as f64 * 60.0;
    if world.day_time {
        if t >= 54000.0 {
            t -= 54000.0;
            world.day_time = false;
        }
    } else if t >= 32400.0 {
        t -= 32400.0;
        world.day_time = true;
        // `Main.cs:19982`: the moon phase advances when night turns into day.
        world.moon_phase = if world.moon_phase >= 7 {
            0
        } else {
            world.moon_phase + 1
        };
    }
    world.time = t;
}

/// `Main.DoUpdate_AnimateDiscoRGB`: step the shared disco-ball colour cycle.
///
/// The original runs its six `if`s sequentially, so a style that completes on
/// this tick also advances the next style in the same tick; that cascade is kept.
pub(crate) fn advance_disco(app: &mut App) {
    const NUM: i32 = 7;
    if app.disco_style == 0 {
        app.disco[1] = (app.disco[1] as i32 + NUM).min(255) as u8;
        if app.disco[1] >= 255 {
            app.disco[1] = 255;
            app.disco_style += 1;
        }
    }
    if app.disco_style == 1 {
        app.disco[0] = (app.disco[0] as i32 - NUM).max(0) as u8;
        if app.disco[0] <= 0 {
            app.disco[0] = 0;
            app.disco_style += 1;
        }
    }
    if app.disco_style == 2 {
        app.disco[2] = (app.disco[2] as i32 + NUM).min(255) as u8;
        if app.disco[2] >= 255 {
            app.disco[2] = 255;
            app.disco_style += 1;
        }
    }
    if app.disco_style == 3 {
        app.disco[1] = (app.disco[1] as i32 - NUM).max(0) as u8;
        if app.disco[1] <= 0 {
            app.disco[1] = 0;
            app.disco_style += 1;
        }
    }
    if app.disco_style == 4 {
        app.disco[0] = (app.disco[0] as i32 + NUM).min(255) as u8;
        if app.disco[0] >= 255 {
            app.disco[0] = 255;
            app.disco_style += 1;
        }
    }
    if app.disco_style == 5 {
        app.disco[2] = (app.disco[2] as i32 - NUM).max(0) as u8;
        if app.disco[2] <= 0 {
            app.disco[2] = 0;
            app.disco_style = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_night_clock_wraps() {
        let mut w = World::new("t".into(), 8, 8);
        w.time = 53999.0;
        w.day_time = true;
        advance_time(&mut w, 1.0);
        assert!(!w.day_time);
        assert!(w.time < 100.0, "time {}", w.time);
        w.time = 32399.0;
        advance_time(&mut w, 1.0);
        assert!(w.day_time);
        assert!(w.time < 100.0, "time {}", w.time);
    }
}
