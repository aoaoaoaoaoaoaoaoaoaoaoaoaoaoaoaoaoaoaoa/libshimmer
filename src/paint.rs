//! Paint: colors, inks, and the row a shimmer paints.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::stage::{Canvas, Caption};

/// Columns of backdrop a row keeps.
const BACKDROP: usize = 80;

/// A 24-bit color whose channels stay fractional until they reach the terminal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb(pub f64, pub f64, pub f64);

impl Rgb {
    /// This color laid over `under` at `alpha`.
    pub(crate) fn blend(self, under: Rgb, alpha: f64) -> Rgb {
        let mix = |a: f64, b: f64| a * alpha + b * (1.0 - alpha);
        Rgb(
            mix(self.0, under.0),
            mix(self.1, under.1),
            mix(self.2, under.2),
        )
    }

    /// The channels as a terminal takes them: clamped to `0..=255` and truncated.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Terminal quantization truncates channels clamped to 0..=255."
    )]
    pub fn bytes(self) -> [u8; 3] {
        let channel = |v: f64| v.clamp(0.0, 255.0) as u8;
        [channel(self.0), channel(self.1), channel(self.2)]
    }
}

impl From<(u8, u8, u8)> for Rgb {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Rgb(f64::from(r), f64::from(g), f64::from(b))
    }
}

/// The terminal's default colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    /// The default foreground.
    pub fg: Rgb,
    /// The default background.
    pub bg: Rgb,
}

impl Palette {
    /// The foreground laid over the background at `alpha`, as terminals grey text.
    pub(crate) fn grey(self, alpha: f64) -> Rgb {
        self.fg.blend(self.bg, alpha)
    }
}

/// Whether the mark and the letters may move.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Motion {
    /// Whether the mark may move; a still mark is absent.
    pub mark: bool,
    /// Whether the letters may move; still letters are the caption verbatim.
    pub letters: bool,
}

/// The paint of one cell. Unset colors are the terminal's defaults, and an unset glyph keeps the
/// text beneath.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ink {
    /// The glyph standing in for the text beneath.
    pub glyph: Option<char>,
    /// The foreground.
    pub fg: Option<Rgb>,
    /// The background.
    pub bg: Option<Rgb>,
    /// Whether the cell is bold.
    pub bold: bool,
    /// An underline in this color.
    pub underline: Option<Rgb>,
}

impl Ink {
    pub(crate) const fn of(glyph: char) -> Ink {
        Ink {
            glyph: Some(glyph),
            fg: None,
            bg: None,
            bold: false,
            underline: None,
        }
    }

    pub(crate) const fn fg(self, fg: Rgb) -> Ink {
        Ink {
            fg: Some(fg),
            ..self
        }
    }

    pub(crate) const fn bg(self, bg: Rgb) -> Ink {
        Ink {
            bg: Some(bg),
            ..self
        }
    }

    pub(crate) const fn bold(self, bold: bool) -> Ink {
        Ink { bold, ..self }
    }

    pub(crate) const fn underline(self, color: Rgb) -> Ink {
        Ink {
            underline: Some(color),
            ..self
        }
    }
}

/// A status line's lead as painted, and the backdrop beneath it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Row {
    /// Every cell's symbol, left to right.
    text: String,
    /// Each cell's ink, beside the byte offset at which its symbol ends.
    cells: Vec<(usize, Ink)>,
    backdrop: Vec<Rgb>,
}

impl Row {
    /// Paints `canvas` over `caption` as `motion` allows.
    pub(crate) fn paint(caption: &Caption, canvas: Canvas, motion: Motion) -> Self {
        let mut row = Self::default();
        if motion.mark && !canvas.mark.is_empty() {
            for ink in canvas.mark {
                row.put(ink, "•");
            }
            row.put(Ink::default(), " ");
        }
        if !motion.letters {
            for letter in caption.iter() {
                row.put(Ink::default(), letter);
            }
            return row;
        }
        let mut letters = canvas.letters.into_iter();
        for letter in caption.iter() {
            row.put(letters.next().unwrap_or_default(), letter);
        }
        for ink in canvas.tail {
            row.put(ink, " ");
        }
        row.backdrop = canvas.backdrop;
        row.backdrop.truncate(BACKDROP);
        row
    }

    /// Paints one cell: `ink` over `beneath`, its glyph standing in only when as wide.
    fn put(&mut self, ink: Ink, beneath: &str) {
        match ink
            .glyph
            .filter(|glyph| glyph.width() == Some(beneath.width()))
        {
            Some(glyph) => self.text.push(glyph),
            None => self.text.push_str(beneath),
        }
        self.cells
            .push((self.text.len(), Ink { glyph: None, ..ink }));
    }

    /// Each cell's symbol and ink, left to right. A symbol spans as many columns as the text it
    /// stands for: its letter's width among the letters, and one elsewhere.
    pub fn cells(&self) -> impl Iterator<Item = (&str, Ink)> {
        self.cells.iter().scan(0, |start, &(end, ink)| {
            let symbol = &self.text[*start..end];
            *start = end;
            Some((symbol, ink))
        })
    }

    /// Background colors laid under the line from its first column, whatever text lands there; at
    /// most 80.
    pub fn backdrop(&self) -> &[Rgb] {
        &self.backdrop
    }
}
