//! The infection rate, patch rate and behaviour of infected agents are defined here.
//!
//! Both structures referenced in this module are created by the AttackParameters file given to the executable via the
//! ModelParameters file.

use std::sync::Arc;

use parking_lot::RwLock;
use rand::{rngs::SmallRng};
// #[cfg(feature = "multi_thread")]
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{agent::{AgentTrait, Household}, util::default_smallrng};
use crate::grid::InfectionState;
#[allow(unused_imports)]
use crate::grid::PowerGeneration;
// #[cfg(feature = "single_thread")]
// use crate::norayon::prelude::*;
use crate::util::{random_percentage, BaseFloat, Steps};

/// Information on the attack and patching behaviour during the model.
///
/// This is built from the AttackParameters defined outside the model.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attack {
    /// Percentage of [PowerGeneration] units that are vulnerable to an [Attack]
    pub percentage_vuln_devices: f32,
    /// Percentage of infections per step.
    pub infection_rate_per_step: f32,
    /// Percentage of patched devices per step.
    pub patch_rate_per_step:     f32,
    /// Starting step of the infection.
    pub infection_start:         Steps,
    /// Last step of the infection.
    pub infection_stop:          Steps,
    /// Starting step of patching.
    pub patch_start:             Steps,
    /// Last step of patching.
    pub patch_stop:              Steps,
    /// A list of behaviours that are exhibited by infected units.
    pub attack_behaviour:        Vec<AttackBehaviour>,
    #[serde(skip)]
    #[serde(default = "default_smallrng")]
    /// A seed to power the rng generators that are used to determine if an agent is impacted.
    pub seed:                    Arc<RwLock<SmallRng>>,
    /// Out of the list of behaviours, defined in [AttackBehaviour], what is the current active one?
    /// If one is active at all.
    pub current_attack:          Option<AttackBehaviour>,
}


impl Attack {
    /// Tries to patch vulnerable or infected [`PowerGeneration`] units. Checks
    /// for [`InfectionState`] within.
    #[inline]
    pub fn try_to_patch_and_infect(&mut self, hh: &Vec<Arc<RwLock<Household>>>) {
        if self.current_attack.is_none() {
            return;
        }
        let will_patch_will_infect: Vec<(bool, bool)> = (0..hh.len())
            .into_iter()
            .map(|_| {
                let seed = &mut self.seed.write_arc();
                (
                    random_percentage(seed) < self.patch_rate_per_step,
                    random_percentage(seed) < self.infection_rate_per_step,
                )
            })
            .collect();

        hh.par_iter()
            .zip(will_patch_will_infect)
            .for_each(|(h, (will_patch, will_infect))| {
                let mut house = h.write_arc();
                match (house.power_generation.infection_state, will_infect, will_patch) {
                    (InfectionState::NotVulnerable, ..) => (),
                    (InfectionState::Patched, ..) => (),
                    (InfectionState::Infected, _, true) => {
                        house.power_generation.infection_state = InfectionState::Patched;
                    }
                    (InfectionState::Vulnerable, _, true) => {
                        house.power_generation.infection_state = InfectionState::Patched;
                    }
                    (InfectionState::Infected, _, false) => (),
                    (InfectionState::Vulnerable, true, _) => {
                        house.power_generation.infection_state = InfectionState::Infected;
                    }
                    (InfectionState::Vulnerable, false, false) => (),
                }
            })
    }

    /// Check if in the list of behaviours an attack is present and if so, set it in the field of the struct. This field
    /// is used by the infected agents to modify their behaviour.
    #[inline]
    #[cfg(not(feature = "single_thread"))]
    pub fn check_current_attack(&mut self, step: Steps) {
        self.current_attack = self
            .attack_behaviour
            .par_iter()
            .find_first(|ab| ab.is_active(step))
            .copied();
    }

    #[inline]
    #[cfg(feature = "single_thread")]
    /// Check if in the list of behaviours an attack is present and if so, set it in the field of the struct. This field
    /// is used by the infected agents to modify their behaviour.
    pub fn check_current_attack(&mut self, step: Steps) {
        self.current_attack = self.attack_behaviour.par_iter().find(|ab| ab.is_active(step)).copied();
    }

    /// Modify the infected households using the attack behaviour.
    #[inline]
    pub fn modify_infected_devices(&self, hh: &Vec<Arc<RwLock<Household>>>) {
        if let Some(modifier) = &self.current_attack {
            hh.par_iter()
                .filter(|h| h.read_arc_recursive().power_generation().infection_state == InfectionState::Infected)
                .for_each(|h| h.write_arc().powerstate_mut().attack(*modifier));
        }
    }
}

/// A step in the behaviour of the attacker. Is generated from the same attack parameters as the attack.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AttackBehaviour {
    /// When does the attack begin? Inclusive.
    pub begin:               Steps,
    /// When does the attack end? Not inclusive.
    pub end:                 Steps,
    /// Percentage modifier of the report field. 1.0 is normal behaviour.
    pub report_modifier:     BaseFloat,
    /// Percentage modifier of the generation field. 1.0 is normal behaviour.
    pub generation_modifier: BaseFloat,
}

impl AttackBehaviour {
    /// Is this substep active on this step? Does not include the end step.
    pub fn is_active(&self, step: Steps) -> bool { step >= self.begin && step <= self.end }
}
