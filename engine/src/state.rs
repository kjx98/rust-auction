#[derive(Debug, PartialEq)]
pub enum State {
    StateIdle,
	StateStart,
	StatePreAuction,
	StateCallAuction,
	StateTrading,
	StatePause,
	StateBreak,
	StateStop,
	StateEnd,
}

use State::*;

impl Default for State {
    fn default() -> Self { State::StateIdle }
}

impl State {
    pub fn review(&self, new_state: &Self) -> bool {
        match new_state {
            State::StateIdle => {
                match *self {
                StateEnd | StateStop => true,
                _ => false
                }
            },
            State::StateStart => {
                match *self {
                StateIdle | StateStop => true,
                _ => false
                }
            },
            State::StatePreAuction => {
                match *self {
                StateStart | StatePause | StateBreak => true,
                _ => false
                }
            },
            State::StateCallAuction => {
                if *self == StatePreAuction {
                    true
                } else {
                    false
                }
            },
            State::StateTrading => {
                match *self {
                StateCallAuction | StatePause | StateBreak => true,
                _ => false
                }
            },
            State::StatePause | State::StateBreak => {
                if *self == StateTrading {
                    true
                } else {
                    false
                }
            },
            State::StateStop => {
                match *self {
                StateIdle | StateTrading | StateBreak | StatePause => true,
                _ => false
                }
            },
            State::StateEnd => {
                match *self {
                StateIdle | StateStop => true,
                _ => false
                }
            },
        }   // end match
    }   // review
}
