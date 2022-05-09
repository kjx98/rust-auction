mod state;
mod engine;
mod order_book;
mod deal;

pub use crate::state::State;
pub use crate::engine::MatchEngine;
pub use crate::deal::{Deal, Deals};

#[cfg(test)]
mod tests {
    #[test]
    fn engine_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
