use std::fmt::{Debug, Display};

use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};

use crate::agent::{AgentKind, AgentList, AgentTrait};
#[allow(unused_imports)]
use crate::agent::{Netstation, Root};
use crate::grid::PowerState;
use crate::util::{BaseUint, Steps};

/// The Area agent. A level below [Root] and above [Netstation].
///
/// An Area is a representation of a big geographical region that is used by a grid operator to divide the grid in
/// multiple pieces. As such it sits below the [Root] node and has Netstation] agents as its children.
#[derive(Clone, Serialize, AvroSchema, Deserialize)]
pub struct Area {
    /// Kind of agent.
    pub kind:       AgentKind,
    /// Index used in the graph.
    pub index:      BaseUint,
    /// Current step of the model.
    pub step:       Steps,
    #[serde(skip)]
    #[avro(skip)]
    /// List of children this Agent has according to the graph.
    pub children:   AgentList,
    /// The current power state of the agent. Changes each step.
    pub powerstate: PowerState,
}

impl Area {
    /// Create a new [Area] agent using the index provided by the graph of the model.
    pub fn new(index: BaseUint) -> Self {
        Self {
            kind: AgentKind::Area,
            index,
            step: Steps(0),
            children: vec![],
            powerstate: PowerState::new(),
        }
    }
}

impl AgentTrait for Area {
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

impl Debug for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        if f.sign_plus() {
            f.debug_struct("Area")
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

impl Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let children_index: Vec<u32> = self.children.iter().map(|c| *c.read_arc_recursive().index()).collect();

        let powerstate = self.powerstate.power_used.to_string();
        f.debug_struct("Area")
            .field("Kind", &self.kind)
            .field("Index", &self.index)
            .field("Step", &self.step)
            .field("Children", &children_index)
            .field("PowerState", &powerstate)
            .finish_non_exhaustive()
    }
}
