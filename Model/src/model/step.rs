use std::sync::Arc;

use log::{debug, error, info, warn};
use parking_lot::RwLock;
// #[cfg(feature = "multi_thread")]
use rayon::{iter::Either, prelude::{IntoParallelRefIterator, ParallelIterator}};
use serde::{Serialize, Deserialize};
use tokio::task::JoinHandle;

use crate::agent::{AgentKind, AgentList, AgentTrait, Area, Household, Netstation, Root};
use crate::attack::Attack;
use crate::grid::{BoundaryAgentTrait, FreqState, Grid, GridState, GridWarning, InfectionState, InfectionStatistics, PowerState, ReservePower};
use crate::model::Model;
// #[cfg(feature = "single_thread")]
// use crate::norayon::prelude::*;
use crate::util::{ModelError, Steps};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GridInformation {
    pub infection_statistics: InfectionStatistics,
    pub freq_state:           FreqState,
    pub power_state:          PowerState,
    pub reserve_power:        ReservePower,
}

impl<'a> Model {
    /// Take the next step in the model. A step is also called a tick in some
    /// literature.

    // #[cfg_attr(feature = "multi_thread", tokio::main)]
    // #[cfg_attr(feature = "single_thread", tokio::main)]
    #[tokio::main]
    pub async fn step(&'a mut self, num_steps: Steps) -> Result<(), ModelError> {
        let mut handles: Vec<JoinHandle<()>> = vec![];

        info!("Model Param: {}", serde_json::to_string(&self.param).unwrap());
        info!("Grid Param: {}", serde_json::to_string(&self.param.grid).unwrap());
        info!("Attack Param: {}", serde_json::to_string(&self.param.attack).unwrap());
        // info!("Graph Layout: {}", &self.graph.get_dot_string());

        info!("Number of Root agents: {}", 1);
        info!("Number of Area agents: {}", self.areas.len());
        info!("Number of Netstation agents: {}", self.netstations.len());
        info!("Number of Household agents: {}", self.households.len());

        for step_inner in 0..num_steps.0 {
            let step = Steps(step_inner);
            info!("{} - Taking step {}", self.param.name, step);

            debug!("Substep update step");
            // Set internal step of agents to current step
            Self::update_step(&self.agents, step);

            debug!("Substep new powerstate");
            // New powerstate for Agents with [`PowerGeneration`] units
            Self::new_power_state(&self.households);

            debug!("Substep attack_and_patch");
            // Attack and Patch
            Self::attack_and_patch(&self.households, &mut self.param.attack, step);

            debug!("Substep powerstate from children");
            // Powerstate from children
            Self::power_state_from_children(&self.netstations, &self.areas, &self.root);

            // Get states
            let states: Vec<InfectionState> = self
                .households
                .par_iter()
                .map(|hh| hh.read_arc_recursive().power_generation.infection_state)
                .collect();
            // #[cfg(feature = "multi_thread")]
            let infstats = tokio::spawn(async move { InfectionStatistics::new(&states) });
            // #[cfg(feature = "single_thread")]
            // let infstats = InfectionStatistics::new(&states);

            // Grid compensation
            debug!("Substep grid compensation");
            Self::grid_frequency_compensation(self);

            debug!("Substep power mismatch impact");
            // Impact from power mismatch
            Self::power_mismatch_impact(&self.netstations, &self.root, &self.param.grid);

            debug!("Substep boundary check");
            // Bounds check
            if Self::boundary_check(self, &self.netstations, &self.root).is_err() {
                return Err(ModelError::NeedToStop);
            }

            // debug!("Substep update Powerstate History");
            // Update PowerGeneration history
            // Self::update_history(&self.agents);

            // Output
            if self.param.enable_output {
                debug!("Substep model output");
                self.output(step, &mut handles)?;
            }
            
            let root_info = self.root.read_arc_recursive();
            
            let grid_information = GridInformation {
                // #[cfg(feature = "multi_thread")]
                infection_statistics:                                   infstats.await?,
                // #[cfg(feature = "single_thread")]
                // infection_statistics:                                   infstats,
                freq_state:                                             root_info.freq_state.clone(),
                power_state:                                            root_info.powerstate.clone(),
                reserve_power:                                          self.reserve_power,
            };
            
            debug!("Grid information - {:?}", &grid_information.infection_statistics);
            debug!("Grid information - {:?}", &grid_information.freq_state);
            debug!("Grid information - {:?}", &grid_information.power_state);
            debug!("Grid information - {:?}", &grid_information.reserve_power);
            info!(
                "Grid information - {}",
                &serde_json::to_string(&grid_information).unwrap()
            );
        }
        for handle in handles {
            handle.await.unwrap();
        }

        Ok(())
    }

    /// Update step
    pub fn update_step(agents: &AgentList, step: Steps) {
        agents.par_iter().for_each(|a| a.write_arc().update_step(step));
    }

    // New powerstate for [`PowerGeneration`] units
    #[inline]
    fn new_power_state(hh: &Vec<Arc<RwLock<Household>>>) {
        // only households
        hh.par_iter().for_each(|a| {
            a.write_arc().clean_power_gen();
        });
    }

    #[inline]
    fn attack_and_patch(hh: &Vec<Arc<RwLock<Household>>>, attack: &mut Attack, step: Steps) {
        // Check on current attack
        attack.check_current_attack(step);
        // only households
        // Patching and infecting
        attack.try_to_patch_and_infect(hh);

        // Attacking
        attack.modify_infected_devices(hh);
    }

    /// Create a vector that copies the Arc of the agents with the desired AgentKind.
    #[inline]
    pub fn subagent(agentkind: AgentKind, agents: &AgentList) -> AgentList {
        agents
            .par_iter()
            .filter(|a| *a.read_arc_recursive().kind() == agentkind)
            .map(std::clone::Clone::clone)
            .collect()
    }

    #[inline]
    fn power_state_from_children(
        ns: &Vec<Arc<RwLock<Netstation>>>,
        area: &Vec<Arc<RwLock<Area>>>,
        root: &Arc<RwLock<Root>>,
    ) {
        // ns
        ns.par_iter()
            .for_each(|agent| agent.write_arc().calc_power_from_child());

        // area
        area.par_iter()
            .for_each(|agent| agent.write_arc().calc_power_from_child());

        // root
        root.write_arc().calc_power_from_child();
    }

    #[inline]
    fn power_mismatch_impact(ns: &[Arc<RwLock<Netstation>>], root: &Arc<RwLock<Root>>, grid_param: &Grid) {
        // Frequency
        let power_total = root.read_arc_recursive().powerstate.power_used;
        let power_error = root.read_arc_recursive().powerstate.power_error;
        let new_freq =
            root.write_arc()
                .freq_state
                .power_mismatch(&power_total, &power_error, &grid_param.bulk_consumption);

        root.write_arc().freq_state.update(new_freq);

        // Voltage
        ns.par_iter().for_each(|net| {
            let mut n = net.write_arc();
            let power_total = n.powerstate.power_used;
            let power_error = n.powerstate.power_error;
            let new_volt = n
                .volt_state
                .power_mismatch(&power_total, &power_error, &grid_param.bulk_consumption);
            n.volt_state.update(new_volt);
        });
    }

    /// Try to compensate for the power mismatch on the [Root] node.
    #[inline]
    pub fn grid_frequency_compensation(&mut self) {
        let power_error = self.root.read_arc_recursive().powerstate().power_error;
        let able = self.reserve_power.compensate(power_error);
        debug!("Compensation from storage - {}", serde_json::to_string(&able).unwrap());
        self.root.write_arc().powerstate_mut().power_error -= able;
    }

    #[inline]

    fn boundary_check(
        model: &Self,
        ns: &Vec<Arc<RwLock<Netstation>>>,
        root: &Arc<RwLock<Root>>,
    ) -> Result<(), ModelError> {
        let frequency_warnings: Option<GridWarning> = root.write_arc().boundary_check();
        let need_to_stop = frequency_warnings.is_some();
        if let Some(fw) = frequency_warnings {
            if fw.critical {
                error!("Frequency error - {}", serde_json::to_string(&fw).unwrap());
            } else {
                warn!("Frequency warning - {}", serde_json::to_string(&fw).unwrap());
            }
        }

        let voltage_warnings = ns.par_iter().filter_map(|a| a.write_arc().boundary_check());
        // #[cfg(feature = "single_thread")]
        // let (v_errors, v_warnings): (Vec<GridWarning>, Vec<GridWarning>) = voltage_warnings.partition(|x| x.critical);
        // #[cfg(feature = "multi_thread")]
        let (v_errors, v_warnings): (Vec<GridWarning>, Vec<GridWarning>) =
            voltage_warnings.partition_map(|x| if x.critical { Either::Left(x) } else { Either::Right(x) });

        if !v_errors.is_empty() {
            error!("Voltage error - {}", serde_json::to_string(&v_errors).unwrap());
        }

        if !v_warnings.is_empty() {
            warn!("Voltage warning - {}", serde_json::to_string(&v_warnings).unwrap());
        }

        if model.param.stop_on_freq_error && need_to_stop {
            Err(ModelError::NeedToStop)
        } else {
            Ok(())
        }
    }

    /// Update the history of the agents given.
    #[inline]
    pub fn update_history(agents: &AgentList) {
        agents.par_iter().for_each(|a| {
            a.write_arc().powerstate_mut().update_history();
        });
    }
}
