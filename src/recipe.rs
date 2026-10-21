//! Recipes, and what each needs of a host.

use std::error::Error;
use std::fmt;

use crate::gauge::Gauge;
use crate::stage::Shader;

/// Glyph repertoires beyond the base set every target terminal draws: those a terminal draws, or
/// those a recipe needs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Glyphs(u8);

impl Glyphs {
    /// The base set alone.
    pub const BASE: Self = Self(0);
    /// Unicode 16's octants, with the other symbols for legacy computing: U+1FB00 to U+1FBFF and
    /// U+1CC00 to U+1CEBF.
    pub const OCTANTS: Self = Self(1);

    /// The repertoires of both sets.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether every repertoire of `other` is in this set.
    pub const fn contains(self, other: Self) -> bool {
        other.0 & !self.0 == 0
    }

    const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}

/// Each repertoire's name.
const REPERTOIRES: [(Glyphs, &str); 1] = [(Glyphs::OCTANTS, "octants")];

/// The repertoire beyond the base set that `glyph` belongs to.
#[cfg(test)]
pub(crate) fn repertoire(glyph: char) -> Glyphs {
    if ('\u{1cc00}'..='\u{1cebf}').contains(&glyph) || ('\u{1fb00}'..='\u{1fbff}').contains(&glyph)
    {
        Glyphs::OCTANTS
    } else {
        Glyphs::BASE
    }
}

/// Why a recipe cannot load for a host: glyphs the terminal lacks, or captions wider than the
/// recipe paints well.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unfit {
    missing: Glyphs,
    narrow: bool,
}

impl Unfit {
    /// The repertoires the recipe needs and the terminal lacks.
    pub fn missing(self) -> Glyphs {
        self.missing
    }

    /// Whether the recipe paints only narrow graphemes, for a shimmer whose gauge is wide.
    pub fn narrow(self) -> bool {
        self.narrow
    }
}

impl fmt::Display for Unfit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut clauses = REPERTOIRES
            .iter()
            .filter(|(repertoire, _)| self.missing.contains(*repertoire))
            .map(|(_, name)| format!("the terminal lacks {name}"))
            .chain(
                self.narrow
                    .then(|| "the recipe paints only narrow graphemes".to_owned()),
            );
        if let Some(first) = clauses.next() {
            f.write_str(&first)?;
        }
        clauses.try_for_each(|clause| write!(f, "; {clause}"))
    }
}

impl Error for Unfit {}

/// A stock animation: its picker entry, what it needs of a host, and how to start its shader.
pub struct Recipe {
    pub(crate) name: &'static str,
    pub(crate) blurb: &'static str,
    pub(crate) credit: &'static str,
    pub(crate) needs: Glyphs,
    pub(crate) wide: bool,
    pub(crate) summon: fn(u64) -> Box<dyn Shader>,
}

impl Recipe {
    /// The name a picker shows, unique among recipes.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// One line on what the recipe paints.
    pub fn blurb(&self) -> &'static str {
        self.blurb
    }

    /// The recipe's author.
    pub fn credit(&self) -> &'static str {
        self.credit
    }

    /// The repertoires the recipe paints beyond the base set.
    pub fn needs(&self) -> Glyphs {
        self.needs
    }

    /// Whether the recipe paints graphemes two cells wide well.
    pub fn wide(&self) -> bool {
        self.wide
    }

    /// Whether a shimmer of gauge `G` can run the recipe on a terminal that draws `glyphs`.
    ///
    /// # Errors
    ///
    /// [`Unfit`] when the terminal lacks glyphs the recipe needs, or when `G` is wide and the
    /// recipe paints only narrow graphemes.
    pub fn fits<G: Gauge>(&self, glyphs: Glyphs) -> Result<(), Unfit> {
        let unfit = Unfit {
            missing: self.needs.without(glyphs),
            narrow: G::WIDE && !self.wide,
        };
        if unfit.missing == Glyphs::BASE && !unfit.narrow {
            Ok(())
        } else {
            Err(unfit)
        }
    }
}

impl fmt::Debug for Recipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Recipe")
            .field("name", &self.name)
            .field("needs", &self.needs)
            .field("wide", &self.wide)
            .finish_non_exhaustive()
    }
}
