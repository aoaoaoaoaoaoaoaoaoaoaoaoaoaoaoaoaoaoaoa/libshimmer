//! Draws libshimmer's status lines with ratatui: a [`Row`] as spans to lay beside the host's own,
//! its backdrop shaded into a buffer, and an [`Ink`], such as a bullet's, as a span.
//!
//! The crate builds on ratatui-core 0.1, the core of ratatui 0.30, so that libshimmer itself
//! depends on no ratatui: a ratatui-core release that changes these types breaks this crate alone.
//!
//! ```
//! use std::time::Duration;
//!
//! use libshimmer::{Glyphs, Intensity, Motion, Palette, Rgb, Shimmer, Spell, stock};
//! use libshimmer_ratatui::RowExt as _;
//!
//! let recipe = stock::named("Lava Lamp").expect("a stock recipe");
//! let mut shimmer = Shimmer::new(recipe, 7, Glyphs::BASE)?;
//! shimmer.set_spell(Spell::Busy);
//! shimmer.show("Compiling");
//! shimmer.set_intensity(Intensity::new(0.6));
//! shimmer.advance(Duration::from_millis(100));
//! let palette = Palette { fg: Rgb::new(208, 208, 208), bg: Rgb::new(21, 21, 21) };
//! let row = shimmer.status(palette, Motion { mark: true, letters: true });
//! let lead: String = row.spans().iter().map(|span| span.content.as_ref()).collect();
//! assert_eq!(lead, "● Compiling");
//! # Ok::<(), libshimmer::Unfit>(())
//! ```

use libshimmer::{Ink, Rgb, Row};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::text::Span;

/// A [`Row`] drawn with ratatui.
pub trait RowExt {
    /// The cells as spans, each run of cells sharing an ink in one span.
    fn spans(&self) -> Vec<Span<'static>>;

    /// Lays the backdrop under the row rendered at the top of `area`, clipped to it.
    fn shade(&self, area: Rect, buf: &mut Buffer);
}

/// An [`Ink`] drawn with ratatui.
pub trait InkExt {
    /// This ink over `text`, unless it carries its own glyph.
    fn span(self, text: &str) -> Span<'static>;
}

impl RowExt for Row {
    fn spans(&self) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let mut run: Option<(String, Ink)> = None;
        for (symbol, ink) in self.cells() {
            match &mut run {
                Some((text, held)) if *held == ink => text.push_str(symbol),
                _ => {
                    if let Some((text, held)) = run.replace((symbol.to_owned(), ink)) {
                        spans.push(Span::styled(text, style(held)));
                    }
                }
            }
        }
        spans.extend(run.map(|(text, ink)| Span::styled(text, style(ink))));
        spans
    }

    fn shade(&self, area: Rect, buf: &mut Buffer) {
        for (x, &bg) in (area.x..area.right()).zip(self.backdrop()) {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_bg(color(bg));
            }
        }
    }
}

impl InkExt for Ink {
    fn span(self, text: &str) -> Span<'static> {
        Span::styled(
            self.glyph.map_or_else(|| text.to_owned(), String::from),
            style(self),
        )
    }
}

/// The ratatui style of `ink`.
fn style(ink: Ink) -> Style {
    let mut style = Style::default();
    if let Some(fg) = ink.fg {
        style = style.fg(color(fg));
    }
    if let Some(bg) = ink.bg {
        style = style.bg(color(bg));
    }
    if ink.bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    if let Some(underline) = ink.underline {
        style = style
            .add_modifier(Modifier::UNDERLINED)
            .underline_color(color(underline));
    }
    style
}

/// The ratatui color of `rgb`.
fn color(rgb: Rgb) -> Color {
    let [r, g, b] = rgb.bytes();
    Color::Rgb(r, g, b)
}
