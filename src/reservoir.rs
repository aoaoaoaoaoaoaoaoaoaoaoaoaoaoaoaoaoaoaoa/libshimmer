//! Reservoirs: lumps of work drained into a steady flow.

use std::time::Duration;

/// Seconds over which a reservoir drains a lump: the time constant of its exponential outflow.
pub(crate) const DRAIN: f64 = 4.0;

/// Turns lumps of work, such as a burst of tokens, into a flow that drains each lump over four
/// seconds. A host converts the flow into an [`Intensity`](crate::Intensity) by its own measure.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reservoir {
    /// Work yet to drain, finite and at least 0.
    level: f64,
}

impl Reservoir {
    /// Adds a lump of work; a negative lump takes work back, never below empty, and a NaN lump
    /// empties the reservoir.
    pub fn pour(&mut self, lump: f64) {
        let level = self.level + lump;
        self.level = if level > 0.0 {
            level.min(f64::MAX)
        } else {
            0.0
        };
    }

    /// Releases the work that drains during `dt`.
    pub fn drain(&mut self, dt: Duration) -> f64 {
        let released = self.level * -(-dt.as_secs_f64() / DRAIN).exp_m1();
        self.level -= released;
        released
    }
}
