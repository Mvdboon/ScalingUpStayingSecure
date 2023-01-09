//! Defines the agents that are part of the model and the trait that is used to create new ones.
//!
//! To take part in the simulation agents need to implement the [`AgentTrait`] trait. This allows them to be a part of
//! the underlying graph and take part in the grid modelling.
//!
//!
//! Furthermore, some types are defined here that make it easier to read the code. These are nothing else than other
//! names for data structures.
//!
//! # New Agents
//! Implementing new agents can be done by implementing [`AgentTrait`] and adding the discriminant to the [`AgentKind`]
//! enum. The behaviour during each step of the model needs to be defined in the model part.

mod area;
mod household;
mod netstation;
mod root;

use std::fmt::Debug;
use std::sync::Arc;

use apache_avro::{AvroSchema, Schema};
pub use area::*;
pub use household::*;
use log::trace;
pub use netstation::*;
use parking_lot::RwLock;
pub use root::*;
use serde::{Deserialize, Serialize};

use crate::grid::{PowerGeneration, PowerState};
use crate::util::{BaseUint, ModelError, Steps};

/// This type can be used to indicate a struct that implements the [AgentTrait] within a RwLock and Arc.
pub type AgentRef = Arc<RwLock<dyn AgentTrait + Send + Sync>>;
/// A vector of the [AgentRef] type.
pub type AgentList = Vec<AgentRef>;

/// The [Area] struct within an Arc and RwLock.
pub type AreaRef = Arc<RwLock<Area>>;
/// A vector of the [AreaRef] type.
pub type AreaList = Vec<AreaRef>;

/// The [Netstation] struct within an Arc and RwLock.
pub type NetstationRef = Arc<RwLock<Netstation>>;
/// A vector of the [NetstationRef] type.
pub type NetstationList = Vec<NetstationRef>;

/// The [Household] struct within an Arc and RwLock.
pub type HouseholdRef = Arc<RwLock<Household>>;
/// A vector of the [HouseholdRef] type.
pub type HouseholdList = Vec<HouseholdRef>;

/// Implementing this trait lets structs participate in the model. New agents need to implement this trait.
///
/// The trait gives access to the inner data of the Agent, therefore this trait forces also that the Agent structs have
/// the correct inner fields. Other methods that you want to use on the agents can be implemented on the struct
/// themselves.
pub trait AgentTrait: Debug {
    /// Read access to kind field.
    fn kind(&self) -> &AgentKind;
    /// Read access to index field.
    fn index(&self) -> &BaseUint;
    /// Read access to step field.
    fn step(&self) -> &Steps;
    /// Read access to powerstate field.
    fn powerstate(&self) -> &PowerState;
    /// Read access to children field.
    fn children(&self) -> &AgentList;
    /// Read access to power_generation field.
    fn power_gen(&self) -> Option<&PowerGeneration> { None }

    /// Mutuable access to the kind field.
    fn kind_mut(&mut self) -> &mut AgentKind;
    /// Mutuable access to the index field.
    fn index_mut(&mut self) -> &mut BaseUint;
    /// Mutable access to the step field.
    fn step_mut(&mut self) -> &mut Steps;
    /// Mutable access to the chlidren field.
    fn children_mut(&mut self) -> &mut AgentList;
    /// Mutable access to the powerstate field.
    fn powerstate_mut(&mut self) -> &mut PowerState;
    /// Mutable access to power_generation field.
    fn power_gen_mut(&mut self) -> Option<&mut PowerGeneration> { None }

    /// Set the step field of the agent.
    fn update_step(&mut self, step: Steps);
    /// Transform the agent to a JSON representation of itself.
    fn get_json(&self) -> Result<Vec<u8>, ModelError>;
    /// Transform the agent to an AVRO representation of itself.
    fn get_avro(&self) -> Result<(Schema, apache_avro::types::Value), ModelError>;

    /// This method is called during a step of the model to determine the powerstate of the Agent if it has children.
    fn calc_power_from_child(&mut self) {
        trace!("calc power from child - Agent {:?}", &self);
        if self.children().is_empty() {
            return;
        }
        let mut res: PowerState = PowerState::new();
        for child in self.children() {
            res.manual_add(child.read_arc_recursive().powerstate());
        }
        self.powerstate_mut().power_used = res.power_used;
        self.powerstate_mut().power_reported = res.power_reported;
        self.powerstate_mut().power_error = res.power_error;
        self.powerstate_mut().power_generated = res.power_generated;
    }
}

/// Agents and results in better readability of the code at the small expense of
/// enum
#[derive(Debug, Serialize, Clone, PartialEq, Eq, Deserialize, Copy, AvroSchema)]
pub enum AgentKind {
    /// Root of the Grid
    Root,
    /// Connections between other agents
    Connection,
    /// Area agent represents a larger geographical area.
    Area,
    /// Netstation are the local transformers that transform from High Voltage
    /// (HV) to Low Voltage (LV). They link households with areas.
    Netstation,
    /// Represents a single household. It may have a PV system. However, that is
    /// determined by the link in the model graph.
    Household,
}
