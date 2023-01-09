use std::cmp::Ordering;

use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};

use crate::grid::gridstate::GridBoundaryState;
#[allow(unused_imports)]
use crate::grid::Boundaries;
use crate::grid::BoundaryUnitTrait;
use crate::util::Steps;

/// Restriction on the grid. How long may the grid pass the border in the number of steps? Generated from the
/// GridParameters file.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, AvroSchema)]
pub struct BoundaryBand<T: BoundaryUnitTrait> {
    /// The border value, that the value must pass to trigger this restriction. Direction is given by the related
    /// [Boundaries].
    pub border:           T,
    /// How many steps can the value pass the border before the restriction is broken?
    pub max_time_allowed: Steps,
    /// How many steps have already been had past the border?
    pub time_passed:      Steps,
}

impl<T: BoundaryUnitTrait> BoundaryBand<T> {
    /// Check if the restriction is active, give the state of the Grid at this point according to this node.
    #[inline]
    pub fn check(&mut self, current: T, direction_regards_normal: Ordering) -> GridBoundaryState {
        let cmp = current.cmp(&self.border);
        if cmp == direction_regards_normal || cmp == Ordering::Equal {
            self.time_passed += Steps(1);
        }
        if self.time_passed > self.max_time_allowed {
            if direction_regards_normal == Ordering::Less {
                GridBoundaryState::TooLow
            } else {
                GridBoundaryState::TooHigh
            }
        } else if direction_regards_normal == Ordering::Less {
            GridBoundaryState::Low
        } else {
            GridBoundaryState::High
        }
    }
}
