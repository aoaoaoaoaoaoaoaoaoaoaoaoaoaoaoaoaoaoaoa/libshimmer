//! Patterns the stock shaders share, kept bit-for-bit with the design swatches.

use crate::paint::Rgb;
use crate::stage::TICK_SECS;

pub(crate) const PINK: Rgb = Rgb(255.0, 78.0, 160.0);
pub(crate) const TANGERINE: Rgb = Rgb(255.0, 140.0, 50.0);
pub(crate) const SUN: Rgb = Rgb(255.0, 214.0, 64.0);
pub(crate) const LIME: Rgb = Rgb(170.0, 240.0, 60.0);
pub(crate) const MINT: Rgb = Rgb(60.0, 240.0, 170.0);
pub(crate) const CYAN: Rgb = Rgb(50.0, 215.0, 255.0);
pub(crate) const AZURE: Rgb = Rgb(70.0, 130.0, 255.0);
pub(crate) const VIOLET: Rgb = Rgb(165.0, 105.0, 255.0);
pub(crate) const MAGENTA: Rgb = Rgb(230.0, 70.0, 230.0);
pub(crate) const CORAL: Rgb = Rgb(255.0, 95.0, 95.0);
pub(crate) const GOLD: Rgb = Rgb(255.0, 190.0, 40.0);
pub(crate) const ICE: Rgb = Rgb(180.0, 230.0, 255.0);
pub(crate) const STEEL: Rgb = Rgb(120.0, 140.0, 165.0);
/// A hot white, brighter than most terminals' default foreground.
pub(crate) const WHITE: Rgb = Rgb(245.0, 245.0, 245.0);

/// A deterministic draw in [0, 1) for `seed`.
pub(crate) fn hash(seed: i64) -> f64 {
    let bytes = seed.to_le_bytes();
    let low = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let mut x = (low ^ 0x9e37_79b9).wrapping_mul(0x85eb_ca6b);
    x ^= x >> 13;
    x = x.wrapping_mul(0xc2b2_ae35);
    f64::from(x ^ (x >> 16)) / 4_294_967_296.0
}

pub(crate) fn chance(seed: i64, odds: f64) -> bool {
    hash(seed) < odds
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "The hash is in [0, 1); flooring its scaled value deliberately selects a nonnegative sample, clamped to the slice's last index."
)]
pub(crate) fn pick<T: Copy>(seed: i64, list: &[T]) -> T {
    list[((hash(seed) * list.len() as f64) as usize).min(list.len() - 1)]
}

/// Every `every` seconds, one draw at `odds`; a won draw lasts `span` seconds.
pub(crate) struct Lottery {
    pub(crate) every: f64,
    pub(crate) odds: f64,
    pub(crate) span: f64,
    pub(crate) seed: i64,
}

impl Lottery {
    /// Progress through the won draw under way at `t`, if one is.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "A lottery draw is deliberately identified by its whole animation-clock bucket."
    )]
    pub(crate) fn at(&self, t: f64) -> Option<f64> {
        let draw = (t / self.every).floor();
        let into = t - draw * self.every;
        (into < self.span && chance(self.seed + 7919 * draw as i64, self.odds))
            .then_some(into / self.span)
    }
}

pub(crate) fn frac(x: f64) -> f64 {
    x - x.floor()
}

/// A cosine ease from 0 to 1 over [0, 1].
pub(crate) fn ease(x: f64) -> f64 {
    0.5 - 0.5 * (std::f64::consts::PI * x).cos()
}

/// A smoothstep from `from` to `to`.
pub(crate) fn smooth(from: f64, to: f64, x: f64) -> f64 {
    let t = ((x - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A raised-cosine bump of half-width `width` around zero.
pub(crate) fn bump(distance: f64, width: f64) -> f64 {
    if distance.abs() < width {
        0.5 + 0.5 * (std::f64::consts::PI * distance / width).cos()
    } else {
        0.0
    }
}

/// One tick of a first-order lag from `value` toward `target` with time constant `tau` seconds.
pub(crate) fn lag(value: f64, target: f64, tau: f64) -> f64 {
    value + (target - value) * (1.0 - (-TICK_SECS / tau).exp())
}

/// The color at `x` in [0, 1] along evenly spaced `stops`.
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "A clamped unit coordinate deliberately selects adjacent color stops; the floored nonnegative index is capped below the last stop."
)]
pub(crate) fn ramp(stops: &[Rgb], x: f64) -> Rgb {
    let y = x.clamp(0.0, 1.0) * (stops.len() - 1) as f64;
    let i = (y.floor() as usize).min(stops.len() - 2);
    stops[i + 1].blend(stops[i], y - i as f64)
}

/// A color from `hue` in turns, `saturation` and `lightness`.
pub(crate) fn hsl(hue: f64, saturation: f64, lightness: f64) -> Rgb {
    let a = saturation * lightness.min(1.0 - lightness);
    let channel = |n: f64| {
        let k = (n + 12.0 * frac(hue)) % 12.0;
        255.0 * (lightness - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0))
    };
    Rgb(channel(0.0), channel(8.0), channel(4.0))
}

/// Quadrant blocks by bits: upper left 8, upper right 4, lower left 2, lower right 1.
pub(crate) const QUADRANTS: [char; 16] = [
    ' ', '▗', '▖', '▄', '▝', '▐', '▞', '▟', '▘', '▚', '▌', '▙', '▀', '▜', '▛', '█',
];

/// Octant blocks by bits: pixel `2 * y + x` of a cell's 2×4 grid is bit `2 * y + x`. Pixels are
/// square on a 1:2 cell. Patterns Unicode encodes outside the octant block map to those characters.
pub(crate) const OCTANTS: [char; 256] = [
    ' ', '𜺨', '𜺫', '🮂', '𜴀', '▘', '𜴁', '𜴂', '𜴃', '𜴄', '▝', '𜴅', '𜴆', '𜴇', '𜴈', '▀', '𜴉', '𜴊', '𜴋',
    '𜴌', '🯦', '𜴍', '𜴎', '𜴏', '𜴐', '𜴑', '𜴒', '𜴓', '𜴔', '𜴕', '𜴖', '𜴗', '𜴘', '𜴙', '𜴚', '𜴛', '𜴜', '𜴝',
    '𜴞', '𜴟', '🯧', '𜴠', '𜴡', '𜴢', '𜴣', '𜴤', '𜴥', '𜴦', '𜴧', '𜴨', '𜴩', '𜴪', '𜴫', '𜴬', '𜴭', '𜴮', '𜴯',
    '𜴰', '𜴱', '𜴲', '𜴳', '𜴴', '𜴵', '🮅', '𜺣', '𜴶', '𜴷', '𜴸', '𜴹', '𜴺', '𜴻', '𜴼', '𜴽', '𜴾', '𜴿', '𜵀',
    '𜵁', '𜵂', '𜵃', '𜵄', '▖', '𜵅', '𜵆', '𜵇', '𜵈', '▌', '𜵉', '𜵊', '𜵋', '𜵌', '▞', '𜵍', '𜵎', '𜵏', '𜵐',
    '▛', '𜵑', '𜵒', '𜵓', '𜵔', '𜵕', '𜵖', '𜵗', '𜵘', '𜵙', '𜵚', '𜵛', '𜵜', '𜵝', '𜵞', '𜵟', '𜵠', '𜵡', '𜵢',
    '𜵣', '𜵤', '𜵥', '𜵦', '𜵧', '𜵨', '𜵩', '𜵪', '𜵫', '𜵬', '𜵭', '𜵮', '𜵯', '𜵰', '𜺠', '𜵱', '𜵲', '𜵳', '𜵴',
    '𜵵', '𜵶', '𜵷', '𜵸', '𜵹', '𜵺', '𜵻', '𜵼', '𜵽', '𜵾', '𜵿', '𜶀', '𜶁', '𜶂', '𜶃', '𜶄', '𜶅', '𜶆', '𜶇',
    '𜶈', '𜶉', '𜶊', '𜶋', '𜶌', '𜶍', '𜶎', '𜶏', '▗', '𜶐', '𜶑', '𜶒', '𜶓', '▚', '𜶔', '𜶕', '𜶖', '𜶗', '▐',
    '𜶘', '𜶙', '𜶚', '𜶛', '▜', '𜶜', '𜶝', '𜶞', '𜶟', '𜶠', '𜶡', '𜶢', '𜶣', '𜶤', '𜶥', '𜶦', '𜶧', '𜶨', '𜶩',
    '𜶪', '𜶫', '▂', '𜶬', '𜶭', '𜶮', '𜶯', '𜶰', '𜶱', '𜶲', '𜶳', '𜶴', '𜶵', '𜶶', '𜶷', '𜶸', '𜶹', '𜶺', '𜶻',
    '𜶼', '𜶽', '𜶾', '𜶿', '𜷀', '𜷁', '𜷂', '𜷃', '𜷄', '𜷅', '𜷆', '𜷇', '𜷈', '𜷉', '𜷊', '𜷋', '𜷌', '𜷍', '𜷎',
    '𜷏', '𜷐', '𜷑', '𜷒', '𜷓', '𜷔', '𜷕', '𜷖', '𜷗', '𜷘', '𜷙', '𜷚', '▄', '𜷛', '𜷜', '𜷝', '𜷞', '▙', '𜷟',
    '𜷠', '𜷡', '𜷢', '▟', '𜷣', '▆', '𜷤', '𜷥', '█',
];

/// A `SplitMix64` stream: a shader's seeded randomness.
#[derive(Clone, Debug)]
pub(crate) struct Rng(u64);

impl Rng {
    pub(crate) const fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// The next 64 random bits.
    pub(crate) fn bits(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A draw from `0..n`, or 0 when `n` is 0.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "The product's high half is below n, which fits a usize."
    )]
    pub(crate) fn below(&mut self, n: usize) -> usize {
        ((u128::from(self.bits()) * n as u128) >> 64) as usize
    }

    /// A draw from [0, 1).
    #[expect(
        clippy::cast_precision_loss,
        reason = "53 random bits convert to f64 exactly."
    )]
    pub(crate) fn unit(&mut self) -> f64 {
        (self.bits() >> 11) as f64 / (1_u64 << 53) as f64
    }

    /// Whether a draw at `odds` wins.
    pub(crate) fn chance(&mut self, odds: f64) -> bool {
        self.unit() < odds
    }

    /// `amount` distinct draws from `0..n`, in draw order; all of `0..n` when `amount` exceeds `n`.
    pub(crate) fn sample(&mut self, n: usize, amount: usize) -> Vec<usize> {
        let mut pool: Vec<usize> = (0..n).collect();
        let amount = amount.min(n);
        for i in 0..amount {
            let j = i + self.below(n - i);
            pool.swap(i, j);
        }
        pool.truncate(amount);
        pool
    }
}
