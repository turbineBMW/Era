/// Physics model for kinetic (inertial) scrolling.
///
/// This is a Rust port of GTK's `GtkKineticScrolling`, restricted to the decelerating phase.
/// Overshoot is omitted.
///
/// The motion follows the differential equation:
///
///   y'' = -friction * y'
///
/// whose solution is:
///
///   position(t) = c1 + c2 * exp(-friction * t)
///   velocity(t) = -friction * c2 * exp(-friction * t)
///
/// where t is the elapsed time in seconds, and c1/c2 are derived from the initial position and
/// velocity.
#[derive(Debug)]
pub struct KineticScrolling {
    /// Exponential decay constant.
    friction: f64,

    c1: f64,
    c2: f64,

    /// Frame time (microsecond) at which this instance was created.
    t0: i64,

    /// Whether the deceleration has finished.
    finished: bool,

    /// The rounded pixel position stored when the animation stops, returned by any subsequent tick
    /// calls.
    final_position: f64,
}

const DECELERATION_FRICTION: f64 = 4.0;

/// Minimum velocity (px/s) below which deceleration is considered finished.
const STOP_VELOCITY: f64 = 0.1;

impl KineticScrolling {
    /// Creates a new instance starting at `initial_position` (pixels) with `initial_velocity`
    /// (pixels / second). `frame_time` is the current frame clock time in microseconds.
    pub fn new(frame_time: i64, initial_position: f64, initial_velocity: f64) -> Self {
        let friction = DECELERATION_FRICTION;
        let c2 = -initial_velocity / friction;
        let c1 = initial_position - c2;

        Self {
            friction,
            c1,
            c2,
            t0: frame_time,
            finished: false,
            final_position: 0.0,
        }
    }

    /// Shifts the physics origin by `delta` pixels, keeping velocity and shape identical.
    ///
    /// Called when a buffer recycle adjusts `scroll_offset` so that the physics continues returning
    /// positions consistent with the new reference frame.
    pub fn shift_origin(&mut self, delta: f64) {
        self.c1 += delta;
        self.final_position += delta
    }

    /// Advances the simulation to `frame_time` (microseconds). Returns `(position, velocity,
    /// still_running)`. When `still_running` is false the called should remove the tick callback.
    pub fn tick(&mut self, frame_time: i64) -> (f64, f64, bool) {
        if self.finished {
            return (self.final_position, 0.0, false);
        }

        let t = (frame_time - self.t0) as f64 / 1_000_000.0;
        let exp_part = (-self.friction * t).exp();
        let position = self.c1 + self.c2 * exp_part;
        let velocity = -self.friction * self.c2 * exp_part;

        if velocity.abs() < STOP_VELOCITY {
            self.finished = true;
            let rounded = position.round();
            self.final_position = rounded;
            return (rounded, 0.0, false);
        }

        (position, velocity, true)
    }
}
