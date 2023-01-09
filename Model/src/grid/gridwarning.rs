use serde::{Serialize, Deserialize};

use crate::grid::gridstate::GridBoundaryState;
#[allow(unused_imports)]
use crate::grid::BoundaryBand;
use crate::grid::{FreqState, InfectionStatistics, PowerState, VoltState};
use crate::util::BaseUint;

/// The warning returned if the grid is outside normal parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GridWarning {
    /// The state the grid is in.
    pub state:               GridBoundaryState,
    /// Has the restriction overstepped its maximum allowed time to pass its boundary? See [BoundaryBand].
    pub critical:            bool,
    /// Index of the agent that created the warning.
    pub agent_index:         Option<BaseUint>,
    /// The powerstate of the agent.
    pub agent_powerstate:    Option<PowerState>,
    /// The frequency state of the agent.
    pub freq_state:          Option<FreqState>,
    /// The voltage state of the agent.
    pub volt_state:          Option<VoltState>,
    /// InfectionStatistics
    pub infectionstatistics: Option<InfectionStatistics>,
}
