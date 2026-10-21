libshimmer: animated status lines for terminal UIs; `SPEC.md` owns the design

## gate
- `./check.py check` after edits; `./check.py verify` runs same gate without writes

## invariants
- No outbound calls: no clock, env, thread, global state or callback; paint is a pure function of recipe, seed, commands, palette, motion
- `Shader`, `Stage`, `kit` stay crate-private; ticks never see colors
