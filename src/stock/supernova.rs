//! Supernova: awaiting its port from codex-tui.

use super::OPUS;
use crate::paint::{Ink, Palette};
use crate::recipe::{Glyphs, Recipe};
use crate::stage::{Canvas, Shader, Stage};

pub(super) const RECIPE: Recipe = Recipe {
    name: "Supernova",
    blurb: "a star grows with output, then blows",
    credit: OPUS,
    needs: Glyphs::BASE,
    wide: true,
    summon: |_| Box::new(Stub),
};

struct Stub;

impl Shader for Stub {
    fn status(&self, _stage: &Stage, _palette: Palette, _canvas: &mut Canvas) {}

    fn bullet(&self, _stage: &Stage, _palette: Palette) -> Ink {
        Ink::of('•')
    }
}
