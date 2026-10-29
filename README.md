# libshimmer

Animated status lines. Let it shimmer!

![The stock recipes, each painting its own name through a bout and the rest after it](media/roster.webp)

Workload-dependent animations, with workload defined by the caller, can spruce up any status line, loading bar or similar.

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

[SPEC.md](SPEC.md) specifies the design. MIT-0 licensed.
