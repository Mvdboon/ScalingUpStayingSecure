mod band;
mod normal;

use std::cmp::Ordering;
use std::fmt::Display;
use std::str::FromStr;

use apache_avro::schema::derive::AvroSchemaComponent;
use apache_avro::AvroSchema;
pub use band::*;
pub use normal::*;
use serde::{Deserialize, Serialize};

use crate::grid::gridstate::GridBoundaryState;
use crate::grid::{Grid, GridWarning};
use crate::util::{mHz, mVolt, BaseInt, Steps, StructTraitBound};

/// A helper trait to enable checking the boundaries
pub trait BoundaryAgentTrait<T: BoundaryUnitTrait> {
    /// Check the boundary and get a warning/error if needed.
    fn boundary_check(&mut self) -> Option<GridWarning>;
}

/// Helper trait, to implement on the units used in the boundary checking.
pub trait BoundaryUnitTrait:
    StructTraitBound + From<BaseInt> + AvroSchemaComponent + FromStr + Display + Default + PartialOrd + Sized
{
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Serialize, AvroSchema, Default)]
/// A struct describing the boundaries that are set for type T. If these are
/// exceeded they will be logged. Currently intended for Voltage and Frequency
/// on the grid.
pub struct Boundaries<T: BoundaryUnitTrait> {
    /// Current State of the grid
    pub state:      GridBoundaryState,
    /// Normal value
    pub normalband: NormalBand<T>,
    /// HashMap of boundary value and max permittable steps.
    pub lowerbands: Vec<BoundaryBand<T>>,
    /// HashMap of boundary value and max permittable steps.
    pub upperbands: Vec<BoundaryBand<T>>,
}

impl<T: BoundaryUnitTrait> Boundaries<T> {
    /// Update the boundary and check all the relevant restrictions to see if the grid is outside its parameters.
    pub fn update(&mut self, current: T) -> Option<GridWarning> {
        let update_cmp = self.normalband.compare(current);
        match update_cmp {
            Ordering::Less => self.check_lowerbands(current),
            Ordering::Equal => {
                self.check_normalband();
                None
            }
            Ordering::Greater => self.check_upperbands(current),
        }
    }

    fn check_normalband(&mut self) {
        if self.state == GridBoundaryState::Normal {
        } else {
            self.change_state(GridBoundaryState::Normal);
            self.reset_bands();
        }
    }

    fn check_upperbands(&mut self, current: T) -> Option<GridWarning> {
        let states: Vec<GridBoundaryState> = self
            .upperbands
            .iter_mut()
            .map(|lb| lb.check(current, Ordering::Greater))
            .collect();
        if states.iter().any(|s| *s == GridBoundaryState::TooHigh) {
            return self.change_state(GridBoundaryState::TooHigh);
        } else if states.iter().any(|s| *s == GridBoundaryState::High) {
            return self.change_state(GridBoundaryState::High);
        }
        None
    }

    fn check_lowerbands(&mut self, current: T) -> Option<GridWarning> {
        let states: Vec<GridBoundaryState> = self
            .lowerbands
            .iter_mut()
            .map(|lb| lb.check(current, Ordering::Less))
            .collect();
        if states.iter().any(|s| *s == GridBoundaryState::TooLow) {
            return self.change_state(GridBoundaryState::TooLow);
        } else if states.iter().any(|s| *s == GridBoundaryState::Low) {
            return self.change_state(GridBoundaryState::Low);
        }
        None
    }

    fn change_state(&mut self, state: GridBoundaryState) -> Option<GridWarning> {
        if self.state != state {
            self.state = state;
        }
        match state {
            GridBoundaryState::TooLow | GridBoundaryState::TooHigh => Some(GridWarning {
                state,
                critical: true,
                agent_index: None,
                agent_powerstate: None,
                freq_state: None,
                volt_state: None,
                infectionstatistics: None,
            }),
            GridBoundaryState::Low | GridBoundaryState::High => Some(GridWarning {
                state,
                critical: false,
                agent_index: None,
                agent_powerstate: None,
                freq_state: None,
                volt_state: None,
                infectionstatistics: None,
            }),
            GridBoundaryState::Normal => None,
        }
    }

    fn reset_bands(&mut self) {
        self.lowerbands.iter_mut().for_each(|b| b.time_passed = Steps(0));
        self.upperbands.iter_mut().for_each(|b| b.time_passed = Steps(0));
    }
}

impl BoundaryUnitTrait for mHz {}
impl BoundaryUnitTrait for mVolt {}

impl From<&Grid> for Boundaries<mHz> {
    fn from(value: &Grid) -> Self { value.freq_boundary.clone() }
}

impl From<&Grid> for Boundaries<mVolt> {
    fn from(value: &Grid) -> Self { value.volt_boundary.clone() }
}
