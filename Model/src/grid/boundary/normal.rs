use std::cmp::Ordering;

use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};

use crate::grid::BoundaryUnitTrait;

/// What is considered normal behaviour of this parameter? Given by the GridParameters file.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Serialize, AvroSchema, Default)]
pub struct NormalBand<T: BoundaryUnitTrait> {
    /// Lower limit of this GridParameter.
    pub lower:  T,
    /// Higher limit of this GridParameter.
    pub higher: T,
}

impl<T: BoundaryUnitTrait> NormalBand<T> {
    /// Is the given value within the normal behaviour, or higher/lower?
    pub fn compare(&self, value: T) -> Ordering {
        if value < self.lower {
            return Ordering::Less;
        }
        if value > self.higher {
            return Ordering::Greater;
        }
        Ordering::Equal
    }
}
