mod state;
mod engine;
mod order_book;

pub use crate::state::State;
pub use crate::engine::MatchEngine;

#[cfg(test)]
mod tests {
    #[test]
    fn engine_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
