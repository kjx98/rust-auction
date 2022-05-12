use std::fmt;

#[derive(PartialEq)]
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

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            StateIdle => write!(f, "State Idle"),
            StateStart => write!(f, "State Start"),
            StatePreAuction => write!(f, "State PreAuction"),
            StateCallAuction => write!(f, "State CallAuction"),
            StateTrading => write!(f, "State Trading"),
            StatePause => write!(f, "State Pause"),
            StateBreak => write!(f, "State Break"),
            StateStop => write!(f, "State Stop"),
            StateEnd => write!(f, "State End"),
        }
    }
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
                StateStart | StateBreak => true,
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
            // only start/callAuction/pause can change state to trading
            State::StateTrading => {
                match *self {
                StateStart | StateCallAuction | StatePause => true,
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
                StateTrading | StateBreak | StatePreAuction => true,
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
    // state == StateTrading
    pub fn is_tc(&self) -> bool {
        *self == StateTrading
    }
    // can place order in orderBook
    pub fn can_book(&self) -> bool {
        *self == StatePreAuction || *self == StateTrading
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn test_state() {
        let mut state: State = Default::default();
        assert!(state == State::StateIdle);
        assert!(state.review(&State::StateStart));
        state = State::StateStart;
        assert!(!state.is_tc());
        assert!(!state.can_book());
        assert!(state.review(&State::StatePreAuction));
        assert!(!state.review(&State::StateCallAuction));
        assert!(state.review(&State::StateTrading));
        state = State::StatePreAuction;
        assert!(!state.is_tc());
        assert!(state.can_book());
        assert!(state.review(&State::StateCallAuction));
        assert!(!state.review(&State::StateTrading));
        state = State::StateCallAuction;
        assert!(!state.is_tc());
        assert!(!state.can_book());
        assert!(state.review(&State::StateTrading));
        state = State::StateTrading;
        assert!(state.review(&State::StatePause));
        assert!(state.review(&State::StateBreak));
        assert!(state.review(&State::StateStop));
        assert!(!state.review(&State::StateEnd));
        assert!(!state.review(&State::StatePreAuction));
        assert!(state.is_tc());
        assert!(state.can_book());
    }
}
