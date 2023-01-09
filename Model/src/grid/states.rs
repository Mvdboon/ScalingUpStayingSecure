use crate::grid::{BoundaryUnitTrait, Grid};
use crate::util::Watt;

mod frequency;
mod infectionstate;
mod power;
mod volt;

pub use frequency::*;
pub use infectionstate::*;
pub use power::*;
pub use volt::*;

/// A trait to be implemented for the grid states. Gives an interface to work with.
pub trait GridState<T: BoundaryUnitTrait> {
    /// Create a new state.
    fn new(grid_param: &Grid) -> Self;

    /// Update the state with its new value.
    fn update(&mut self, new: T);

    /// Calculate the impact of the power_mismatch. Gives back the new value.
    fn power_mismatch(&mut self, power_total: &Watt, power_error: &Watt, bulk_consumption: &Watt) -> T;
}
