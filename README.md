# libshimmer

Animated status lines for terminal UIs, paced by the host's work.

A host makes a `Shimmer` from one of 17 stock recipes, tells it what the host is doing and how hard it works, and each frame draws the status line it paints: a mark, the caption's letters, and a tail, over a backdrop of background colors. Paint is plain data; the `ratatui` feature converts it into ratatui spans and shades a ratatui buffer. libshimmer reads no clock and keeps no global state: the host passes the time between frames, and paint depends only on the recipe, the seed, the commands so far, and the colors and motion the host paints with.

```rust
use std::time::Duration;

use libshimmer::{Glyphs, Intensity, Motion, Palette, Rgb, Shimmer, Spell, stock};

let recipe = stock::named("Lava Lamp").expect("a stock recipe");
let mut shimmer: Shimmer = Shimmer::new(recipe, 7, Glyphs::BASE)?;
shimmer.set_spell(Spell::Busy);
shimmer.show("Compiling");

// Each frame:
shimmer.set_intensity(Intensity::new(tokens_per_second / 100.0));
shimmer.advance(since_last_frame);
let palette = Palette { fg: Rgb(208.0, 208.0, 208.0), bg: Rgb(21.0, 21.0, 21.0) };
let row = shimmer.status(palette, Motion { mark: true, letters: true });
let spans = row.spans(); // with the `ratatui` feature; `row.cells()` otherwise
row.shade(area, buf);

// When the work ends:
shimmer.set_spell(Spell::Idle);
```

[SPEC.md](SPEC.md) specifies the design. MIT licensed.
