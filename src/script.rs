//! The preview script: a canned bout and the rest after it, playing in a loop, with which a picker
//! previews a recipe.
//!
//! The bout is the design swatches' cycle. The host works under three captions, then waits on a
//! background terminal, whose output arrives as the wait ends and drains as a [`Reservoir`]
//! drains it. Then it rests, so that the preview shows the recipe between bouts too.
//!
//! [`Reservoir`]: crate::Reservoir

use std::time::Duration;

use crate::reservoir::DRAIN;
use crate::shimmer::QUANTA;
use crate::{Intensity, Spell};

/// What the host does during a segment, in units that count as quanta up to brisk work.
#[derive(Clone, Copy)]
enum Work {
    /// It produces `rate` units a second.
    Busy { rate: f64 },
    /// It waits on a terminal, whose output of `lump` units arrives as the wait ends.
    Wait { lump: f64 },
    /// It rests between bouts.
    Idle,
}

struct Segment {
    caption: &'static str,
    /// Seconds.
    length: f64,
    work: Work,
}

static CYCLE: [Segment; 5] = [
    Segment {
        caption: "Working",
        length: 2.5,
        work: Work::Busy { rate: 60.0 },
    },
    Segment {
        caption: "Tracing shimmer cadence",
        length: 3.5,
        work: Work::Busy { rate: 80.0 },
    },
    Segment {
        caption: "Drafting the fix",
        length: 4.0,
        work: Work::Busy { rate: 50.0 },
    },
    Segment {
        caption: "Waiting for background terminal",
        length: 7.0,
        work: Work::Wait { lump: 600.0 },
    },
    Segment {
        caption: "Finished",
        length: 3.0,
        work: Work::Idle,
    },
];

/// Seconds in one cycle.
fn length() -> f64 {
    CYCLE.iter().map(|segment| segment.length).sum()
}

/// The segment under way `t` seconds into a cycle.
fn segment(t: f64) -> &'static Segment {
    let mut end = 0.0;
    for segment in &CYCLE[..CYCLE.len() - 1] {
        end += segment.length;
        if t < end {
            return segment;
        }
    }
    &CYCLE[CYCLE.len() - 1]
}

/// Units a second arriving `t` seconds into a cycle: the host's own output while it works, and
/// every lump draining.
fn inflow(t: f64) -> f64 {
    let length = length();
    // A lump drains on through later cycles, so each cycle meets every earlier one's remainder.
    let echoes = 1.0 / -(-length / DRAIN).exp_m1();
    let mut end = 0.0;
    let mut drained = 0.0;
    for segment in &CYCLE {
        end += segment.length;
        if let Work::Wait { lump } = segment.work {
            drained += lump / DRAIN * (-(t - end).rem_euclid(length) / DRAIN).exp() * echoes;
        }
    }
    match segment(t).work {
        Work::Busy { rate } => rate + drained,
        Work::Wait { .. } | Work::Idle => drained,
    }
}

/// What the script commands at one moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cue {
    /// The spell to cast.
    pub spell: Spell,
    /// The caption to show.
    pub caption: &'static str,
    /// The intensity to set.
    pub intensity: Intensity,
}

/// The cue `at` into the looping script.
pub fn cue(at: Duration) -> Cue {
    let t = at.as_secs_f64() % length();
    let segment = segment(t);
    Cue {
        spell: match segment.work {
            Work::Busy { .. } => Spell::Busy,
            Work::Wait { .. } => Spell::Wait,
            Work::Idle => Spell::Idle,
        },
        caption: segment.caption,
        intensity: Intensity::new(inflow(t) / QUANTA),
    }
}
