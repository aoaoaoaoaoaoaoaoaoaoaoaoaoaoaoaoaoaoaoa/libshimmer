//! Paint as ratatui types.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::text::Span;

use crate::paint::{Ink, Rgb, Row};

impl Row {
    /// The cells as spans, each run of cells sharing an ink in one span.
    pub fn spans(&self) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let mut run: Option<(String, Ink)> = None;
        for (symbol, ink) in self.cells() {
            match &mut run {
                Some((text, held)) if *held == ink => text.push_str(symbol),
                _ => {
                    if let Some((text, held)) = run.replace((symbol.to_owned(), ink)) {
                        spans.push(Span::styled(text, held.style()));
                    }
                }
            }
        }
        spans.extend(run.map(|(text, ink)| Span::styled(text, ink.style())));
        spans
    }

    /// Lays the backdrop under the row rendered at the top of `area`, clipped to it.
    pub fn shade(&self, area: Rect, buf: &mut Buffer) {
        for (x, &bg) in (area.x..area.right()).zip(self.backdrop()) {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_bg(bg.into());
            }
        }
    }
}

impl Ink {
    /// This ink over `text`, unless it carries its own glyph.
    pub fn span(self, text: &str) -> Span<'static> {
        Span::styled(
            self.glyph.map_or_else(|| text.to_owned(), String::from),
            self.style(),
        )
    }

    fn style(self) -> Style {
        let mut style = Style::default();
        if let Some(fg) = self.fg {
            style = style.fg(fg.into());
        }
        if let Some(bg) = self.bg {
            style = style.bg(bg.into());
        }
        if self.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        if let Some(underline) = self.underline {
            style = style
                .add_modifier(Modifier::UNDERLINED)
                .underline_color(underline.into());
        }
        style
    }
}

impl From<Rgb> for Color {
    fn from(rgb: Rgb) -> Self {
        let [r, g, b] = rgb.bytes();
        Color::Rgb(r, g, b)
    }
}
