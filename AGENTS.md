libshimmer: animated status lines for terminal UIs; `SPEC.md` owns the design

## gate
- `./check.py check` after edits; `./check.py verify` runs same gate without writes
- Release only with current README reel: `media/roster.py check`; stale → `media/roster.py`, commit `media/`

## invariants
- No outbound calls: no clock, env, thread, global state or callback; paint is a pure function of recipe, seed, commands, palette, motion
- `Shader`, `Stage`, `kit` stay crate-private; ticks never see colors
