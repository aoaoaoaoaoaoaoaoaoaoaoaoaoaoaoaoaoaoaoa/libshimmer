//! Gauges: the widest grapheme a shimmer's captions may hold.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub(crate) mod sealed {
    /// Whether a gauge admits graphemes two cells wide. Its module is private, so that only this
    /// crate's gauges exist.
    pub trait Gauge {
        const WIDE: bool;
    }
}

/// The widest grapheme a shimmer's captions may hold, fixed by the host in the shimmer's type.
/// Sealed: [`Wide`] and [`Narrow`] are the only gauges.
pub trait Gauge: sealed::Gauge {
    /// The caption [`Shimmer::show`](crate::Shimmer::show) takes.
    type Caption<'a>: AsRef<str> + Copy;
}

/// Graphemes one or two cells wide: captions arrive as `&str`.
#[derive(Debug)]
pub enum Wide {}

/// Graphemes one cell wide: captions arrive checked, as [`NarrowStr`].
#[derive(Debug)]
pub enum Narrow {}

impl sealed::Gauge for Wide {
    const WIDE: bool = true;
}

impl sealed::Gauge for Narrow {
    const WIDE: bool = false;
}

impl Gauge for Wide {
    type Caption<'a> = &'a str;
}

impl Gauge for Narrow {
    type Caption<'a> = NarrowStr<'a>;
}

/// Text holding no grapheme wider than one cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NarrowStr<'a>(&'a str);

impl<'a> NarrowStr<'a> {
    /// `text`, if none of its graphemes is wider than one cell.
    pub fn new(text: &'a str) -> Option<Self> {
        text.graphemes(/*is_extended*/ true)
            .all(|letter| letter.width() <= 1)
            .then_some(Self(text))
    }
}

impl AsRef<str> for NarrowStr<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}
