use rand::Rng;

pub mod physics;
pub mod unit;

/// A quantified data from an arbitrary value. This is can only be derived for enums.
pub trait Quantified<T> {
    fn quantify(value: T) -> Self;
    fn sample(self, rng: &mut impl Rng) -> T;
}
