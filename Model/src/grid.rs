//! All things related to the grid and power can be found here.
//!
//! A few different topics are part of this module.
//!
//! # Construction of the Grid
//! The construction of the gird is done by the parameters given in the GridParameters file that was given to the
//! executable by the ModelParameters file.
//!
//! # Parameters
//! - Bulk consumption.    
//!   The amount of Watt that is consumed by big energy consumers.
//!   The bigger this number the lower the instability of the grid.  
//! - Boundary values.    
//!   For both frequency and voltage, the allowed limits are given.   
//!   These are extracted from EU regulations.  
//! - Number of agents per layer.    
//!   How many agents are needed per layer? Currently, no connections are made on a horizontal level.  
//!   So this is the number of children per node.
//! - PV adoption.    
//!   The number of households that have a PV installation at the home.  
//!   This may or may not be infectable.  
//!
//! # Power generation
//! Generation of power is done in this model by PV installations. However, this is a subset of Distributed Energy
//! Resources (DER) that can be used in its place instead.
//!
//! Generation characteristics are based on actual household data given by the [MFFBAS document](https://www.mffbas.nl/documenten/).
//! PV households are assumed to be net zero throughout the year. But do not have energy storage in the house.
//! To let each house be unique the standardized profile is adjusted with noise functions and a linear modifier. This
//! gives dynamic in the grid.
//!
//! # Power mismatch
//! When there is too much or too little power being generated with regard to the power being consumed this has an
//! impact on the voltage and frequency of the grid. This is monitored by the [Root] and [Netstation] agents.

mod boundary;
mod gridstate;
mod gridwarning;
mod powergeneration;
mod reservepower;
mod states;

use std::fmt::Debug;

pub use boundary::*;
pub use gridstate::*;
pub use gridwarning::*;
pub use powergeneration::*;
pub use reservepower::*;
use serde::{Serialize, Deserialize};
pub use states::*;

#[allow(unused_imports)]
use crate::agent::{Netstation, Root};
use crate::util::{mHz, mVolt, BaseFloat, BaseInt, Watt};

/// Struct that defines the grid parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grid {
    /// Number of areas
    pub n_areas:                                  BaseInt,
    /// Energy regulating room in W
    pub energy_storage:                           Watt,
    /// Max energy generation in W per time tick
    pub max_gen_inc_tick:                         Watt,
    /// Bounds for the number of netstation per
    /// area. Intended to be used with a uniform
    /// distribution
    pub ns_per_a:                                 (BaseInt, BaseInt),
    /// Bounds for the number of households per
    /// netstation. Intended to be used with a
    /// uniform distribution
    pub hs_per_ns:                                (BaseInt, BaseInt),
    /// Percentage of PV adoption by households
    /// i.e. chance of a household having a PV.
    pub pv_adoption:                              BaseFloat,
    /// Amount of noise functions per [PowerGeneration] unit.
    pub num_noise_functions:                      BaseInt,
    /// The normal distribution parameters that determine [PowerGeneration] units.
    /// First is center, second value is std deviation.
    pub household_power_consumption_distribution: (Watt, Watt),
    /// Percentage of noise function based on the total power consumption of the
    /// household.
    pub percentage_noise_on_power:                f32,
    /// The percentage of power with regards to the average consumption of the household that is being generated on average.
    pub percentage_generation_of_usage:           f32,
    /// Bulk consumption, i.e. factories
    pub bulk_consumption:                         Watt,
    /// Impact of voltage due to power mismatch. Linear assumption.
    pub volt_modifier:                            BaseFloat,
    pub(crate) volt_boundary:                     Boundaries<mVolt>,
    pub(crate) freq_boundary:                     Boundaries<mHz>,
}
