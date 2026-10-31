//! Rest: how every recipe looks between bouts. Whatever a shader paints then settles near grey, its
//! inks near the grey a terminal greys text in and its backgrounds near the palette's background,
//! each keeping a breathing trace of its own color and no trace more colorful than another.

use std::f64::consts::TAU;

use crate::kit::ease;
use crate::paint::{Palette, Rgb};
use crate::shimmer::Spell;
use crate::stage::{Canvas, Stage};

/// Seconds of the host's per second of shimmer time at rest, so that a resting recipe moves a
/// quarter as fast. A whole number, so that the clock keeps rest's time exactly.
pub(crate) const DRAG: u32 = 4;

/// Shimmer time per second of the host's at rest.
pub(crate) const TEMPO: f64 = 1.0 / DRAG as f64;

/// The shortest lull, in seconds of shimmer time, between a resting recipe's moves: five of the
/// host's. A move may be a brief burst, such as a moth's two wingbeats.
pub(crate) const LULL: f64 = 5.0 * TEMPO;

/// The resting grey: the foreground laid over the background at this alpha, as terminals grey
/// text.
const GREY: f64 = 0.6;

/// The share of an ink's departure from the resting grey that rest keeps, between breaths.
const KEEP: f64 = 0.3;

/// The share of a background's departure from the palette's background that rest keeps, between
/// breaths.
const SHADE: f64 = 0.35;

/// The most saturated a kept trace may be between breaths: the spread of its channels over its
/// strongest.
const TINT: f64 = 0.16;

/// The most an ink's kept trace strays from the resting grey's luma between breaths, out of 1.
const SWAY: f64 = 0.04;

/// How far the kept shares swell and ebb with each breath.
const BREATH: f64 = 0.35;

/// Seconds of the host's time per breath.
const BREATH_SECS: f64 = 8.0;

/// Seconds of the host's time over which a bout's colors settle into rest.
const SETTLE_SECS: f64 = 2.0;

/// Settles `canvas` into rest while the spell is idle; bold letters let go halfway.
pub(crate) fn settle(canvas: &mut Canvas, stage: &Stage, palette: Palette) {
    if stage.spell != Spell::Idle {
        return;
    }
    let rested = stage.spell_t / TEMPO;
    let depth = ease((rested / SETTLE_SECS).min(1.0));
    let breath = 1.0 + BREATH * (TAU * rested / BREATH_SECS).sin();
    let grey = palette.grey(GREY);
    let ink = |color: Rgb| {
        trace(color, grey, KEEP * breath, TINT * breath, SWAY * breath).blend(color, depth)
    };
    let shade = |color: Rgb| {
        trace(
            color,
            palette.bg,
            SHADE * breath,
            TINT * breath,
            f64::INFINITY,
        )
        .blend(color, depth)
    };
    for paint in canvas
        .mark
        .iter_mut()
        .chain(&mut canvas.letters)
        .chain(&mut canvas.tail)
    {
        paint.fg = Some(ink(paint.fg.unwrap_or(palette.fg)));
        paint.bg = paint.bg.map(shade);
        paint.underline = paint.underline.map(ink);
        paint.bold &= depth < 0.5;
    }
    for color in &mut canvas.backdrop {
        *color = shade(*color);
    }
}

/// What rest keeps of `color`'s departure from `home`: `share` of it, straying no further than
/// `sway` from `home`'s luma, and no more saturated than `tint`.
fn trace(color: Rgb, home: Rgb, share: f64, tint: f64, sway: f64) -> Rgb {
    let kept = color.blend(home, share);
    let stray = kept.luma() - home.luma();
    let lift = 255.0 * (stray.clamp(-sway, sway) - stray);
    let Rgb(r, g, b) = kept;
    let lifted = Rgb(r + lift, g + lift, b + lift);
    let top = r.max(g).max(b) + lift;
    let spread = r.max(g).max(b) - r.min(g).min(b);
    if spread <= tint * top {
        return lifted;
    }
    // Drawn toward its own grey by alpha, the trace's spread scales by alpha and its top nears the
    // grey: alpha * spread = tint * (grey + alpha * (top - grey)).
    let grey = 255.0 * lifted.luma();
    lifted.blend(
        Rgb(grey, grey, grey),
        tint * grey / (spread - tint * (top - grey)),
    )
}
