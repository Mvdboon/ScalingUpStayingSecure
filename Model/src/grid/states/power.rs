use std::fmt::{Debug, Display};
use std::iter::Sum;
use std::ops::{Add, AddAssign};

use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};
use thousands::Separable;

use crate::attack::AttackBehaviour;
use crate::util::{BaseUint, Watt};

/// A struct containing the power state of an
/// agent.
#[derive(PartialOrd, Clone, Serialize, AvroSchema, Deserialize, Default)]
pub struct PowerState {
    /// Power that is generated
    pub power_generated:                Watt,
    /// Current amount of power measured. Positive
    /// indicates a draw by the children.
    pub power_used:                     Watt,
    /// Current amount of power being reported to
    /// be used. Positive indicates a draw by the
    /// children.
    pub power_reported:                 Watt,
    /// Current error between power used and power
    /// reported.
    pub power_error:                    Watt,
    pub(crate) history_power_used:      Vec<Watt>,
    pub(crate) history_power_generated: Vec<Watt>,
    pub(crate) history_power_reported:  Vec<Watt>,
    pub(crate) history_power_error:     Vec<Watt>,
    pub(crate) history_len:             BaseUint,
}

impl PowerState {
    /// Creates a default `PowerState`. Needs to
    /// init before usage.
    pub const fn new() -> Self {
        Self {
            power_used:              Watt(0),
            power_generated:         Watt(0),
            power_reported:          Watt(0),
            power_error:             Watt(0),
            history_power_used:      vec![],
            history_power_reported:  vec![],
            history_power_error:     vec![],
            history_power_generated: vec![],
            history_len:             10,
        }
    }

    /// Update the history of the unit using internal data.
    pub fn update_history(&mut self) {
        self.history_power_used.push(self.power_used);
        self.history_power_reported.push(self.power_reported);
        self.history_power_error.push(self.power_error);
        self.history_power_generated.push(self.power_generated);

        if self.history_power_error.len() > self.history_len as usize {
            self.history_power_error.remove(0);
            self.history_power_reported.remove(0);
            self.history_power_error.remove(0);
        }
    }

    pub(crate) fn manual_add(&mut self, rhs: &Self) {
        self.power_used += rhs.power_used;
        self.power_reported += rhs.power_reported;
        self.power_error += rhs.power_error;
        self.power_generated += rhs.power_generated;
    }

    /// Attack the PowerState using the [AttackBehaviour] given.
    pub fn attack(&mut self, attack: AttackBehaviour) {
        self.power_generated = Watt((self.power_generated.0 as f32 * attack.generation_modifier) as i64);
        self.power_reported = Watt((self.power_reported.0 as f32 * attack.report_modifier) as i64);
        self.power_error = self.power_reported - (self.power_used - self.power_generated);
    }
}

impl PartialEq for PowerState {
    fn eq(&self, other: &Self) -> bool {
        self.power_used == other.power_used
            && self.power_reported == other.power_reported
            && self.power_error == other.power_error
            && self.power_generated == other.power_generated
    }
}

impl Add for PowerState {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            power_used:              self.power_used + rhs.power_used,
            power_reported:          self.power_reported + rhs.power_reported,
            power_error:             self.power_error + rhs.power_error,
            power_generated:         self.power_generated + rhs.power_generated,
            history_power_used:      vec![],
            history_power_reported:  vec![],
            history_power_error:     vec![],
            history_power_generated: vec![],
            history_len:             10,
        }
    }
}

impl AddAssign for PowerState {
    fn add_assign(&mut self, rhs: Self) {
        self.power_used += rhs.power_used;
        self.power_reported += rhs.power_reported;
        self.power_error += rhs.power_error;
        self.power_generated += rhs.power_generated;
    }
}

impl Sum for PowerState {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self::new(), |a, b| (a + b))
    }
}

impl Display for PowerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PowerState")
            .field(
                "Power used",
                &format!("Watt({})", &self.power_used.0.separate_with_underscores()),
            )
            .field(
                "Power reported",
                &format!("Watt({})", &self.power_reported.0.separate_with_underscores()),
            )
            .field(
                "Power error",
                &format!("Watt({})", &self.power_error.0.separate_with_underscores()),
            )
            .field(
                "Power generated",
                &format!("Watt({})", &self.power_generated.0.separate_with_underscores()),
            )
            .finish_non_exhaustive()
    }
}

impl Debug for PowerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.sign_minus() {
            f.debug_struct("PowerState")
                .field("power_error", &self.power_error)
                .field("history_power_used", &self.history_power_used)
                .field("history_power_reported", &self.history_power_reported)
                .field("history_power_error", &self.history_power_error)
                .field("history_len", &self.history_len)
                .finish()
        } else {
            f.debug_struct("PowerState")
                .field(
                    "Power used",
                    &format!("Watt({})", &self.power_used.0.separate_with_underscores()),
                )
                .field(
                    "Power reported",
                    &format!("Watt({})", &self.power_reported.0.separate_with_underscores()),
                )
                .field(
                    "Power error",
                    &format!("Watt({})", &self.power_error.0.separate_with_underscores()),
                )
                .field(
                    "Power generated",
                    &format!("Watt({})", &self.power_generated.0.separate_with_underscores()),
                )
                .finish_non_exhaustive()
        }
    }
}

#[cfg(test)]
mod states_tests {

    use std::sync::Arc;

    use parking_lot::RwLock;

    use super::*;
    use crate::agent::{AgentList, AgentTrait, Household, Netstation};
    use crate::grid::PowerGeneration;
    use crate::model::ModelParameters;

    const fn powerstate_test() -> (PowerState, PowerState, PowerState) {
        let ps1: PowerState = PowerState {
            power_used:              Watt(40),
            power_reported:          Watt(40),
            power_error:             Watt(40),
            power_generated:         Watt(40),
            history_power_generated: vec![],
            history_power_used:      vec![],
            history_power_reported:  vec![],
            history_power_error:     vec![],
            history_len:             10,
        };
        let ps2: PowerState = PowerState {
            power_used:              Watt(40),
            power_reported:          Watt(40),
            power_error:             Watt(40),
            power_generated:         Watt(40),
            history_power_generated: vec![],
            history_power_used:      vec![],
            history_power_reported:  vec![],
            history_power_error:     vec![],
            history_len:             10,
        };
        let ps3: PowerState = PowerState {
            power_used:              Watt(80),
            power_reported:          Watt(80),
            power_error:             Watt(80),
            power_generated:         Watt(80),
            history_power_generated: vec![],
            history_power_used:      vec![],
            history_power_reported:  vec![],
            history_power_error:     vec![],
            history_len:             10,
        };
        (ps1, ps2, ps3)
    }
    #[test]
    fn powerstate_sum_test() {
        let (ps1, ps2, ps3) = powerstate_test();
        assert_eq!((ps1 + ps2), (ps3));
    }
    #[test]
    fn powerstate_iter_sum_test() {
        let (ps1, ps2, ps3) = powerstate_test();
        let it = vec![ps1, ps2].into_iter();
        assert_eq!(it.sum::<PowerState>(), (ps3));
    }

    #[test]
    fn check_powerstate_calc() {
        let mut param = ModelParameters::test();
        let mut agent = Netstation::new(117, &param.grid);
        let mut child1 = Household::new(117, PowerGeneration::new_no_pv(Some(117), &mut param).unwrap());
        let mut child2 = Household::new(117, PowerGeneration::new_pv(Some(117), &mut param).unwrap());

        let (ps1, ps2, ps3) = powerstate_test();
        *child1.powerstate_mut() = ps1;
        *child2.powerstate_mut() = ps2;
        let children: AgentList = vec![Arc::new(RwLock::new(child1)), Arc::new(RwLock::new(child2))];
        *agent.children_mut() = children;

        agent.calc_power_from_child();

        assert_eq!(*agent.powerstate(), ps3);
    }
}
