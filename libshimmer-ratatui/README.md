# libshimmer-ratatui

Draws [libshimmer](https://crates.io/crates/libshimmer)'s animated status lines with ratatui: a row as spans, its backdrop shaded into a buffer, and an ink as a span.

```rust
use libshimmer_ratatui::{InkExt as _, RowExt as _};

let lead = row.spans(); // lay beside your own spans
row.shade(area, buf); // the backdrop under the line
let bullet = shimmer.bullet(palette).span("•");
```

Built on ratatui-core 0.1, the core of ratatui 0.30. MIT-0 licensed.
