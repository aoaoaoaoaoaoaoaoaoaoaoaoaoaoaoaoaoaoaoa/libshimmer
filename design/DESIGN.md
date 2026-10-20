# libshimmer design

Draft 1, 2026-10-20. Subject: moving codex-tui's shimmers into a crate that any terminal UI can use.

Each view answers one question in a fixed notation, with its legend beneath it. Numbered items are review handles: "D4: (b)", "P7 is too strong", "Q1: (a)".

| § | View | Question | Notation |
|---|---|---|---|
| 0 | Remit | What is libshimmer for? | purpose, operational principle, exclusions |
| 1 | Boundary | What does libshimmer own, and what crosses its edge? | context diagram, ownership table |
| 2 | Cut | Where does today's code divide? | module graph |
| 3 | Nouns | Which types exist, and how do they relate? | UML class diagram, glossary |
| 4 | Lifecycle | Which states does a shimmer pass through? | UML state machine |
| 5 | Frame | What does a host do each frame? | UML sequence diagram |
| 6 | Promises | What always holds? | invariants with their checks |
| 7 | Decisions | What was chosen over what? | options, choice, reason |
| 8 | Questions | What remains open? | options, current lean |

## 0 Remit

**Purpose.** Dress a terminal status line with an animation that follows the host's work.

**Operational principle.** If the host captions its work and reports how hard it is working, then each frame the shimmer paints the caption, a mark, and a backdrop that quicken as work intensifies, change while the host waits, and settle when the bout ends.

**Excluded.** Terminal I/O and frame scheduling; status-line layout; measuring work; detecting terminal colors or color depth; remembering the user's choice; picker UI.

**Prior art.** `tui-shimmer`, written for the VT Code coding agent, styles text spans by animation phase; `tachyonfx` composes time-driven effects for ratatui. Neither takes a measure of the host's work.

## 1 Boundary

```mermaid
flowchart LR
  classDef outside fill:#1a1f26,stroke:#6e7681,color:#c9d1d9
  classDef host fill:#0f2740,stroke:#58a6ff,color:#e6edf3
  classDef lib fill:#2a1d45,stroke:#bc8cff,color:#f5f0ff,stroke-width:2px
  classDef dep fill:#12161c,stroke:#6e7681,color:#8b949e,stroke-dasharray:5 4

  work["<b>Work source</b><br/>codex-tui: the app server"]:::outside
  host["<b>Host TUI</b><br/>codex-tui, memview, …<br/><br/>owns the terminal,<br/>the frame loop, layout,<br/>measuring work, colors,<br/>motion preference,<br/>the user's choice"]:::host
  lib["<b>libshimmer</b><br/><br/>owns the stock catalog,<br/>each world and its clock,<br/>pacing, paint"]:::lib
  term["<b>Terminal</b>"]:::outside
  core["ratatui-core"]:::dep
  seg["unicode-segmentation"]:::dep

  work -->|"turn, tool and<br/>usage events"| host
  host -->|"recipe, seed<br/>caption, spell, intensity<br/>Δt; palette, motion"| lib
  lib -->|"paint: Row, Ink<br/>catalog: ROSTER"| host
  host -->|"frames"| term
  lib -.->|"feature ratatui"| core
  lib -.-> seg
```

Blue: the host. Violet: libshimmer. Grey: outside both. Dashed grey box and arrow: a Cargo dependency. A solid arrow carries the data on its label in its direction. The host initiates every exchange; libshimmer never calls out.

| Concern | Owner | Crossing |
|---|---|---|
| terminal, frame loop, redraw requests | host | `TICK` advises the frame interval |
| wall time | host | `advance(Δt)` |
| shimmer time: fixed ticks, skipped gaps | libshimmer | none |
| measuring work | host; in codex-tui, `Feed` | `set_intensity`, `set_spell` |
| converting intensity into quanta | libshimmer | none |
| caption | host | `show` |
| terminal colors and the truecolor check | host | `Palette`, at paint |
| reduced motion | host | `Motion`, at paint |
| status-line layout and clipping | host | `Row` |
| shimmer choice, persistence, picker | host | `ROSTER`, `Recipe`, `Script` |
| randomness | libshimmer, seeded by the host | `seed` |
| stock catalog | libshimmer | `ROSTER` |

## 2 Cut

```mermaid
flowchart TB
  classDef caller fill:#0f2740,stroke:#58a6ff,color:#e6edf3
  classDef changed fill:#0f2740,stroke:#58a6ff,color:#e6edf3,stroke-width:3px
  classDef moved fill:#2a1d45,stroke:#bc8cff,color:#f5f0ff

  subgraph tui["stays in codex-tui"]
    protocol["chatwidget/protocol.rs"]:::caller
    indicator["status_indicator_widget.rs"]:::caller
    rows["motion.rs<br/>running-tool rows"]:::caller
    boot["lib.rs<br/>startup_draft.rs"]:::caller
    slash["chatwidget/<br/>slash_dispatch.rs"]:::caller
    live["shimmers/live.rs<br/>Feed, singleton, colors,<br/>presentation gate"]:::changed
    picker["shimmers/picker.rs"]:::changed
    reel["shimmers/reel.rs<br/>test only"]:::changed
  end
  subgraph lib["moves to libshimmer"]
    engine["engine.rs<br/>→ Shimmer"]:::moved
    script["script.rs<br/>→ preview::Script"]:::moved
    stock["17 shimmer files, ROSTER<br/>→ stock"]:::moved
    stagefile["stage.rs<br/>→ Stage, Shader, paint"]:::moved
    kit["kit.rs<br/>→ kit"]:::moved
  end

  protocol -->|observe| live
  indicator -->|"dress_status<br/>shade_status"| live
  rows -->|tool_mark| live
  boot -->|"engage_random<br/>follow_presentation"| live
  slash -->|picker| picker
  picker --> live
  reel --> picker
  live ==>|"Shimmer, Spell, Palette,<br/>ROSTER, TICK"| lib
  picker ==>|"Shimmer, Script,<br/>ROSTER"| lib
  reel ==>|"Shimmer"| lib
  engine --> stagefile
  stock --> stagefile
  stock --> kit
  script --> stagefile
  style tui fill:#0b1622,stroke:#58a6ff
  style lib fill:#160f24,stroke:#bc8cff
```

Thin arrows are calls inside one side; thick arrows cross the cut, and their labels are the public API they need. Bold-bordered files stay but change. The seven calls codex-tui makes into `shimmers` keep their signatures, because `live.rs` absorbs the change.

| Today, in `codex-rs/tui/src/shimmers/` | Destination |
|---|---|
| `stage.rs` | `Stage`, `Shader`, `Spell`, paint types |
| `engine.rs` | `Shimmer`; its span conversion moves to the `ratatui` feature |
| `kit.rs` | `kit`, crate-private |
| `script.rs` | `preview::Script` |
| 17 shimmer files and `ROSTER` | `stock` |
| `live.rs` | stays: `Feed`, the process singleton, color detection, the presentation gate |
| `picker.rs`, `reel.rs` | stay, rebuilt on the public API |

## 3 Nouns

```mermaid
classDiagram
  direction TB
  class Shimmer {
    +new(recipe: Recipe, seed: u64) Shimmer
    +show(caption: str)
    +set_spell(spell: Spell)
    +set_intensity(intensity: Intensity)
    +advance(dt: Duration)
    +status(palette: Palette, motion: Motion) Row
    +bullet(palette: Palette) Ink
  }
  class Recipe {
    +name
    +blurb
    +credit
  }
  class ROSTER {
    <<static>>
  }
  class Spell {
    <<enumeration>>
    Idle
    Busy
    Wait
  }
  class Intensity {
    <<f64 at least 0>>
  }
  class Row {
    +mark
    +letters
    +insert
    +tail
    +backdrop
  }
  class Ink {
    +glyph
    +fg
    +bg
    +bold
    +underline
  }
  class Shader {
    <<trait>>
    ~tick(stage)
    ~pause(stage)
    ~status(stage, row)
    ~bullet(stage) Ink
  }
  class Stage {
    ~t
    ~quanta
    ~caption
    ~flow(window) f64
  }
  class Clock {
    ~owed
    ~due(dt) u32
  }
  class LavaLamp
  Shimmer "1" *-- "1" Clock
  Shimmer "1" *-- "1" Stage
  Shimmer "1" *-- "1" Shader
  Shimmer --> "1" Recipe
  ROSTER "1" *-- "17" Recipe
  Recipe ..> Shader : summons
  LavaLamp ..|> Shader
  Shader ..> Stage : reads
  Shader ..> Row : paints
  Row "1" *-- "*" Ink
  Stage *-- Spell
  Stage *-- Intensity
  style Shimmer fill:#2a1d45,stroke:#bc8cff,color:#f5f0ff
  style Recipe fill:#2a1d45,stroke:#bc8cff,color:#f5f0ff
  style ROSTER fill:#2a1d45,stroke:#bc8cff,color:#f5f0ff
  style Spell fill:#2a1d45,stroke:#bc8cff,color:#f5f0ff
  style Intensity fill:#2a1d45,stroke:#bc8cff,color:#f5f0ff
  style Row fill:#2a1d45,stroke:#bc8cff,color:#f5f0ff
  style Ink fill:#2a1d45,stroke:#bc8cff,color:#f5f0ff
```

Violet: public, its members marked `+`. Grey: crate-private, marked `~`. Filled diamond: owns, with multiplicities; arrow: refers to; dashed arrow: uses; dashed arrow with hollow triangle: implements. Members appear only where they carry the design. `LavaLamp` stands for all 17 stock shaders; `Palette`, `Motion`, and `Script` appear in the glossary only.

| Term | Meaning | Today |
|---|---|---|
| Shimmer | One recipe's shader running on its stage and clock; what a host holds | `Engine` |
| Shader | The animation: reads the stage each tick and paints a row | trait `Shimmer` |
| Recipe | Catalog entry: name, blurb, credit, and a constructor for its shader | `Recipe` and `RosterEntry` |
| ROSTER | The stock recipes, in picker order | `ROSTER` |
| Stage | What a shader reads each tick, as a GPU shader reads uniforms | `Stage` |
| Tick | One 32 ms step of shimmer time | `TICK` |
| Clock | Converts the host's Δt into whole ticks; a gap longer than `STALE` yields one | `stepped` and `STALE` in `live.rs` |
| Spell | `Busy`: producing; `Wait`: blocked on something external; `Idle`: no bout | `Spell` without `Idle`, plus `pause` |
| Bout | A stretch of `Busy` and `Wait` between two `Idle`s | a codex turn |
| Intensity | How hard the host works: at least 0, where 1.0 is brisk, the most the stage shows linearly | the paced token rate and `KNEE` |
| Quantum | One whole unit of paced work, as the stage counts it | a paced token |
| Caption | The text a shimmer dresses | `header` |
| Palette | The terminal's default foreground and background | `Palette` |
| Motion | Whether the mark and the letters may move | the `mark` and `letters` booleans |
| Row | The paint of the status line's lead: mark, letters, insert, tail, backdrop | `Row` |
| Ink | The paint of one cell | `Ink` |
| Bullet | The one-cell mark of a running sub-task | the tool mark |
| Script | A looping canned bout that previews a recipe | `Script` |

## 4 Lifecycle

```mermaid
stateDiagram-v2
  direction LR
  [*] --> Idle: new(recipe, seed)
  Idle --> Busy: set_spell(Busy)
  Idle --> Wait: set_spell(Wait)
  state Bout {
    Busy --> Wait: set_spell(Wait)
    Wait --> Busy: set_spell(Busy)
  }
  Bout --> Idle: set_spell(Idle) / pause
  note left of Idle
    Any state, no transition:
    show(caption)
    set_intensity(i)
    advance(Δt)
  end note
```

Transitions read `command / effect`, and `pause` goes to the shader. Setting the current spell again is not a transition. The note's commands change data, not state: a changed caption restarts its age, and `advance` runs the ticks the clock owes (§3).

## 5 Frame

```mermaid
sequenceDiagram
  autonumber
  participant W as Work source
  participant H as Host
  participant S as Shimmer
  participant D as Shader
  H->>S: new(recipe, seed)
  W-)H: bout starts
  H->>S: set_spell(Busy), show("Planning"), set_intensity(0.4)
  loop each frame, every TICK (32 ms)
    opt work changed since the last frame
      H->>S: show, set_spell, set_intensity
    end
    H->>S: advance(Δt)
    loop each whole tick owed
      S->>S: stage takes the tick's quanta
      S->>D: tick(stage)
    end
    H->>S: status(palette, motion)
    S->>D: status(stage, row)
    S-->>H: Row
    Note over H: lays the lead beside its own spans,<br/>shades the backdrop
  end
  W-)H: bout ends
  H->>S: set_spell(Idle)
  S->>D: pause(stage)
```

Solid arrows are calls, the dashed arrow a return, open arrowheads events. `loop` repeats its contents; `opt` runs them only when its condition holds.

## 6 Promises

| # | Promise | Today | Check |
|---|---|---|---|
| P1 | libshimmer makes no outbound call: it reads no clock or environment, spawns no thread, keeps no global state, and takes no callback. | broken by three shaders' `rand` | clippy `disallowed-methods`; review |
| P2 | Paint is a function of the recipe, the seed, the command sequence, the palette, and the motion. | broken by `rand` | property test: twin shimmers under one random command sequence paint alike |
| P3 | The world changes only at whole ticks: `advance(a)` then `advance(b)` equals `advance(a + b)` while both gaps stay within `STALE`. | holds, in `live.rs` | property test |
| P4 | A gap longer than `STALE` (1 s) advances exactly one tick. | holds, in `live.rs` | unit test |
| P5 | Each entry into `Idle` delivers exactly one `pause` to the shader, when commanded; nothing else delivers one. | new | unit test with a recording shader |
| P6 | Painting changes nothing: `status` and `bullet` take `&self`. | new: `Engine::status` takes `&mut self` | signature |
| P7 | The lead spans `len(mark) + (1 if mark) + width(caption) + (1 if insert) + len(tail)` columns, and the backdrop at most 80. | backdrop tested; lead unverified | test across `ROSTER` |
| P8 | With still letters, the lead is the caption verbatim, without styling, tail, or backdrop; a still mark is absent. | holds | test across `ROSTER` |
| P9 | No caption, intensity, or Δt causes a panic. | unverified | `Intensity::new` saturates; property test |
| P10 | Ticks never read colors; colors exist only at paint. | broken by critter's tint | by construction: the stage holds no palette |

## 7 Decisions

**D1 Home.** (a) Own repository and crate, `libshimmer`; crates.io's `shimmer` is a wikitext parser, and `libgrid` set the naming precedent. (b) A crate in codex-1337's workspace. Chosen: (a), because other hosts should not depend on a fork of codex.

**D2 Paint target.** (a) Paint as plain data (`Row`, `Ink`, `Rgb`), with an optional `ratatui` feature built on `ratatui-core`, the crate ratatui designates for widget libraries. (b) Paint as ratatui types throughout. Chosen: (a). The reel already flattens paint into cells, and codex-tui and memview both build on ratatui 0.30.2, whose types come from `ratatui-core`, so the feature costs codex-tui nothing.

**D3 Time.** (a) The host passes Δt; libshimmer owns the fixed-step accumulator and the stale cutoff. (b) libshimmer reads the wall clock. Chosen: (a), for P1 to P4: the picker restarts its script, the reel films synthetic time, and tests should not wait.

**D4 Work signal.** (a) A sticky `Intensity` beside `Spell`. (b) The host pushes whole quanta. (c) The host pushes lumps of work, which libshimmer drains. Chosen: (a). The host sets one number whenever it likes, codex-tui's `Feed` already computes a rate, and draining lumps models the host's domain. The sketch that prompted this design had two inputs, a text and a scalar; the shaders read a third, `stage.busy()`, more than any other stage member (56 reads), so `Spell` joins them.

**D5 Openness.** (a) Closed catalog: `Shader`, `Stage`, and `kit` stay crate-private. (b) Public `Shader` for third-party shimmers. Chosen: (a). Opening later is a minor release and closing a major one, and (b) would freeze the stage's members.

**D6 Randomness.** Each shimmer takes a seed at construction and draws from a crate-internal SplitMix64; the `rand` dependency, used today by lfsr, cambrian, and turing, goes. Reason: P2, which also stops the reel's Turing and Cambrian takes from changing between exports.

**D7 Bout end.** (a) `Spell::Idle`, whose entry delivers `pause`. (b) `pause()` beside a two-state `Spell`. Chosen: (a). Busy, waiting, and between bouts are exclusive states, so they belong to one enum, which also makes a lost or doubled pause impossible (P5).

**D8 Reset.** No reset command. Clearing the caption is `show("")`; ending a bout is `set_spell(Idle)`; a new world is `Shimmer::new`. Reason: each of the three meanings of "reset" gets its own name.

**D9 Timer.** The status-line timer leaves the stage. Only critter reads it, for a gesture in the first two seconds of each minute, which moves onto shimmer time. Reason: hosts without a timer need not invent one.

**D10 Palette.** Colors arrive only at paint. Critter, which smooths a color in `tick` today, smooths blend weights instead. Reason: P10; codex-tui also loses its `Engaged` state, which exists only to wait for colors before constructing an engine.

**D11 Names.** `Engine` becomes `Shimmer`, the trait `Shimmer` becomes `Shader`, the header becomes the caption, and paced tokens become quanta. Reason: the type a host holds carries the library's name, and a token is a language model's unit.

## 8 Questions

**Q1** What does a shimmer paint while `Idle`? codex-tui never paints then; a host that keeps its status line would. (a) As in `Wait`. (b) As in `Busy` at zero intensity. (c) Painting while `Idle` is a host error. Lean: (a), which every shader already does, because `stage.busy()` is false.

**Q2** Should the host pass the line's width? The only bound today is a test that holds backdrops to 80 columns; the host clips the rest. Lean: no, until a host needs it.

**Q3** Recipes that paint Unicode 16 octants (phyllotaxis, tetromino) show missing glyphs in terminals and fonts without them. (a) A quadrant fallback the host can request. (b) Octants required. Lean: (b), until a host reports the problem.

**Q4** Distribution. (a) crates.io: any host, and every machine that builds codex-1337, fetches it without credentials. (b) A private git dependency, which every such machine must be able to fetch. Lean: (a).

**Q5** Should memview, a second ratatui 0.30.2 host, adopt libshimmer in the first release? Lean: yes; a boundary drawn from one host is untested.

**Q6** codex-tui's `Feed` and `Script` both drain lumps of work over `DRAIN` (4 s) and share that constant today. (a) libshimmer exports a `Reservoir` that turns lumps into intensity, used by both. (b) Each keeps its own constant. Lean: (a), so the concept keeps one owner.

## Appendix: public API sketch

Uncompiled; the first implementation step compiles it.

```rust
pub const TICK: Duration = Duration::from_millis(32);
pub const STALE: Duration = Duration::from_secs(1);

pub struct Shimmer { /* recipe, Box<dyn Shader>, Stage, Clock */ }

impl Shimmer {
    pub fn new(recipe: &'static Recipe, seed: u64) -> Self;
    pub fn recipe(&self) -> &'static Recipe;

    // Commands. Each input holds until the next command replaces it.
    pub fn show(&mut self, caption: &str);
    pub fn set_spell(&mut self, spell: Spell);
    pub fn set_intensity(&mut self, intensity: Intensity);
    pub fn advance(&mut self, dt: Duration);

    // Queries.
    pub fn status(&self, palette: Palette, motion: Motion) -> Row<'_>;
    pub fn bullet(&self, palette: Palette) -> Ink;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Spell { #[default] Idle, Busy, Wait }

/// How hard the host works; 1.0 is brisk, the most the stage shows linearly.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Intensity(f64);

impl Intensity {
    pub const BRISK: Self = Self(1.0);
    /// NaN and negative levels read as zero.
    pub fn new(level: f64) -> Self;
}

pub struct Recipe { /* name, blurb, credit, summon: fn(u64) -> Box<dyn Shader> */ }

impl Recipe {
    pub fn name(&self) -> &'static str;
    pub fn blurb(&self) -> &'static str;
    pub fn credit(&self) -> &'static str;
}

pub mod stock {
    pub static ROSTER: [Recipe; 17];
    pub fn named(name: &str) -> Option<&'static Recipe>;
}

pub struct Rgb(pub f64, pub f64, pub f64);
pub struct Palette { pub fg: Rgb, pub bg: Rgb }
pub struct Motion { pub mark: bool, pub letters: bool }
pub struct Ink { pub glyph: Option<char>, pub fg: Option<Rgb>, pub bg: Option<Rgb>, pub bold: bool, pub underline: Option<Rgb> }

/// The status line's lead as painted, and the backdrop beneath it.
pub struct Row<'a> { /* caption graphemes and their inks */ }

impl Row<'_> {
    /// Symbols and their inks, left to right.
    pub fn cells(&self) -> impl Iterator<Item = (&str, Ink)>;
    pub fn backdrop(&self) -> &[Rgb];
}

pub mod preview {
    pub struct Script;
    pub struct Cue { pub spell: Spell, pub caption: &'static str, pub intensity: Intensity }
    impl Script {
        pub fn new() -> Self;
        pub fn cue(&self, at: Duration) -> Cue;
    }
}

#[cfg(feature = "ratatui")]
pub mod ratatui {
    impl Row<'_> {
        pub fn spans(&self) -> Vec<Span<'static>>;
        pub fn shade(&self, area: Rect, buf: &mut Buffer);
    }
    impl Ink {
        pub fn span(self, text: &str) -> Span<'static>;
    }
}
```
