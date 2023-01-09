use std::fmt::{Debug, Display};

use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use crate::agent::Netstation;
use crate::agent::{AgentKind, AgentList, AgentTrait};
use crate::grid::{PowerGeneration, PowerState};
use crate::util::{BaseUint, Steps, Watt};
/// The Household agent. A level below [Netstation] and lowest on the grid.
///
/// An Household is a representation of a single house that is connected to the powergrid. It can have a PV system to
/// generate its own power. These [PowerGeneration] devices can be vulnerable to a cyber attack and infect the grid as
/// a result of an attack.
#[derive(Clone, Serialize, AvroSchema, Deserialize)]
pub struct Household {
    /// Kind of agent.
    pub kind:             AgentKind,
    /// Index used in the graph.
    pub index:            BaseUint,
    /// Current step of the model.
    pub step:             Steps,
    #[serde(skip)]
    #[avro(skip)]
    /// List of children this Agent has according to the graph.
    pub children:         AgentList,
    /// The current power state of the agent. Changes each step.
    pub powerstate:       PowerState,
    /// A [Household] is the only agent that generates power using a DER / PV.
    /// This field contains that power generation unit.
    pub power_generation: PowerGeneration,
}

impl Household {
    /// Read access to the PowerGeneration field.
    pub fn power_generation(&self) -> &PowerGeneration { &self.power_generation }

    /// Gives mutable access to the PowerGeneration field.
    pub fn power_generation_mut(&mut self) -> &mut PowerGeneration { &mut self.power_generation }

    /// Creates a new household agent, using the index of the graph and the PowerGeneration provided.
    pub fn new(index: BaseUint, power_generation: PowerGeneration) -> Self {
        Self {
            kind: AgentKind::Household,
            index,
            step: Steps(0),
            children: vec![],
            powerstate: PowerState::new(),
            power_generation,
        }
    }

    /// Calculates the power state of the agent, without it being impacted by a cyber attack.
    pub fn clean_power_gen(&mut self) {
        let (generated, used, reported) = self.power_generation.calc_power(&self.step);
        self.powerstate.power_used = used;
        self.powerstate.power_generated = generated;
        self.powerstate.power_reported = reported;
        self.powerstate.power_error = Watt(0);
    }
}

impl AgentTrait for Household {
    fn kind(&self) -> &AgentKind { &self.kind }

    fn step(&self) -> &Steps { &self.step }

    fn index(&self) -> &BaseUint { &self.index }

    fn powerstate(&self) -> &PowerState { &self.powerstate }

    fn children(&self) -> &AgentList { &self.children }

    fn power_gen(&self) -> Option<&PowerGeneration> { Some(&self.power_generation) }

    fn kind_mut(&mut self) -> &mut super::AgentKind { &mut self.kind }

    fn index_mut(&mut self) -> &mut crate::util::BaseUint { &mut self.index }

    fn step_mut(&mut self) -> &mut crate::util::Steps { &mut self.step }

    fn children_mut(&mut self) -> &mut super::AgentList { &mut self.children }

    fn powerstate_mut(&mut self) -> &mut crate::grid::PowerState { &mut self.powerstate }

    fn power_gen_mut(&mut self) -> Option<&mut PowerGeneration> { Some(&mut self.power_generation) }

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

impl Debug for Household {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        if f.sign_plus() {
            f.debug_struct("Household")
                .field("Kind", &self.kind)
                .field("PV", &self.power_generation.generation_param.is_empty())
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

impl Display for Household {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let children_index: Vec<u32> = self
            .children
            .iter()
            .map(|c| (*c.read_arc_recursive().index()))
            .collect();

        let powerstate = self.powerstate.power_used.to_string();
        f.debug_struct("Household")
            .field("Kind", &self.kind)
            .field("Index", &self.index)
            .field("Step", &self.step)
            .field("Children", &children_index)
            .field("PowerState", &powerstate)
            .finish_non_exhaustive()
    }
}
