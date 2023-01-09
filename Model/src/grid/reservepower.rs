use std::fmt::Debug;

use serde::{Serialize, Deserialize};
use thousands::Separable;

use crate::util::Watt;

/// Grid energy storage, that can be used to regulate.
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReservePower {
    /// How much power can the grid operator let produce less in total?
    pub lower_limit:   Watt,
    /// How much power can the grid operator let produce more in total?
    pub upper_limit:   Watt,
    /// How much of that room is currently in use?
    pub current_usage: Watt,
    /// How much of that room can be used in a single step?
    pub watt_per_step: Watt,
}

impl ReservePower {
    /// Try to compensate the power mismatch with the available room.
    /// Returns the amount of Watt that is compensated.
    pub fn compensate(&mut self, power_error: Watt) -> Watt {
        let new_usage = self.current_usage + power_error;
        if new_usage > self.upper_limit {
            let res = self.upper_limit - self.current_usage;
            self.current_usage = self.upper_limit;
            res
        } else if new_usage < self.lower_limit {
            let res = self.lower_limit - self.current_usage;
            self.current_usage = self.lower_limit;
            res
        } else {
            let take = match (power_error.0.abs().cmp(&self.watt_per_step.0), power_error.0 < 0) {
                (std::cmp::Ordering::Less, _) => power_error,
                (std::cmp::Ordering::Equal, _) => power_error,
                (std::cmp::Ordering::Greater, true) => self.watt_per_step * -1,
                (std::cmp::Ordering::Greater, false) => self.watt_per_step,
            };
            self.current_usage += take;
            take
        }
    }
}

impl Debug for ReservePower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReservePower")
            .field(
                "lower_limit",
                &format!("Watt({})", &self.lower_limit.0.separate_with_underscores()),
            )
            .field(
                "upper_limit",
                &format!("Watt({})", &self.upper_limit.0.separate_with_underscores()),
            )
            .field(
                "current_usage",
                &format!("Watt({})", &self.current_usage.0.separate_with_underscores()),
            )
            .field(
                "watt_per_step",
                &format!("Watt({})", &self.watt_per_step.0.separate_with_underscores()),
            )
            .finish()
    }
}
