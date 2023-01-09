use std::fmt::Debug;

use apache_avro::AvroSchema;
use derive_more::{Add, Sum};
#[cfg(not(feature = "single_thread"))]
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::grid::PowerGeneration;
#[cfg(feature = "single_thread")]
use crate::norayon::prelude::*;
use crate::util::{BaseFloat, BaseInt};

/// All possible states that a PV system can have regarding being infected.
#[derive(Clone, Copy, Debug, Serialize, AvroSchema, Deserialize, PartialEq, Eq, PartialOrd, Hash)]
pub enum InfectionState {
    /// Can not be infected.
    NotVulnerable,
    /// Can be infected.
    Vulnerable,
    /// Currently infected by malware.
    Infected,
    /// Not infected but was previously infected before being patched. Is not
    /// vulnerable anymore.
    Patched,
}

/// A helper struct to determine the state of the grid.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Sum, Add)]
pub struct InfectionCount {
    /// Can not be infected.
    pub not_vulnerable: BaseInt,
    /// Can be infected.
    pub vulnerable:     BaseInt,
    /// Currently infected by malware.
    pub infected:       BaseInt,
    /// Not infected but was previously infected before being patched. Is not
    /// vulnerable anymore.
    pub patched:        BaseInt,
}

/// A helper struct to determine the statistics on the infection in the grid.
#[derive(Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct InfectionStatistics {
    /// Total amount of [PowerGeneration] devices
    pub(crate) total:               BaseInt,
    pub(crate) num_not_vulnerable:  BaseInt,
    pub(crate) num_vulnerable:      BaseInt,
    pub(crate) num_infected:        BaseInt,
    pub(crate) num_patched:         BaseInt,
    pub(crate) perc_not_vulnerable: BaseFloat,
    pub(crate) perc_vulnerable:     BaseFloat,
    pub(crate) perc_infected:       BaseFloat,
    pub(crate) perc_patched:        BaseFloat,
}

// --------------------------------------- //

impl InfectionStatistics {
    /// Only input Households
    pub(crate) fn new(states: &[InfectionState]) -> Self {
        // Info
        let (num_not_vulnerable, num_vulnerable, num_infected, num_patched) = count_states(states);

        let total = num_not_vulnerable + num_infected + num_vulnerable + num_patched;
        let perc_not_vulnerable = (num_not_vulnerable * 100) as f32 / total as f32;
        let perc_infected = (num_infected * 100) as f32 / total as f32;
        let perc_vulnerable = (num_vulnerable * 100) as f32 / total as f32;
        let perc_patched = (num_patched * 100) as f32 / total as f32;

        Self {
            total,
            num_not_vulnerable,
            num_vulnerable,
            num_infected,
            num_patched,
            perc_not_vulnerable,
            perc_vulnerable,
            perc_infected,
            perc_patched,
        }
    }
}
#[cfg(not(feature = "single_thread"))]
fn count_states(states: &[InfectionState]) -> (i32, i32, i32, i32) {
    let (num_not_vulnerable, num_vulnerable, num_infected, num_patched) = states
        .par_iter()
        .map(|pg| match pg {
            InfectionState::NotVulnerable => (1, 0, 0, 0),
            InfectionState::Vulnerable => (0, 1, 0, 0),
            InfectionState::Infected => (0, 0, 1, 0),
            InfectionState::Patched => (0, 0, 0, 1),
        })
        // .reduce(|a,b| (a.0 + b.0, a.1 + b.1, a.2 + b.2, a.3 + b.3)).unwrap();
        .reduce(|| (0, 0, 0, 0), |a, b| (a.0 + b.0, a.1 + b.1, a.2 + b.2, a.3 + b.3));
    (num_not_vulnerable, num_vulnerable, num_infected, num_patched)
}
#[cfg(feature = "single_thread")]
fn count_states(states: &[InfectionState]) -> (i32, i32, i32, i32) {
    let (num_not_vulnerable, num_vulnerable, num_infected, num_patched) = states
        .par_iter()
        .map(|pg| match pg {
            InfectionState::NotVulnerable => (1, 0, 0, 0),
            InfectionState::Vulnerable => (0, 1, 0, 0),
            InfectionState::Infected => (0, 0, 1, 0),
            InfectionState::Patched => (0, 0, 0, 1),
        })
        .reduce(|a, b| (a.0 + b.0, a.1 + b.1, a.2 + b.2, a.3 + b.3))
        .unwrap();
    (num_not_vulnerable, num_vulnerable, num_infected, num_patched)
}

impl Debug for InfectionStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InfectionStatistics")
            .field("total", &self.total)
            .field(
                "num_not_vulnerable",
                &format!("{} - ({:.2}%)", &self.num_not_vulnerable, &self.perc_not_vulnerable),
            )
            .field(
                "num_vulnerable",
                &format!("{} - ({:.2}%)", &self.num_vulnerable, &self.perc_vulnerable),
            )
            .field(
                "num_infected",
                &format!("{} - ({:.2}%)", &self.num_infected, &self.perc_infected),
            )
            .field(
                "num_patched",
                &format!("{} - ({:.2}%)", &self.num_patched, &self.perc_patched),
            )
            .finish()
    }
}

// --------------------------------------- //

impl From<InfectionState> for InfectionCount {
    fn from(value: InfectionState) -> Self {
        match value {
            InfectionState::NotVulnerable => Self {
                not_vulnerable: 1,
                ..Self::default()
            },
            InfectionState::Vulnerable => Self {
                vulnerable: 1,
                ..Self::default()
            },
            InfectionState::Infected => Self {
                infected: 1,
                ..Self::default()
            },
            InfectionState::Patched => Self {
                patched: 1,
                ..Self::default()
            },
        }
    }
}

impl From<PowerGeneration> for InfectionCount {
    fn from(value: PowerGeneration) -> Self { value.infection_state.into() }
}
