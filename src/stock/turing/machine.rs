//! Blank-start programs for the 37-cell ring; no search or parsing runs in the animation.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Machine {
    Wolfram,
    Counter,
    Sawtooth,
    Harrow,
}

impl Machine {
    pub(super) const ALL: [Self; 4] = [Self::Wolfram, Self::Counter, Self::Sawtooth, Self::Harrow];

    pub(super) fn next(self) -> Self {
        Self::ALL[(self as usize + 1) % Self::ALL.len()]
    }

    pub(super) fn instruction(self, state: State, bit: bool) -> Option<Transition> {
        let program: &[[Option<Transition>; 2]] = match self {
            Self::Wolfram => &WOLFRAM,
            Self::Counter => &COUNTER,
            Self::Sawtooth => &SAWTOOTH,
            Self::Harrow => &HARROW,
        };
        program[state as usize][usize::from(bit)]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum State {
    A,
    B,
    C,
    D,
    E,
}

impl State {
    const ALL: [Self; 5] = [Self::A, Self::B, Self::C, Self::D, Self::E];

    pub(super) fn glyph(self) -> char {
        char::from(b'A' + self as u8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Direction {
    Left,
    Right,
}

impl Direction {
    pub(super) fn glyph(self) -> char {
        match self {
            Self::Left => '◂',
            Self::Right => '▸',
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Transition {
    pub(super) write: bool,
    pub(super) direction: Direction,
    pub(super) next: State,
}

// Wolfram, NKS p.81: https://www.wolframscience.com/nks/p81--turing-machines/
const WOLFRAM: [[Option<Transition>; 2]; 4] = program(b"1LC0RC_1LD0RB_1RB1LD_1RA0LD");
// Ligocki's counter: https://www.sligocki.com/2022/06/14/counter-induction.html
// Its undefined instruction remains a halt; arbitrary initial tapes are not valid seeds.
const COUNTER: [[Option<Transition>; 2]; 5] = program(b"1RB1LA_0LC0RB_0LD0LB_1RE---_1LE1LA");
// Locally screened tables, not historical names. Blank 37-ring periods are 2,097,148 and
// 567,247 transitions respectively; finite-ring claims do not describe infinite tapes.
const SAWTOOTH: [[Option<Transition>; 2]; 3] = program(b"0RB0LA_1RC1RB_1LC1LA");
const HARROW: [[Option<Transition>; 2]; 4] = program(b"1RC0LB_0RD1RC_0RB1RA_1RC1RD");

/// Standard write/direction/state notation, checked at compile time against each program's
/// actual state count. Three dashes mean halt, never an invented continuation.
const fn program<const N: usize>(source: &[u8]) -> [[Option<Transition>; 2]; N] {
    assert!(
        N > 0 && N <= State::ALL.len(),
        "supported machine state count"
    );
    assert!(source.len() == N * 7 - 1, "two instructions per state");
    let mut rules = [[None; 2]; N];
    let mut state = 0;
    while state < N {
        if state > 0 {
            assert!(source[state * 7 - 1] == b'_', "state separator");
        }
        let mut symbol = 0;
        while symbol < 2 {
            let at = state * 7 + symbol * 3;
            if source[at] == b'-' {
                assert!(
                    source[at + 1] == b'-' && source[at + 2] == b'-',
                    "halt instruction"
                );
            } else {
                assert!(
                    source[at] == b'0' || source[at] == b'1',
                    "binary tape symbol"
                );
                assert!(
                    source[at + 1] == b'L' || source[at + 1] == b'R',
                    "head direction"
                );
                assert!(source[at + 2] >= b'A', "next state");
                let next = (source[at + 2] - b'A') as usize;
                assert!(next < N, "next state belongs to this machine");
                rules[state][symbol] = Some(Transition {
                    write: source[at] == b'1',
                    direction: if source[at + 1] == b'L' {
                        Direction::Left
                    } else {
                        Direction::Right
                    },
                    next: State::ALL[next],
                });
            }
            symbol += 1;
        }
        state += 1;
    }
    rules
}
