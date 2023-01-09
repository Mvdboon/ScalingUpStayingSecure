use std::fmt::{Debug, Display};

use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};

use crate::agent::{AgentKind, AgentList, AgentTrait};
#[allow(unused_imports)]
use crate::agent::{Area, Household};
use crate::grid::{Boundaries, BoundaryAgentTrait, Grid, GridState, GridWarning, InfectionState, InfectionStatistics, PowerState, VoltState};
use crate::util::{mVolt, BaseUint, Steps};

/// The Netstation agent. A level below [Area] and one above [Household].
///
/// A [Netstation] is a power distribution house that is situated in a neighbourhood. They take the High Voltage of
/// the grid and transform this to the known 230V. As this is the point where the voltage of the grid is determined,
/// this agent is responsible for the Voltage stability monitoring of the grid.
#[derive(Clone, Serialize, Deserialize, AvroSchema)]
pub struct Netstation {
    /// Kind of agent.
    pub kind:          AgentKind,
    /// Index used in the graph.
    pub index:         BaseUint,
    /// Current step of the model.
    pub step:          Steps,
    #[serde(skip)]
    #[avro(skip)]
    /// List of children this Agent has according to the graph.
    pub children:      AgentList,
    /// The current power state of the agent. Changes each step.
    pub powerstate:    PowerState,
    /// Voltage boundary of the grid, that needs to be monitored at this agent.
    pub volt_boundary: Boundaries<mVolt>,
    /// The current voltage and history of voltage within the agent.    
    pub volt_state:    VoltState,
}

impl Netstation {
    /// Gives mutable access to the volt_state field.
    pub fn volt_state(&mut self) -> &mut VoltState { &mut self.volt_state }

    /// Gives mutable access to the volt_boundary field.
    pub fn volt_boundary(&mut self) -> &mut Boundaries<mVolt> { &mut self.volt_boundary }

    /// Creates a new netstation agent, given the grid parameters and index of the graph.
    pub fn new(index: BaseUint, grid_param: &Grid) -> Self {
        Self {
            kind: AgentKind::Netstation,
            index,
            step: Steps(0),
            children: vec![],
            powerstate: PowerState::new(),
            volt_boundary: Boundaries::<mVolt>::from(grid_param),
            volt_state: VoltState::new(grid_param),
        }
    }

    /// Get the infection statistics for the children of the netstation
    pub fn get_infection_state_children(&self) -> InfectionStatistics {
        let states: Vec<InfectionState> = self
            .children()
            .iter()
            .filter_map(|c| c.read_arc_recursive().power_gen().map(|pg| pg.infection_state))
            .collect();
        InfectionStatistics::new(&states)
    }
}

impl BoundaryAgentTrait<mVolt> for Netstation {
    fn boundary_check(&mut self) -> Option<GridWarning> {
        if let Some(mut gw) = self.volt_boundary.update(self.volt_state.now) {
            gw.agent_index = Some(self.index);
            gw.agent_powerstate = Some(self.powerstate.clone());
            gw.volt_state = Some(self.volt_state.clone());
            gw.infectionstatistics = Some(self.get_infection_state_children());
            Some(gw)
        } else {
            None
        }
    }
}

impl AgentTrait for Netstation {
    fn kind(&self) -> &AgentKind { &self.kind }

    fn step(&self) -> &Steps { &self.step }

    fn index(&self) -> &BaseUint { &self.index }

    fn powerstate(&self) -> &PowerState { &self.powerstate }

    fn children(&self) -> &AgentList { &self.children }

    fn kind_mut(&mut self) -> &mut super::AgentKind { &mut self.kind }

    fn index_mut(&mut self) -> &mut crate::util::BaseUint { &mut self.index }

    fn step_mut(&mut self) -> &mut crate::util::Steps { &mut self.step }

    fn children_mut(&mut self) -> &mut super::AgentList { &mut self.children }

    fn powerstate_mut(&mut self) -> &mut crate::grid::PowerState { &mut self.powerstate }

    fn update_step(&mut self, step: crate::util::Steps) { self.step = step; }

    fn get_json(&self) -> Result<Vec<u8>, crate::util::ModelError> {
        match serde_json::to_vec(&self) {
            Ok(v) => Ok(v),
            Err(e) => Err(crate::util::ModelError::LogStateErrorJson {
                agent:  self.to_string(),
                source: e,
            }),
        }
    }

    fn get_avro(&self) -> Result<(apache_avro::Schema, apache_avro::types::Value), crate::util::ModelError> {
        let schema = Self::get_schema();
        match apache_avro::to_value(self) {
            Ok(v) => Ok((schema, v)),
            Err(e) => Err(crate::util::ModelError::LogStateErrorAvro {
                agent:  self.to_string(),
                source: e,
            }),
        }
    }
}

impl Debug for Netstation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        if f.sign_plus() {
            f.debug_struct("Netstation")
                .field("Kind", &self.kind)
                .field("Step", &self.step)
                .field("Index", &self.index)
                .field("Children", &self.children)
                .field("PowerState", &self.powerstate)
                .finish_non_exhaustive()
        } else {
            let children_index: Vec<u32> = self.children.iter().map(|c| *c.read_arc_recursive().index()).collect();
            f.debug_struct("Agent")
                .field("Kind", &self.kind)
                .field("Step", &self.step)
                .field("Index", &self.index)
                .field("Children", &children_index)
                .finish_non_exhaustive()
        }
    }
}

impl Display for Netstation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let children_index: Vec<u32> = self.children.iter().map(|c| *c.read_arc_recursive().index()).collect();

        let powerstate = self.powerstate.power_used.to_string();
        f.debug_struct("Netstation")
            .field("Kind", &self.kind)
            .field("Index", &self.index)
            .field("Step", &self.step)
            .field("Children", &children_index)
            .field("PowerState", &powerstate)
            .finish_non_exhaustive()
    }
}
