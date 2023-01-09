use std::fmt::{Debug, Display};

use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use crate::agent::Area;
use crate::agent::{AgentKind, AgentList, AgentTrait};
use crate::grid::{Boundaries, BoundaryAgentTrait, FreqState, Grid, GridState, GridWarning, PowerState};
use crate::util::{mHz, BaseUint, Steps};

/// The Root agent. Top-level of the grid and sits above [Area].
///
/// The root agent does not have a true physical representative in the grid. It is the combination of all the area nodes
/// and gives insight into the frequency of the grid. In the actual grid, this can be measured at each node in the grid.
///
/// This agent is only here for calculation and programming purposes.
#[derive(Clone, Serialize, AvroSchema, Deserialize)]
pub struct Root {
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
    /// Frequency boundary of the grid, that needs to be monitored at this agent.
    pub freq_boundary: Boundaries<mHz>,
    /// The current frequency and history of the frequency of the grid.
    pub freq_state:    FreqState,
}

impl Root {
    /// Gives mutable access to the freq_state field.
    pub fn freq_state(&mut self) -> &mut FreqState { &mut self.freq_state }

    /// Gives mutable access to the freq_boundary field.
    pub fn freq_boundary(&mut self) -> &mut Boundaries<mHz> { &mut self.freq_boundary }

    /// Creates a new root agent, given the grid parameters and index of the graph.
    pub fn new(index: BaseUint, grid_param: &Grid) -> Self {
        Self {
            kind: AgentKind::Root,
            index,
            step: Steps(0),
            children: vec![],
            powerstate: PowerState::new(),
            freq_boundary: Boundaries::<mHz>::from(grid_param),
            freq_state: FreqState::new(grid_param),
        }
    }
}

impl BoundaryAgentTrait<mHz> for Root {
    fn boundary_check(&mut self) -> Option<GridWarning> {
        if let Some(mut gw) = self.freq_boundary.update(self.freq_state.now) {
            gw.agent_index = Some(self.index);
            gw.agent_powerstate = Some(self.powerstate.clone());
            gw.freq_state = Some(self.freq_state.clone());
            Some(gw)
        } else {
            None
        }
    }
}

impl AgentTrait for Root {
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

impl Debug for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        if f.sign_plus() {
            f.debug_struct("Root")
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

impl Display for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let children_index: Vec<u32> = self.children.iter().map(|c| *c.read_arc_recursive().index()).collect();

        let powerstate = self.powerstate.power_used.to_string();
        f.debug_struct("Root")
            .field("Kind", &self.kind)
            .field("Index", &self.index)
            .field("Step", &self.step)
            .field("Children", &children_index)
            .field("PowerState", &powerstate)
            .finish_non_exhaustive()
    }
}
